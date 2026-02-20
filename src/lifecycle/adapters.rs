use crate::error::Error;
use crate::lifecycle::TerminalStatus;
use crate::protocols::{self, EventType, Protocol};
use crate::types::{RawEvent, RawInstruction, ResolveContext};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorrelationOutcome {
    NotRequired,
    Correlated(Vec<String>),
    Uncorrelated { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventPayload {
    None,
    DcaFill {
        in_amount: i64,
        out_amount: i64,
    },
    DcaClosed {
        status: TerminalStatus,
    },
    LimitFill {
        in_amount: i64,
        out_amount: i64,
        remaining_in_amount: i64,
        counterparty: String,
    },
    KaminoDisplay {
        remaining_input_amount: i64,
        filled_output_amount: i64,
        terminal_status: Option<TerminalStatus>,
    },
}

pub trait ProtocolAdapter: Sync {
    fn protocol(&self) -> Protocol;

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType>;

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>>;
}

#[derive(Debug)]
pub struct DcaAdapter;

#[derive(Debug)]
pub struct LimitV1Adapter;

#[derive(Debug)]
pub struct LimitV2Adapter;

#[derive(Debug)]
pub struct KaminoAdapter;

static DCA_ADAPTER: DcaAdapter = DcaAdapter;
static LIMIT_V1_ADAPTER: LimitV1Adapter = LimitV1Adapter;
static LIMIT_V2_ADAPTER: LimitV2Adapter = LimitV2Adapter;
static KAMINO_ADAPTER: KaminoAdapter = KaminoAdapter;

impl ProtocolAdapter for DcaAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::Dca
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        protocols::dca::classify_instruction_envelope(ix)
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        protocols::dca::resolve_event_envelope(fields)
    }
}

impl ProtocolAdapter for LimitV1Adapter {
    fn protocol(&self) -> Protocol {
        Protocol::LimitV1
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        protocols::limit_v1::classify_instruction_envelope(ix)
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        protocols::limit_v1::resolve_event_envelope(fields)
    }
}

impl ProtocolAdapter for LimitV2Adapter {
    fn protocol(&self) -> Protocol {
        Protocol::LimitV2
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        protocols::limit_v2::classify_instruction_envelope(ix)
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        protocols::limit_v2::resolve_event_envelope(fields)
    }
}

impl ProtocolAdapter for KaminoAdapter {
    fn protocol(&self) -> Protocol {
        Protocol::Kamino
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        protocols::kamino::classify_instruction_envelope(ix)
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        protocols::kamino::resolve_event_envelope(fields, &ev.signature, ctx)
    }
}

pub fn dca_closed_terminal_status(closed: &protocols::dca::DcaClosedEvent) -> TerminalStatus {
    if closed.user_closed {
        TerminalStatus::Cancelled
    } else if closed.unfilled_amount == 0 {
        TerminalStatus::Completed
    } else {
        TerminalStatus::Expired
    }
}

pub fn kamino_display_terminal_status(status_code: i64) -> Result<Option<TerminalStatus>, Error> {
    let status = protocols::kamino::parse_display_status(status_code)?;
    match status {
        protocols::kamino::KaminoDisplayStatus::Open => Ok(None),
        protocols::kamino::KaminoDisplayStatus::Filled => Ok(Some(TerminalStatus::Completed)),
        protocols::kamino::KaminoDisplayStatus::Cancelled => Ok(Some(TerminalStatus::Cancelled)),
        protocols::kamino::KaminoDisplayStatus::Expired => Ok(Some(TerminalStatus::Expired)),
    }
}

pub fn adapter_for(protocol: Protocol) -> &'static dyn ProtocolAdapter {
    match protocol {
        Protocol::Dca => &DCA_ADAPTER,
        Protocol::LimitV1 => &LIMIT_V1_ADAPTER,
        Protocol::LimitV2 => &LIMIT_V2_ADAPTER,
        Protocol::Kamino => &KAMINO_ADAPTER,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::protocols::{EventType, Protocol};
    use crate::types::{RawEvent, RawInstruction, ResolveContext};

    #[test]
    fn adapter_selection_matches_protocol() {
        assert_eq!(adapter_for(Protocol::Dca).protocol(), Protocol::Dca);
        assert_eq!(adapter_for(Protocol::LimitV1).protocol(), Protocol::LimitV1);
        assert_eq!(adapter_for(Protocol::LimitV2).protocol(), Protocol::LimitV2);
        assert_eq!(adapter_for(Protocol::Kamino).protocol(), Protocol::Kamino);
    }

    #[test]
    fn instruction_classifiers_map_known_names() {
        let dca = adapter_for(Protocol::Dca);
        let ix = RawInstruction {
            id: 1,
            signature: "sig".to_string(),
            instruction_index: 0,
            program_id: "p".to_string(),
            inner_program_id: "p".to_string(),
            instruction_name: "OpenDca".to_string(),
            accounts: None,
            args: None,
            slot: 1,
        };
        assert_eq!(dca.classify_instruction(&ix), Some(EventType::Created));
    }

    #[test]
    fn dca_closed_terminal_status_user_cancelled() {
        let closed = protocols::dca::DcaClosedEvent {
            order_pda: "pda".to_string(),
            user_closed: true,
            unfilled_amount: 500,
        };
        assert_eq!(
            dca_closed_terminal_status(&closed),
            TerminalStatus::Cancelled
        );
    }

    #[test]
    fn dca_closed_terminal_status_completed() {
        let closed = protocols::dca::DcaClosedEvent {
            order_pda: "pda".to_string(),
            user_closed: false,
            unfilled_amount: 0,
        };
        assert_eq!(
            dca_closed_terminal_status(&closed),
            TerminalStatus::Completed
        );
    }

    #[test]
    fn dca_closed_terminal_status_expired() {
        let closed = protocols::dca::DcaClosedEvent {
            order_pda: "pda".to_string(),
            user_closed: false,
            unfilled_amount: 1000,
        };
        assert_eq!(dca_closed_terminal_status(&closed), TerminalStatus::Expired);
    }

    #[test]
    fn kamino_display_terminal_status_all_codes() {
        assert_eq!(kamino_display_terminal_status(0).unwrap(), None);
        assert_eq!(
            kamino_display_terminal_status(1).unwrap(),
            Some(TerminalStatus::Completed)
        );
        assert_eq!(
            kamino_display_terminal_status(2).unwrap(),
            Some(TerminalStatus::Cancelled)
        );
        assert_eq!(
            kamino_display_terminal_status(3).unwrap(),
            Some(TerminalStatus::Expired)
        );
    }

    #[test]
    fn kamino_resolve_uncorrelated_without_context() {
        let adapter = adapter_for(Protocol::Kamino);
        let ev = RawEvent {
            id: 1,
            signature: "test_sig".to_string(),
            event_index: 0,
            program_id: "p".to_string(),
            inner_program_id: "p".to_string(),
            event_name: "OrderDisplayEvent".to_string(),
            fields: Some(serde_json::json!({
                "OrderDisplayEvent": {
                    "remaining_input_amount": 0,
                    "filled_output_amount": 100,
                    "number_of_fills": 1,
                    "status": 1
                }
            })),
            slot: 1,
        };
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };

        let result = adapter
            .classify_and_resolve_event(&ev, &ctx)
            .unwrap()
            .unwrap();
        let (_event_type, correlation, payload) = result;

        assert!(matches!(
            correlation,
            CorrelationOutcome::Uncorrelated { .. }
        ));
        assert_eq!(payload, EventPayload::None);
    }
}
