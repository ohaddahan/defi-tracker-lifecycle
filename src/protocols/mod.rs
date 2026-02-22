pub mod dca;
pub mod kamino;
pub mod limit_v1;
pub mod limit_v2;

use serde::{Deserialize, Serialize};

use crate::error::Error;

#[cfg(feature = "wasm")]
pub const DCA_PROGRAM_ID: &str = "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M";
#[cfg(feature = "wasm")]
pub const LIMIT_V1_PROGRAM_ID: &str = "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu";
#[cfg(feature = "wasm")]
pub const LIMIT_V2_PROGRAM_ID: &str = "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X";
#[cfg(feature = "wasm")]
pub const KAMINO_PROGRAM_ID: &str = "LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF";

/// Supported DeFi protocols.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Serialize, strum_macros::Display, strum_macros::AsRefStr,
)]
#[strum(serialize_all = "snake_case")]
pub enum Protocol {
    /// Jupiter Dollar-Cost Averaging.
    Dca,
    /// Jupiter Limit Order v1.
    LimitV1,
    /// Jupiter Limit Order v2.
    LimitV2,
    /// Kamino Limit Order.
    Kamino,
}

impl Protocol {
    /// Resolves a base58 program id string to its [`Protocol`], or `None` if unrecognised.
    #[cfg(feature = "native")]
    pub fn from_program_id(program_id: &str) -> Option<Self> {
        let key: solana_pubkey::Pubkey = program_id.parse().ok()?;
        match key {
            carbon_jupiter_dca_decoder::PROGRAM_ID => Some(Self::Dca),
            carbon_jupiter_limit_order_decoder::PROGRAM_ID => Some(Self::LimitV1),
            carbon_jupiter_limit_order_2_decoder::PROGRAM_ID => Some(Self::LimitV2),
            carbon_kamino_limit_order_decoder::PROGRAM_ID => Some(Self::Kamino),
            _ => None,
        }
    }

    #[cfg(all(feature = "wasm", not(feature = "native")))]
    pub fn from_program_id(program_id: &str) -> Option<Self> {
        match program_id {
            DCA_PROGRAM_ID => Some(Self::Dca),
            LIMIT_V1_PROGRAM_ID => Some(Self::LimitV1),
            LIMIT_V2_PROGRAM_ID => Some(Self::LimitV2),
            KAMINO_PROGRAM_ID => Some(Self::Kamino),
            _ => None,
        }
    }

    /// Returns the on-chain program id for every supported protocol.
    #[cfg(feature = "native")]
    pub fn all_program_ids() -> [solana_pubkey::Pubkey; 4] {
        [
            carbon_jupiter_dca_decoder::PROGRAM_ID,
            carbon_jupiter_limit_order_decoder::PROGRAM_ID,
            carbon_jupiter_limit_order_2_decoder::PROGRAM_ID,
            carbon_kamino_limit_order_decoder::PROGRAM_ID,
        ]
    }

    #[cfg(feature = "wasm")]
    pub fn program_id_str(&self) -> &'static str {
        match self {
            Self::Dca => DCA_PROGRAM_ID,
            Self::LimitV1 => LIMIT_V1_PROGRAM_ID,
            Self::LimitV2 => LIMIT_V2_PROGRAM_ID,
            Self::Kamino => KAMINO_PROGRAM_ID,
        }
    }
}

/// Canonical event classification shared across all protocols.
#[derive(Debug, Clone, PartialEq, Eq, strum_macros::Display, strum_macros::AsRefStr)]
#[strum(serialize_all = "snake_case")]
pub enum EventType {
    /// Order was created on-chain.
    Created,
    /// A fill was initiated (e.g. flash-fill start).
    FillInitiated,
    /// A fill was completed (partial or full).
    FillCompleted,
    /// Order was explicitly cancelled.
    Cancelled,
    /// Order expired without completing.
    Expired,
    /// Order reached a terminal close (protocol-level).
    Closed,
    /// Protocol fee was collected.
    FeeCollected,
    /// Funds were withdrawn from the order.
    Withdrawn,
    /// Funds were deposited into the order.
    Deposited,
}

/// A single account entry from a decoded instruction's account list.
#[derive(Debug, Deserialize)]
pub struct AccountInfo {
    /// Base58-encoded account public key.
    pub pubkey: String,
    /// Whether this account signed the transaction.
    #[serde(default)]
    pub is_signer: bool,
    /// Whether this account was marked writable.
    #[serde(default)]
    pub is_writable: bool,
    /// Optional IDL-derived account name (e.g. `"dca"`, `"order"`).
    pub name: Option<String>,
}

/// Shared stateless helpers used across all protocol adapters.
pub struct ProtocolHelpers;

impl ProtocolHelpers {
    /// Deserializes a JSON array of accounts into [`AccountInfo`] structs.
    pub fn parse_accounts(accounts_json: &serde_json::Value) -> Result<Vec<AccountInfo>, Error> {
        serde_json::from_value(accounts_json.clone()).map_err(|e| Error::Protocol {
            reason: format!("failed to parse accounts: {e}"),
        })
    }

    /// Returns the pubkey of the first signer in the account list.
    pub fn find_signer(accounts: &[AccountInfo]) -> Option<&str> {
        accounts
            .iter()
            .find(|a| a.is_signer)
            .map(|a| a.pubkey.as_str())
    }

    /// Finds an account by its IDL-derived name.
    pub fn find_account_by_name<'a>(
        accounts: &'a [AccountInfo],
        name: &str,
    ) -> Option<&'a AccountInfo> {
        accounts.iter().find(|a| a.name.as_deref() == Some(name))
    }

    /// Returns `true` if the JSON object's keys contain any of the `known_names`.
    pub fn contains_known_variant(fields: &serde_json::Value, known_names: &[&str]) -> bool {
        fields
            .as_object()
            .is_some_and(|obj| obj.keys().any(|name| known_names.contains(&name.as_str())))
    }

    /// Converts `u64` to `i64`, returning an error if the value exceeds `i64::MAX`.
    pub fn checked_u64_to_i64(value: u64, field: &str) -> Result<i64, Error> {
        i64::try_from(value).map_err(|_| Error::Protocol {
            reason: format!("{field} exceeds i64::MAX: {value}"),
        })
    }

    /// Converts `u64` to `i64` for optional fields.
    /// Returns `None` if the value exceeds `i64::MAX` (e.g. `u64::MAX` sentinel for "no limit").
    pub fn optional_u64_to_i64(value: u64) -> Option<i64> {
        i64::try_from(value).ok()
    }

    /// Converts `u16` to `i16`, returning an error if the value exceeds `i16::MAX`.
    pub fn checked_u16_to_i16(value: u16, field: &str) -> Result<i16, Error> {
        i16::try_from(value).map_err(|_| Error::Protocol {
            reason: format!("{field} exceeds i16::MAX: {value}"),
        })
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::lifecycle::adapters::{ProtocolAdapter, adapter_for};
    use crate::types::{RawEvent, RawInstruction, ResolveContext};
    use std::collections::HashSet;

    #[cfg(feature = "native")]
    #[test]
    fn protocol_program_id_mapping_and_string_names_are_stable() {
        let cases = [
            (
                &carbon_jupiter_dca_decoder::PROGRAM_ID,
                Protocol::Dca,
                "dca",
            ),
            (
                &carbon_jupiter_limit_order_decoder::PROGRAM_ID,
                Protocol::LimitV1,
                "limit_v1",
            ),
            (
                &carbon_jupiter_limit_order_2_decoder::PROGRAM_ID,
                Protocol::LimitV2,
                "limit_v2",
            ),
            (
                &carbon_kamino_limit_order_decoder::PROGRAM_ID,
                Protocol::Kamino,
                "kamino",
            ),
        ];
        for (program_id, expected_protocol, expected_name) in cases {
            assert_eq!(
                Protocol::from_program_id(&program_id.to_string()),
                Some(expected_protocol)
            );
            assert_eq!(expected_protocol.as_ref(), expected_name);
        }

        assert_eq!(Protocol::from_program_id("unknown_program"), None);
        assert_eq!(Protocol::from_program_id("not_even_base58!@#"), None);
        assert_eq!(
            Protocol::all_program_ids(),
            [
                carbon_jupiter_dca_decoder::PROGRAM_ID,
                carbon_jupiter_limit_order_decoder::PROGRAM_ID,
                carbon_jupiter_limit_order_2_decoder::PROGRAM_ID,
                carbon_kamino_limit_order_decoder::PROGRAM_ID,
            ]
        );
    }

    #[cfg(all(feature = "native", feature = "wasm"))]
    #[test]
    fn hardcoded_program_ids_match_carbon_constants() {
        assert_eq!(
            carbon_jupiter_dca_decoder::PROGRAM_ID.to_string(),
            DCA_PROGRAM_ID
        );
        assert_eq!(
            carbon_jupiter_limit_order_decoder::PROGRAM_ID.to_string(),
            LIMIT_V1_PROGRAM_ID
        );
        assert_eq!(
            carbon_jupiter_limit_order_2_decoder::PROGRAM_ID.to_string(),
            LIMIT_V2_PROGRAM_ID
        );
        assert_eq!(
            carbon_kamino_limit_order_decoder::PROGRAM_ID.to_string(),
            KAMINO_PROGRAM_ID
        );
    }

    #[cfg(feature = "wasm")]
    #[test]
    fn protocol_program_id_str_roundtrips() {
        for protocol in [
            Protocol::Dca,
            Protocol::LimitV1,
            Protocol::LimitV2,
            Protocol::Kamino,
        ] {
            assert_eq!(
                Protocol::from_program_id(protocol.program_id_str()),
                Some(protocol)
            );
        }
    }

    #[test]
    fn event_type_strings_match_expected_labels() {
        let cases = [
            (EventType::Created, "created"),
            (EventType::FillInitiated, "fill_initiated"),
            (EventType::FillCompleted, "fill_completed"),
            (EventType::Cancelled, "cancelled"),
            (EventType::Expired, "expired"),
            (EventType::Closed, "closed"),
            (EventType::FeeCollected, "fee_collected"),
            (EventType::Withdrawn, "withdrawn"),
            (EventType::Deposited, "deposited"),
        ];
        for (event_type, expected_label) in cases {
            assert_eq!(event_type.as_ref(), expected_label);
        }
    }

    #[test]
    fn parse_accounts_supports_defaults_and_find_helpers() {
        let accounts_json = serde_json::json!([
            {
                "pubkey": "signer_pubkey",
                "is_signer": true,
                "is_writable": true,
                "name": "order"
            },
            {
                "pubkey": "readonly_pubkey"
            }
        ]);

        let parsed = ProtocolHelpers::parse_accounts(&accounts_json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].pubkey, "signer_pubkey");
        assert!(parsed[0].is_signer);
        assert!(parsed[0].is_writable);
        assert_eq!(parsed[0].name.as_deref(), Some("order"));

        assert_eq!(parsed[1].pubkey, "readonly_pubkey");
        assert!(!parsed[1].is_signer);
        assert!(!parsed[1].is_writable);
        assert!(parsed[1].name.is_none());

        assert_eq!(ProtocolHelpers::find_signer(&parsed), Some("signer_pubkey"));
        assert_eq!(
            ProtocolHelpers::find_account_by_name(&parsed, "order").map(|a| a.pubkey.as_str()),
            Some("signer_pubkey")
        );
        assert!(ProtocolHelpers::find_account_by_name(&parsed, "missing").is_none());
    }

    #[test]
    fn parse_accounts_rejects_non_array() {
        let err = ProtocolHelpers::parse_accounts(&serde_json::json!({"pubkey": "not-an-array"}))
            .unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert!(reason.contains("failed to parse accounts"), "{reason}");
    }

    #[test]
    fn parse_accounts_rejects_missing_pubkey() {
        let err =
            ProtocolHelpers::parse_accounts(&serde_json::json!([{"is_signer": true}])).unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert!(reason.contains("failed to parse accounts"), "{reason}");
    }

    #[test]
    fn find_signer_returns_none_when_no_signer_present() {
        let accounts = vec![AccountInfo {
            pubkey: "p1".to_string(),
            is_signer: false,
            is_writable: false,
            name: Some("order".to_string()),
        }];

        assert_eq!(ProtocolHelpers::find_signer(&accounts), None);
    }

    fn make_ix(name: &str) -> RawInstruction {
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

    fn collect_instruction_event_types(
        instruction_names: &[&str],
        adapter: &dyn ProtocolAdapter,
    ) -> HashSet<String> {
        instruction_names
            .iter()
            .filter_map(|name| adapter.classify_instruction(&make_ix(name)))
            .map(|et| et.as_ref().to_string())
            .collect()
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

    fn resolve_event_type(
        json: serde_json::Value,
        adapter: &dyn ProtocolAdapter,
        ctx: &ResolveContext,
    ) -> Option<String> {
        adapter
            .classify_and_resolve_event(&make_event(json), ctx)
            .and_then(|r| r.ok())
            .map(|(et, _, _)| et.as_ref().to_string())
    }

    #[test]
    fn event_type_reachability_all_variants_covered() {
        let mut all_event_types: HashSet<String> = HashSet::new();
        let default_ctx = ResolveContext {
            pre_fetched_order_pdas: None,
        };

        let dca = adapter_for(Protocol::Dca);
        let dca_ix_names = [
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
        all_event_types.extend(collect_instruction_event_types(&dca_ix_names, dca));

        let dca_event_payloads = [
            serde_json::json!({"OpenedEvent": {"dca_key": "t"}}),
            serde_json::json!({"FilledEvent": {"dca_key": "t", "in_amount": 1_u64, "out_amount": 1_u64}}),
            serde_json::json!({"ClosedEvent": {"dca_key": "t", "user_closed": false, "unfilled_amount": 0_u64}}),
            serde_json::json!({"CollectedFeeEvent": {"dca_key": "t"}}),
            serde_json::json!({"WithdrawEvent": {"dca_key": "t"}}),
            serde_json::json!({"DepositEvent": {"dca_key": "t"}}),
        ];
        for json in &dca_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), dca, &default_ctx) {
                all_event_types.insert(et);
            }
        }

        let v1 = adapter_for(Protocol::LimitV1);
        let v1_ix_names = [
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
        all_event_types.extend(collect_instruction_event_types(&v1_ix_names, v1));

        let v1_event_payloads = [
            serde_json::json!({"CreateOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"CancelOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"TradeEvent": {"order_key": "t", "in_amount": 1_u64, "out_amount": 1_u64, "remaining_in_amount": 0_u64, "remaining_out_amount": 0_u64}}),
        ];
        for json in &v1_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), v1, &default_ctx) {
                all_event_types.insert(et);
            }
        }

        let v2 = adapter_for(Protocol::LimitV2);
        let v2_ix_names = [
            "InitializeOrder",
            "PreFlashFillOrder",
            "FlashFillOrder",
            "CancelOrder",
            "UpdateFee",
            "WithdrawFee",
        ];
        all_event_types.extend(collect_instruction_event_types(&v2_ix_names, v2));

        let v2_event_payloads = [
            serde_json::json!({"CreateOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"CancelOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"TradeEvent": {"order_key": "t", "making_amount": 1_u64, "taking_amount": 1_u64, "remaining_making_amount": 0_u64, "remaining_taking_amount": 0_u64}}),
        ];
        for json in &v2_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), v2, &default_ctx) {
                all_event_types.insert(et);
            }
        }

        let kamino = adapter_for(Protocol::Kamino);
        let kamino_ix_names = [
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
        all_event_types.extend(collect_instruction_event_types(&kamino_ix_names, kamino));

        let kamino_ctx = ResolveContext {
            pre_fetched_order_pdas: Some(vec!["test_pda".to_string()]),
        };
        let kamino_event_payloads = [
            serde_json::json!({"OrderDisplayEvent": {"status": 1_u8}}),
            serde_json::json!({"UserSwapBalancesEvent": {}}),
        ];
        for json in &kamino_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), kamino, &kamino_ctx) {
                all_event_types.insert(et);
            }
        }

        let expected: HashSet<String> = [
            "created",
            "fill_initiated",
            "fill_completed",
            "cancelled",
            "expired",
            "closed",
            "fee_collected",
            "withdrawn",
            "deposited",
        ]
        .into_iter()
        .map(String::from)
        .collect();

        assert_eq!(
            all_event_types,
            expected,
            "missing EventTypes: {:?}, extra: {:?}",
            expected.difference(&all_event_types).collect::<Vec<_>>(),
            all_event_types.difference(&expected).collect::<Vec<_>>()
        );
    }
}
