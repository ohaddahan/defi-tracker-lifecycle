use crate::error::Error;
use crate::lifecycle::adapters::{CorrelationOutcome, EventPayload, ProtocolAdapter};
use crate::protocols::{AccountInfo, EventType, Protocol, ProtocolHelpers};
use crate::types::{RawEvent, RawInstruction, ResolveContext};
use strum::VariantNames;

/// Serde-tagged envelope for Jupiter Limit Order v2 event variants.
#[derive(serde::Deserialize, strum_macros::VariantNames)]
pub enum LimitV2EventEnvelope {
    CreateOrderEvent(OrderKeyHolder),
    CancelOrderEvent(OrderKeyHolder),
    TradeEvent(TradeEventFields),
}

/// Serde-tagged envelope for Jupiter Limit Order v2 instruction variants.
#[derive(serde::Deserialize)]
pub enum LimitV2InstructionKind {
    InitializeOrder(serde_json::Value),
    PreFlashFillOrder(serde_json::Value),
    FlashFillOrder(serde_json::Value),
    CancelOrder(serde_json::Value),
    UpdateFee(serde_json::Value),
    WithdrawFee(serde_json::Value),
}

#[cfg(feature = "wasm")]
pub const INSTRUCTION_EVENT_TYPES: &[(&str, EventType)] = &[
    ("InitializeOrder", EventType::Created),
    ("PreFlashFillOrder", EventType::FillInitiated),
    ("FlashFillOrder", EventType::FillCompleted),
    ("CancelOrder", EventType::Cancelled),
];

#[cfg(feature = "wasm")]
pub const EVENT_EVENT_TYPES: &[(&str, EventType)] = &[
    ("CreateOrderEvent", EventType::Created),
    ("CancelOrderEvent", EventType::Cancelled),
    ("TradeEvent", EventType::FillCompleted),
];

#[cfg(feature = "wasm")]
pub const CLOSED_VARIANTS: &[&str] = &[];

/// Jupiter Limit Order v2 protocol adapter (zero-sized, stored as a static).
#[derive(Debug)]
pub struct LimitV2Adapter;

/// Serde intermediate for events that only carry an `order_key`.
#[derive(serde::Deserialize)]
pub struct OrderKeyHolder {
    order_key: String,
}

/// Serde intermediate for `TradeEvent` payload fields (v2 field names).
#[derive(serde::Deserialize)]
pub struct TradeEventFields {
    order_key: String,
    #[serde(default = "LimitV2Adapter::default_unknown")]
    taker: String,
    making_amount: u64,
    taking_amount: u64,
    remaining_making_amount: u64,
    #[expect(dead_code, reason = "consumed by serde for completeness")]
    remaining_taking_amount: u64,
}

/// Extracted Limit v2 trade event with checked-cast amounts.
pub struct LimitV2TradeEvent {
    pub order_pda: String,
    pub taker: String,
    pub in_amount: i64,
    pub out_amount: i64,
    pub remaining_in_amount: i64,
    pub remaining_out_amount: i64,
}

/// Parsed arguments from an `InitializeOrder` instruction (Limit v2).
///
/// `fee_bps` and `unique_id` are v2-specific fields not present in v1.
pub struct LimitV2CreateArgs {
    pub unique_id: Option<i64>,
    pub making_amount: i64,
    pub taking_amount: i64,
    pub expired_at: Option<i64>,
    pub fee_bps: Option<i16>,
}

/// Input and output mint addresses extracted from a Limit v2 create instruction.
pub struct LimitV2CreateMints {
    pub input_mint: String,
    pub output_mint: String,
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

impl ProtocolAdapter for LimitV2Adapter {
    fn protocol(&self) -> Protocol {
        Protocol::LimitV2
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
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

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        let envelope: LimitV2EventEnvelope = match serde_json::from_value(fields.clone()) {
            Ok(e) => e,
            Err(err) => {
                if !ProtocolHelpers::contains_known_variant(fields, LimitV2EventEnvelope::VARIANTS)
                {
                    return None;
                }
                return Some(Err(Error::Protocol {
                    reason: format!("failed to parse Limit v2 event payload: {err}"),
                }));
            }
        };

        Some(Self::resolve_event(envelope))
    }
}

impl LimitV2Adapter {
    fn default_unknown() -> String {
        "unknown".to_string()
    }

    fn resolve_event(
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
                    in_amount: ProtocolHelpers::checked_u64_to_i64(making_amount, "making_amount")?,
                    out_amount: ProtocolHelpers::checked_u64_to_i64(
                        taking_amount,
                        "taking_amount",
                    )?,
                    remaining_in_amount: ProtocolHelpers::checked_u64_to_i64(
                        remaining_making_amount,
                        "remaining_making_amount",
                    )?,
                    counterparty: taker,
                },
            )),
        }
    }

    /// Extracts the order PDA from instruction accounts.
    ///
    /// Prefers the named `"order"` account; falls back to positional index per instruction variant.
    pub fn extract_order_pda(
        accounts: &[AccountInfo],
        instruction_name: &str,
    ) -> Result<String, Error> {
        if let Some(acc) = ProtocolHelpers::find_account_by_name(accounts, "order") {
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
                reason: format!(
                    "Limit v2 account index {idx} out of bounds for {instruction_name}"
                ),
            })
    }

    /// Extracts input/output mint addresses from a Limit v2 create instruction's accounts.
    ///
    /// Prefers named accounts; falls back to positional indexes 7 (input) and 8 (output).
    pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<LimitV2CreateMints, Error> {
        let by_name_input =
            ProtocolHelpers::find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
        let by_name_output = ProtocolHelpers::find_account_by_name(accounts, "output_mint")
            .map(|a| a.pubkey.clone());

        if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
            return Ok(LimitV2CreateMints {
                input_mint,
                output_mint,
            });
        }

        let input_mint =
            accounts
                .get(7)
                .map(|a| a.pubkey.clone())
                .ok_or_else(|| Error::Protocol {
                    reason: "Limit v2 input_mint index 7 out of bounds".into(),
                })?;
        let output_mint =
            accounts
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

    /// Parses `InitializeOrder` instruction args into checked [`LimitV2CreateArgs`].
    ///
    /// Handles both `{"params": {...}}` wrapper format and flat format.
    pub fn parse_create_args(args: &serde_json::Value) -> Result<LimitV2CreateArgs, Error> {
        let params =
            if let Ok(wrapper) = serde_json::from_value::<InitializeOrderWrapper>(args.clone()) {
                wrapper.params
            } else {
                serde_json::from_value::<InitializeOrderParamsFields>(args.clone()).map_err(
                    |e| Error::Protocol {
                        reason: format!("failed to parse Limit v2 create args: {e}"),
                    },
                )?
            };

        let InitializeOrderParamsFields {
            unique_id,
            making_amount,
            taking_amount,
            expired_at,
            fee_bps,
        } = params;

        Ok(LimitV2CreateArgs {
            unique_id: unique_id.and_then(ProtocolHelpers::optional_u64_to_i64),
            making_amount: ProtocolHelpers::checked_u64_to_i64(making_amount, "making_amount")?,
            taking_amount: ProtocolHelpers::checked_u64_to_i64(taking_amount, "taking_amount")?,
            expired_at,
            fee_bps: fee_bps
                .map(|v| ProtocolHelpers::checked_u16_to_i16(v, "fee_bps"))
                .transpose()?,
        })
    }

    #[cfg(all(test, feature = "native"))]
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

    fn make_event(fields: serde_json::Value) -> RawEvent {
        RawEvent {
            id: 1,
            signature: "sig".to_string(),
            event_index: 0,
            program_id: "p".to_string(),
            inner_program_id: "p".to_string(),
            event_name: "test".to_string(),
            fields: Some(fields),
            slot: 1,
        }
    }

    fn resolve(
        fields: serde_json::Value,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), crate::error::Error>> {
        let ev = make_event(fields);
        let ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };
        LimitV2Adapter.classify_and_resolve_event(&ev, &ctx)
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
                LimitV2Adapter.classify_instruction(&ix),
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
        let (event_type, correlation, payload) = resolve(fields).unwrap().unwrap();
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
        let result = resolve(fields).unwrap();
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
        let result = resolve(fields).unwrap();
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
        let (_, _, payload) = resolve(fields).unwrap().unwrap();
        let EventPayload::LimitFill { counterparty, .. } = payload else {
            panic!("expected LimitFill");
        };
        assert_eq!(counterparty, "unknown");
    }

    #[test]
    fn unknown_event_returns_none() {
        let fields = serde_json::json!({"UnknownEvent": {"some_field": 1}});
        assert!(resolve(fields).is_none());
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
        let parsed = LimitV2Adapter::parse_create_args(&args).unwrap();
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
        let parsed = LimitV2Adapter::parse_create_args(&args).unwrap();
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
        assert!(LimitV2Adapter::parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_rejects_fee_bps_out_of_range() {
        let args = serde_json::json!({
            "making_amount": 1_u64,
            "taking_amount": 1_u64,
            "fee_bps": 65_535_u16
        });
        assert!(LimitV2Adapter::parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_rejects_malformed_payload() {
        let args = serde_json::json!({
            "making_amount": "bad",
            "taking_amount": 1_u64
        });
        assert!(LimitV2Adapter::parse_create_args(&args).is_err());
    }

    #[test]
    fn extract_order_pda_prefers_named_account() {
        let accounts = vec![
            account("idx1", None),
            account("idx2", None),
            account("named_order", Some("order")),
        ];
        let extracted = LimitV2Adapter::extract_order_pda(&accounts, "CancelOrder").unwrap();
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
            LimitV2Adapter::extract_order_pda(&init_accounts, "InitializeOrder").unwrap(),
            "init_idx2"
        );

        let pre_flash_accounts = vec![account("0", None), account("pre_flash_idx1", None)];
        assert_eq!(
            LimitV2Adapter::extract_order_pda(&pre_flash_accounts, "PreFlashFillOrder").unwrap(),
            "pre_flash_idx1"
        );
    }

    #[test]
    fn extract_order_pda_rejects_unknown_instruction() {
        let err = LimitV2Adapter::extract_order_pda(&[], "Unknown").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "unknown Limit v2 instruction: Unknown");
    }

    #[test]
    fn extract_order_pda_rejects_out_of_bounds_index() {
        let err = LimitV2Adapter::extract_order_pda(&[], "CancelOrder").unwrap_err();
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
        let extracted = LimitV2Adapter::extract_create_mints(&accounts).unwrap();
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
        let extracted = LimitV2Adapter::extract_create_mints(&accounts).unwrap();
        assert_eq!(extracted.input_mint, "fallback_input");
        assert_eq!(extracted.output_mint, "fallback_output");
    }

    #[test]
    fn extract_create_mints_rejects_missing_input_fallback_index() {
        let err = LimitV2Adapter::extract_create_mints(&[])
            .err()
            .expect("expected error");
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
        let err = LimitV2Adapter::extract_create_mints(&accounts)
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Limit v2 output_mint index 8 out of bounds");
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn instruction_constants_match_classify() {
        for (name, expected) in INSTRUCTION_EVENT_TYPES {
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
                LimitV2Adapter.classify_instruction(&ix).as_ref(),
                Some(expected),
                "INSTRUCTION_EVENT_TYPES mismatch for {name}"
            );
        }
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn event_constants_match_resolve() {
        for (name, expected) in EVENT_EVENT_TYPES {
            let fields = match *name {
                "TradeEvent" => {
                    serde_json::json!({(*name): {"order_key": "t", "making_amount": 1_u64, "taking_amount": 1_u64, "remaining_making_amount": 0_u64, "remaining_taking_amount": 0_u64}})
                }
                _ => serde_json::json!({(*name): {"order_key": "t"}}),
            };
            let result = resolve(fields);
            let (event_type, _, _) = result.expect("should return Some").expect("should be Ok");
            assert_eq!(
                &event_type, expected,
                "EVENT_EVENT_TYPES mismatch for {name}"
            );
        }
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
