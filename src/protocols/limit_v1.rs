use crate::error::Error;
use crate::protocols::{AccountInfo, EventType, find_account_by_name, value_to_pubkey};

pub fn classify_instruction(name: &str) -> Option<EventType> {
    match name {
        "InitializeOrder" => Some(EventType::Created),
        "PreFlashFillOrder" => Some(EventType::FillInitiated),
        "FillOrder" | "FlashFillOrder" => Some(EventType::FillCompleted),
        "CancelOrder" => Some(EventType::Cancelled),
        "CancelExpiredOrder" => Some(EventType::Expired),
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
        "FillOrder" | "PreFlashFillOrder" | "FlashFillOrder" => 0,
        "CancelOrder" | "CancelExpiredOrder" => 0,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown Limit v1 instruction: {instruction_name}"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("Limit v1 account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct LimitV1CreateArgs {
    pub making_amount: i64,
    pub taking_amount: i64,
    pub expired_at: Option<i64>,
}

pub struct LimitV1CreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<LimitV1CreateMints, Error> {
    let by_name_input = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let by_name_output = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
        return Ok(LimitV1CreateMints {
            input_mint,
            output_mint,
        });
    }

    let input_mint = accounts
        .get(5)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Limit v1 input_mint index 5 out of bounds".into(),
        })?;
    let output_mint = accounts
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

pub fn classify_event(name: &str) -> Option<EventType> {
    match name {
        "CreateOrderEvent" => Some(EventType::Created),
        "CancelOrderEvent" => Some(EventType::Cancelled),
        "TradeEvent" => Some(EventType::FillCompleted),
        _ => None,
    }
}

pub fn parse_order_pda_event(
    fields: &serde_json::Value,
    event_name: &str,
) -> Result<String, Error> {
    fields
        .get("order_key")
        .and_then(value_to_pubkey)
        .ok_or_else(|| Error::Protocol {
            reason: format!("missing order_key in {event_name}"),
        })
}

pub struct LimitTradeEvent {
    pub order_pda: String,
    pub taker: String,
    pub in_amount: i64,
    pub out_amount: i64,
    pub remaining_in_amount: i64,
    pub remaining_out_amount: i64,
}

pub fn parse_trade_event(fields: &serde_json::Value) -> Result<LimitTradeEvent, Error> {
    let order_pda = fields
        .get("order_key")
        .and_then(value_to_pubkey)
        .ok_or_else(|| Error::Protocol {
            reason: "missing order_key in Limit v1 TradeEvent".into(),
        })?;
    let taker = fields
        .get("taker")
        .and_then(value_to_pubkey)
        .unwrap_or_else(|| "unknown".to_string());
    let in_amount = fields
        .get("making_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let out_amount = fields
        .get("taking_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let remaining_in_amount = fields
        .get("remaining_making_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let remaining_out_amount = fields
        .get("remaining_taking_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(LimitTradeEvent {
        order_pda,
        taker,
        in_amount,
        out_amount,
        remaining_in_amount,
        remaining_out_amount,
    })
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<LimitV1CreateArgs, Error> {
    let making_amount = args
        .get("making_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing making_amount in Limit v1 create args".into(),
        })?;
    let taking_amount = args
        .get("taking_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing taking_amount in Limit v1 create args".into(),
        })?;
    let expired_at = args.get("expired_at").and_then(|v| v.as_i64());

    Ok(LimitV1CreateArgs {
        making_amount,
        taking_amount,
        expired_at,
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
            classify_instruction("FillOrder"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(
            classify_instruction("FlashFillOrder"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(
            classify_instruction("CancelOrder"),
            Some(EventType::Cancelled)
        );
        assert_eq!(
            classify_instruction("CancelExpiredOrder"),
            Some(EventType::Expired)
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
    fn parse_trade_event_extracts_fill() {
        let fields = serde_json::json!({
            "order_key": "HkLZgYy93cEi3Fn96SvdWeJk8DNeHeU5wiNV5SeRLiJC",
            "taker": "j1oeQoPeuEDmjvyMwBmCWexzCQup77kbKKxV59CnYbd",
            "making_amount": 724_773_829,
            "taking_amount": 51_821_329,
            "remaining_making_amount": 89_147_181_051_i64,
            "remaining_taking_amount": 6_374_023_074_i64
        });
        let trade = parse_trade_event(&fields).unwrap();
        assert_eq!(
            trade.order_pda,
            "HkLZgYy93cEi3Fn96SvdWeJk8DNeHeU5wiNV5SeRLiJC"
        );
        assert_eq!(trade.taker, "j1oeQoPeuEDmjvyMwBmCWexzCQup77kbKKxV59CnYbd");
        assert_eq!(trade.in_amount, 724_773_829);
        assert_eq!(trade.out_amount, 51_821_329);
        assert_eq!(trade.remaining_in_amount, 89_147_181_051);
        assert_eq!(trade.remaining_out_amount, 6_374_023_074);
    }

    #[test]
    fn parse_trade_event_missing_taker_defaults() {
        let fields = serde_json::json!({
            "order_key": "ABC",
            "making_amount": 100,
            "taking_amount": 50,
            "remaining_making_amount": 0,
            "remaining_taking_amount": 0
        });
        let trade = parse_trade_event(&fields).unwrap();
        assert_eq!(trade.taker, "unknown");
    }

    #[test]
    fn parse_trade_event_missing_order_key_errors() {
        let fields = serde_json::json!({
            "taker": "abc",
            "making_amount": 100,
            "taking_amount": 50
        });
        assert!(parse_trade_event(&fields).is_err());
    }
}
