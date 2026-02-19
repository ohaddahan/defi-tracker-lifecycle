use crate::error::Error;
use crate::protocols::{AccountInfo, EventType, find_account_by_name};

pub fn classify_instruction(name: &str) -> Option<EventType> {
    match name {
        "CreateOrder" => Some(EventType::Created),
        "TakeOrder" => Some(EventType::FillCompleted),
        "FlashTakeOrderStart" => Some(EventType::FillInitiated),
        "FlashTakeOrderEnd" => Some(EventType::FillCompleted),
        "CloseOrderAndClaimTip" => Some(EventType::Closed),
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
        "CreateOrder" => 3,
        "TakeOrder" => 4,
        "FlashTakeOrderStart" | "FlashTakeOrderEnd" => 4,
        "CloseOrderAndClaimTip" => 1,
        _ => {
            return Err(Error::Protocol {
                reason: format!("unknown Kamino instruction: {instruction_name}"),
            });
        }
    };

    accounts
        .get(idx)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: format!("Kamino account index {idx} out of bounds for {instruction_name}"),
        })
}

pub struct KaminoCreateArgs {
    pub input_amount: i64,
    pub output_amount: i64,
    pub order_type: i16,
}

pub struct KaminoCreateMints {
    pub input_mint: String,
    pub output_mint: String,
}

pub fn extract_create_mints(accounts: &[AccountInfo]) -> Result<KaminoCreateMints, Error> {
    let by_name_input = find_account_by_name(accounts, "input_mint").map(|a| a.pubkey.clone());
    let by_name_output = find_account_by_name(accounts, "output_mint").map(|a| a.pubkey.clone());

    if let (Some(input_mint), Some(output_mint)) = (by_name_input, by_name_output) {
        return Ok(KaminoCreateMints {
            input_mint,
            output_mint,
        });
    }

    let input_mint = accounts
        .get(4)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Kamino input_mint index 4 out of bounds".into(),
        })?;
    let output_mint = accounts
        .get(5)
        .map(|a| a.pubkey.clone())
        .ok_or_else(|| Error::Protocol {
            reason: "Kamino output_mint index 5 out of bounds".into(),
        })?;

    Ok(KaminoCreateMints {
        input_mint,
        output_mint,
    })
}

pub fn parse_create_args(args: &serde_json::Value) -> Result<KaminoCreateArgs, Error> {
    let input_amount = args
        .get("input_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing input_amount in Kamino create args".into(),
        })?;
    let output_amount = args
        .get("output_amount")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| Error::Protocol {
            reason: "missing output_amount in Kamino create args".into(),
        })?;
    let order_type = args.get("order_type").and_then(|v| v.as_i64()).unwrap_or(0) as i16;

    Ok(KaminoCreateArgs {
        input_amount,
        output_amount,
        order_type,
    })
}

pub fn classify_event(name: &str) -> Option<EventType> {
    match name {
        "OrderDisplayEvent" => Some(EventType::FillCompleted),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KaminoDisplayStatus {
    Open,
    Filled,
    Cancelled,
    Expired,
}

pub fn parse_display_status(status: i64) -> Result<KaminoDisplayStatus, Error> {
    match status {
        0 => Ok(KaminoDisplayStatus::Open),
        1 => Ok(KaminoDisplayStatus::Filled),
        2 => Ok(KaminoDisplayStatus::Cancelled),
        3 => Ok(KaminoDisplayStatus::Expired),
        _ => Err(Error::Protocol {
            reason: format!("unknown Kamino display status code: {status}"),
        }),
    }
}

pub struct KaminoOrderDisplayEvent {
    pub remaining_input_amount: i64,
    pub filled_output_amount: i64,
    pub number_of_fills: i64,
    pub status: i64,
}

pub fn parse_order_display_event(
    fields: &serde_json::Value,
) -> Result<KaminoOrderDisplayEvent, Error> {
    let remaining_input_amount = fields
        .get("remaining_input_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let filled_output_amount = fields
        .get("filled_output_amount")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let number_of_fills = fields
        .get("number_of_fills")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let status = fields.get("status").and_then(|v| v.as_i64()).unwrap_or(0);

    Ok(KaminoOrderDisplayEvent {
        remaining_input_amount,
        filled_output_amount,
        number_of_fills,
        status,
    })
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    use super::{
        KaminoDisplayStatus, classify_instruction, extract_order_pda, parse_display_status,
        parse_order_display_event,
    };
    use crate::protocols::{AccountInfo, EventType};

    #[test]
    fn flash_take_instructions_are_classified() {
        assert_eq!(
            classify_instruction("FlashTakeOrderStart"),
            Some(EventType::FillInitiated)
        );
        assert_eq!(
            classify_instruction("FlashTakeOrderEnd"),
            Some(EventType::FillCompleted)
        );
    }

    #[test]
    fn extract_order_pda_supports_flash_take_fallback_index() {
        let order = "7ee61prCHrJwHNYJKojPu1GcyWDjQEUTeuRT4gbA1EDo";
        let accounts = vec![
            AccountInfo {
                pubkey: "a".to_string(),
                is_signer: true,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "b".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "c".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: "d".to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
            AccountInfo {
                pubkey: order.to_string(),
                is_signer: false,
                is_writable: false,
                name: None,
            },
        ];

        let extracted = extract_order_pda(&accounts, "FlashTakeOrderEnd").unwrap();
        assert_eq!(extracted, order);
    }

    #[test]
    fn parse_display_event_reads_status_field() {
        let fields = serde_json::json!({
            "remaining_input_amount": 0,
            "filled_output_amount": 11_744_711,
            "number_of_fills": 1,
            "status": 1
        });
        let display = parse_order_display_event(&fields).unwrap();
        assert_eq!(display.remaining_input_amount, 0);
        assert_eq!(display.status, 1);
    }

    #[test]
    fn parses_known_display_status_codes() {
        assert_eq!(parse_display_status(0).unwrap(), KaminoDisplayStatus::Open);
        assert_eq!(
            parse_display_status(1).unwrap(),
            KaminoDisplayStatus::Filled
        );
        assert_eq!(
            parse_display_status(2).unwrap(),
            KaminoDisplayStatus::Cancelled
        );
        assert_eq!(
            parse_display_status(3).unwrap(),
            KaminoDisplayStatus::Expired
        );
    }

    #[test]
    fn rejects_unknown_display_status_codes() {
        assert!(parse_display_status(99).is_err());
    }
}
