use crate::error::Error;
use crate::lifecycle::adapters::{CorrelationOutcome, EventPayload, dca_closed_terminal_status};
use crate::protocols::{AccountInfo, EventType, find_account_by_name};
use crate::types::RawInstruction;

#[derive(serde::Deserialize)]
#[expect(
    clippy::enum_variant_names,
    reason = "variant names mirror Carbon decoder crate"
)]
pub(crate) enum DcaEventEnvelope {
    OpenedEvent(DcaKeyHolder),
    FilledEvent(FilledEventFields),
    ClosedEvent(ClosedEventFields),
    CollectedFeeEvent(DcaKeyHolder),
    WithdrawEvent(DcaKeyHolder),
    DepositEvent(DcaKeyHolder),
}

#[derive(serde::Deserialize)]
#[expect(
    dead_code,
    reason = "variant data consumed by serde, not read directly"
)]
pub(crate) enum DcaInstructionKind {
    OpenDca(serde_json::Value),
    OpenDcaV2(serde_json::Value),
    InitiateFlashFill(serde_json::Value),
    InitiateDlmmFill(serde_json::Value),
    FulfillFlashFill(serde_json::Value),
    FulfillDlmmFill(serde_json::Value),
    CloseDca(serde_json::Value),
    EndAndClose(serde_json::Value),
    Transfer(serde_json::Value),
    Deposit(serde_json::Value),
    Withdraw(serde_json::Value),
    WithdrawFees(serde_json::Value),
}

pub fn classify_instruction_envelope(ix: &RawInstruction) -> Option<EventType> {
    let wrapper = serde_json::json!({ &ix.instruction_name: ix.args });
    let kind: DcaInstructionKind = serde_json::from_value(wrapper).ok()?;
    match kind {
        DcaInstructionKind::OpenDca(_) | DcaInstructionKind::OpenDcaV2(_) => {
            Some(EventType::Created)
        }
        DcaInstructionKind::InitiateFlashFill(_) | DcaInstructionKind::InitiateDlmmFill(_) => {
            Some(EventType::FillInitiated)
        }
        DcaInstructionKind::FulfillFlashFill(_) | DcaInstructionKind::FulfillDlmmFill(_) => {
            Some(EventType::FillCompleted)
        }
        DcaInstructionKind::CloseDca(_) | DcaInstructionKind::EndAndClose(_) => {
            Some(EventType::Closed)
        }
        DcaInstructionKind::Transfer(_)
        | DcaInstructionKind::Deposit(_)
        | DcaInstructionKind::Withdraw(_)
        | DcaInstructionKind::WithdrawFees(_) => None,
    }
}

pub fn resolve_event_envelope(
    fields: &serde_json::Value,
) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
    let envelope: DcaEventEnvelope = match serde_json::from_value(fields.clone()) {
        Ok(e) => e,
        Err(_) => return None,
    };

    Some(resolve_dca_event(envelope))
}

fn resolve_dca_event(
    envelope: DcaEventEnvelope,
) -> Result<(EventType, CorrelationOutcome, EventPayload), Error> {
    match envelope {
        DcaEventEnvelope::FilledEvent(FilledEventFields {
            dca_key,
            in_amount,
            out_amount,
        }) => Ok((
            EventType::FillCompleted,
            CorrelationOutcome::Correlated(vec![dca_key]),
            EventPayload::DcaFill {
                in_amount: in_amount as i64,
                out_amount: out_amount as i64,
            },
        )),
        DcaEventEnvelope::ClosedEvent(ClosedEventFields {
            dca_key,
            user_closed,
            unfilled_amount,
        }) => {
            let closed = DcaClosedEvent {
                order_pda: dca_key,
                user_closed,
                unfilled_amount: unfilled_amount as i64,
            };
            let status = dca_closed_terminal_status(&closed);
            Ok((
                EventType::Closed,
                CorrelationOutcome::Correlated(vec![closed.order_pda]),
                EventPayload::DcaClosed { status },
            ))
        }
        DcaEventEnvelope::OpenedEvent(DcaKeyHolder { dca_key }) => Ok((
            EventType::Created,
            CorrelationOutcome::Correlated(vec![dca_key]),
            EventPayload::None,
        )),
        DcaEventEnvelope::CollectedFeeEvent(DcaKeyHolder { dca_key }) => Ok((
            EventType::FeeCollected,
            CorrelationOutcome::Correlated(vec![dca_key]),
            EventPayload::None,
        )),
        DcaEventEnvelope::WithdrawEvent(DcaKeyHolder { dca_key }) => Ok((
            EventType::Withdrawn,
            CorrelationOutcome::Correlated(vec![dca_key]),
            EventPayload::None,
        )),
        DcaEventEnvelope::DepositEvent(DcaKeyHolder { dca_key }) => Ok((
            EventType::Deposited,
            CorrelationOutcome::Correlated(vec![dca_key]),
            EventPayload::None,
        )),
    }
}

pub fn extract_order_pda(
    accounts: &[AccountInfo],
    instruction_name: &str,
) -> Result<String, Error> {
    if let Some(acc) = find_account_by_name(accounts, "dca") {
        return Ok(acc.pubkey.clone());
    }

    let idx = match instruction_name {
        "OpenDca" | "OpenDcaV2" => 0,
        "InitiateFlashFill" | "FulfillFlashFill" | "InitiateDlmmFill" | "FulfillDlmmFill" => 1,
        "CloseDca" | "EndAndClose" => 1,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown DCA instruction: {instruction_name}"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct DcaCreateArgs {
    pub in_amount: i64,
    pub in_amount_per_cycle: i64,
    pub cycle_frequency: i64,
    pub min_out_amount: Option<i64>,
    pub max_out_amount: Option<i64>,
    pub start_at: Option<i64>,
}

pub struct DcaCreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(
    accounts: &[AccountInfo],
    instruction_name: &str,
) -> Result<DcaCreateMints, Error> {
    let input_mint = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let output_mint = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (input_mint, output_mint) {
        return Ok(DcaCreateMints {
            input_mint,
            output_mint,
        });
    }

    let (input_idx, output_idx) = match instruction_name {
        "OpenDca" => (2, 3),
        "OpenDcaV2" => (3, 4),
        _ => {
            return Err(Error::Protocol {
                reason: format!("not a DCA create instruction: {instruction_name}"),
            });
        }
    };

    let input_mint = accounts
        .get(input_idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA input_mint index {input_idx} out of bounds"),
        })?;
    let output_mint = accounts
        .get(output_idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA output_mint index {output_idx} out of bounds"),
        })?;

    Ok(DcaCreateMints {
        input_mint,
        output_mint,
    })
}

#[derive(serde::Deserialize)]
pub(crate) struct FilledEventFields {
    dca_key: String,
    in_amount: u64,
    out_amount: u64,
}

#[derive(serde::Deserialize)]
pub(crate) struct ClosedEventFields {
    dca_key: String,
    user_closed: bool,
    unfilled_amount: u64,
}

#[derive(serde::Deserialize)]
pub(crate) struct DcaKeyHolder {
    dca_key: String,
}

pub struct DcaClosedEvent {
    pub order_pda: String,
    pub user_closed: bool,
    pub unfilled_amount: i64,
}

pub struct DcaFillEvent {
    pub order_pda: String,
    pub in_amount: i64,
    pub out_amount: i64,
}

#[derive(serde::Deserialize)]
struct OpenDcaFields {
    in_amount: u64,
    in_amount_per_cycle: u64,
    cycle_frequency: i64,
    min_out_amount: Option<u64>,
    max_out_amount: Option<u64>,
    start_at: Option<i64>,
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<DcaCreateArgs, Error> {
    let OpenDcaFields {
        in_amount,
        in_amount_per_cycle,
        cycle_frequency,
        min_out_amount,
        max_out_amount,
        start_at,
    } = serde_json::from_value(args.clone()).map_err(|e| Error::Protocol {
        reason: format!("failed to parse DCA create args: {e}"),
    })?;

    Ok(DcaCreateArgs {
        in_amount: in_amount as i64,
        in_amount_per_cycle: in_amount_per_cycle as i64,
        cycle_frequency,
        min_out_amount: min_out_amount.map(|v| v as i64),
        max_out_amount: max_out_amount.map(|v| v as i64),
        start_at,
    })
}

#[cfg(test)]
pub fn classify_decoded(
    decoded: &carbon_jupiter_dca_decoder::instructions::JupiterDcaInstruction,
) -> Option<EventType> {
    use carbon_jupiter_dca_decoder::instructions::JupiterDcaInstruction;
    match decoded {
        JupiterDcaInstruction::OpenDca(_) | JupiterDcaInstruction::OpenDcaV2(_) => {
            Some(EventType::Created)
        }
        JupiterDcaInstruction::InitiateFlashFill(_)
        | JupiterDcaInstruction::InitiateDlmmFill(_) => Some(EventType::FillInitiated),
        JupiterDcaInstruction::FulfillFlashFill(_) | JupiterDcaInstruction::FulfillDlmmFill(_) => {
            Some(EventType::FillCompleted)
        }
        JupiterDcaInstruction::CloseDca(_) | JupiterDcaInstruction::EndAndClose(_) => {
            Some(EventType::Closed)
        }
        JupiterDcaInstruction::OpenedEvent(_) => Some(EventType::Created),
        JupiterDcaInstruction::FilledEvent(_) => Some(EventType::FillCompleted),
        JupiterDcaInstruction::ClosedEvent(_) => Some(EventType::Closed),
        JupiterDcaInstruction::CollectedFeeEvent(_) => Some(EventType::FeeCollected),
        JupiterDcaInstruction::WithdrawEvent(_) => Some(EventType::Withdrawn),
        JupiterDcaInstruction::DepositEvent(_) => Some(EventType::Deposited),
        JupiterDcaInstruction::Transfer(_)
        | JupiterDcaInstruction::Deposit(_)
        | JupiterDcaInstruction::Withdraw(_)
        | JupiterDcaInstruction::WithdrawFees(_) => None,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::lifecycle::TerminalStatus;

    #[test]
    fn classify_known_instructions_via_envelope() {
        let cases = [
            ("OpenDca", Some(EventType::Created)),
            ("OpenDcaV2", Some(EventType::Created)),
            ("InitiateFlashFill", Some(EventType::FillInitiated)),
            ("InitiateDlmmFill", Some(EventType::FillInitiated)),
            ("FulfillFlashFill", Some(EventType::FillCompleted)),
            ("FulfillDlmmFill", Some(EventType::FillCompleted)),
            ("CloseDca", Some(EventType::Closed)),
            ("EndAndClose", Some(EventType::Closed)),
            ("Transfer", None),
            ("Deposit", None),
            ("Withdraw", None),
            ("WithdrawFees", None),
            ("Unknown", None),
        ];
        for (name, expected) in cases {
            let ix = RawInstruction {
                id: 1,
                signature: "sig".to_string(),
                instruction_index: 0,
                program_id: "p".to_string(),
                inner_program_id: "p".to_string(),
                instruction_name: name.to_string(),
                accounts: None,
                args: None,
                slot: 1,
            };
            assert_eq!(
                classify_instruction_envelope(&ix),
                expected,
                "mismatch for {name}"
            );
        }
    }

    #[test]
    fn resolve_fill_event_from_envelope() {
        let fields = serde_json::json!({
            "FilledEvent": {
                "dca_key": "3nsTjVJTwwGvXqDRgqNCZAQKwt4QMVhHHqvyseCNR3YX",
                "in_amount": 21_041_666_667_u64,
                "out_amount": 569_529_644_u64,
                "fee": 570_099_u64,
                "fee_mint": "A7b",
                "input_mint": "So1",
                "output_mint": "A7b",
                "user_key": "31o"
            }
        });
        let (event_type, correlation, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::FillCompleted);
        let CorrelationOutcome::Correlated(pdas) = correlation else {
            panic!("expected Correlated");
        };
        assert_eq!(pdas, vec!["3nsTjVJTwwGvXqDRgqNCZAQKwt4QMVhHHqvyseCNR3YX"]);
        let EventPayload::DcaFill {
            in_amount,
            out_amount,
        } = payload
        else {
            panic!("expected DcaFill");
        };
        assert_eq!(in_amount, 21_041_666_667);
        assert_eq!(out_amount, 569_529_644);
    }

    #[test]
    fn resolve_closed_event_completed() {
        let fields = serde_json::json!({
            "ClosedEvent": {
                "dca_key": "pda1",
                "user_closed": false,
                "unfilled_amount": 0_u64,
                "created_at": 0, "in_amount_per_cycle": 0, "in_deposited": 0,
                "input_mint": "x", "output_mint": "y", "total_in_withdrawn": 0,
                "total_out_withdrawn": 0, "user_key": "z", "cycle_frequency": 60
            }
        });
        let (event_type, _, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::Closed);
        assert_eq!(
            payload,
            EventPayload::DcaClosed {
                status: TerminalStatus::Completed
            }
        );
    }

    #[test]
    fn resolve_opened_event_correlates() {
        let fields = serde_json::json!({
            "OpenedEvent": {
                "dca_key": "my_pda",
                "created_at": 0, "cycle_frequency": 60, "in_amount_per_cycle": 100,
                "in_deposited": 500, "input_mint": "a", "output_mint": "b", "user_key": "c"
            }
        });
        let (event_type, correlation, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::Created);
        assert_eq!(
            correlation,
            CorrelationOutcome::Correlated(vec!["my_pda".to_string()])
        );
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn unknown_event_returns_none() {
        let fields = serde_json::json!({"UnknownEvent": {"some_field": 1}});
        assert!(resolve_event_envelope(&fields).is_none());
    }

    #[test]
    fn resolve_deposit_event_from_envelope() {
        let fields = serde_json::json!({
            "DepositEvent": {
                "dca_key": "deposit_pda_123",
                "amount": 1_000_000_u64,
                "user_key": "user123"
            }
        });
        let (event_type, correlation, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::Deposited);
        assert_eq!(
            correlation,
            CorrelationOutcome::Correlated(vec!["deposit_pda_123".to_string()])
        );
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn mirror_enums_cover_all_carbon_variants() {
        let instruction_variants = [
            "OpenDca",
            "OpenDcaV2",
            "InitiateFlashFill",
            "InitiateDlmmFill",
            "FulfillFlashFill",
            "FulfillDlmmFill",
            "CloseDca",
            "EndAndClose",
            "Transfer",
            "Deposit",
            "Withdraw",
            "WithdrawFees",
        ];
        for name in instruction_variants {
            let json = serde_json::json!({ name: serde_json::Value::Null });
            assert!(
                serde_json::from_value::<DcaInstructionKind>(json).is_ok(),
                "DcaInstructionKind missing variant: {name}"
            );
        }

        let key_holder_variants = [
            "OpenedEvent",
            "CollectedFeeEvent",
            "WithdrawEvent",
            "DepositEvent",
        ];
        for name in key_holder_variants {
            let json = serde_json::json!({ name: { "dca_key": "test" } });
            assert!(
                serde_json::from_value::<DcaEventEnvelope>(json).is_ok(),
                "DcaEventEnvelope missing variant: {name}"
            );
        }

        let filled = serde_json::json!({
            "FilledEvent": { "dca_key": "t", "in_amount": 1_u64, "out_amount": 1_u64 }
        });
        assert!(serde_json::from_value::<DcaEventEnvelope>(filled).is_ok());

        let closed = serde_json::json!({
            "ClosedEvent": { "dca_key": "t", "user_closed": false, "unfilled_amount": 0_u64 }
        });
        assert!(serde_json::from_value::<DcaEventEnvelope>(closed).is_ok());
    }
}
