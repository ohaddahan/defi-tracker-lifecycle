use crate::error::Error;
use crate::protocols::{AccountInfo, EventType, find_account_by_name};

pub fn classify_instruction(name: &str) -> Option<EventType> {
    match name {
        "InitializeOrder" => Some(EventType::Created),
        "PreFlashFillOrder" => Some(EventType::FillInitiated),
        "FlashFillOrder" => Some(EventType::FillCompleted),
        "CancelOrder" => Some(EventType::Cancelled),
        _ => None,
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
        "InitializeOrder" => 2,
        "FlashFillOrder" | "CancelOrder" => 2,
        "PreFlashFillOrder" => 1,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown Limit v2 instruction: {instruction_name}"),
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

pub fn classify_event(name: &str) -> Option<EventType> {
    match name {
        "CreateOrderEvent" => Some(EventType::Created),
        "CancelOrderEvent" => Some(EventType::Cancelled),
        "TradeEvent" => Some(EventType::FillCompleted),
        _ => None,
    }
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<LimitV2CreateArgs, Error> {
    let source = args.get("params").unwrap_or(args);

    let making_amount = source
        .get("making_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing making_amount in Limit v2 create args".into(),
        })?;
    let taking_amount = source
        .get("taking_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing taking_amount in Limit v2 create args".into(),
        })?;
    let unique_id = source.get("unique_id").and_then(|v| v.as_i64());
    let expired_at = source.get("expired_at").and_then(|v| v.as_i64());
    let fee_bps = source
        .get("fee_bps")
        .and_then(|v| v.as_i64())
        .map(|v| v as i16);

    Ok(LimitV2CreateArgs {
        unique_id,
        making_amount,
        taking_amount,
        expired_at,
        fee_bps,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn classify_all_instruction_names() {
        assert_eq!(
            classify_instruction("InitializeOrder"),
            Some(EventType::Created)
        );
        assert_eq!(
            classify_instruction("PreFlashFillOrder"),
            Some(EventType::FillInitiated)
        );
        assert_eq!(
            classify_instruction("FlashFillOrder"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(
            classify_instruction("CancelOrder"),
            Some(EventType::Cancelled)
        );
        assert_eq!(classify_instruction("Unknown"), None);
    }

    #[test]
    fn classify_all_event_names() {
        assert_eq!(classify_event("CreateOrderEvent"), Some(EventType::Created));
        assert_eq!(
            classify_event("CancelOrderEvent"),
            Some(EventType::Cancelled)
        );
        assert_eq!(classify_event("TradeEvent"), Some(EventType::FillCompleted));
        assert_eq!(classify_event("Unknown"), None);
    }

    #[test]
    fn parse_create_args_with_params_wrapper() {
        let args = serde_json::json!({
            "params": {
                "making_amount": 1000,
                "taking_amount": 500,
                "unique_id": 42,
                "expired_at": 1_700_000_000,
                "fee_bps": 25
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
            "making_amount": 2000,
            "taking_amount": 1000
        });
        let parsed = parse_create_args(&args).unwrap();
        assert_eq!(parsed.making_amount, 2000);
        assert_eq!(parsed.taking_amount, 1000);
        assert_eq!(parsed.unique_id, None);
        assert_eq!(parsed.expired_at, None);
        assert_eq!(parsed.fee_bps, None);
    }
}
