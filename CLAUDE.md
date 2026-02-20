# defi-tracker-lifecycle

Pure-logic crate for DeFi order lifecycle tracking on Solana. Zero IO/DB — classification, correlation, and state machine logic only.

## Crate Structure

```
src/
  lib.rs                    # Public API re-exports, cfg_attr deny for production
  error.rs                  # Error enum (Parse, Protocol, Json)
  types.rs                  # RawInstruction, RawEvent, ResolveContext, AccountInfo helpers
  lifecycle/
    mod.rs                  # LifecycleEngine state machine, TerminalStatus, SnapshotDelta
    adapters.rs             # ProtocolAdapter trait, adapter_for(), CorrelationOutcome, EventPayload
  protocols/
    mod.rs                  # Protocol/EventType enums, program IDs, shared account + checked-cast + known-variant helpers
    dca.rs                  # Jupiter DCA adapter (mirror enums: DcaEventEnvelope, DcaInstructionKind)
    limit_v1.rs             # Jupiter Limit V1 adapter (mirror enums: LimitV1EventEnvelope, LimitV1InstructionKind)
    limit_v2.rs             # Jupiter Limit V2 adapter (mirror enums: LimitV2EventEnvelope, LimitV2InstructionKind)
    kamino.rs               # Kamino adapter (mirror enums: KaminoEventEnvelope, KaminoInstructionKind)
tests/
  adapter_fixtures.rs       # Integration tests using real JSON fixtures + end-to-end lifecycle tests
  fixtures/                 # dca_*.json, kamino_*.json, limit_v1_*.json, limit_v2_*.json
```

## Key Architecture

**Pipeline**: `RawInstruction → classify_instruction_envelope() → EventType` and `RawEvent → classify_and_resolve_event() → (EventType, CorrelationOutcome, EventPayload)`

**Enum-based dispatch**: Each protocol defines mirror enums (`*EventEnvelope`, `*InstructionKind`) whose variant names match Carbon decoder crate variants exactly. Event JSON (already `{"EventName": {...}}` format) deserializes directly via serde's externally-tagged enum. Instructions use a constructed `{"Name": args}` wrapper. Classification + field extraction happen in a single `serde_json::from_value` call.

**State machine**: `LifecycleEngine::decide_transition(Option<TerminalStatus>, LifecycleTransition)` — `None` (non-terminal) accepts all transitions; `Some(TerminalStatus)` only accepts `MetadataOnly`.

**Adapters are stateless** — zero-sized structs stored as statics. Each protocol implements `ProtocolAdapter` with `classify_instruction` and `classify_and_resolve_event`.

**Typed deserialization**: Inner types use `String` for pubkeys since `solana_pubkey::Pubkey` v3 serde expects byte arrays, not base58 strings in JSON.

**Program IDs from Carbon**: `Protocol::from_program_id(&str)` parses the input to `solana_pubkey::Pubkey` and compares directly against each Carbon decoder crate's `PROGRAM_ID` constant. No duplicated string constants — correctness by construction.

**Compile-time guardrails**: Each protocol has a `#[cfg(test)]` `classify_decoded()` function with exhaustive match on the Carbon instruction enum. When upstream adds new variants, tests break at compile time.

**Runtime guardrails (mirror enum alignment tests)**: Each protocol has a `mirror_enums_cover_all_carbon_variants` test that constructs `{"VariantName": <minimal_payload>}` JSON for every Carbon variant and asserts the mirror enum (`*InstructionKind`, `*EventEnvelope`) deserializes it. This bridges the compile-time `classify_decoded()` guard with the runtime serde dispatch — if someone adds a Carbon variant to `classify_decoded()` but forgets the mirror enum, this test catches it.

**Known-variant detection from strum**: Event envelopes derive `strum_macros::VariantNames`, providing `VARIANTS` at runtime. The `contains_known_variant()` helper uses `*EventEnvelope::VARIANTS` directly — no manually maintained `KNOWN_EVENT_NAMES` arrays. Correct by construction.

**EventType reachability test**: `event_type_reachability_all_variants_covered` in `protocols/mod.rs` runs all instruction+event variant names through classify/resolve across all protocols, collects produced `EventType` values, and asserts all 9 variants are hit. Catches dead/unreachable variants.

**End-to-end lifecycle tests**: `adapter_fixtures.rs` contains `lifecycle_*` tests that bridge the adapter layer with the state machine. A `LifecycleState` struct tracks status across steps, flowing raw JSON → adapter classification → `EventType` → `LifecycleTransition` → `decide_transition` → status update. Each test simulates a complete order lifecycle (create → fills → close → terminal rejection).

## Protocol-Specific Notes

- **DCA**: ClosedEvent terminal status derived from `user_closed` + `unfilled_amount` fields (not a direct status code)
- **Kamino**: `OrderDisplayEvent` has no order PDA — requires `ResolveContext::pre_fetched_order_pdas` from instruction-level account parsing. Returns `Uncorrelated` if PDAs missing. `UserSwapBalancesEvent` is diagnostic-only (NotRequired correlation; should map to `MetadataOnly` transition).
- **Limit V2**: Args may be nested in `{"params": {...}}` wrapper or flat — adapter handles both. V2 TradeEvent uses `making_amount`/`taking_amount` field names; V1 uses `in_amount`/`out_amount` with `#[serde(alias)]` for backward compat.
- **Limit V1**: `CancelExpiredOrder` instruction maps to `Expired` EventType (distinct from V2 which has no expiry instruction)

## Conventions

- `thiserror` for all errors. `reason` field for string context (not `source`).
- `#[expect(clippy::unwrap_used, reason = "...")]` on test modules (not `#[allow(...)]`)
- `#[expect(dead_code, reason = "...")]` on instruction kind enums (inner `serde_json::Value` consumed by serde, not read)
- `#[expect(clippy::enum_variant_names, reason = "...")]` on event envelopes (variant names mirror Carbon)
- `cfg_attr(not(test), deny(...))` in lib.rs for production-only denies
- Fixtures loaded via `env!("CARGO_MANIFEST_DIR")` + path
- Serde `default` behavior ignores unknown fields — fixtures from main crate (with extra fields) work directly

## Commands

```bash
cargo test                  # 137 tests (110 unit + 27 integration)
cargo clippy                # pedantic + deny(unwrap_used, expect_used, panic, ...)
cargo fmt                   # format
cargo llvm-cov              # coverage
cargo llvm-cov --html       # HTML report → target/llvm-cov/html/
```

## Test Layers

```
Layer 1: Compile-time       classify_decoded() exhaustive match on Carbon enum
                            → breaks if upstream adds new variant
Layer 2: Mirror alignment   mirror_enums_cover_all_carbon_variants()
                            → bridges compile-time guard to runtime serde dispatch
Layer 3: EventType reach    event_type_reachability_all_variants_covered()
                            → all 9 EventType variants are reachable
Layer 4: Unit tests         per-protocol classify/resolve tests with inline JSON
Layer 5: Fixture tests      adapter_fixtures.rs with real JSON from defi-tracker
Layer 6: Lifecycle E2E      lifecycle_* tests: raw JSON → adapter → state machine → status

Known-variant detection uses strum::VariantNames on event envelopes — no manual string arrays.
```

## Gotchas

- Global `~/.cargo/config.toml` sets `-D clippy::unwrap-used` and `-D clippy::allow-attributes` via RUSTFLAGS — overrides Cargo.toml and code-level `#[allow]`. Use `#[expect(...)]` instead.
- `protocols/mod.rs` helper functions (`parse_accounts`, `find_signer`, checked numeric casts, known-variant detection) are shared across all adapters — changes affect all protocols.
- Carbon decoder crates (`carbon-jupiter-dca-decoder`, etc.) are used for exhaustive enum matching in `classify_decoded()` functions (test-only), NOT for direct serde deserialization — `solana_pubkey::Pubkey` v3 doesn't deserialize from base58 strings in JSON.
- Mirror enums must keep `serde_json::Value` inner type on instruction variants to consume any JSON payload (including `null` for args-less instructions).
- Mirror enum alignment tests need minimal valid JSON payloads (not just `{}`), because inner structs like `DcaKeyHolder { dca_key: String }` have required fields.
- `SnapshotDelta::delta` is always `>= 0` even if snapshot regresses. Regression tracked as separate bool.
- No `EventType → LifecycleTransition` mapping function exists in this crate — the consumer (defi-tracker) defines it. End-to-end lifecycle tests define the mapping inline via `event_type_to_transition()` and `event_to_transition()`, including `NotRequired -> MetadataOnly`.
- Limit V1 instruction fixtures contain only `CancelOrder` records (793 from real data). V1 event fixtures are synthetic.
- Pre-commit hooks via `cargo-husky`: runs test, clippy, fmt on commit.
