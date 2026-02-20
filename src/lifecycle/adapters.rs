use crate::error::Error;
use crate::lifecycle::TerminalStatus;
use crate::protocols::dca::DcaAdapter;
use crate::protocols::kamino::KaminoAdapter;
use crate::protocols::limit_v1::LimitV1Adapter;
use crate::protocols::limit_v2::LimitV2Adapter;
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
        Protocol::Dca => &DcaAdapter,
        Protocol::LimitV1 => &LimitV1Adapter,
        Protocol::LimitV2 => &LimitV2Adapter,
        Protocol::Kamino => &KaminoAdapter,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::protocols::{EventType, Protocol};
    use crate::types::{RawEvent, RawInstruction, ResolveContext};

    fn make_instruction(name: &str) -> RawInstruction {
        RawInstruction {
            id: 1,
            signature: "sig".to_string(),
            instruction_index: 0,
            program_id: "p".to_string(),
            inner_program_id: "p".to_string(),
            instruction_name: name.to_string(),
            accounts: None,
            args: None,
            slot: 1,
        }
    }

    fn make_event(event_name: &str, fields: Option<serde_json::Value>) -> RawEvent {
        RawEvent {
            id: 1,
            signature: "sig".to_string(),
            event_index: 0,
            program_id: "p".to_string(),
            inner_program_id: "p".to_string(),
            event_name: event_name.to_string(),
            fields,
            slot: 1,
        }
    }

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
        assert_eq!(
            dca.classify_instruction(&make_instruction("OpenDca")),
            Some(EventType::Created)
        );

        let limit_v1 = adapter_for(Protocol::LimitV1);
        assert_eq!(
            limit_v1.classify_instruction(&make_instruction("FillOrder")),
            Some(EventType::FillCompleted)
        );

        let limit_v2 = adapter_for(Protocol::LimitV2);
        assert_eq!(
            limit_v2.classify_instruction(&make_instruction("PreFlashFillOrder")),
            Some(EventType::FillInitiated)
        );

        let kamino = adapter_for(Protocol::Kamino);
        assert_eq!(
            kamino.classify_instruction(&make_instruction("CreateOrder")),
            Some(EventType::Created)
        );
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
            signature: "test_sig".to_string(),
            ..make_event(
                "OrderDisplayEvent",
                Some(serde_json::json!({
                    "OrderDisplayEvent": {
                        "remaining_input_amount": 0,
                        "filled_output_amount": 100,
                        "number_of_fills": 1,
                        "status": 1
                    }
                })),
            )
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

    #[test]
    fn dca_adapter_resolves_opened_event() {
        let adapter = adapter_for(Protocol::Dca);
        let ev = make_event(
            "OpenedEvent",
            Some(serde_json::json!({
                "OpenedEvent": { "dca_key": "dca_pda" }
            })),
        );

        let (event_type, correlation, payload) = adapter
            .classify_and_resolve_event(
                &ev,
                &ResolveContext {
                    pre_fetched_order_pdas: None,
                },
            )
            .unwrap()
            .unwrap();

        assert_eq!(event_type, EventType::Created);
        assert_eq!(
            correlation,
            CorrelationOutcome::Correlated(vec!["dca_pda".to_string()])
        );
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn limit_adapters_resolve_create_events() {
        let limit_v1 = adapter_for(Protocol::LimitV1);
        let limit_v1_event = make_event(
            "CreateOrderEvent",
            Some(serde_json::json!({
                "CreateOrderEvent": { "order_key": "v1_order" }
            })),
        );
        let (event_type_v1, _, _) = limit_v1
            .classify_and_resolve_event(
                &limit_v1_event,
                &ResolveContext {
                    pre_fetched_order_pdas: None,
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(event_type_v1, EventType::Created);

        let limit_v2 = adapter_for(Protocol::LimitV2);
        let limit_v2_event = make_event(
            "CreateOrderEvent",
            Some(serde_json::json!({
                "CreateOrderEvent": { "order_key": "v2_order" }
            })),
        );
        let (event_type_v2, _, _) = limit_v2
            .classify_and_resolve_event(
                &limit_v2_event,
                &ResolveContext {
                    pre_fetched_order_pdas: None,
                },
            )
            .unwrap()
            .unwrap();
        assert_eq!(event_type_v2, EventType::Created);
    }

    #[test]
    fn classify_and_resolve_event_returns_none_when_fields_are_absent() {
        let ev = make_event("AnyEvent", None);
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };

        assert!(
            adapter_for(Protocol::Dca)
                .classify_and_resolve_event(&ev, &ctx)
                .is_none()
        );
        assert!(
            adapter_for(Protocol::LimitV1)
                .classify_and_resolve_event(&ev, &ctx)
                .is_none()
        );
        assert!(
            adapter_for(Protocol::LimitV2)
                .classify_and_resolve_event(&ev, &ctx)
                .is_none()
        );
        assert!(
            adapter_for(Protocol::Kamino)
                .classify_and_resolve_event(&ev, &ctx)
                .is_none()
        );
    }
}
