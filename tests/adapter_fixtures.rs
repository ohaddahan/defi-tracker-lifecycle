#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test code uses unwrap/panic for concise assertions"
)]

use defi_tracker_lifecycle::{
    CorrelationOutcome, EventPayload, EventType, LifecycleEngine, LifecycleTransition, Protocol,
    RawEvent, RawInstruction, ResolveContext, TerminalStatus, TransitionDecision, adapter_for,
};

fn load_instructions(filename: &str) -> Vec<RawInstruction> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/fixtures/{filename}");
    let data =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

fn load_events(filename: &str) -> Vec<RawEvent> {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let path = format!("{manifest_dir}/tests/fixtures/{filename}");
    let data =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("failed to read {path}: {e}"));
    serde_json::from_str(&data).unwrap_or_else(|e| panic!("failed to parse {path}: {e}"))
}

fn resolve_event_from_fixture(
    protocol: Protocol,
    ev: &RawEvent,
    ctx: &ResolveContext,
) -> (EventType, CorrelationOutcome, EventPayload) {
    let adapter = adapter_for(protocol);
    adapter
        .classify_and_resolve_event(ev, ctx)
        .unwrap_or_else(|| panic!("unclassified event: {}", ev.event_name))
        .unwrap_or_else(|e| panic!("resolve failed for {}: {e}", ev.event_name))
}

fn no_context() -> ResolveContext {
    ResolveContext {
        pre_fetched_order_pdas: None,
    }
}

// ──────────────────── DCA ────────────────────

#[test]
fn dca_classify_instructions_from_fixture() {
    let adapter = adapter_for(Protocol::Dca);
    let expected: &[(&str, EventType)] = &[
        ("OpenDcaV2", EventType::Created),
        ("InitiateFlashFill", EventType::FillInitiated),
        ("FulfillFlashFill", EventType::FillCompleted),
        ("CloseDca", EventType::Closed),
        ("EndAndClose", EventType::Closed),
    ];

    let instructions = load_instructions("dca_instructions.json");
    for (name, event_type) in expected {
        let ix = instructions
            .iter()
            .find(|ix| ix.instruction_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        assert_eq!(
            adapter.classify_instruction(ix),
            Some(event_type.clone()),
            "wrong classification for {name}"
        );
    }
}

#[test]
fn dca_classify_events_from_fixture() {
    let expected: &[(&str, EventType)] = &[
        ("OpenedEvent", EventType::Created),
        ("FilledEvent", EventType::FillCompleted),
        ("ClosedEvent", EventType::Closed),
        ("CollectedFeeEvent", EventType::FeeCollected),
        ("WithdrawEvent", EventType::Withdrawn),
    ];

    let events = load_events("dca_events.json");
    for (name, event_type) in expected {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        let (et, _, _) = resolve_event_from_fixture(Protocol::Dca, ev, &no_context());
        assert_eq!(et, *event_type, "wrong classification for {name}");
    }
}

#[test]
fn dca_malformed_known_event_surfaces_error() {
    let adapter = adapter_for(Protocol::Dca);
    let ev = make_event(
        "FilledEvent",
        serde_json::json!({
            "FilledEvent": {
                "dca_key": "order",
                "in_amount": "bad",
                "out_amount": 1_u64
            }
        }),
    );
    let result = adapter.classify_and_resolve_event(&ev, &no_context());
    assert!(matches!(result, Some(Err(_))));
}

#[test]
fn dca_resolve_fill_event_extracts_amounts() {
    let events = load_events("dca_events.json");
    let ev = events
        .iter()
        .find(|ev| ev.event_name == "FilledEvent")
        .unwrap();

    let (event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

    assert_eq!(event_type, EventType::FillCompleted);

    let CorrelationOutcome::Correlated(pdas) = &correlation else {
        panic!("expected Correlated, got {correlation:?}");
    };
    assert_eq!(pdas.len(), 1);
    assert!(!pdas[0].is_empty());

    let EventPayload::DcaFill {
        in_amount,
        out_amount,
    } = payload
    else {
        panic!("expected DcaFill, got {payload:?}");
    };
    assert!(in_amount > 0, "in_amount should be positive");
    assert!(out_amount > 0, "out_amount should be positive");
}

#[test]
fn dca_resolve_closed_completed() {
    let events = load_events("dca_events.json");
    let ev = events
        .iter()
        .filter(|ev| ev.event_name == "ClosedEvent")
        .find(|ev| {
            let fields = ev.fields.as_ref().unwrap();
            let inner = &fields["ClosedEvent"];
            !inner["user_closed"].as_bool().unwrap_or(false)
                && inner["unfilled_amount"].as_i64().unwrap_or(1) == 0
        })
        .unwrap();

    let (_event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

    assert!(matches!(correlation, CorrelationOutcome::Correlated(_)));
    assert_eq!(
        payload,
        EventPayload::DcaClosed {
            status: TerminalStatus::Completed
        }
    );
}

#[test]
fn dca_resolve_closed_user_cancelled() {
    let events = load_events("dca_events.json");
    let ev = events
        .iter()
        .filter(|ev| ev.event_name == "ClosedEvent")
        .find(|ev| {
            let fields = ev.fields.as_ref().unwrap();
            let inner = &fields["ClosedEvent"];
            inner["user_closed"].as_bool().unwrap_or(false)
        })
        .unwrap();

    let (_event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

    assert!(matches!(correlation, CorrelationOutcome::Correlated(_)));
    assert_eq!(
        payload,
        EventPayload::DcaClosed {
            status: TerminalStatus::Cancelled
        }
    );
}

#[test]
fn dca_resolve_other_events_produce_none_payload() {
    let events = load_events("dca_events.json");
    let other_event_names = ["OpenedEvent", "CollectedFeeEvent", "WithdrawEvent"];

    for name in other_event_names {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));

        let (_event_type, correlation, payload) =
            resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

        assert!(
            matches!(correlation, CorrelationOutcome::Correlated(_)),
            "{name} should correlate via dca_key"
        );
        assert_eq!(
            payload,
            EventPayload::None,
            "{name} should have None payload"
        );
    }
}

// ──────────────────── Kamino ────────────────────

#[test]
fn kamino_classify_instructions_from_fixture() {
    let adapter = adapter_for(Protocol::Kamino);
    let expected: &[(&str, EventType)] = &[
        ("CreateOrder", EventType::Created),
        ("TakeOrder", EventType::FillCompleted),
        ("FlashTakeOrderStart", EventType::FillInitiated),
        ("FlashTakeOrderEnd", EventType::FillCompleted),
        ("CloseOrderAndClaimTip", EventType::Closed),
    ];

    let instructions = load_instructions("kamino_instructions.json");
    for (name, event_type) in expected {
        let ix = instructions
            .iter()
            .find(|ix| ix.instruction_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        assert_eq!(
            adapter.classify_instruction(ix),
            Some(event_type.clone()),
            "wrong classification for {name}"
        );
    }
}

#[test]
fn kamino_classify_events_from_fixture() {
    let events = load_events("kamino_events.json");
    let pda = "FakeOrderPda".to_string();
    let ctx = ResolveContext {
        pre_fetched_order_pdas: Some(vec![pda]),
    };

    for ev in &events {
        let (event_type, _, _) = resolve_event_from_fixture(Protocol::Kamino, ev, &ctx);
        assert_eq!(
            event_type,
            EventType::FillCompleted,
            "OrderDisplayEvent should always be FillCompleted"
        );
    }
}

#[test]
fn kamino_resolve_display_event_with_pdas() {
    let events = load_events("kamino_events.json");
    let ev = &events[0];

    let pda = "FakeOrderPda1111111111111111111111111111111".to_string();
    let ctx = ResolveContext {
        pre_fetched_order_pdas: Some(vec![pda.clone()]),
    };

    let (_event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::Kamino, ev, &ctx);

    assert_eq!(correlation, CorrelationOutcome::Correlated(vec![pda]));

    let EventPayload::KaminoDisplay {
        remaining_input_amount,
        filled_output_amount,
        terminal_status,
    } = payload
    else {
        panic!("expected KaminoDisplay, got {payload:?}");
    };
    assert!(remaining_input_amount >= 0);
    assert!(filled_output_amount >= 0);
    assert!(terminal_status.is_some(), "status 2 should be terminal");
}

#[test]
fn kamino_resolve_display_event_without_pdas() {
    let events = load_events("kamino_events.json");
    let ev = &events[0];

    let (_event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::Kamino, ev, &no_context());

    assert!(
        matches!(correlation, CorrelationOutcome::Uncorrelated { .. }),
        "should be Uncorrelated without pre_fetched_order_pdas"
    );
    assert_eq!(payload, EventPayload::None);
}

#[test]
fn kamino_malformed_known_event_surfaces_error() {
    let adapter = adapter_for(Protocol::Kamino);
    let ev = make_event(
        "OrderDisplayEvent",
        serde_json::json!({
            "OrderDisplayEvent": {
                "remaining_input_amount": "bad",
                "filled_output_amount": 1_u64,
                "status": 1_u8
            }
        }),
    );
    let ctx = ResolveContext {
        pre_fetched_order_pdas: Some(vec!["pda".to_string()]),
    };
    let result = adapter.classify_and_resolve_event(&ev, &ctx);
    assert!(matches!(result, Some(Err(_))));
}

// ──────────────────── Limit V1 ────────────────────

#[test]
fn limit_v1_classify_instructions_from_fixture() {
    let adapter = adapter_for(Protocol::LimitV1);
    let instructions = load_instructions("limit_v1_instructions.json");
    let cancel = instructions
        .iter()
        .find(|ix| ix.instruction_name == "CancelOrder")
        .unwrap_or_else(|| panic!("missing CancelOrder fixture"));
    assert_eq!(
        adapter.classify_instruction(cancel),
        Some(EventType::Cancelled),
        "CancelOrder should classify as Cancelled"
    );
}

#[test]
fn limit_v1_classify_events_from_fixture() {
    let expected: &[(&str, EventType)] = &[
        ("CreateOrderEvent", EventType::Created),
        ("CancelOrderEvent", EventType::Cancelled),
        ("TradeEvent", EventType::FillCompleted),
    ];

    let events = load_events("limit_v1_events.json");
    for (name, event_type) in expected {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        let (et, _, _) = resolve_event_from_fixture(Protocol::LimitV1, ev, &no_context());
        assert_eq!(et, *event_type, "wrong classification for {name}");
    }
}

#[test]
fn limit_v1_resolve_trade_event_extracts_fill() {
    let events = load_events("limit_v1_events.json");
    let ev = events
        .iter()
        .find(|ev| ev.event_name == "TradeEvent")
        .unwrap();

    let (event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::LimitV1, ev, &no_context());

    assert_eq!(event_type, EventType::FillCompleted);

    let CorrelationOutcome::Correlated(pdas) = &correlation else {
        panic!("expected Correlated, got {correlation:?}");
    };
    assert_eq!(pdas.len(), 1);
    assert!(!pdas[0].is_empty());

    let EventPayload::LimitFill {
        in_amount,
        out_amount,
        remaining_in_amount,
        counterparty,
    } = &payload
    else {
        panic!("expected LimitFill, got {payload:?}");
    };
    assert!(*in_amount > 0, "in_amount should be positive");
    assert!(*out_amount > 0, "out_amount should be positive");
    assert!(
        *remaining_in_amount >= 0,
        "remaining should be non-negative"
    );
    assert!(!counterparty.is_empty(), "counterparty should be present");
}

#[test]
fn limit_v1_resolve_create_and_cancel_events() {
    let events = load_events("limit_v1_events.json");

    for name in ["CreateOrderEvent", "CancelOrderEvent"] {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));

        let (_event_type, correlation, payload) =
            resolve_event_from_fixture(Protocol::LimitV1, ev, &no_context());

        assert!(
            matches!(correlation, CorrelationOutcome::Correlated(_)),
            "{name} should correlate via order_key"
        );
        assert_eq!(
            payload,
            EventPayload::None,
            "{name} should have None payload"
        );
    }
}

// ──────────────────── Limit V2 ────────────────────

#[test]
fn limit_v2_classify_instructions_from_fixture() {
    let adapter = adapter_for(Protocol::LimitV2);
    let expected: &[(&str, EventType)] = &[
        ("InitializeOrder", EventType::Created),
        ("PreFlashFillOrder", EventType::FillInitiated),
        ("FlashFillOrder", EventType::FillCompleted),
        ("CancelOrder", EventType::Cancelled),
    ];

    let instructions = load_instructions("limit_v2_instructions.json");
    for (name, event_type) in expected {
        let ix = instructions
            .iter()
            .find(|ix| ix.instruction_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        assert_eq!(
            adapter.classify_instruction(ix),
            Some(event_type.clone()),
            "wrong classification for {name}"
        );
    }
}

#[test]
fn limit_v2_classify_events_from_fixture() {
    let expected: &[(&str, EventType)] = &[
        ("CreateOrderEvent", EventType::Created),
        ("CancelOrderEvent", EventType::Cancelled),
        ("TradeEvent", EventType::FillCompleted),
    ];

    let events = load_events("limit_v2_events.json");
    for (name, event_type) in expected {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == *name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));
        let (et, _, _) = resolve_event_from_fixture(Protocol::LimitV2, ev, &no_context());
        assert_eq!(et, *event_type, "wrong classification for {name}");
    }
}

#[test]
fn limit_v2_resolve_trade_event_extracts_fill() {
    let events = load_events("limit_v2_events.json");
    let ev = events
        .iter()
        .find(|ev| ev.event_name == "TradeEvent")
        .unwrap();

    let (event_type, correlation, payload) =
        resolve_event_from_fixture(Protocol::LimitV2, ev, &no_context());

    assert_eq!(event_type, EventType::FillCompleted);

    let CorrelationOutcome::Correlated(pdas) = &correlation else {
        panic!("expected Correlated, got {correlation:?}");
    };
    assert_eq!(pdas.len(), 1);
    assert!(!pdas[0].is_empty());

    let EventPayload::LimitFill {
        in_amount,
        out_amount,
        remaining_in_amount,
        counterparty,
    } = &payload
    else {
        panic!("expected LimitFill, got {payload:?}");
    };
    assert!(*in_amount > 0, "in_amount should be positive");
    assert!(*out_amount > 0, "out_amount should be positive");
    assert!(
        *remaining_in_amount >= 0,
        "remaining should be non-negative"
    );
    assert!(!counterparty.is_empty(), "counterparty should be present");
}

#[test]
fn limit_v2_resolve_create_and_cancel_events() {
    let events = load_events("limit_v2_events.json");

    for name in ["CreateOrderEvent", "CancelOrderEvent"] {
        let ev = events
            .iter()
            .find(|ev| ev.event_name == name)
            .unwrap_or_else(|| panic!("missing fixture for {name}"));

        let (_event_type, correlation, payload) =
            resolve_event_from_fixture(Protocol::LimitV2, ev, &no_context());

        assert!(
            matches!(correlation, CorrelationOutcome::Correlated(_)),
            "{name} should correlate via order_key"
        );
        assert_eq!(
            payload,
            EventPayload::None,
            "{name} should have None payload"
        );
    }
}

// ──────────────────── End-to-End Lifecycle ────────────────────
//
// These tests bridge the adapter layer (raw JSON → EventType + Correlation + EventPayload)
// with the lifecycle state machine (LifecycleTransition → TransitionDecision).
// The EventType→LifecycleTransition mapping mirrors what the parent defi-tracker
// crate does at runtime.

fn event_type_to_transition(event_type: &EventType, payload: &EventPayload) -> LifecycleTransition {
    match event_type {
        EventType::Created => LifecycleTransition::Create,
        EventType::FillCompleted | EventType::FillInitiated => LifecycleTransition::FillDelta,
        EventType::Cancelled => LifecycleTransition::Close {
            status: TerminalStatus::Cancelled,
        },
        EventType::Expired => LifecycleTransition::Close {
            status: TerminalStatus::Expired,
        },
        EventType::Closed => {
            let status = match payload {
                EventPayload::DcaClosed { status } => *status,
                EventPayload::KaminoDisplay {
                    terminal_status: Some(s),
                    ..
                } => *s,
                _ => TerminalStatus::Completed,
            };
            LifecycleTransition::Close { status }
        }
        EventType::FeeCollected | EventType::Withdrawn | EventType::Deposited => {
            LifecycleTransition::MetadataOnly
        }
    }
}

fn event_to_transition(
    event_type: &EventType,
    correlation: &CorrelationOutcome,
    payload: &EventPayload,
) -> LifecycleTransition {
    if matches!(correlation, CorrelationOutcome::NotRequired) {
        return LifecycleTransition::MetadataOnly;
    }
    event_type_to_transition(event_type, payload)
}

fn make_event(name: &str, fields: serde_json::Value) -> RawEvent {
    RawEvent {
        id: 1,
        signature: "test_sig".to_string(),
        event_index: 0,
        program_id: "p".to_string(),
        inner_program_id: "p".to_string(),
        event_name: name.to_string(),
        fields: Some(fields),
        slot: 1,
    }
}

struct LifecycleState {
    status: Option<String>,
    protocol: Protocol,
}

impl LifecycleState {
    fn new(protocol: Protocol) -> Self {
        Self {
            status: None,
            protocol,
        }
    }

    fn apply_event(&mut self, ev: &RawEvent, ctx: &ResolveContext) -> TransitionDecision {
        let adapter = adapter_for(self.protocol);
        let (event_type, correlation, payload) = adapter
            .classify_and_resolve_event(ev, ctx)
            .unwrap_or_else(|| panic!("unclassified event: {}", ev.event_name))
            .unwrap_or_else(|e| panic!("resolve failed: {e}"));

        let transition = event_to_transition(&event_type, &correlation, &payload);
        let decision = LifecycleEngine::decide_transition(self.status.as_deref(), transition);

        if decision == TransitionDecision::Apply {
            self.status = match transition {
                LifecycleTransition::Create => Some("created".to_string()),
                LifecycleTransition::FillDelta => self.status.take().or(Some("active".to_string())),
                LifecycleTransition::Close { status } => Some(status.as_str().to_string()),
                LifecycleTransition::MetadataOnly => self.status.take(),
            };
        }

        decision
    }

    fn apply_instruction(&mut self, ix: &RawInstruction) -> Option<TransitionDecision> {
        let adapter = adapter_for(self.protocol);
        let event_type = adapter.classify_instruction(ix)?;
        let transition = event_type_to_transition(&event_type, &EventPayload::None);
        let decision = LifecycleEngine::decide_transition(self.status.as_deref(), transition);

        if decision == TransitionDecision::Apply {
            self.status = match transition {
                LifecycleTransition::Create => Some("created".to_string()),
                LifecycleTransition::FillDelta => self.status.take().or(Some("active".to_string())),
                LifecycleTransition::Close { status } => Some(status.as_str().to_string()),
                LifecycleTransition::MetadataOnly => self.status.take(),
            };
        }

        Some(decision)
    }
}

#[test]
fn lifecycle_dca_create_fill_fill_close_completed() {
    let mut state = LifecycleState::new(Protocol::Dca);
    let ctx = no_context();

    let opened = make_event(
        "OpenedEvent",
        serde_json::json!({"OpenedEvent": {"dca_key": "order1"}}),
    );
    assert_eq!(state.apply_event(&opened, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("created"));

    let fill1 = make_event(
        "FilledEvent",
        serde_json::json!({"FilledEvent": {"dca_key": "order1", "in_amount": 1000_u64, "out_amount": 50_u64}}),
    );
    assert_eq!(state.apply_event(&fill1, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("created"));

    let fill2 = make_event(
        "FilledEvent",
        serde_json::json!({"FilledEvent": {"dca_key": "order1", "in_amount": 1000_u64, "out_amount": 50_u64}}),
    );
    assert_eq!(state.apply_event(&fill2, &ctx), TransitionDecision::Apply);

    let closed = make_event(
        "ClosedEvent",
        serde_json::json!({"ClosedEvent": {"dca_key": "order1", "user_closed": false, "unfilled_amount": 0_u64}}),
    );
    assert_eq!(state.apply_event(&closed, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("completed"));

    let late_fill = make_event(
        "FilledEvent",
        serde_json::json!({"FilledEvent": {"dca_key": "order1", "in_amount": 500_u64, "out_amount": 25_u64}}),
    );
    assert_eq!(
        state.apply_event(&late_fill, &ctx),
        TransitionDecision::IgnoreTerminalViolation
    );
    assert_eq!(state.status.as_deref(), Some("completed"));
}

#[test]
fn lifecycle_dca_create_fill_user_cancel() {
    let mut state = LifecycleState::new(Protocol::Dca);
    let ctx = no_context();

    let opened = make_event(
        "OpenedEvent",
        serde_json::json!({"OpenedEvent": {"dca_key": "order2"}}),
    );
    assert_eq!(state.apply_event(&opened, &ctx), TransitionDecision::Apply);

    let fill = make_event(
        "FilledEvent",
        serde_json::json!({"FilledEvent": {"dca_key": "order2", "in_amount": 500_u64, "out_amount": 25_u64}}),
    );
    assert_eq!(state.apply_event(&fill, &ctx), TransitionDecision::Apply);

    let closed = make_event(
        "ClosedEvent",
        serde_json::json!({"ClosedEvent": {"dca_key": "order2", "user_closed": true, "unfilled_amount": 500_u64}}),
    );
    assert_eq!(state.apply_event(&closed, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("cancelled"));

    let late_create = make_event(
        "OpenedEvent",
        serde_json::json!({"OpenedEvent": {"dca_key": "order2"}}),
    );
    assert_eq!(
        state.apply_event(&late_create, &ctx),
        TransitionDecision::IgnoreTerminalViolation
    );
    assert_eq!(state.status.as_deref(), Some("cancelled"));
}

#[test]
fn lifecycle_dca_create_close_expired_then_metadata_still_works() {
    let mut state = LifecycleState::new(Protocol::Dca);
    let ctx = no_context();

    let opened = make_event(
        "OpenedEvent",
        serde_json::json!({"OpenedEvent": {"dca_key": "order3"}}),
    );
    state.apply_event(&opened, &ctx);

    let closed = make_event(
        "ClosedEvent",
        serde_json::json!({"ClosedEvent": {"dca_key": "order3", "user_closed": false, "unfilled_amount": 1000_u64}}),
    );
    assert_eq!(state.apply_event(&closed, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("expired"));

    let fee = make_event(
        "CollectedFeeEvent",
        serde_json::json!({"CollectedFeeEvent": {"dca_key": "order3"}}),
    );
    assert_eq!(state.apply_event(&fee, &ctx), TransitionDecision::Apply);
    assert_eq!(
        state.status.as_deref(),
        Some("expired"),
        "metadata-only should not change terminal status"
    );
}

#[test]
fn lifecycle_limit_v1_create_fill_cancel() {
    let mut state = LifecycleState::new(Protocol::LimitV1);
    let ctx = no_context();

    let create = make_event(
        "CreateOrderEvent",
        serde_json::json!({"CreateOrderEvent": {"order_key": "lv1_order"}}),
    );
    assert_eq!(state.apply_event(&create, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("created"));

    let trade = make_event(
        "TradeEvent",
        serde_json::json!({"TradeEvent": {
            "order_key": "lv1_order", "taker": "taker1",
            "in_amount": 100_u64, "out_amount": 10_u64,
            "remaining_in_amount": 900_u64, "remaining_out_amount": 90_u64
        }}),
    );
    assert_eq!(state.apply_event(&trade, &ctx), TransitionDecision::Apply);

    let cancel = make_event(
        "CancelOrderEvent",
        serde_json::json!({"CancelOrderEvent": {"order_key": "lv1_order"}}),
    );
    assert_eq!(state.apply_event(&cancel, &ctx), TransitionDecision::Apply);
    assert_eq!(state.status.as_deref(), Some("cancelled"));

    let late_trade = make_event(
        "TradeEvent",
        serde_json::json!({"TradeEvent": {
            "order_key": "lv1_order", "taker": "taker2",
            "in_amount": 50_u64, "out_amount": 5_u64,
            "remaining_in_amount": 850_u64, "remaining_out_amount": 85_u64
        }}),
    );
    assert_eq!(
        state.apply_event(&late_trade, &ctx),
        TransitionDecision::IgnoreTerminalViolation
    );
}

#[test]
fn lifecycle_limit_v2_instruction_driven_create_fill_cancel() {
    let mut state = LifecycleState::new(Protocol::LimitV2);

    let make_ix = |name: &str| RawInstruction {
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

    let create = make_ix("InitializeOrder");
    assert_eq!(
        state.apply_instruction(&create),
        Some(TransitionDecision::Apply)
    );
    assert_eq!(state.status.as_deref(), Some("created"));

    let fill_start = make_ix("PreFlashFillOrder");
    assert_eq!(
        state.apply_instruction(&fill_start),
        Some(TransitionDecision::Apply)
    );

    let fill_end = make_ix("FlashFillOrder");
    assert_eq!(
        state.apply_instruction(&fill_end),
        Some(TransitionDecision::Apply)
    );

    let cancel = make_ix("CancelOrder");
    assert_eq!(
        state.apply_instruction(&cancel),
        Some(TransitionDecision::Apply)
    );
    assert_eq!(state.status.as_deref(), Some("cancelled"));

    let late_fill = make_ix("FlashFillOrder");
    assert_eq!(
        state.apply_instruction(&late_fill),
        Some(TransitionDecision::IgnoreTerminalViolation)
    );
    assert_eq!(
        state.status.as_deref(),
        Some("cancelled"),
        "terminal status must not change"
    );
}

#[test]
fn lifecycle_kamino_create_fill_close_completed() {
    let mut state = LifecycleState::new(Protocol::Kamino);

    let make_ix = |name: &str| RawInstruction {
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

    let create = make_ix("CreateOrder");
    assert_eq!(
        state.apply_instruction(&create),
        Some(TransitionDecision::Apply)
    );
    assert_eq!(state.status.as_deref(), Some("created"));

    let take = make_ix("TakeOrder");
    assert_eq!(
        state.apply_instruction(&take),
        Some(TransitionDecision::Apply)
    );

    let close = make_ix("CloseOrderAndClaimTip");
    assert_eq!(
        state.apply_instruction(&close),
        Some(TransitionDecision::Apply)
    );
    assert_eq!(state.status.as_deref(), Some("completed"));

    let late_take = make_ix("TakeOrder");
    assert_eq!(
        state.apply_instruction(&late_take),
        Some(TransitionDecision::IgnoreTerminalViolation)
    );
}

#[test]
fn lifecycle_kamino_user_swap_balances_is_metadata_only() {
    let mut state = LifecycleState::new(Protocol::Kamino);
    let ctx = no_context();

    state.status = Some("completed".to_string());

    let swap_balances = make_event(
        "UserSwapBalancesEvent",
        serde_json::json!({"UserSwapBalancesEvent": {"some_field": 42}}),
    );
    assert_eq!(
        state.apply_event(&swap_balances, &ctx),
        TransitionDecision::Apply
    );
    assert_eq!(
        state.status.as_deref(),
        Some("completed"),
        "diagnostic event should not mutate lifecycle status"
    );
}
