use wasm_bindgen::prelude::*;

use crate::lifecycle::mapping;
use crate::lifecycle::{LifecycleEngine, LifecycleTransition, TerminalStatus};
use crate::protocols::{self, EventType, Protocol};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = JSON)]
    fn parse(s: &str) -> JsValue;
}

fn to_js(value: &serde_json::Value) -> JsValue {
    match serde_json::to_string(value) {
        Ok(json_str) => parse(&json_str),
        Err(_) => JsValue::NULL,
    }
}

fn parse_terminal_status(s: &str) -> Option<TerminalStatus> {
    s.parse::<TerminalStatus>().ok()
}

fn parse_event_type(s: &str) -> Option<EventType> {
    match s {
        "Created" => Some(EventType::Created),
        "FillInitiated" => Some(EventType::FillInitiated),
        "FillCompleted" => Some(EventType::FillCompleted),
        "Cancelled" => Some(EventType::Cancelled),
        "Expired" => Some(EventType::Expired),
        "Closed" => Some(EventType::Closed),
        "FeeCollected" => Some(EventType::FeeCollected),
        "Withdrawn" => Some(EventType::Withdrawn),
        "Deposited" => Some(EventType::Deposited),
        _ => None,
    }
}

fn event_type_to_pascal(et: &EventType) -> &'static str {
    match et {
        EventType::Created => "Created",
        EventType::FillInitiated => "FillInitiated",
        EventType::FillCompleted => "FillCompleted",
        EventType::Cancelled => "Cancelled",
        EventType::Expired => "Expired",
        EventType::Closed => "Closed",
        EventType::FeeCollected => "FeeCollected",
        EventType::Withdrawn => "Withdrawn",
        EventType::Deposited => "Deposited",
    }
}

fn parse_protocol(s: &str) -> Option<Protocol> {
    match s {
        "dca" => Some(Protocol::Dca),
        "limitV1" => Some(Protocol::LimitV1),
        "limitV2" => Some(Protocol::LimitV2),
        "kamino" => Some(Protocol::Kamino),
        _ => None,
    }
}

#[expect(
    clippy::type_complexity,
    reason = "simple lookup, no benefit from type alias"
)]
fn variant_maps_for_protocol(
    protocol: Protocol,
) -> (
    &'static [(&'static str, EventType)],
    &'static [(&'static str, EventType)],
    &'static [&'static str],
) {
    match protocol {
        Protocol::Dca => (
            protocols::dca::INSTRUCTION_EVENT_TYPES,
            protocols::dca::EVENT_EVENT_TYPES,
            protocols::dca::CLOSED_VARIANTS,
        ),
        Protocol::LimitV1 => (
            protocols::limit_v1::INSTRUCTION_EVENT_TYPES,
            protocols::limit_v1::EVENT_EVENT_TYPES,
            protocols::limit_v1::CLOSED_VARIANTS,
        ),
        Protocol::LimitV2 => (
            protocols::limit_v2::INSTRUCTION_EVENT_TYPES,
            protocols::limit_v2::EVENT_EVENT_TYPES,
            protocols::limit_v2::CLOSED_VARIANTS,
        ),
        Protocol::Kamino => (
            protocols::kamino::INSTRUCTION_EVENT_TYPES,
            protocols::kamino::EVENT_EVENT_TYPES,
            protocols::kamino::CLOSED_VARIANTS,
        ),
    }
}

/// Returns protocol configs with instruction/event→EventType mappings.
#[wasm_bindgen]
pub fn get_all_protocols() -> JsValue {
    let protocols = [
        Protocol::Dca,
        Protocol::LimitV1,
        Protocol::LimitV2,
        Protocol::Kamino,
    ];
    let result: Vec<serde_json::Value> = protocols
        .iter()
        .map(|p| {
            let (ix_map, ev_map, closed) = variant_maps_for_protocol(*p);
            let instructions: serde_json::Map<String, serde_json::Value> = ix_map
                .iter()
                .map(|(name, et)| {
                    (
                        (*name).to_string(),
                        serde_json::Value::String(event_type_to_pascal(et).to_string()),
                    )
                })
                .collect();
            let events: serde_json::Map<String, serde_json::Value> = ev_map
                .iter()
                .map(|(name, et)| {
                    (
                        (*name).to_string(),
                        serde_json::Value::String(event_type_to_pascal(et).to_string()),
                    )
                })
                .collect();
            let closed_list: Vec<serde_json::Value> = closed
                .iter()
                .map(|s| serde_json::Value::String((*s).to_string()))
                .collect();
            serde_json::json!({
                "id": p.as_ref(),
                "programId": p.program_id_str(),
                "instructions": instructions,
                "events": events,
                "closedVariants": closed_list,
            })
        })
        .collect();
    to_js(&serde_json::Value::Array(result))
}

/// Classify a JSON payload against a protocol's known variant names.
#[wasm_bindgen]
pub fn classify_json(protocol: &str, json: &str) -> JsValue {
    let Some(proto) = parse_protocol(protocol) else {
        return error_result("Unknown protocol");
    };

    let parsed: serde_json::Value = match serde_json::from_str(json) {
        Ok(v) => v,
        Err(_) => return error_result("Invalid JSON"),
    };

    let obj = match parsed.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return error_result("Expected a JSON object with a variant key"),
    };

    let variant_name = match obj.keys().next() {
        Some(k) => k.clone(),
        None => return error_result("Empty JSON object"),
    };

    let (ix_map, ev_map, _) = variant_maps_for_protocol(proto);

    if let Some(et) = ev_map
        .iter()
        .find(|(n, _)| *n == variant_name)
        .map(|(_, et)| et)
    {
        let transition = mapping::event_type_to_transition(et, None);
        let result = serde_json::json!({
            "variantName": variant_name,
            "source": "event",
            "eventType": event_type_to_pascal(et),
            "transition": mapping::transition_to_display(&transition),
            "decision": format!("{:?}", LifecycleEngine::decide_transition(None, transition)),
        });
        return to_js(&result);
    }

    if let Some(et) = ix_map
        .iter()
        .find(|(n, _)| *n == variant_name)
        .map(|(_, et)| et)
    {
        let transition = mapping::event_type_to_transition(et, None);
        let result = serde_json::json!({
            "variantName": variant_name,
            "source": "instruction",
            "eventType": event_type_to_pascal(et),
            "transition": mapping::transition_to_display(&transition),
            "decision": format!("{:?}", LifecycleEngine::decide_transition(None, transition)),
        });
        return to_js(&result);
    }

    let all_events: Vec<&str> = ev_map.iter().map(|(n, _)| *n).collect();
    let all_ix: Vec<&str> = ix_map.iter().map(|(n, _)| *n).collect();
    error_result(&format!(
        "Unknown variant \"{variant_name}\". Known events: {}. Known instructions: {}.",
        all_events.join(", "),
        all_ix.join(", ")
    ))
}

/// Decide whether a transition should be applied given current terminal status.
#[wasm_bindgen]
pub fn decide_transition(
    current_terminal: Option<String>,
    transition_type: &str,
    close_status: Option<String>,
) -> String {
    let terminal = current_terminal.as_deref().and_then(parse_terminal_status);

    let transition = match transition_type {
        "Create" => LifecycleTransition::Create,
        "FillDelta" => LifecycleTransition::FillDelta,
        "MetadataOnly" => LifecycleTransition::MetadataOnly,
        "Close" => {
            let status = close_status
                .as_deref()
                .and_then(parse_terminal_status)
                .unwrap_or(TerminalStatus::Completed);
            LifecycleTransition::Close { status }
        }
        _ => return "IgnoreTerminalViolation".to_string(),
    };

    format!(
        "{:?}",
        LifecycleEngine::decide_transition(terminal, transition)
    )
}

/// Normalize a cumulative snapshot into a delta.
#[wasm_bindgen]
pub fn normalize_snapshot(stored: f64, snapshot: f64) -> JsValue {
    let result = LifecycleEngine::normalize_snapshot_to_delta(stored as i64, snapshot as i64);
    let obj = serde_json::json!({
        "delta": result.delta,
        "regression": result.regression,
    });
    to_js(&obj)
}

/// Map an EventType to a LifecycleTransition.
#[wasm_bindgen]
pub fn wasm_event_type_to_transition(event_type: &str, closed_status: Option<String>) -> JsValue {
    let Some(et) = parse_event_type(event_type) else {
        return JsValue::NULL;
    };
    let cs = closed_status.as_deref().and_then(parse_terminal_status);
    let transition = mapping::event_type_to_transition(&et, cs);
    let obj = match &transition {
        LifecycleTransition::Close { status } => serde_json::json!({
            "type": "Close",
            "status": status.as_ref(),
        }),
        LifecycleTransition::Create => serde_json::json!({"type": "Create"}),
        LifecycleTransition::FillDelta => serde_json::json!({"type": "FillDelta"}),
        LifecycleTransition::MetadataOnly => serde_json::json!({"type": "MetadataOnly"}),
    };
    to_js(&obj)
}

/// Check if a status string is terminal.
#[wasm_bindgen]
pub fn is_terminal(status: Option<String>) -> bool {
    status.as_deref().and_then(parse_terminal_status).is_some()
}

/// Get the display string for a transition.
#[wasm_bindgen]
pub fn wasm_transition_to_string(transition_type: &str, close_status: Option<String>) -> String {
    let transition = match transition_type {
        "Create" => LifecycleTransition::Create,
        "FillDelta" => LifecycleTransition::FillDelta,
        "MetadataOnly" => LifecycleTransition::MetadataOnly,
        "Close" => {
            let status = close_status
                .as_deref()
                .and_then(parse_terminal_status)
                .unwrap_or(TerminalStatus::Completed);
            LifecycleTransition::Close { status }
        }
        _ => return transition_type.to_string(),
    };
    mapping::transition_to_display(&transition)
}

/// Get the target status of a transition, or null.
#[wasm_bindgen]
pub fn wasm_transition_target(
    transition_type: &str,
    close_status: Option<String>,
) -> Option<String> {
    let transition = match transition_type {
        "Create" => LifecycleTransition::Create,
        "FillDelta" => LifecycleTransition::FillDelta,
        "MetadataOnly" => LifecycleTransition::MetadataOnly,
        "Close" => {
            let status = close_status
                .as_deref()
                .and_then(parse_terminal_status)
                .unwrap_or(TerminalStatus::Completed);
            LifecycleTransition::Close { status }
        }
        _ => return None,
    };
    mapping::transition_target(&transition).map(str::to_string)
}

fn error_result(msg: &str) -> JsValue {
    let obj = serde_json::json!({"error": msg});
    to_js(&obj)
}
