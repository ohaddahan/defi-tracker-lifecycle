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

/// Unwrap Carbon's named wrapper: `{"EventName": {...}}` -> `{...}`.
/// If the value is an object with exactly one key, returns the inner value.
/// Otherwise returns the value as-is.
pub fn unwrap_named(value: &serde_json::Value) -> &serde_json::Value {
    if let Some(obj) = value.as_object()
        && obj.len() == 1
        && let Some(inner) = obj.values().next()
    {
        return inner;
    }
    value
}

/// Convert a JSON value to a base58 pubkey string.
/// Handles both byte arrays (`[u8; 32]`) and direct strings.
pub fn value_to_pubkey(value: &serde_json::Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    let arr = value.as_array()?;
    let bytes: Vec<u8> = arr
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n as u8))
        .collect();
    if bytes.len() == 32 {
        Some(bs58::encode(&bytes).into_string())
    } else {
        None
    }
}
