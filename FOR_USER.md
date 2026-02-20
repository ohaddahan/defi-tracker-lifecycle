# defi-tracker-lifecycle: What's Going On Here

## The 30-Second Version

This crate takes raw Solana transaction data (instructions + events) from DeFi protocols and figures out: "What order does this belong to? What happened to it? Is it done yet?"

It does zero I/O — no database, no RPC calls. Pure logic: classify, correlate, and track state transitions.

## Technical Architecture

### The Pipeline

```
                                  EVENTS
Raw event JSON                    Instructions
{"EventName": {fields}}           ix.instruction_name + ix.args
        |                                 |
        v                                 v
serde_json::from_value()          serde_json::from_value()
  into DcaEventEnvelope             into DcaInstructionKind
  (externally-tagged enum)          (constructed {"Name": args})
        |                                 |
        v                                 v
  exhaustive match                  exhaustive match
        |                                 |
        v                                 v
(EventType, CorrelationOutcome,     Option<EventType>
 EventPayload)
```

Classification + field extraction happen in a single serde pass. No string matching, no `unwrap_named()` — the JSON structure IS the dispatch mechanism.

### State Machine

Orders move through states: Created -> (Fill cycles) -> Terminal (Completed/Cancelled/Expired).

The `LifecycleEngine` enforces: once terminal, no more state changes (except metadata-only updates).

### Protocols Supported

| Protocol | Program | Key Behavior |
|---|---|---|
| Jupiter DCA | `DCA265Vj...` | Dollar-cost averaging. Terminal status inferred from `user_closed` + `unfilled_amount` |
| Jupiter Limit V1 | `jupoNjAx...` | Limit orders. Has `CancelExpiredOrder` instruction |
| Jupiter Limit V2 | `j1o2qRpj...` | Newer limit orders. Params wrapped in `{"params": {...}}` |
| Kamino | `LiMoM9rM...` | Limit orders. `OrderDisplayEvent` has no PDA -- needs pre-fetched PDAs from instruction accounts |

## How the Codebase is Connected

```
lib.rs          -- Public API surface, re-exports everything
    |
    +-- types.rs          -- Input types (RawInstruction, RawEvent, ResolveContext)
    +-- error.rs          -- Error enum (Parse, Protocol, Json)
    |
    +-- protocols/
    |       mod.rs         -- Protocol/EventType + shared account helpers + checked casts + known-variant detection
    |       dca.rs         -- DcaEventEnvelope + DcaInstructionKind (mirror enums)
    |       limit_v1.rs    -- LimitV1EventEnvelope + LimitV1InstructionKind
    |       limit_v2.rs    -- LimitV2EventEnvelope + LimitV2InstructionKind
    |       kamino.rs      -- KaminoEventEnvelope + KaminoInstructionKind
    |
    +-- lifecycle/
            mod.rs         -- LifecycleEngine, TerminalStatus, SnapshotDelta
            adapters.rs    -- ProtocolAdapter trait (classify_instruction, classify_and_resolve_event)
```

## Technologies and Why

- **Rust** -- correctness matters when tracking financial state machines
- **serde/serde_json** -- event JSON is already in serde's externally-tagged enum format; one `from_value` call does classification + extraction
- **Carbon decoder crates** -- IDL-generated enums for compile-time variant coverage (test-only `classify_decoded()`)
- **thiserror** -- structured error types with context
- **Zero dependencies on solana-sdk** -- only the thin `solana-pubkey` (pulled transitively by Carbon)

## Key Design Decisions

### Mirror Enums Over String Matching

Each protocol defines two mirror enums:
- `*EventEnvelope` — variant names match Carbon's event variants, inner types are local structs with `String` for pubkeys
- `*InstructionKind` — variant names match Carbon's instruction variants, inner `serde_json::Value` (consumed by serde, not read)

Why "mirror" instead of using Carbon types directly? Because `solana_pubkey::Pubkey` v3 deserializes from byte arrays `[u8; 32]`, not base58 strings. Our JSON uses base58 strings. So we define local types with `String` fields and do checked `u64 -> i64` conversion at the boundary.

### Single-Pass Classify + Resolve

The old approach was: classify event name (string match) -> unwrap JSON wrapper -> resolve fields (second parse). The new approach: `serde_json::from_value::<DcaEventEnvelope>(fields)` does variant matching AND field extraction in one shot. The trait method `classify_and_resolve_event` returns `Option<Result<(EventType, CorrelationOutcome, EventPayload), Error>>` — `None` means unknown event variant, `Some(Ok(...))` means classified + resolved, `Some(Err(...))` means malformed known payload.

### Multi-Layer Safety Net

The crate has a layered defense strategy that catches different classes of bugs at different stages:

**Layer 1 — Compile-time (`classify_decoded()`)**
Each protocol has a `#[cfg(test)]` function that exhaustively matches Carbon's instruction enum. When the upstream protocol adds a new instruction type, this match becomes non-exhaustive and tests won't compile. This is the first tripwire.

**Layer 2 — Mirror enum alignment tests**
But `classify_decoded()` and the mirror enums are independent code paths — someone could add a variant to `classify_decoded()` without touching the mirror enum. The `mirror_enums_cover_all_carbon_variants` test closes this gap: it constructs `{"VariantName": <payload>}` JSON for every Carbon variant and asserts the mirror enum deserializes it. If the two sides drift, this test fails.

There is also a second runtime sync guard: `known_event_names_match_event_envelope_variants` in each protocol module. It asserts `KNOWN_EVENT_NAMES` (used for malformed-known vs unknown discrimination) exactly matches `*EventEnvelope` variant names via `strum::VariantNames`.

**Layer 3 — EventType reachability**
The `event_type_reachability_all_variants_covered` test runs every instruction and event variant across all four protocols, collects all produced `EventType` values, and asserts all 9 variants (`Created`, `FillInitiated`, `FillCompleted`, `Cancelled`, `Expired`, `Closed`, `FeeCollected`, `Withdrawn`, `Deposited`) are hit. If a variant becomes dead, this catches it.

**Layer 4 — End-to-end lifecycle tests**
The `lifecycle_*` tests in `adapter_fixtures.rs` bridge both layers of the crate: they flow raw JSON through the adapter classification layer, map to lifecycle transitions, and track order status through the state machine. Each test tells a complete story:
- DCA: Create → Fill → Fill → Close(completed) → late fill gets rejected
- DCA: Create → Fill → Close(cancelled by user) → late create gets rejected
- DCA: Create → Close(expired) → fee collection metadata still accepted
- Limit V1: Create → Trade → Cancel → late trade gets rejected
- Limit V2: InitializeOrder → PreFlashFill → FlashFill → Cancel → late fill rejected
- Kamino: Create → TakeOrder → Close → late take rejected

This is like a full integration test of the crate as a consumer would use it.

## Lessons and Pitfalls

### The Pubkey Serde Trap

`solana_pubkey::Pubkey` v3's `Deserialize` uses `visit_seq` (expects `[u8; 32]`), not `visit_str` (base58). This means you **cannot** directly deserialize JSON like `{"dca_key": "3nsT..."}` into a struct with `dca_key: Pubkey`. The fix: use `String` in your deserialization structs.

### Global Clippy Config Overrides

`~/.cargo/config.toml` with `RUSTFLAGS = -D clippy::unwrap-used` overrides `Cargo.toml` lint settings AND code-level `#[allow]` attributes. Only `#[expect(...)]` works because it's checked by the compiler, not clippy RUSTFLAGS.

### Serde Externally-Tagged Enum Format

Carbon-decoded events arrive as `{"EventName": {"field1": ..., "field2": ...}}` — this is exactly serde's default externally-tagged enum representation. By defining `enum DcaEventEnvelope { FilledEvent(FilledEventFields), ... }`, a single `serde_json::from_value` does the dispatch. No manual wrapper stripping needed.

For instructions, the JSON has `instruction_name` and `args` as separate fields. We construct `{"Name": args}` to create the externally-tagged format, then deserialize into the instruction kind enum.

### Kamino's Missing PDA Problem

`OrderDisplayEvent` is the only event from Kamino, and it doesn't contain the order PDA. The PDA must be extracted from the instruction's account list (`pre_fetched_order_pdas` in `ResolveContext`). Without it, the event is `Uncorrelated` — usable for metadata but can't be tied to a specific order.

### Dead Code in Instruction Enums

Instruction kind enum variants have `serde_json::Value` inner types that are never read — they exist only so serde can consume the JSON payload during variant name matching. These are suppressed with `#[expect(dead_code, reason = "...")]` on the enum. Don't remove the inner types or serde will fail to deserialize non-null payloads.

### Mirror Enum Alignment Tests Need Real Payloads

When writing alignment tests, you can't just use `{"EventName": {}}` for event variants — many inner structs have required fields (e.g., `DcaKeyHolder { dca_key: String }`). The test must provide minimal valid JSON: `{"OpenedEvent": {"dca_key": "test"}}`. For instruction variants, `null` works because they all use `serde_json::Value` which accepts anything.

### The Missing Bridge: EventType → LifecycleTransition

This crate provides two layers that don't connect to each other: adapters produce `EventType`, and the state machine consumes `LifecycleTransition`. The mapping between them lives in the consumer (the parent defi-tracker crate). The end-to-end lifecycle tests define this mapping inline via `event_type_to_transition()` and `event_to_transition()`, including `CorrelationOutcome::NotRequired -> LifecycleTransition::MetadataOnly` for diagnostic events.
