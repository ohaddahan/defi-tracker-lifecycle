#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "test code uses unwrap/panic for concise assertions"
)]

use defi_tracker_lifecycle::protocols::unwrap_named;
use defi_tracker_lifecycle::{
    CorrelationOutcome, EventPayload, EventType, Protocol, RawEvent, RawInstruction,
    ResolveContext, TerminalStatus, adapter_for,
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
) -> (CorrelationOutcome, EventPayload) {
    let adapter = adapter_for(protocol);
    let event_type = adapter
        .classify_event(ev)
        .unwrap_or_else(|| panic!("unclassified event: {}", ev.event_name));
    let null = serde_json::Value::Null;
    let raw_fields = ev.fields.as_ref().unwrap_or(&null);
    let fields = unwrap_named(raw_fields);
    adapter
        .resolve_event(ev, fields, event_type, ctx)
        .unwrap_or_else(|e| panic!("resolve_event failed for {}: {e}", ev.event_name))
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
    let adapter = adapter_for(Protocol::Dca);
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
        assert_eq!(
            adapter.classify_event(ev),
            Some(event_type.clone()),
            "wrong classification for {name}"
        );
    }
}

#[test]
fn dca_resolve_fill_event_extracts_amounts() {
    let events = load_events("dca_events.json");
    let ev = events
        .iter()
        .find(|ev| ev.event_name == "FilledEvent")
        .unwrap();

    let (correlation, payload) = resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

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
            let fields = unwrap_named(ev.fields.as_ref().unwrap());
            !fields
                .get("user_closed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
                && fields
                    .get("unfilled_amount")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1)
                    == 0
        })
        .unwrap();

    let (correlation, payload) = resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

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
            let fields = unwrap_named(ev.fields.as_ref().unwrap());
            fields
                .get("user_closed")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .unwrap();

    let (correlation, payload) = resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

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

        let (correlation, payload) = resolve_event_from_fixture(Protocol::Dca, ev, &no_context());

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
    let adapter = adapter_for(Protocol::Kamino);
    let events = load_events("kamino_events.json");

    for ev in &events {
        assert_eq!(
            adapter.classify_event(ev),
            Some(EventType::FillCompleted),
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

    let (correlation, payload) = resolve_event_from_fixture(Protocol::Kamino, ev, &ctx);

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

    let (correlation, payload) = resolve_event_from_fixture(Protocol::Kamino, ev, &no_context());

    assert!(
        matches!(correlation, CorrelationOutcome::Uncorrelated { .. }),
        "should be Uncorrelated without pre_fetched_order_pdas"
    );
    assert_eq!(payload, EventPayload::None);
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
    let adapter = adapter_for(Protocol::LimitV2);
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
        assert_eq!(
            adapter.classify_event(ev),
            Some(event_type.clone()),
            "wrong classification for {name}"
        );
    }
}

#[test]
fn limit_v2_resolve_trade_event_extracts_fill() {
    let events = load_events("limit_v2_events.json");
    let ev = events
        .iter()
        .find(|ev| ev.event_name == "TradeEvent")
        .unwrap();

    let (correlation, payload) = resolve_event_from_fixture(Protocol::LimitV2, ev, &no_context());

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

        let (correlation, payload) =
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
