# defi-tracker-lifecycle

Pure-logic crate for DeFi order lifecycle tracking on Solana. No IO, no database — just classification, correlation, and state machine logic.

## Supported Protocols

| Protocol | Program | Description |
|----------|---------|-------------|
| **DCA** | `DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M` | Jupiter Dollar-Cost Averaging |
| **Limit V1** | `jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu` | Jupiter Limit Orders V1 |
| **Limit V2** | `j1o2qRpjcyUwEvwtcfhEQefh773ZgjxcVRry7LDqg5X` | Jupiter Limit Orders V2 |
| **Kamino** | `LiMoM9rMhrdYrfzUCxQppvxCSG1FcrUK9G8uLq4A1GF` | Kamino Limit Orders |

## Architecture

### Event Processing Pipeline

```mermaid
flowchart LR
    Raw["RawInstruction / RawEvent"] --> Lookup["Protocol::from_program_id()"]
    Lookup --> Adapter["adapter_for(protocol)"]
    Adapter --> Classify["classify_instruction()\nclassify_event()"]
    Classify --> ET["EventType"]
    ET --> Resolve["resolve_event()"]
    Resolve --> Out["(CorrelationOutcome, EventPayload)"]
```

### Order Lifecycle State Machine

```mermaid
stateDiagram-v2
    [*] --> Active : Create
    Active --> Active : FillDelta
    Active --> Active : MetadataOnly
    Active --> Completed : Close(Completed)
    Active --> Cancelled : Close(Cancelled)
    Active --> Expired : Close(Expired)

    Completed --> Completed : MetadataOnly
    Cancelled --> Cancelled : MetadataOnly
    Expired --> Expired : MetadataOnly

    note right of Completed : Terminal — rejects\nCreate, FillDelta, Close
    note right of Cancelled : Terminal
    note right of Expired : Terminal
```

### Protocol Adapter Selection

```mermaid
flowchart TD
    PID["program_id"] --> Match{"Protocol::from_program_id()"}
    Match -->|DCA265...| DCA["DcaAdapter"]
    Match -->|jupoNj...| LV1["LimitV1Adapter"]
    Match -->|j1o2qR...| LV2["LimitV2Adapter"]
    Match -->|LiMoM9...| KAM["KaminoAdapter"]
    Match -->|unknown| None["None"]
```

## Core Concepts

**`ProtocolAdapter`** — trait implemented by each protocol. Two phases:
- **Classify**: maps instruction/event names to `EventType` (Created, FillCompleted, Closed, etc.)
- **Resolve**: extracts order PDAs (`CorrelationOutcome`) and structured data (`EventPayload`)

**`LifecycleEngine`** — stateless state machine that enforces transition rules:
- Non-terminal orders accept all transitions
- Terminal orders (Completed/Cancelled/Expired) only accept `MetadataOnly`
- Snapshot deltas are always non-negative; regressions tracked separately

**`ResolveContext`** — carries pre-fetched data needed for correlation (Kamino requires pre-fetched order PDAs since its events don't contain them directly)

## Usage

```rust
use defi_tracker_lifecycle::{
    Protocol, adapter_for, ResolveContext,
    LifecycleEngine, LifecycleTransition, TransitionDecision,
};

// 1. Identify protocol from program ID
let protocol = Protocol::from_program_id("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M");
let adapter = adapter_for(protocol.unwrap());

// 2. Classify + resolve an event in one pass
let ctx = ResolveContext { pre_fetched_order_pdas: None };
let (event_type, correlation, payload) = adapter
    .classify_and_resolve_event(&raw_event, &ctx)
    .unwrap()  // None = unknown event variant
    .unwrap(); // Err = malformed known event payload

// 3. Map EventType to LifecycleTransition (your responsibility)
//    Recommended: treat CorrelationOutcome::NotRequired as MetadataOnly.
let transition = LifecycleTransition::FillDelta;

// 4. Check state transition
let decision = LifecycleEngine::decide_transition(
    current_status,
    transition,
);
match decision {
    TransitionDecision::Apply => { /* update order status */ }
    TransitionDecision::IgnoreTerminalViolation => { /* order is terminal, skip */ }
}
```

## Testing

```bash
cargo test                  # run all 92 tests (65 unit + 27 integration)
cargo clippy                # lint check
```

### Test Layers

| Layer | What it catches |
|-------|----------------|
| Compile-time (`classify_decoded()`) | Upstream Carbon adds a new variant |
| Mirror enum alignment | Mirror enum drifts from Carbon variants |
| EventType reachability | A variant becomes dead/unreachable |
| Unit tests | Individual classify/resolve logic per protocol |
| Fixture tests | Real JSON from defi-tracker parses and classifies correctly |
| End-to-end lifecycle | Full pipeline: raw JSON → adapter → state machine → status tracking |

### Coverage (requires [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov))

```bash
cargo llvm-cov                                        # text summary
cargo llvm-cov --html                                 # HTML report → target/llvm-cov/html/
cargo llvm-cov --lcov --output-path lcov.info         # LCOV for CI upload
```
