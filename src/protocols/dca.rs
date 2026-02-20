use crate::error::Error;
use crate::lifecycle::adapters::{CorrelationOutcome, EventPayload, dca_closed_terminal_status};
use crate::protocols::{
    AccountInfo, EventType, checked_u64_to_i64, contains_known_variant, find_account_by_name,
};
use crate::types::RawInstruction;
use strum::VariantNames;

#[derive(serde::Deserialize, strum_macros::VariantNames)]
pub enum DcaEventEnvelope {
    OpenedEvent(DcaKeyHolder),
    FilledEvent(FilledEventFields),
    ClosedEvent(ClosedEventFields),
    CollectedFeeEvent(DcaKeyHolder),
    WithdrawEvent(DcaKeyHolder),
    DepositEvent(DcaKeyHolder),
}

#[derive(serde::Deserialize)]
pub enum DcaInstructionKind {
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
        Err(err) => {
            if !contains_known_variant(fields, DcaEventEnvelope::VARIANTS) {
                return None;
            }
            return Some(Err(Error::Protocol {
                reason: format!("failed to parse DCA event payload: {err}"),
            }));
        }
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
                in_amount: checked_u64_to_i64(in_amount, "in_amount")?,
                out_amount: checked_u64_to_i64(out_amount, "out_amount")?,
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
                unfilled_amount: checked_u64_to_i64(unfilled_amount, "unfilled_amount")?,
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

    let wrapper = serde_json::json!({ instruction_name: serde_json::Value::Null });
    let kind: DcaInstructionKind =
        serde_json::from_value(wrapper).map_err(|_| Error::Protocol {
            reason: format!("unknown DCA instruction: {instruction_name}"),
        })?;

    let idx = match kind {
        DcaInstructionKind::OpenDca(_) | DcaInstructionKind::OpenDcaV2(_) => 0,
        DcaInstructionKind::InitiateFlashFill(_)
        | DcaInstructionKind::FulfillFlashFill(_)
        | DcaInstructionKind::InitiateDlmmFill(_)
        | DcaInstructionKind::FulfillDlmmFill(_) => 1,
        DcaInstructionKind::CloseDca(_) | DcaInstructionKind::EndAndClose(_) => 1,
        DcaInstructionKind::Transfer(_)
        | DcaInstructionKind::Deposit(_)
        | DcaInstructionKind::Withdraw(_)
        | DcaInstructionKind::WithdrawFees(_) => {
            return Err(Error::Protocol {
                reason: format!("DCA instruction {instruction_name} has no order PDA"),
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

    let wrapper = serde_json::json!({ instruction_name: serde_json::Value::Null });
    let kind: DcaInstructionKind =
        serde_json::from_value(wrapper).map_err(|_| Error::Protocol {
            reason: format!("unknown DCA instruction: {instruction_name}"),
        })?;

    let (input_idx, output_idx) = match kind {
        DcaInstructionKind::OpenDca(_) => (2, 3),
        DcaInstructionKind::OpenDcaV2(_) => (3, 4),
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
pub struct FilledEventFields {
    dca_key: String,
    in_amount: u64,
    out_amount: u64,
}

#[derive(serde::Deserialize)]
pub struct ClosedEventFields {
    dca_key: String,
    user_closed: bool,
    unfilled_amount: u64,
}

#[derive(serde::Deserialize)]
pub struct DcaKeyHolder {
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
        in_amount: checked_u64_to_i64(in_amount, "in_amount")?,
        in_amount_per_cycle: checked_u64_to_i64(in_amount_per_cycle, "in_amount_per_cycle")?,
        cycle_frequency,
        min_out_amount: min_out_amount
            .map(|v| checked_u64_to_i64(v, "min_out_amount"))
            .transpose()?,
        max_out_amount: max_out_amount
            .map(|v| checked_u64_to_i64(v, "max_out_amount"))
            .transpose()?,
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

    fn account(pubkey: &str, name: Option<&str>) -> AccountInfo {
        AccountInfo {
            pubkey: pubkey.to_string(),
            is_signer: false,
            is_writable: false,
            name: name.map(str::to_string),
        }
    }

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
    fn malformed_known_event_returns_error() {
        let fields = serde_json::json!({
            "FilledEvent": {
                "dca_key": "pda",
                "in_amount": "bad",
                "out_amount": 1_u64
            }
        });
        let result = resolve_event_envelope(&fields).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_fill_event_rejects_amount_overflow() {
        let fields = serde_json::json!({
            "FilledEvent": {
                "dca_key": "pda",
                "in_amount": (i64::MAX as u64) + 1,
                "out_amount": 1_u64
            }
        });
        let result = resolve_event_envelope(&fields).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn parse_create_args_rejects_overflow_amounts() {
        let args = serde_json::json!({
            "in_amount": (i64::MAX as u64) + 1,
            "in_amount_per_cycle": 1_u64,
            "cycle_frequency": 60_i64,
            "min_out_amount": 1_u64,
            "max_out_amount": 1_u64
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_accepts_valid_payload() {
        let args = serde_json::json!({
            "in_amount": 1_000_u64,
            "in_amount_per_cycle": 100_u64,
            "cycle_frequency": 60_i64,
            "min_out_amount": 10_u64,
            "max_out_amount": 500_u64,
            "start_at": 1_700_000_000_i64
        });
        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.in_amount, 1_000);
        assert_eq!(parsed.in_amount_per_cycle, 100);
        assert_eq!(parsed.cycle_frequency, 60);
        assert_eq!(parsed.min_out_amount, Some(10));
        assert_eq!(parsed.max_out_amount, Some(500));
        assert_eq!(parsed.start_at, Some(1_700_000_000));
    }

    #[test]
    fn parse_create_args_rejects_malformed_payload() {
        let args = serde_json::json!({
            "in_amount": "bad",
            "in_amount_per_cycle": 100_u64,
            "cycle_frequency": 60_i64
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn extract_order_pda_prefers_named_account() {
        let accounts = vec![
            account("idx0", None),
            account("idx1", None),
            account("named_dca", Some("dca")),
        ];
        let extracted = extract_order_pda(&accounts, "CloseDca").unwrap();
        assert_eq!(extracted, "named_dca");
    }

    #[test]
    fn extract_order_pda_uses_instruction_fallback_indexes() {
        let open_accounts = vec![account("open_idx0", None)];
        assert_eq!(
            extract_order_pda(&open_accounts, "OpenDca").unwrap(),
            "open_idx0"
        );

        let close_accounts = vec![account("ignore0", None), account("close_idx1", None)];
        assert_eq!(
            extract_order_pda(&close_accounts, "CloseDca").unwrap(),
            "close_idx1"
        );
    }

    #[test]
    fn extract_order_pda_rejects_unknown_instruction() {
        let err = extract_order_pda(&[account("a", None)], "Unknown").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "unknown DCA instruction: Unknown");
    }

    #[test]
    fn extract_order_pda_rejects_out_of_bounds_fallback() {
        let err = extract_order_pda(&[account("only0", None)], "CloseDca").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "DCA account index 1 out of bounds for CloseDca");
    }

    #[test]
    fn extract_create_mints_prefers_named_accounts() {
        let accounts = vec![
            account("fallback_input", None),
            account("fallback_output", None),
            account("named_input", Some("input_mint")),
            account("named_output", Some("output_mint")),
        ];
        let extracted = extract_create_mints(&accounts, "OpenDca").unwrap();
        assert_eq!(extracted.input_mint, "named_input");
        assert_eq!(extracted.output_mint, "named_output");
    }

    #[test]
    fn extract_create_mints_uses_fallback_indexes_for_create_variants() {
        let open_accounts = vec![
            account("0", None),
            account("1", None),
            account("open_input", None),
            account("open_output", None),
        ];
        let open = extract_create_mints(&open_accounts, "OpenDca").unwrap();
        assert_eq!(open.input_mint, "open_input");
        assert_eq!(open.output_mint, "open_output");

        let open_v2_accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("open_v2_input", None),
            account("open_v2_output", None),
        ];
        let open_v2 = extract_create_mints(&open_v2_accounts, "OpenDcaV2").unwrap();
        assert_eq!(open_v2.input_mint, "open_v2_input");
        assert_eq!(open_v2.output_mint, "open_v2_output");
    }

    #[test]
    fn extract_create_mints_rejects_non_create_instruction() {
        let err = extract_create_mints(&[], "CloseDca")
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "not a DCA create instruction: CloseDca");
    }

    #[test]
    fn extract_create_mints_rejects_missing_fallback_input_index() {
        let err = extract_create_mints(&[], "OpenDca")
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "DCA input_mint index 2 out of bounds");
    }

    #[test]
    fn extract_create_mints_rejects_missing_fallback_output_index() {
        let accounts = vec![account("0", None), account("1", None), account("2", None)];
        let err = extract_create_mints(&accounts, "OpenDca")
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "DCA output_mint index 3 out of bounds");
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
