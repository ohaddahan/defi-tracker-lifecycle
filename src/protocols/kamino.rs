use crate::error::Error;
use crate::lifecycle::adapters::{
    CorrelationOutcome, EventPayload, kamino_display_terminal_status,
};
use crate::protocols::{AccountInfo, EventType, find_account_by_name};
use crate::types::{RawInstruction, ResolveContext};

#[derive(serde::Deserialize)]
#[expect(
    dead_code,
    reason = "variant data consumed by serde, not read directly"
)]
pub(crate) enum KaminoEventEnvelope {
    OrderDisplayEvent(OrderDisplayEventFields),
    UserSwapBalancesEvent(serde_json::Value),
}

#[derive(serde::Deserialize)]
#[expect(
    dead_code,
    reason = "variant data consumed by serde, not read directly"
)]
pub(crate) enum KaminoInstructionKind {
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
        Err(_) => return None,
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
                    remaining_input_amount: display_fields.remaining_input_amount as i64,
                    filled_output_amount: display_fields.filled_output_amount as i64,
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

    let idx = match instruction_name {
        "CreateOrder" => 3,
        "TakeOrder" => 4,
        "FlashTakeOrderStart" | "FlashTakeOrderEnd" => 4,
        "CloseOrderAndClaimTip" => 1,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown Kamino instruction: {instruction_name}"),
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
        input_amount: input_amount as i64,
        output_amount: output_amount as i64,
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
pub(crate) struct OrderDisplayEventFields {
    #[serde(default)]
    pub remaining_input_amount: u64,
    #[serde(default)]
    pub filled_output_amount: u64,
    #[serde(default)]
    #[expect(dead_code, reason = "consumed by serde for completeness")]
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
