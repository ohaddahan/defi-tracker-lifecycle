use crate::error::Error;
use crate::lifecycle::adapters::{CorrelationOutcome, EventPayload, ProtocolAdapter};
use crate::protocols::{
    AccountInfo, EventType, Protocol, checked_u16_to_i16, checked_u64_to_i64,
    contains_known_variant, find_account_by_name,
};
use crate::types::{RawEvent, RawInstruction, ResolveContext};
use strum::VariantNames;

#[derive(serde::Deserialize, strum_macros::VariantNames)]
pub enum LimitV2EventEnvelope {
    CreateOrderEvent(OrderKeyHolder),
    CancelOrderEvent(OrderKeyHolder),
    TradeEvent(TradeEventFields),
}

#[derive(serde::Deserialize)]
pub enum LimitV2InstructionKind {
    InitializeOrder(serde_json::Value),
    PreFlashFillOrder(serde_json::Value),
    FlashFillOrder(serde_json::Value),
    CancelOrder(serde_json::Value),
    UpdateFee(serde_json::Value),
    WithdrawFee(serde_json::Value),
}

#[derive(Debug)]
pub struct LimitV2Adapter;

impl ProtocolAdapter for LimitV2Adapter {
    fn protocol(&self) -> Protocol {
        Protocol::LimitV2
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        classify_instruction_envelope(ix)
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        resolve_event_envelope(fields)
    }
}

pub fn classify_instruction_envelope(ix: &RawInstruction) -> Option<EventType> {
    let wrapper = serde_json::json!({ &ix.instruction_name: ix.args });
    let kind: LimitV2InstructionKind = serde_json::from_value(wrapper).ok()?;
    match kind {
        LimitV2InstructionKind::InitializeOrder(_) => Some(EventType::Created),
        LimitV2InstructionKind::PreFlashFillOrder(_) => Some(EventType::FillInitiated),
        LimitV2InstructionKind::FlashFillOrder(_) => Some(EventType::FillCompleted),
        LimitV2InstructionKind::CancelOrder(_) => Some(EventType::Cancelled),
        LimitV2InstructionKind::UpdateFee(_) | LimitV2InstructionKind::WithdrawFee(_) => None,
    }
}

pub fn resolve_event_envelope(
    fields: &serde_json::Value,
) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
    let envelope: LimitV2EventEnvelope = match serde_json::from_value(fields.clone()) {
        Ok(e) => e,
        Err(err) => {
            if !contains_known_variant(fields, LimitV2EventEnvelope::VARIANTS) {
                return None;
            }
            return Some(Err(Error::Protocol {
                reason: format!("failed to parse Limit v2 event payload: {err}"),
            }));
        }
    };

    Some(resolve_limit_v2_event(envelope))
}

fn resolve_limit_v2_event(
    envelope: LimitV2EventEnvelope,
) -> Result<(EventType, CorrelationOutcome, EventPayload), Error> {
    match envelope {
        LimitV2EventEnvelope::CreateOrderEvent(OrderKeyHolder { order_key }) => Ok((
            EventType::Created,
            CorrelationOutcome::Correlated(vec![order_key]),
            EventPayload::None,
        )),
        LimitV2EventEnvelope::CancelOrderEvent(OrderKeyHolder { order_key }) => Ok((
            EventType::Cancelled,
            CorrelationOutcome::Correlated(vec![order_key]),
            EventPayload::None,
        )),
        LimitV2EventEnvelope::TradeEvent(TradeEventFields {
            order_key,
            taker,
            making_amount,
            taking_amount,
            remaining_making_amount,
            ..
        }) => Ok((
            EventType::FillCompleted,
            CorrelationOutcome::Correlated(vec![order_key]),
            EventPayload::LimitFill {
                in_amount: checked_u64_to_i64(making_amount, "making_amount")?,
                out_amount: checked_u64_to_i64(taking_amount, "taking_amount")?,
                remaining_in_amount: checked_u64_to_i64(
                    remaining_making_amount,
                    "remaining_making_amount",
                )?,
                counterparty: taker,
            },
        )),
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
    let kind: LimitV2InstructionKind =
        serde_json::from_value(wrapper).map_err(|_| Error::Protocol {
            reason: format!("unknown Limit v2 instruction: {instruction_name}"),
        })?;

    let idx = match kind {
        LimitV2InstructionKind::InitializeOrder(_) => 2,
        LimitV2InstructionKind::FlashFillOrder(_) | LimitV2InstructionKind::CancelOrder(_) => 2,
        LimitV2InstructionKind::PreFlashFillOrder(_) => 1,
        LimitV2InstructionKind::UpdateFee(_) | LimitV2InstructionKind::WithdrawFee(_) => {
            return Err(Error::Protocol {
                reason: format!("Limit v2 instruction {instruction_name} has no order PDA"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("Limit v2 account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct LimitV2CreateArgs {
    pub unique_id: Option<i64>,
    pub making_amount: i64,
    pub taking_amount: i64,
    pub expired_at: Option<i64>,
    pub fee_bps: Option<i16>,
}

pub struct LimitV2CreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<LimitV2CreateMints, Error> {
    let by_name_input = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let by_name_output = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
        return Ok(LimitV2CreateMints {
            input_mint,
            output_mint,
        });
    }

    let input_mint = accounts
        .get(7)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Limit v2 input_mint index 7 out of bounds".into(),
        })?;
    let output_mint = accounts
        .get(8)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Limit v2 output_mint index 8 out of bounds".into(),
        })?;

    Ok(LimitV2CreateMints {
        input_mint,
        output_mint,
    })
}

#[derive(serde::Deserialize)]
pub struct OrderKeyHolder {
    order_key: String,
}

#[derive(serde::Deserialize)]
pub struct TradeEventFields {
    order_key: String,
    #[serde(default = "default_unknown")]
    taker: String,
    making_amount: u64,
    taking_amount: u64,
    remaining_making_amount: u64,
    #[expect(dead_code, reason = "consumed by serde for completeness")]
    remaining_taking_amount: u64,
}

fn default_unknown() -> String {
    "unknown".to_string()
}

pub struct LimitV2TradeEvent {
    pub order_pda: String,
    pub taker: String,
    pub in_amount: i64,
    pub out_amount: i64,
    pub remaining_in_amount: i64,
    pub remaining_out_amount: i64,
}

#[derive(serde::Deserialize)]
struct InitializeOrderParamsFields {
    #[serde(default)]
    unique_id: Option<u64>,
    making_amount: u64,
    taking_amount: u64,
    expired_at: Option<i64>,
    #[serde(default)]
    fee_bps: Option<u16>,
}

#[derive(serde::Deserialize)]
struct InitializeOrderWrapper {
    params: InitializeOrderParamsFields,
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<LimitV2CreateArgs, Error> {
    let params = if let Ok(wrapper) = serde_json::from_value::<InitializeOrderWrapper>(args.clone())
    {
        wrapper.params
    } else {
        serde_json::from_value::<InitializeOrderParamsFields>(args.clone()).map_err(|e| {
            Error::Protocol {
                reason: format!("failed to parse Limit v2 create args: {e}"),
            }
        })?
    };

    let InitializeOrderParamsFields {
        unique_id,
        making_amount,
        taking_amount,
        expired_at,
        fee_bps,
    } = params;

    Ok(LimitV2CreateArgs {
        unique_id: unique_id
            .map(|v| checked_u64_to_i64(v, "unique_id"))
            .transpose()?,
        making_amount: checked_u64_to_i64(making_amount, "making_amount")?,
        taking_amount: checked_u64_to_i64(taking_amount, "taking_amount")?,
        expired_at,
        fee_bps: fee_bps
            .map(|v| checked_u16_to_i16(v, "fee_bps"))
            .transpose()?,
    })
}

#[cfg(test)]
pub fn classify_decoded(
    decoded: &carbon_jupiter_limit_order_2_decoder::instructions::JupiterLimitOrder2Instruction,
) -> Option<EventType> {
    use carbon_jupiter_limit_order_2_decoder::instructions::JupiterLimitOrder2Instruction;
    match decoded {
        JupiterLimitOrder2Instruction::InitializeOrder(_) => Some(EventType::Created),
        JupiterLimitOrder2Instruction::PreFlashFillOrder(_) => Some(EventType::FillInitiated),
        JupiterLimitOrder2Instruction::FlashFillOrder(_) => Some(EventType::FillCompleted),
        JupiterLimitOrder2Instruction::CancelOrder(_) => Some(EventType::Cancelled),
        JupiterLimitOrder2Instruction::CreateOrderEvent(_) => Some(EventType::Created),
        JupiterLimitOrder2Instruction::CancelOrderEvent(_) => Some(EventType::Cancelled),
        JupiterLimitOrder2Instruction::TradeEvent(_) => Some(EventType::FillCompleted),
        JupiterLimitOrder2Instruction::UpdateFee(_)
        | JupiterLimitOrder2Instruction::WithdrawFee(_) => None,
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
            ("InitializeOrder", Some(EventType::Created)),
            ("PreFlashFillOrder", Some(EventType::FillInitiated)),
            ("FlashFillOrder", Some(EventType::FillCompleted)),
            ("CancelOrder", Some(EventType::Cancelled)),
            ("UpdateFee", None),
            ("WithdrawFee", None),
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
    fn resolve_trade_event_from_envelope() {
        let fields = serde_json::json!({
            "TradeEvent": {
                "order_key": "HkLZgYy93cEi3Fn96SvdWeJk8DNeHeU5wiNV5SeRLiJC",
                "taker": "j1oeQoPeuEDmjvyMwBmCWexzCQup77kbKKxV59CnYbd",
                "making_amount": 724_773_829_u64,
                "taking_amount": 51_821_329_u64,
                "remaining_making_amount": 89_147_181_051_u64,
                "remaining_taking_amount": 6_374_023_074_u64
            }
        });
        let (event_type, correlation, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::FillCompleted);
        let CorrelationOutcome::Correlated(pdas) = correlation else {
            panic!("expected Correlated");
        };
        assert_eq!(pdas, vec!["HkLZgYy93cEi3Fn96SvdWeJk8DNeHeU5wiNV5SeRLiJC"]);
        let EventPayload::LimitFill {
            in_amount,
            out_amount,
            remaining_in_amount,
            counterparty,
        } = payload
        else {
            panic!("expected LimitFill");
        };
        assert_eq!(in_amount, 724_773_829);
        assert_eq!(out_amount, 51_821_329);
        assert_eq!(remaining_in_amount, 89_147_181_051);
        assert_eq!(counterparty, "j1oeQoPeuEDmjvyMwBmCWexzCQup77kbKKxV59CnYbd");
    }

    #[test]
    fn malformed_known_event_returns_error() {
        let fields = serde_json::json!({
            "TradeEvent": {
                "order_key": "order",
                "making_amount": "bad",
                "taking_amount": 1_u64,
                "remaining_making_amount": 0_u64,
                "remaining_taking_amount": 0_u64
            }
        });
        let result = resolve_event_envelope(&fields).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_trade_event_rejects_amount_overflow() {
        let fields = serde_json::json!({
            "TradeEvent": {
                "order_key": "order",
                "making_amount": (i64::MAX as u64) + 1,
                "taking_amount": 1_u64,
                "remaining_making_amount": 0_u64,
                "remaining_taking_amount": 0_u64
            }
        });
        let result = resolve_event_envelope(&fields).unwrap();
        assert!(result.is_err());
    }

    #[test]
    fn resolve_trade_event_defaults_missing_taker() {
        let fields = serde_json::json!({
            "TradeEvent": {
                "order_key": "order",
                "making_amount": 10_u64,
                "taking_amount": 5_u64,
                "remaining_making_amount": 1_u64,
                "remaining_taking_amount": 0_u64
            }
        });
        let (_, _, payload) = resolve_event_envelope(&fields).unwrap().unwrap();
        let EventPayload::LimitFill { counterparty, .. } = payload else {
            panic!("expected LimitFill");
        };
        assert_eq!(counterparty, "unknown");
    }

    #[test]
    fn unknown_event_returns_none() {
        let fields = serde_json::json!({"UnknownEvent": {"some_field": 1}});
        assert!(resolve_event_envelope(&fields).is_none());
    }

    #[test]
    fn parse_create_args_with_params_wrapper() {
        let args = serde_json::json!({
            "params": {
                "making_amount": 1000_u64,
                "taking_amount": 500_u64,
                "unique_id": 42_u64,
                "expired_at": 1_700_000_000_i64,
                "fee_bps": 25_u16
            }
        });
        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.making_amount, 1000);
        assert_eq!(parsed.taking_amount, 500);
        assert_eq!(parsed.unique_id, Some(42));
        assert_eq!(parsed.expired_at, Some(1_700_000_000));
        assert_eq!(parsed.fee_bps, Some(25));
    }

    #[test]
    fn parse_create_args_without_params_wrapper() {
        let args = serde_json::json!({
            "making_amount": 2000_u64,
            "taking_amount": 1000_u64
        });
        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.making_amount, 2000);
        assert_eq!(parsed.taking_amount, 1000);
        assert_eq!(parsed.unique_id, None);
        assert_eq!(parsed.expired_at, None);
        assert_eq!(parsed.fee_bps, None);
    }

    #[test]
    fn parse_create_args_rejects_overflow_values() {
        let args = serde_json::json!({
            "making_amount": (i64::MAX as u64) + 1,
            "taking_amount": 1_u64,
            "unique_id": (i64::MAX as u64) + 1
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_rejects_fee_bps_out_of_range() {
        let args = serde_json::json!({
            "making_amount": 1_u64,
            "taking_amount": 1_u64,
            "fee_bps": 65_535_u16
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_rejects_malformed_payload() {
        let args = serde_json::json!({
            "making_amount": "bad",
            "taking_amount": 1_u64
        });
        assert!(parse_create_args(&args).is_err());
    }

    #[test]
    fn extract_order_pda_prefers_named_account() {
        let accounts = vec![
            account("idx1", None),
            account("idx2", None),
            account("named_order", Some("order")),
        ];
        let extracted = extract_order_pda(&accounts, "CancelOrder").unwrap();
        assert_eq!(extracted, "named_order");
    }

    #[test]
    fn extract_order_pda_uses_fallback_indexes() {
        let init_accounts = vec![
            account("0", None),
            account("1", None),
            account("init_idx2", None),
        ];
        assert_eq!(
            extract_order_pda(&init_accounts, "InitializeOrder").unwrap(),
            "init_idx2"
        );

        let pre_flash_accounts = vec![account("0", None), account("pre_flash_idx1", None)];
        assert_eq!(
            extract_order_pda(&pre_flash_accounts, "PreFlashFillOrder").unwrap(),
            "pre_flash_idx1"
        );
    }

    #[test]
    fn extract_order_pda_rejects_unknown_instruction() {
        let err = extract_order_pda(&[], "Unknown").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "unknown Limit v2 instruction: Unknown");
    }

    #[test]
    fn extract_order_pda_rejects_out_of_bounds_index() {
        let err = extract_order_pda(&[], "CancelOrder").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(
            reason,
            "Limit v2 account index 2 out of bounds for CancelOrder"
        );
    }

    #[test]
    fn extract_create_mints_prefers_named_accounts() {
        let accounts = vec![
            account("idx7", None),
            account("idx8", None),
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
            account("4", None),
            account("5", None),
            account("6", None),
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
        assert_eq!(reason, "Limit v2 input_mint index 7 out of bounds");
    }

    #[test]
    fn extract_create_mints_rejects_missing_output_fallback_index() {
        let accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("3", None),
            account("4", None),
            account("5", None),
            account("6", None),
            account("fallback_input", None),
        ];
        let err = extract_create_mints(&accounts)
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Limit v2 output_mint index 8 out of bounds");
    }

    #[test]
    fn mirror_enums_cover_all_carbon_variants() {
        let instruction_variants = [
            "InitializeOrder",
            "PreFlashFillOrder",
            "FlashFillOrder",
            "CancelOrder",
            "UpdateFee",
            "WithdrawFee",
        ];
        for name in instruction_variants {
            let json = serde_json::json!({ name: serde_json::Value::Null });
            assert!(
                serde_json::from_value::<LimitV2InstructionKind>(json).is_ok(),
                "LimitV2InstructionKind missing variant: {name}"
            );
        }

        for name in ["CreateOrderEvent", "CancelOrderEvent"] {
            let json = serde_json::json!({ name: { "order_key": "test" } });
            assert!(
                serde_json::from_value::<LimitV2EventEnvelope>(json).is_ok(),
                "LimitV2EventEnvelope missing variant: {name}"
            );
        }

        let trade = serde_json::json!({
            "TradeEvent": { "order_key": "t", "making_amount": 1_u64, "taking_amount": 1_u64,
                "remaining_making_amount": 0_u64, "remaining_taking_amount": 0_u64 }
        });
        assert!(serde_json::from_value::<LimitV2EventEnvelope>(trade).is_ok());
    }
}
