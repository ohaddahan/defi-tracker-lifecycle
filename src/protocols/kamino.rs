use crate::error::Error;
use crate::lifecycle::adapters::{
    CorrelationOutcome, EventPayload, kamino_display_terminal_status,
};
use crate::protocols::{
    AccountInfo, EventType, checked_u64_to_i64, contains_known_variant, find_account_by_name,
};
use crate::types::{RawInstruction, ResolveContext};
use strum::VariantNames;

#[derive(serde::Deserialize, strum_macros::VariantNames)]
pub enum KaminoEventEnvelope {
    OrderDisplayEvent(OrderDisplayEventFields),
    UserSwapBalancesEvent(serde_json::Value),
}

#[derive(serde::Deserialize)]
pub enum KaminoInstructionKind {
    CreateOrder(serde_json::Value),
    TakeOrder(serde_json::Value),
    FlashTakeOrderStart(serde_json::Value),
    FlashTakeOrderEnd(serde_json::Value),
    CloseOrderAndClaimTip(serde_json::Value),
    InitializeGlobalConfig(serde_json::Value),
    InitializeVault(serde_json::Value),
    UpdateGlobalConfig(serde_json::Value),
    UpdateGlobalConfigAdmin(serde_json::Value),
    WithdrawHostTip(serde_json::Value),
    LogUserSwapBalances(serde_json::Value),
}

pub fn classify_instruction_envelope(ix: &RawInstruction) -> Option<EventType> {
    let wrapper = serde_json::json!({ &ix.instruction_name: ix.args });
    let kind: KaminoInstructionKind = serde_json::from_value(wrapper).ok()?;
    match kind {
        KaminoInstructionKind::CreateOrder(_) => Some(EventType::Created),
        KaminoInstructionKind::TakeOrder(_) => Some(EventType::FillCompleted),
        KaminoInstructionKind::FlashTakeOrderStart(_) => Some(EventType::FillInitiated),
        KaminoInstructionKind::FlashTakeOrderEnd(_) => Some(EventType::FillCompleted),
        KaminoInstructionKind::CloseOrderAndClaimTip(_) => Some(EventType::Closed),
        KaminoInstructionKind::InitializeGlobalConfig(_)
        | KaminoInstructionKind::InitializeVault(_)
        | KaminoInstructionKind::UpdateGlobalConfig(_)
        | KaminoInstructionKind::UpdateGlobalConfigAdmin(_)
        | KaminoInstructionKind::WithdrawHostTip(_)
        | KaminoInstructionKind::LogUserSwapBalances(_) => None,
    }
}

pub fn resolve_event_envelope(
    fields: &serde_json::Value,
    signature: &str,
    ctx: &ResolveContext,
) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
    let envelope: KaminoEventEnvelope = match serde_json::from_value(fields.clone()) {
        Ok(e) => e,
        Err(err) => {
            if !contains_known_variant(fields, KaminoEventEnvelope::VARIANTS) {
                return None;
            }
            return Some(Err(Error::Protocol {
                reason: format!("failed to parse Kamino event payload: {err}"),
            }));
        }
    };

    Some(resolve_kamino_event(envelope, signature, ctx))
}

fn resolve_kamino_event(
    envelope: KaminoEventEnvelope,
    signature: &str,
    ctx: &ResolveContext,
) -> Result<(EventType, CorrelationOutcome, EventPayload), Error> {
    match envelope {
        KaminoEventEnvelope::UserSwapBalancesEvent(_) => Ok((
            EventType::FillCompleted,
            CorrelationOutcome::NotRequired,
            EventPayload::None,
        )),
        KaminoEventEnvelope::OrderDisplayEvent(display_fields) => {
            let order_pdas = ctx.pre_fetched_order_pdas.clone().unwrap_or_default();

            if order_pdas.is_empty() {
                return Ok((
                    EventType::FillCompleted,
                    CorrelationOutcome::Uncorrelated {
                        reason: format!(
                            "cannot correlate Kamino OrderDisplayEvent for signature {signature}"
                        ),
                    },
                    EventPayload::None,
                ));
            }

            let terminal_status = kamino_display_terminal_status(i64::from(display_fields.status))?;
            Ok((
                EventType::FillCompleted,
                CorrelationOutcome::Correlated(order_pdas),
                EventPayload::KaminoDisplay {
                    remaining_input_amount: checked_u64_to_i64(
                        display_fields.remaining_input_amount,
                        "remaining_input_amount",
                    )?,
                    filled_output_amount: checked_u64_to_i64(
                        display_fields.filled_output_amount,
                        "filled_output_amount",
                    )?,
                    terminal_status,
                },
            ))
        }
    }
}

pub fn extract_order_pda(
    accounts: &[AccountInfo],
    instruction_name: &str,
) -> Result<String, Error> {
    if let Some(acc) = find_account_by_name(accounts, "order") {
        return Ok(acc.pubkey.clone());
    }

    let wrapper = serde_json::json!({ instruction_name: serde_json::Value::Null });
    let kind: KaminoInstructionKind =
        serde_json::from_value(wrapper).map_err(|_| Error::Protocol {
            reason: format!("unknown Kamino instruction: {instruction_name}"),
        })?;

    let idx = match kind {
        KaminoInstructionKind::CreateOrder(_) => 3,
        KaminoInstructionKind::TakeOrder(_) => 4,
        KaminoInstructionKind::FlashTakeOrderStart(_)
        | KaminoInstructionKind::FlashTakeOrderEnd(_) => 4,
        KaminoInstructionKind::CloseOrderAndClaimTip(_) => 1,
        KaminoInstructionKind::InitializeGlobalConfig(_)
        | KaminoInstructionKind::InitializeVault(_)
        | KaminoInstructionKind::UpdateGlobalConfig(_)
        | KaminoInstructionKind::UpdateGlobalConfigAdmin(_)
        | KaminoInstructionKind::WithdrawHostTip(_)
        | KaminoInstructionKind::LogUserSwapBalances(_) => {
            return Err(Error::Protocol {
                reason: format!("Kamino instruction {instruction_name} has no order PDA"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("Kamino account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct KaminoCreateArgs {
    pub input_amount: i64,
    pub output_amount: i64,
    pub order_type: i16,
}

pub struct KaminoCreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<KaminoCreateMints, Error> {
    let by_name_input = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let by_name_output = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
        return Ok(KaminoCreateMints {
            input_mint,
            output_mint,
        });
    }

    let input_mint = accounts
        .get(4)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Kamino input_mint index 4 out of bounds".into(),
        })?;
    let output_mint = accounts
        .get(5)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Kamino output_mint index 5 out of bounds".into(),
        })?;

    Ok(KaminoCreateMints {
        input_mint,
        output_mint,
    })
}

#[derive(serde::Deserialize)]
struct CreateOrderFields {
    input_amount: u64,
    output_amount: u64,
    #[serde(default)]
    order_type: u8,
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<KaminoCreateArgs, Error> {
    let CreateOrderFields {
        input_amount,
        output_amount,
        order_type,
    } = serde_json::from_value(args.clone()).map_err(|e| Error::Protocol {
        reason: format!("failed to parse Kamino create args: {e}"),
    })?;

    Ok(KaminoCreateArgs {
        input_amount: checked_u64_to_i64(input_amount, "input_amount")?,
        output_amount: checked_u64_to_i64(output_amount, "output_amount")?,
        order_type: i16::from(order_type),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KaminoDisplayStatus {
    Open,
    Filled,
    Cancelled,
    Expired,
}

pub fn parse_display_status(status: i64) -> Result<KaminoDisplayStatus, Error> {
    match status {
        0 => Ok(KaminoDisplayStatus::Open),
        1 => Ok(KaminoDisplayStatus::Filled),
        2 => Ok(KaminoDisplayStatus::Cancelled),
        3 => Ok(KaminoDisplayStatus::Expired),
        _ => Err(Error::Protocol {
            reason: format!("unknown Kamino display status code: {status}"),
        }),
    }
}

#[derive(serde::Deserialize)]
pub struct OrderDisplayEventFields {
    #[serde(default)]
    pub remaining_input_amount: u64,
    #[serde(default)]
    pub filled_output_amount: u64,
    #[serde(default)]
    pub number_of_fills: u64,
    #[serde(default)]
    pub status: u8,
}

pub struct KaminoOrderDisplayEvent {
    pub remaining_input_amount: i64,
    pub filled_output_amount: i64,
    pub number_of_fills: i64,
    pub status: i64,
}

#[cfg(test)]
pub fn classify_decoded(
    decoded: &carbon_kamino_limit_order_decoder::instructions::KaminoLimitOrderInstruction,
) -> Option<EventType> {
    use carbon_kamino_limit_order_decoder::instructions::KaminoLimitOrderInstruction;
    match decoded {
        KaminoLimitOrderInstruction::CreateOrder(_) => Some(EventType::Created),
        KaminoLimitOrderInstruction::TakeOrder(_) => Some(EventType::FillCompleted),
        KaminoLimitOrderInstruction::FlashTakeOrderStart(_) => Some(EventType::FillInitiated),
        KaminoLimitOrderInstruction::FlashTakeOrderEnd(_) => Some(EventType::FillCompleted),
        KaminoLimitOrderInstruction::CloseOrderAndClaimTip(_) => Some(EventType::Closed),
        KaminoLimitOrderInstruction::OrderDisplayEvent(_) => Some(EventType::FillCompleted),
        KaminoLimitOrderInstruction::InitializeGlobalConfig(_)
        | KaminoLimitOrderInstruction::InitializeVault(_)
        | KaminoLimitOrderInstruction::UpdateGlobalConfig(_)
        | KaminoLimitOrderInstruction::UpdateGlobalConfigAdmin(_)
        | KaminoLimitOrderInstruction::WithdrawHostTip(_)
        | KaminoLimitOrderInstruction::LogUserSwapBalances(_)
        | KaminoLimitOrderInstruction::UserSwapBalancesEvent(_) => None,
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

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
            ("CreateOrder", Some(EventType::Created)),
            ("TakeOrder", Some(EventType::FillCompleted)),
            ("FlashTakeOrderStart", Some(EventType::FillInitiated)),
            ("FlashTakeOrderEnd", Some(EventType::FillCompleted)),
            ("CloseOrderAndClaimTip", Some(EventType::Closed)),
            ("InitializeGlobalConfig", None),
            ("InitializeVault", None),
            ("UpdateGlobalConfig", None),
            ("UpdateGlobalConfigAdmin", None),
            ("WithdrawHostTip", None),
            ("LogUserSwapBalances", None),
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
    fn resolve_display_event_with_pdas() {
        let fields = serde_json::json!({
            "OrderDisplayEvent": {
                "remaining_input_amount": 0_u64,
                "filled_output_amount": 11_744_711_u64,
                "number_of_fills": 1_u64,
                "status": 1_u8
            }
        });
        let ctx = ResolveContext {
            pre_fetched_order_pdas: Some(vec!["pda1".to_string()]),
        };
        let (event_type, correlation, payload) = resolve_event_envelope(&fields, "sig", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(event_type, EventType::FillCompleted);
        assert_eq!(
            correlation,
            CorrelationOutcome::Correlated(vec!["pda1".to_string()])
        );
        let EventPayload::KaminoDisplay {
            remaining_input_amount,
            filled_output_amount,
            terminal_status,
        } = payload
        else {
            panic!("expected KaminoDisplay");
        };
        assert_eq!(remaining_input_amount, 0);
        assert_eq!(filled_output_amount, 11_744_711);
        assert!(terminal_status.is_some());
    }

    #[test]
    fn resolve_display_event_without_pdas() {
        let fields = serde_json::json!({
            "OrderDisplayEvent": {
                "remaining_input_amount": 0_u64,
                "filled_output_amount": 100_u64,
                "number_of_fills": 1_u64,
                "status": 1_u8
            }
        });
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };
        let (_, correlation, payload) = resolve_event_envelope(&fields, "sig", &ctx)
            .unwrap()
            .unwrap();
        assert!(matches!(
            correlation,
            CorrelationOutcome::Uncorrelated { .. }
        ));
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn flash_take_instructions_are_classified() {
        assert_eq!(
            classify_instruction_envelope(&RawInstruction {
                id: 1,
                signature: "s".to_string(),
                instruction_index: 0,
                program_id: "p".to_string(),
                inner_program_id: "p".to_string(),
                instruction_name: "FlashTakeOrderStart".to_string(),
                accounts: None,
                args: None,
                slot: 1,
            }),
            Some(EventType::FillInitiated)
        );
        assert_eq!(
            classify_instruction_envelope(&RawInstruction {
                id: 1,
                signature: "s".to_string(),
                instruction_index: 0,
                program_id: "p".to_string(),
                inner_program_id: "p".to_string(),
                instruction_name: "FlashTakeOrderEnd".to_string(),
                accounts: None,
                args: None,
                slot: 1,
            }),
            Some(EventType::FillCompleted)
        );
    }

    #[test]
    fn extract_order_pda_supports_flash_take_fallback_index() {
        let order = "7ee61prCHrJwHNYJKojPu1GcyWDjQEUTeuRT4gbA1EDo";
        let accounts = vec![
            AccountInfo {
                pubkey: "a".to_string(),
                is_signer: true,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "b".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "c".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "d".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: order.to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
        ];

        let extracted = extract_order_pda(&accounts, "FlashTakeOrderEnd").unwrap();
        assert_eq!(extracted, order);
    }

    #[test]
    fn extract_order_pda_prefers_named_account() {
        let accounts = vec![
            account("idx3", None),
            account("idx4", None),
            account("named_order", Some("order")),
        ];
        let extracted = extract_order_pda(&accounts, "TakeOrder").unwrap();
        assert_eq!(extracted, "named_order");
    }

    #[test]
    fn extract_order_pda_supports_all_known_fallback_indexes() {
        let create_accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("create_idx3", None),
        ];
        assert_eq!(
            extract_order_pda(&create_accounts, "CreateOrder").unwrap(),
            "create_idx3"
        );

        let take_accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("3", None),
            account("take_idx4", None),
        ];
        assert_eq!(
            extract_order_pda(&take_accounts, "TakeOrder").unwrap(),
            "take_idx4"
        );

        let close_accounts = vec![account("0", None), account("close_idx1", None)];
        assert_eq!(
            extract_order_pda(&close_accounts, "CloseOrderAndClaimTip").unwrap(),
            "close_idx1"
        );
    }

    #[test]
    fn extract_order_pda_rejects_unknown_instruction() {
        let err = extract_order_pda(&[], "Unknown").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "unknown Kamino instruction: Unknown");
    }

    #[test]
    fn extract_order_pda_rejects_out_of_bounds_index() {
        let err = extract_order_pda(&[], "TakeOrder").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Kamino account index 4 out of bounds for TakeOrder");
    }

    #[test]
    fn extract_create_mints_prefers_named_accounts() {
        let accounts = vec![
            account("idx4", None),
            account("idx5", None),
            account("named_input", Some("input_mint")),
            account("named_output", Some("output_mint")),
        ];
        let extracted = extract_create_mints(&accounts).unwrap();
        assert_eq!(extracted.input_mint, "named_input");
        assert_eq!(extracted.output_mint, "named_output");
    }

    #[test]
    fn extract_create_mints_uses_fallback_indexes() {
        let accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("3", None),
            account("fallback_input", None),
            account("fallback_output", None),
        ];
        let extracted = extract_create_mints(&accounts).unwrap();
        assert_eq!(extracted.input_mint, "fallback_input");
        assert_eq!(extracted.output_mint, "fallback_output");
    }

    #[test]
    fn extract_create_mints_rejects_missing_input_fallback_index() {
        let err = extract_create_mints(&[]).err().expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Kamino input_mint index 4 out of bounds");
    }

    #[test]
    fn extract_create_mints_rejects_missing_output_fallback_index() {
        let accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("3", None),
            account("only_input", None),
        ];
        let err = extract_create_mints(&accounts)
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Kamino output_mint index 5 out of bounds");
    }

    #[test]
    fn parses_known_display_status_codes() {
        assert_eq!(parse_display_status(0).unwrap(), KaminoDisplayStatus::Open);
        assert_eq!(
            parse_display_status(1).unwrap(),
            KaminoDisplayStatus::Filled
        );
        assert_eq!(
            parse_display_status(2).unwrap(),
            KaminoDisplayStatus::Cancelled
        );
        assert_eq!(
            parse_display_status(3).unwrap(),
            KaminoDisplayStatus::Expired
        );
    }

    #[test]
    fn rejects_unknown_display_status_codes() {
        assert!(parse_display_status(99).is_err());
    }

    #[test]
    fn unknown_event_returns_none() {
        let fields = serde_json::json!({"UnknownEvent": {"some_field": 1}});
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };
        assert!(resolve_event_envelope(&fields, "sig", &ctx).is_none());
    }

    #[test]
    fn malformed_known_event_returns_error() {
        let fields = serde_json::json!({
            "OrderDisplayEvent": {
                "remaining_input_amount": "bad",
                "filled_output_amount": 1_u64,
                "number_of_fills": 1_u64,
                "status": 1_u8
            }
        });
        let ctx = ResolveContext {
            pre_fetched_order_pdas: Some(vec!["pda1".to_string()]),
        };
        let result = resolve_event_envelope(&fields, "sig", &ctx).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_display_event_rejects_amount_overflow() {
        let fields = serde_json::json!({
            "OrderDisplayEvent": {
                "remaining_input_amount": (i64::MAX as u64) + 1,
                "filled_output_amount": 1_u64,
                "number_of_fills": 1_u64,
                "status": 1_u8
            }
        });
        let ctx = ResolveContext {
            pre_fetched_order_pdas: Some(vec!["pda1".to_string()]),
        };
        let result = resolve_event_envelope(&fields, "sig", &ctx).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_user_swap_balances_event() {
        let fields = serde_json::json!({
            "UserSwapBalancesEvent": {
                "some_field": 42
            }
        });
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };
        let (event_type, correlation, payload) = resolve_event_envelope(&fields, "sig", &ctx)
            .unwrap()
            .unwrap();
        assert_eq!(event_type, EventType::FillCompleted);
        assert_eq!(correlation, CorrelationOutcome::NotRequired);
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn parse_create_args_rejects_amount_overflow() {
        let args = serde_json::json!({
            "input_amount": (i64::MAX as u64) + 1,
            "output_amount": 1_u64,
            "order_type": 0_u8
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_accepts_valid_payload() {
        let args = serde_json::json!({
            "input_amount": 5_000_u64,
            "output_amount": 4_500_u64,
            "order_type": 2_u8
        });
        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.input_amount, 5_000);
        assert_eq!(parsed.output_amount, 4_500);
        assert_eq!(parsed.order_type, 2);
    }

    #[test]
    fn parse_create_args_rejects_malformed_payload() {
        let args = serde_json::json!({
            "input_amount": "bad",
            "output_amount": 4_500_u64
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn mirror_enums_cover_all_carbon_variants() {
        let instruction_variants = [
            "CreateOrder",
            "TakeOrder",
            "FlashTakeOrderStart",
            "FlashTakeOrderEnd",
            "CloseOrderAndClaimTip",
            "InitializeGlobalConfig",
            "InitializeVault",
            "UpdateGlobalConfig",
            "UpdateGlobalConfigAdmin",
            "WithdrawHostTip",
            "LogUserSwapBalances",
        ];
        for name in instruction_variants {
            let json = serde_json::json!({ name: serde_json::Value::Null });
            assert!(
                serde_json::from_value::<KaminoInstructionKind>(json).is_ok(),
                "KaminoInstructionKind missing variant: {name}"
            );
        }

        let event_variants = ["OrderDisplayEvent", "UserSwapBalancesEvent"];
        for name in event_variants {
            let json = serde_json::json!({ name: {} });
            assert!(
                serde_json::from_value::<KaminoEventEnvelope>(json).is_ok(),
                "KaminoEventEnvelope missing variant: {name}"
            );
        }
    }
}
