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
