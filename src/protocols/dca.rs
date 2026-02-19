use crate::error::Error;
use crate::protocols::{AccountInfo, EventType, find_account_by_name, value_to_pubkey};

pub fn classify_instruction(name: &str) -> Option<EventType> {
    match name {
        "OpenDca" | "OpenDcaV2" => Some(EventType::Created),
        "InitiateFlashFill" | "InitiateDlmmFill" => Some(EventType::FillInitiated),
        "FulfillFlashFill" | "FulfillDlmmFill" => Some(EventType::FillCompleted),
        "CloseDca" | "EndAndClose" => Some(EventType::Closed),
        _ => None,
    }
}

pub fn extract_order_pda(
    accounts: &[AccountInfo],
    instruction_name: &str,
) -> Result<String, Error> {
    if let Some(acc) = find_account_by_name(accounts, "dca") {
        return Ok(acc.pubkey.clone());
    }

    let idx = match instruction_name {
        "OpenDca" | "OpenDcaV2" => 0,
        "InitiateFlashFill" | "FulfillFlashFill" | "InitiateDlmmFill" | "FulfillDlmmFill" => 1,
        "CloseDca" | "EndAndClose" => 1,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown DCA instruction: {instruction_name}"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct DcaCreateArgs {
    pub in_amount: i64,
    pub in_amount_per_cycle: i64,
    pub cycle_frequency: i64,
    pub min_out_amount: Option<i64>,
    pub max_out_amount: Option<i64>,
    pub start_at: Option<i64>,
}

pub struct DcaCreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(
    accounts: &[AccountInfo],
    instruction_name: &str,
) -> Result<DcaCreateMints, Error> {
    let input_mint = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let output_mint = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (input_mint, output_mint) {
        return Ok(DcaCreateMints {
            input_mint,
            output_mint,
        });
    }

    let (input_idx, output_idx) = match instruction_name {
        "OpenDca" => (2, 3),
        "OpenDcaV2" => (3, 4),
        _ => {
            return Err(Error::Protocol {
                reason: format!("not a DCA create instruction: {instruction_name}"),
            });
        }
    };

    let input_mint = accounts
        .get(input_idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA input_mint index {input_idx} out of bounds"),
        })?;
    let output_mint = accounts
        .get(output_idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("DCA output_mint index {output_idx} out of bounds"),
        })?;

    Ok(DcaCreateMints {
        input_mint,
        output_mint,
    })
}

pub fn classify_event(name: &str) -> Option<EventType> {
    match name {
        "OpenedEvent" => Some(EventType::Created),
        "FilledEvent" => Some(EventType::FillCompleted),
        "ClosedEvent" => Some(EventType::Closed),
        "CollectedFeeEvent" => Some(EventType::FeeCollected),
        "WithdrawEvent" => Some(EventType::Withdrawn),
        "DepositEvent" => Some(EventType::Deposited),
        _ => None,
    }
}

pub fn parse_event_order_pda(
    fields: &serde_json::Value,
    event_name: &str,
) -> Result<String, Error> {
    fields
        .get("dca_key")
        .and_then(value_to_pubkey)
        .ok_or_else(|| Error::Protocol {
            reason: format!("missing dca_key in DCA {event_name}"),
        })
}

pub struct DcaFillEvent {
    pub order_pda: String,
    pub in_amount: i64,
    pub out_amount: i64,
}

pub fn parse_fill_event(fields: &serde_json::Value) -> Result<DcaFillEvent, Error> {
    let order_pda = fields
        .get("dca_key")
        .and_then(value_to_pubkey)
        .ok_or_else(|| Error::Protocol {
            reason: "missing dca_key in DCA FilledEvent".into(),
        })?;
    let in_amount = fields
        .get("in_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let out_amount = fields
        .get("out_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(DcaFillEvent {
        order_pda,
        in_amount,
        out_amount,
    })
}

pub struct DcaClosedEvent {
    pub order_pda: String,
    pub user_closed: bool,
    pub unfilled_amount: i64,
}

pub fn parse_closed_event(fields: &serde_json::Value) -> Result<DcaClosedEvent, Error> {
    let order_pda = fields
        .get("dca_key")
        .and_then(value_to_pubkey)
        .ok_or_else(|| Error::Protocol {
            reason: "missing dca_key in DCA ClosedEvent".into(),
        })?;
    let user_closed = fields
        .get("user_closed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let unfilled_amount = fields
        .get("unfilled_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    Ok(DcaClosedEvent {
        order_pda,
        user_closed,
        unfilled_amount,
    })
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<DcaCreateArgs, Error> {
    let in_amount = args
        .get("in_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing in_amount in DCA create args".into(),
        })?;
    let in_amount_per_cycle = args
        .get("in_amount_per_cycle")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing in_amount_per_cycle in DCA create args".into(),
        })?;
    let cycle_frequency = args
        .get("cycle_frequency")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing cycle_frequency in DCA create args".into(),
        })?;
    let min_out_amount = args.get("min_out_amount").and_then(|v| v.as_i64());
    let max_out_amount = args.get("max_out_amount").and_then(|v| v.as_i64());
    let start_at = args.get("start_at").and_then(|v| v.as_i64());

    Ok(DcaCreateArgs {
        in_amount,
        in_amount_per_cycle,
        cycle_frequency,
        min_out_amount,
        max_out_amount,
        start_at,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::*;

    #[test]
    fn classify_all_instruction_names() {
        assert_eq!(classify_instruction("OpenDca"), Some(EventType::Created));
        assert_eq!(classify_instruction("OpenDcaV2"), Some(EventType::Created));
        assert_eq!(
            classify_instruction("InitiateFlashFill"),
            Some(EventType::FillInitiated)
        );
        assert_eq!(
            classify_instruction("InitiateDlmmFill"),
            Some(EventType::FillInitiated)
        );
        assert_eq!(
            classify_instruction("FulfillFlashFill"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(
            classify_instruction("FulfillDlmmFill"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(classify_instruction("CloseDca"), Some(EventType::Closed));
        assert_eq!(classify_instruction("EndAndClose"), Some(EventType::Closed));
        assert_eq!(classify_instruction("Unknown"), None);
    }

    #[test]
    fn classify_all_event_names() {
        assert_eq!(classify_event("OpenedEvent"), Some(EventType::Created));
        assert_eq!(
            classify_event("FilledEvent"),
            Some(EventType::FillCompleted)
        );
        assert_eq!(classify_event("ClosedEvent"), Some(EventType::Closed));
        assert_eq!(
            classify_event("CollectedFeeEvent"),
            Some(EventType::FeeCollected)
        );
        assert_eq!(classify_event("WithdrawEvent"), Some(EventType::Withdrawn));
        assert_eq!(classify_event("DepositEvent"), Some(EventType::Deposited));
        assert_eq!(classify_event("Unknown"), None);
    }

    #[test]
    fn parse_fill_event_extracts_amounts() {
        let fields = serde_json::json!({
            "dca_key": "3nsTjVJTwwGvXqDRgqNCZAQKwt4QMVhHHqvyseCNR3YX",
            "in_amount": 21_041_666_667_i64,
            "out_amount": 569_529_644,
            "fee": 570_099,
            "fee_mint": "A7b",
            "input_mint": "So1",
            "output_mint": "A7b",
            "user_key": "31o"
        });
        let fill = parse_fill_event(&fields).unwrap();
        assert_eq!(
            fill.order_pda,
            "3nsTjVJTwwGvXqDRgqNCZAQKwt4QMVhHHqvyseCNR3YX"
        );
        assert_eq!(fill.in_amount, 21_041_666_667);
        assert_eq!(fill.out_amount, 569_529_644);
    }

    #[test]
    fn parse_closed_event_extracts_fields() {
        let fields = serde_json::json!({
            "dca_key": "5gadwswXkAacjtyPsPFGhfEtTdyxthUtvwRekfhUgYX",
            "user_closed": true,
            "unfilled_amount": 1_400_000_000_i64,
            "created_at": 0,
            "in_amount_per_cycle": 0,
            "in_deposited": 0,
            "input_mint": "x",
            "output_mint": "y",
            "total_in_withdrawn": 0,
            "total_out_withdrawn": 0,
            "user_key": "z",
            "cycle_frequency": 60
        });
        let closed = parse_closed_event(&fields).unwrap();
        assert_eq!(
            closed.order_pda,
            "5gadwswXkAacjtyPsPFGhfEtTdyxthUtvwRekfhUgYX"
        );
        assert!(closed.user_closed);
        assert_eq!(closed.unfilled_amount, 1_400_000_000);
    }

    #[test]
    fn parse_fill_event_missing_dca_key_errors() {
        let fields = serde_json::json!({"in_amount": 100, "out_amount": 50});
        assert!(parse_fill_event(&fields).is_err());
    }

    #[test]
    fn parse_closed_event_missing_dca_key_errors() {
        let fields = serde_json::json!({"user_closed": true, "unfilled_amount": 0});
        assert!(parse_closed_event(&fields).is_err());
    }
}
