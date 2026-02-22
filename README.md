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
flowchart TD
    subgraph Input
        Raw["RawInstruction / RawEvent"]
    end
    subgraph Dispatch
        Lookup["from_program_id()"] --> Adapter["adapter_for()"]
    end
    subgraph Classify
        CI["classify_instruction()"]
        CE["classify_and_resolve_event()"]
    end
    subgraph Output
        ET["EventType"]
        CO["CorrelationOutcome"]
        EP["EventPayload"]
    end

    Raw --> Lookup
    Adapter --> CI & CE
    CI --> ET
    CE --> ET & CO & EP
```

### Order Lifecycle State Machine

```mermaid
flowchart TD
    Start(( )) -->|Create| Active
    Active -->|FillDelta / MetadataOnly| Active
    Active -->|"Close(Completed)"| Completed
    Active -->|"Close(Cancelled)"| Cancelled
    Active -->|"Close(Expired)"| Expired

    subgraph Terminal ["Terminal — only MetadataOnly accepted"]
        Completed
        Cancelled
        Expired
    end
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
    Protocol, adapter_for, event_type_to_transition, ResolveContext,
    LifecycleEngine, TerminalStatus, TransitionDecision,
};

// 1. Identify protocol from program ID
let protocol = Protocol::from_program_id("DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M")
    .ok_or("unknown program")?;
let adapter = adapter_for(protocol);

// 2. Classify + resolve an event in one pass
let ctx = ResolveContext { pre_fetched_order_pdas: None };
let (event_type, correlation, payload) = adapter
    .classify_and_resolve_event(&raw_event, &ctx)
    .ok_or("unknown event variant")?  // None = unknown event variant
    .map_err(|e| e.to_string())?;     // Err = malformed known event payload

// 3. Map EventType to LifecycleTransition via the canonical mapping
let transition = event_type_to_transition(&event_type, None);

// 4. Check state transition (pass None if not terminal)
let current_terminal: Option<TerminalStatus> = None;
let decision = LifecycleEngine::decide_transition(current_terminal, transition);
match decision {
    TransitionDecision::Apply => { /* update order status */ }
    TransitionDecision::IgnoreTerminalViolation => { /* order is terminal, skip */ }
}
```

## Testing

```bash
cargo test                  # run all 140 tests (113 unit + 27 integration)
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
