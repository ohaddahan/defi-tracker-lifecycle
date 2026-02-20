use crate::error::Error;
use crate::lifecycle::adapters::{CorrelationOutcome, EventPayload, ProtocolAdapter};
use crate::protocols::{AccountInfo, EventType, Protocol, ProtocolHelpers};
use crate::types::{RawEvent, RawInstruction, ResolveContext};
use strum::VariantNames;

/// Serde-tagged envelope for Jupiter Limit Order v1 event variants.
#[derive(serde::Deserialize, strum_macros::VariantNames)]
pub enum LimitV1EventEnvelope {
    CreateOrderEvent(OrderKeyHolder),
    CancelOrderEvent(OrderKeyHolder),
    TradeEvent(TradeEventFields),
}

/// Serde-tagged envelope for Jupiter Limit Order v1 instruction variants.
#[derive(serde::Deserialize)]
pub enum LimitV1InstructionKind {
    InitializeOrder(serde_json::Value),
    PreFlashFillOrder(serde_json::Value),
    FillOrder(serde_json::Value),
    FlashFillOrder(serde_json::Value),
    CancelOrder(serde_json::Value),
    CancelExpiredOrder(serde_json::Value),
    WithdrawFee(serde_json::Value),
    InitFee(serde_json::Value),
    UpdateFee(serde_json::Value),
}

/// Jupiter Limit Order v1 protocol adapter (zero-sized, stored as a static).
#[derive(Debug)]
pub struct LimitV1Adapter;

/// Serde intermediate for events that only carry an `order_key`.
#[derive(serde::Deserialize)]
pub struct OrderKeyHolder {
    order_key: String,
}

/// Serde intermediate for `TradeEvent` payload fields.
///
/// Uses `serde(alias)` to accept both V1 (`in_amount`) and V2 (`making_amount`) field names.
#[derive(serde::Deserialize)]
pub struct TradeEventFields {
    order_key: String,
    #[serde(default = "LimitV1Adapter::default_unknown")]
    taker: String,
    #[serde(alias = "making_amount", default)]
    in_amount: u64,
    #[serde(alias = "taking_amount", default)]
    out_amount: u64,
    #[serde(alias = "remaining_making_amount", default)]
    remaining_in_amount: u64,
    #[expect(dead_code, reason = "consumed by serde for completeness")]
    #[serde(alias = "remaining_taking_amount", default)]
    remaining_out_amount: u64,
}

/// Extracted Limit v1 trade event with checked-cast amounts.
pub struct LimitTradeEvent {
    pub order_pda: String,
    pub taker: String,
    pub in_amount: i64,
    pub out_amount: i64,
    pub remaining_in_amount: i64,
    pub remaining_out_amount: i64,
}

/// Parsed arguments from an `InitializeOrder` instruction (Limit v1).
pub struct LimitV1CreateArgs {
    pub making_amount: i64,
    pub taking_amount: i64,
    pub expired_at: Option<i64>,
}

/// Input and output mint addresses extracted from a Limit v1 create instruction.
pub struct LimitV1CreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

#[derive(serde::Deserialize)]
struct InitializeOrderFields {
    making_amount: u64,
    taking_amount: u64,
    expired_at: Option<i64>,
}

impl ProtocolAdapter for LimitV1Adapter {
    fn protocol(&self) -> Protocol {
        Protocol::LimitV1
    }

    fn classify_instruction(&self, ix: &RawInstruction) -> Option<EventType> {
        let wrapper = serde_json::json!({ &ix.instruction_name: ix.args });
        let kind: LimitV1InstructionKind = serde_json::from_value(wrapper).ok()?;
        match kind {
            LimitV1InstructionKind::InitializeOrder(_) => Some(EventType::Created),
            LimitV1InstructionKind::PreFlashFillOrder(_) => Some(EventType::FillInitiated),
            LimitV1InstructionKind::FillOrder(_) | LimitV1InstructionKind::FlashFillOrder(_) => {
                Some(EventType::FillCompleted)
            }
            LimitV1InstructionKind::CancelOrder(_) => Some(EventType::Cancelled),
            LimitV1InstructionKind::CancelExpiredOrder(_) => Some(EventType::Expired),
            LimitV1InstructionKind::WithdrawFee(_)
            | LimitV1InstructionKind::InitFee(_)
            | LimitV1InstructionKind::UpdateFee(_) => None,
        }
    }

    fn classify_and_resolve_event(
        &self,
        ev: &RawEvent,
        _ctx: &ResolveContext,
    ) -> Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>> {
        let fields = ev.fields.as_ref()?;
        let envelope: LimitV1EventEnvelope = match serde_json::from_value(fields.clone()) {
            Ok(e) => e,
            Err(err) => {
                if !ProtocolHelpers::contains_known_variant(fields, LimitV1EventEnvelope::VARIANTS)
                {
                    return None;
                }
                return Some(Err(Error::Protocol {
                    reason: format!("failed to parse Limit v1 event payload: {err}"),
                }));
            }
        };

        Some(Self::resolve_event(envelope))
    }
}

impl LimitV1Adapter {
    fn default_unknown() -> String {
        "unknown".to_string()
    }

    fn resolve_event(
        envelope: LimitV1EventEnvelope,
    ) -> Result<(EventType, CorrelationOutcome, EventPayload), Error> {
        match envelope {
            LimitV1EventEnvelope::CreateOrderEvent(OrderKeyHolder { order_key }) => Ok((
                EventType::Created,
                CorrelationOutcome::Correlated(vec![order_key]),
                EventPayload::None,
            )),
            LimitV1EventEnvelope::CancelOrderEvent(OrderKeyHolder { order_key }) => Ok((
                EventType::Cancelled,
                CorrelationOutcome::Correlated(vec![order_key]),
                EventPayload::None,
            )),
            LimitV1EventEnvelope::TradeEvent(TradeEventFields {
                order_key,
                taker,
                in_amount,
                out_amount,
                remaining_in_amount,
                ..
            }) => Ok((
                EventType::FillCompleted,
                CorrelationOutcome::Correlated(vec![order_key]),
                EventPayload::LimitFill {
                    in_amount: ProtocolHelpers::checked_u64_to_i64(in_amount, "in_amount")?,
                    out_amount: ProtocolHelpers::checked_u64_to_i64(out_amount, "out_amount")?,
                    remaining_in_amount: ProtocolHelpers::checked_u64_to_i64(
                        remaining_in_amount,
                        "remaining_in_amount",
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
        let kind: LimitV1InstructionKind =
            serde_json::from_value(wrapper).map_err(|_| Error::Protocol {
                reason: format!("unknown Limit v1 instruction: {instruction_name}"),
            })?;

        let idx = match kind {
            LimitV1InstructionKind::InitializeOrder(_) => 2,
            LimitV1InstructionKind::FillOrder(_)
            | LimitV1InstructionKind::PreFlashFillOrder(_)
            | LimitV1InstructionKind::FlashFillOrder(_) => 0,
            LimitV1InstructionKind::CancelOrder(_)
            | LimitV1InstructionKind::CancelExpiredOrder(_) => 0,
            LimitV1InstructionKind::WithdrawFee(_)
            | LimitV1InstructionKind::InitFee(_)
            | LimitV1InstructionKind::UpdateFee(_) => {
                return Err(Error::Protocol {
                    reason: format!("Limit v1 instruction {instruction_name} has no order PDA"),
                });
            }
        };

        accounts
            .get(idx)
            .map(|a| a.pubkey.clone())
            .ok_or_else(|| Error::Protocol {
                reason: format!(
                    "Limit v1 account index {idx} out of bounds for {instruction_name}"
                ),
            })
    }

    /// Extracts input/output mint addresses from a Limit v1 create instruction's accounts.
    ///
    /// Prefers named accounts; falls back to positional indexes 5 (input) and 8 (output).
    pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<LimitV1CreateMints, Error> {
        let by_name_input =
            ProtocolHelpers::find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
        let by_name_output = ProtocolHelpers::find_account_by_name(accounts, "output_mint")
            .map(|a| a.pubkey.clone());

        if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
            return Ok(LimitV1CreateMints {
                input_mint,
                output_mint,
            });
        }

        let input_mint =
            accounts
                .get(5)
                .map(|a| a.pubkey.clone())
                .ok_or_else(|| Error::Protocol {
                    reason: "Limit v1 input_mint index 5 out of bounds".into(),
                })?;
        let output_mint =
            accounts
                .get(8)
                .map(|a| a.pubkey.clone())
                .ok_or_else(|| Error::Protocol {
                    reason: "Limit v1 output_mint index 8 out of bounds".into(),
                })?;

        Ok(LimitV1CreateMints {
            input_mint,
            output_mint,
        })
    }

    /// Parses `InitializeOrder` instruction args into checked [`LimitV1CreateArgs`].
    pub fn parse_create_args(args: &serde_json::Value) -> Result<LimitV1CreateArgs, Error> {
        let InitializeOrderFields {
            making_amount,
            taking_amount,
            expired_at,
        } = serde_json::from_value(args.clone()).map_err(|e| Error::Protocol {
            reason: format!("failed to parse Limit v1 create args: {e}"),
        })?;

        Ok(LimitV1CreateArgs {
            making_amount: ProtocolHelpers::checked_u64_to_i64(making_amount, "making_amount")?,
            taking_amount: ProtocolHelpers::checked_u64_to_i64(taking_amount, "taking_amount")?,
            expired_at,
        })
    }

    #[cfg(test)]
    pub fn classify_decoded(
        decoded: &carbon_jupiter_limit_order_decoder::instructions::JupiterLimitOrderInstruction,
    ) -> Option<EventType> {
        use carbon_jupiter_limit_order_decoder::instructions::JupiterLimitOrderInstruction;
        match decoded {
            JupiterLimitOrderInstruction::InitializeOrder(_) => Some(EventType::Created),
            JupiterLimitOrderInstruction::PreFlashFillOrder(_) => Some(EventType::FillInitiated),
            JupiterLimitOrderInstruction::FillOrder(_)
            | JupiterLimitOrderInstruction::FlashFillOrder(_) => Some(EventType::FillCompleted),
            JupiterLimitOrderInstruction::CancelOrder(_) => Some(EventType::Cancelled),
            JupiterLimitOrderInstruction::CancelExpiredOrder(_) => Some(EventType::Expired),
            JupiterLimitOrderInstruction::CreateOrderEvent(_) => Some(EventType::Created),
            JupiterLimitOrderInstruction::CancelOrderEvent(_) => Some(EventType::Cancelled),
            JupiterLimitOrderInstruction::TradeEvent(_) => Some(EventType::FillCompleted),
            JupiterLimitOrderInstruction::WithdrawFee(_)
            | JupiterLimitOrderInstruction::InitFee(_)
            | JupiterLimitOrderInstruction::UpdateFee(_) => None,
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
        LimitV1Adapter.classify_and_resolve_event(&ev, &ctx)
    }

    #[test]
    fn classify_known_instructions_via_envelope() {
        let cases = [
            ("InitializeOrder", Some(EventType::Created)),
            ("PreFlashFillOrder", Some(EventType::FillInitiated)),
            ("FillOrder", Some(EventType::FillCompleted)),
            ("FlashFillOrder", Some(EventType::FillCompleted)),
            ("CancelOrder", Some(EventType::Cancelled)),
            ("CancelExpiredOrder", Some(EventType::Expired)),
            ("WithdrawFee", None),
            ("InitFee", None),
            ("UpdateFee", None),
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
                LimitV1Adapter.classify_instruction(&ix),
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
    fn resolve_create_order_event_from_envelope() {
        let fields = serde_json::json!({
            "CreateOrderEvent": {
                "order_key": "ABC123"
            }
        });
        let (event_type, correlation, payload) = resolve(fields).unwrap().unwrap();
        assert_eq!(event_type, EventType::Created);
        assert_eq!(
            correlation,
            CorrelationOutcome::Correlated(vec!["ABC123".to_string()])
        );
        assert_eq!(payload, EventPayload::None);
    }

    #[test]
    fn unknown_event_returns_none() {
        let fields = serde_json::json!({"UnknownEvent": {"some_field": 1}});
        assert!(resolve(fields).is_none());
    }

    #[test]
    fn malformed_known_event_returns_error() {
        let fields = serde_json::json!({
            "TradeEvent": {
                "order_key": "order",
                "in_amount": "bad",
                "out_amount": 1_u64,
                "remaining_in_amount": 0_u64,
                "remaining_out_amount": 0_u64
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
                "in_amount": (i64::MAX as u64) + 1,
                "out_amount": 1_u64,
                "remaining_in_amount": 0_u64,
                "remaining_out_amount": 0_u64
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
                "in_amount": 10_u64,
                "out_amount": 5_u64,
                "remaining_in_amount": 1_u64,
                "remaining_out_amount": 0_u64
            }
        });
        let (_, _, payload) = resolve(fields).unwrap().unwrap();
        let EventPayload::LimitFill { counterparty, .. } = payload else {
            panic!("expected LimitFill");
        };
        assert_eq!(counterparty, "unknown");
    }

    #[test]
    fn parse_create_args_rejects_amount_overflow() {
        let args = serde_json::json!({
            "making_amount": (i64::MAX as u64) + 1,
            "taking_amount": 1_u64
        });
        assert!(LimitV1Adapter::parse_create_args(&args).is_err());
    }

    #[test]
    fn parse_create_args_accepts_valid_payload() {
        let args = serde_json::json!({
            "making_amount": 5_000_u64,
            "taking_amount": 4_500_u64,
            "expired_at": 1_700_000_000_i64
        });
        let parsed = LimitV1Adapter::parse_create_args(&args).unwrap();
        assert_eq!(parsed.making_amount, 5_000);
        assert_eq!(parsed.taking_amount, 4_500);
        assert_eq!(parsed.expired_at, Some(1_700_000_000));
    }

    #[test]
    fn parse_create_args_rejects_malformed_payload() {
        let args = serde_json::json!({
            "making_amount": "bad",
            "taking_amount": 1_u64
        });
        assert!(LimitV1Adapter::parse_create_args(&args).is_err());
    }

    #[test]
    fn extract_order_pda_prefers_named_account() {
        let accounts = vec![
            account("idx0", None),
            account("idx2", None),
            account("named_order", Some("order")),
        ];
        let extracted = LimitV1Adapter::extract_order_pda(&accounts, "InitializeOrder").unwrap();
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
            LimitV1Adapter::extract_order_pda(&init_accounts, "InitializeOrder").unwrap(),
            "init_idx2"
        );

        let fill_accounts = vec![account("fill_idx0", None)];
        assert_eq!(
            LimitV1Adapter::extract_order_pda(&fill_accounts, "FillOrder").unwrap(),
            "fill_idx0"
        );
    }

    #[test]
    fn extract_order_pda_rejects_unknown_instruction() {
        let err = LimitV1Adapter::extract_order_pda(&[], "Unknown").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "unknown Limit v1 instruction: Unknown");
    }

    #[test]
    fn extract_order_pda_rejects_out_of_bounds_index() {
        let err = LimitV1Adapter::extract_order_pda(&[], "InitializeOrder").unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(
            reason,
            "Limit v1 account index 2 out of bounds for InitializeOrder"
        );
    }

    #[test]
    fn extract_create_mints_prefers_named_accounts() {
        let accounts = vec![
            account("idx5", None),
            account("idx8", None),
            account("named_input", Some("input_mint")),
            account("named_output", Some("output_mint")),
        ];
        let extracted = LimitV1Adapter::extract_create_mints(&accounts).unwrap();
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
            account("fallback_input", None),
            account("6", None),
            account("7", None),
            account("fallback_output", None),
        ];
        let extracted = LimitV1Adapter::extract_create_mints(&accounts).unwrap();
        assert_eq!(extracted.input_mint, "fallback_input");
        assert_eq!(extracted.output_mint, "fallback_output");
    }

    #[test]
    fn extract_create_mints_rejects_missing_input_fallback_index() {
        let err = LimitV1Adapter::extract_create_mints(&[])
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Limit v1 input_mint index 5 out of bounds");
    }

    #[test]
    fn extract_create_mints_rejects_missing_output_fallback_index() {
        let accounts = vec![
            account("0", None),
            account("1", None),
            account("2", None),
            account("3", None),
            account("4", None),
            account("fallback_input", None),
        ];
        let err = LimitV1Adapter::extract_create_mints(&accounts)
            .err()
            .expect("expected error");
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "Limit v1 output_mint index 8 out of bounds");
    }

    #[test]
    fn mirror_enums_cover_all_carbon_variants() {
        let instruction_variants = [
            "InitializeOrder",
            "PreFlashFillOrder",
            "FillOrder",
            "FlashFillOrder",
            "CancelOrder",
            "CancelExpiredOrder",
            "WithdrawFee",
            "InitFee",
            "UpdateFee",
        ];
        for name in instruction_variants {
            let json = serde_json::json!({ name: serde_json::Value::Null });
            assert!(
                serde_json::from_value::<LimitV1InstructionKind>(json).is_ok(),
                "LimitV1InstructionKind missing variant: {name}"
            );
        }

        for name in ["CreateOrderEvent", "CancelOrderEvent"] {
            let json = serde_json::json!({ name: { "order_key": "test" } });
            assert!(
                serde_json::from_value::<LimitV1EventEnvelope>(json).is_ok(),
                "LimitV1EventEnvelope missing variant: {name}"
            );
        }

        let trade = serde_json::json!({
            "TradeEvent": { "order_key": "t", "in_amount": 1_u64, "out_amount": 1_u64,
                "remaining_in_amount": 0_u64, "remaining_out_amount": 0_u64 }
        });
        assert!(serde_json::from_value::<LimitV1EventEnvelope>(trade).is_ok());
    }
}
