pub mod dca;
pub mod kamino;
pub mod limit_v1;
pub mod limit_v2;

use serde::Serialize;

use crate::error::Error;

pub const JUPITER_DCA_PROGRAM_ID: &str = "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M";
pub const JUPITER_LIMIT_ORDER_PROGRAM_ID: &str = "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu";
pub const JUPITER_LIMIT_ORDER_2_PROGRAM_ID: &str = "j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X";
pub const KAMINO_LIMIT_ORDER_PROGRAM_ID: &str = "LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Protocol {
    Dca,
    LimitV1,
    LimitV2,
    Kamino,
}

impl Protocol {
    pub fn from_program_id(program_id: &str) -> Option<Self> {
        match program_id {
            JUPITER_DCA_PROGRAM_ID => Some(Self::Dca),
            JUPITER_LIMIT_ORDER_PROGRAM_ID => Some(Self::LimitV1),
            JUPITER_LIMIT_ORDER_2_PROGRAM_ID => Some(Self::LimitV2),
            KAMINO_LIMIT_ORDER_PROGRAM_ID => Some(Self::Kamino),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Dca => "dca",
            Self::LimitV1 => "limit_v1",
            Self::LimitV2 => "limit_v2",
            Self::Kamino => "kamino",
        }
    }

    pub fn all_program_ids() -> &'static [&'static str] {
        &[
            JUPITER_DCA_PROGRAM_ID,
            JUPITER_LIMIT_ORDER_PROGRAM_ID,
            JUPITER_LIMIT_ORDER_2_PROGRAM_ID,
            KAMINO_LIMIT_ORDER_PROGRAM_ID,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventType {
    Created,
    FillInitiated,
    FillCompleted,
    Cancelled,
    Expired,
    Closed,
    FeeCollected,
    Withdrawn,
    Deposited,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::FillInitiated => "fill_initiated",
            Self::FillCompleted => "fill_completed",
            Self::Cancelled => "cancelled",
            Self::Expired => "expired",
            Self::Closed => "closed",
            Self::FeeCollected => "fee_collected",
            Self::Withdrawn => "withdrawn",
            Self::Deposited => "deposited",
        }
    }
}

#[derive(Debug)]
pub struct AccountInfo {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
    pub name: Option<String>,
}

pub fn parse_accounts(accounts_json: &serde_json::Value) -> Result<Vec<AccountInfo>, Error> {
    let arr = accounts_json.as_array().ok_or_else(|| Error::Protocol {
        reason: "accounts is not an array".into(),
    })?;

    let mut result = Vec::with_capacity(arr.len());
    for item in arr {
        let pubkey = item
            .get("pubkey")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::Protocol {
                reason: "account missing pubkey".into(),
            })?
            .to_string();
        let is_signer = item
            .get("is_signer")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let is_writable = item
            .get("is_writable")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let name = item.get("name").and_then(|v| v.as_str()).map(String::from);
        result.push(AccountInfo {
            pubkey,
            is_signer,
            is_writable,
            name,
        });
    }
    Ok(result)
}

pub fn find_signer(accounts: &[AccountInfo]) -> Option<&str> {
    accounts
        .iter()
        .find(|a| a.is_signer)
        .map(|a| a.pubkey.as_str())
}

pub fn find_account_by_name<'a>(
    accounts: &'a [AccountInfo],
    name: &str,
) -> Option<&'a AccountInfo> {
    accounts.iter().find(|a| a.name.as_deref() == Some(name))
}

pub(crate) fn contains_known_variant(fields: &serde_json::Value, known_names: &[&str]) -> bool {
    fields
        .as_object()
        .is_some_and(|obj| obj.keys().any(|name| known_names.contains(&name.as_str())))
}

pub(crate) fn checked_u64_to_i64(value: u64, field: &str) -> Result<i64, Error> {
    i64::try_from(value).map_err(|_| Error::Protocol {
        reason: format!("{field} exceeds i64::MAX: {value}"),
    })
}

pub(crate) fn checked_u16_to_i16(value: u16, field: &str) -> Result<i16, Error> {
    i16::try_from(value).map_err(|_| Error::Protocol {
        reason: format!("{field} exceeds i16::MAX: {value}"),
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;
    use crate::types::RawInstruction;
    use std::collections::HashSet;

    #[test]
    fn program_ids_match_carbon_constants() {
        assert_eq!(
            JUPITER_DCA_PROGRAM_ID,
            carbon_jupiter_dca_decoder::PROGRAM_ID.to_string()
        );
        assert_eq!(
            JUPITER_LIMIT_ORDER_PROGRAM_ID,
            carbon_jupiter_limit_order_decoder::PROGRAM_ID.to_string()
        );
        assert_eq!(
            JUPITER_LIMIT_ORDER_2_PROGRAM_ID,
            carbon_jupiter_limit_order_2_decoder::PROGRAM_ID.to_string()
        );
        assert_eq!(
            KAMINO_LIMIT_ORDER_PROGRAM_ID,
            carbon_kamino_limit_order_decoder::PROGRAM_ID.to_string()
        );
    }

    #[test]
    fn protocol_program_id_mapping_and_string_names_are_stable() {
        let cases = [
            (JUPITER_DCA_PROGRAM_ID, Protocol::Dca, "dca"),
            (
                JUPITER_LIMIT_ORDER_PROGRAM_ID,
                Protocol::LimitV1,
                "limit_v1",
            ),
            (
                JUPITER_LIMIT_ORDER_2_PROGRAM_ID,
                Protocol::LimitV2,
                "limit_v2",
            ),
            (KAMINO_LIMIT_ORDER_PROGRAM_ID, Protocol::Kamino, "kamino"),
        ];
        for (program_id, expected_protocol, expected_name) in cases {
            assert_eq!(
                Protocol::from_program_id(program_id),
                Some(expected_protocol)
            );
            assert_eq!(expected_protocol.as_str(), expected_name);
        }

        assert_eq!(Protocol::from_program_id("unknown_program"), None);
        assert_eq!(
            Protocol::all_program_ids(),
            &[
                JUPITER_DCA_PROGRAM_ID,
                JUPITER_LIMIT_ORDER_PROGRAM_ID,
                JUPITER_LIMIT_ORDER_2_PROGRAM_ID,
                KAMINO_LIMIT_ORDER_PROGRAM_ID
            ]
        );
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
            assert_eq!(event_type.as_str(), expected_label);
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

        let parsed = parse_accounts(&accounts_json).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].pubkey, "signer_pubkey");
        assert!(parsed[0].is_signer);
        assert!(parsed[0].is_writable);
        assert_eq!(parsed[0].name.as_deref(), Some("order"));

        assert_eq!(parsed[1].pubkey, "readonly_pubkey");
        assert!(!parsed[1].is_signer);
        assert!(!parsed[1].is_writable);
        assert!(parsed[1].name.is_none());

        assert_eq!(find_signer(&parsed), Some("signer_pubkey"));
        assert_eq!(
            find_account_by_name(&parsed, "order").map(|a| a.pubkey.as_str()),
            Some("signer_pubkey")
        );
        assert!(find_account_by_name(&parsed, "missing").is_none());
    }

    #[test]
    fn parse_accounts_rejects_non_array() {
        let err = parse_accounts(&serde_json::json!({"pubkey": "not-an-array"})).unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "accounts is not an array");
    }

    #[test]
    fn parse_accounts_rejects_missing_pubkey() {
        let err = parse_accounts(&serde_json::json!([{"is_signer": true}])).unwrap_err();
        let Error::Protocol { reason } = err else {
            panic!("expected protocol error");
        };
        assert_eq!(reason, "account missing pubkey");
    }

    #[test]
    fn find_signer_returns_none_when_no_signer_present() {
        let accounts = vec![AccountInfo {
            pubkey: "p1".to_string(),
            is_signer: false,
            is_writable: false,
            name: Some("order".to_string()),
        }];

        assert_eq!(find_signer(&accounts), None);
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
        classify: fn(&RawInstruction) -> Option<EventType>,
    ) -> HashSet<String> {
        instruction_names
            .iter()
            .filter_map(|name| classify(&make_ix(name)))
            .map(|et| et.as_str().to_string())
            .collect()
    }

    fn resolve_event_type(
        json: serde_json::Value,
        resolve: &dyn Fn(&serde_json::Value) -> Option<EventType>,
    ) -> Option<String> {
        resolve(&json).map(|et| et.as_str().to_string())
    }

    #[test]
    fn event_type_reachability_all_variants_covered() {
        let mut all_event_types: HashSet<String> = HashSet::new();

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
        all_event_types.extend(collect_instruction_event_types(
            &dca_ix_names,
            dca::classify_instruction_envelope,
        ));

        let dca_resolve = |json: &serde_json::Value| {
            dca::resolve_event_envelope(json)
                .and_then(|r| r.ok())
                .map(|(et, _, _)| et)
        };
        let dca_event_payloads = [
            serde_json::json!({"OpenedEvent": {"dca_key": "t"}}),
            serde_json::json!({"FilledEvent": {"dca_key": "t", "in_amount": 1_u64, "out_amount": 1_u64}}),
            serde_json::json!({"ClosedEvent": {"dca_key": "t", "user_closed": false, "unfilled_amount": 0_u64}}),
            serde_json::json!({"CollectedFeeEvent": {"dca_key": "t"}}),
            serde_json::json!({"WithdrawEvent": {"dca_key": "t"}}),
            serde_json::json!({"DepositEvent": {"dca_key": "t"}}),
        ];
        for json in &dca_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), &dca_resolve) {
                all_event_types.insert(et);
            }
        }

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
        all_event_types.extend(collect_instruction_event_types(
            &v1_ix_names,
            limit_v1::classify_instruction_envelope,
        ));

        let v1_resolve = |json: &serde_json::Value| {
            limit_v1::resolve_event_envelope(json)
                .and_then(|r| r.ok())
                .map(|(et, _, _)| et)
        };
        let v1_event_payloads = [
            serde_json::json!({"CreateOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"CancelOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"TradeEvent": {"order_key": "t", "in_amount": 1_u64, "out_amount": 1_u64, "remaining_in_amount": 0_u64, "remaining_out_amount": 0_u64}}),
        ];
        for json in &v1_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), &v1_resolve) {
                all_event_types.insert(et);
            }
        }

        let v2_ix_names = [
            "InitializeOrder",
            "PreFlashFillOrder",
            "FlashFillOrder",
            "CancelOrder",
            "UpdateFee",
            "WithdrawFee",
        ];
        all_event_types.extend(collect_instruction_event_types(
            &v2_ix_names,
            limit_v2::classify_instruction_envelope,
        ));

        let v2_resolve = |json: &serde_json::Value| {
            limit_v2::resolve_event_envelope(json)
                .and_then(|r| r.ok())
                .map(|(et, _, _)| et)
        };
        let v2_event_payloads = [
            serde_json::json!({"CreateOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"CancelOrderEvent": {"order_key": "t"}}),
            serde_json::json!({"TradeEvent": {"order_key": "t", "making_amount": 1_u64, "taking_amount": 1_u64, "remaining_making_amount": 0_u64, "remaining_taking_amount": 0_u64}}),
        ];
        for json in &v2_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), &v2_resolve) {
                all_event_types.insert(et);
            }
        }

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
        all_event_types.extend(collect_instruction_event_types(
            &kamino_ix_names,
            kamino::classify_instruction_envelope,
        ));

        let kamino_resolve = |json: &serde_json::Value| {
            let ctx = crate::types::ResolveContext {
                pre_fetched_order_pdas: Some(vec!["test_pda".to_string()]),
            };
            kamino::resolve_event_envelope(json, "sig", &ctx)
                .and_then(|r| r.ok())
                .map(|(et, _, _)| et)
        };
        let kamino_event_payloads = [
            serde_json::json!({"OrderDisplayEvent": {"status": 1_u8}}),
            serde_json::json!({"UserSwapBalancesEvent": {}}),
        ];
        for json in &kamino_event_payloads {
            if let Some(et) = resolve_event_type(json.clone(), &kamino_resolve) {
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
