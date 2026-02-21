# defi-tracker-lifecycle

Pure-logic crate for DeFi order lifecycle tracking on Solana. Zero IO/DB — classification, correlation, and state machine logic only.

## Crate Structure

```
src/
  lib.rs                    # Public API re-exports, cfg_attr deny for production
  error.rs                  # Error enum (Parse, Protocol, Json)
  types.rs                  # RawInstruction, RawEvent, ResolveContext, AccountInfo helpers
  wasm.rs                   # WASM-bindgen API surface (cfg(feature = "wasm"))
  lifecycle/
    mod.rs                  # LifecycleEngine state machine, TerminalStatus, SnapshotDelta
    adapters.rs             # ProtocolAdapter trait, adapter_for(), CorrelationOutcome, EventPayload
    mapping.rs              # Canonical EventType→LifecycleTransition mapping + display helpers
  protocols/
    mod.rs                  # Protocol/EventType enums, program IDs, shared helpers, hardcoded program ID constants
    dca.rs                  # Jupiter DCA adapter + INSTRUCTION/EVENT_EVENT_TYPES + CLOSED_VARIANTS constants
    limit_v1.rs             # Jupiter Limit V1 adapter + variant→EventType constants
    limit_v2.rs             # Jupiter Limit V2 adapter + variant→EventType constants
    kamino.rs               # Kamino adapter + variant→EventType constants
tests/
  adapter_fixtures.rs       # Integration tests using real JSON fixtures + end-to-end lifecycle tests
  fixtures/                 # dca_*.json, kamino_*.json, limit_v1_*.json, limit_v2_*.json
docs-site/
  src/engine/wasm.ts        # Typed wrappers for WASM imports
  src/engine/lifecycle.ts   # TS types + WASM-backed functions
  src/engine/classifier.ts  # JSON classifier via WASM
  src/engine/eventMapping.ts # EventType→Transition via WASM
  src/data/protocols.ts     # UI metadata merged with WASM protocol data
```

## Key Architecture

**Pipeline**: `RawInstruction → classify_instruction_envelope() → EventType` and `RawEvent → classify_and_resolve_event() → (EventType, CorrelationOutcome, EventPayload)`

**Enum-based dispatch**: Each protocol defines mirror enums (`*EventEnvelope`, `*InstructionKind`) whose variant names match Carbon decoder crate variants exactly. Event JSON (already `{"EventName": {...}}` format) deserializes directly via serde's externally-tagged enum. Instructions use a constructed `{"Name": args}` wrapper. Classification + field extraction happen in a single `serde_json::from_value` call.

**State machine**: `LifecycleEngine::decide_transition(Option<TerminalStatus>, LifecycleTransition)` — `None` (non-terminal) accepts all transitions; `Some(TerminalStatus)` only accepts `MetadataOnly`.

**Adapters are stateless** — zero-sized structs stored as statics. Each protocol implements `ProtocolAdapter` with `classify_instruction` and `classify_and_resolve_event`.

**Typed deserialization**: Inner types use `String` for pubkeys since `solana_pubkey::Pubkey` v3 serde expects byte arrays, not base58 strings in JSON.

**Program IDs**: Hardcoded base58 constants (`DCA_PROGRAM_ID`, etc.) in `protocols/mod.rs`. With `native` feature, `from_program_id()` parses to `Pubkey` and compares against Carbon's `PROGRAM_ID` constants. With `wasm` feature, uses string comparison. Native-only test verifies hardcoded strings match Carbon constants.

**WASM API**: Feature-gated (`wasm`) `src/wasm.rs` exposes `get_all_protocols`, `classify_json`, `decide_transition`, `normalize_snapshot`, `event_type_to_transition`, `is_terminal`, `transition_to_string`, `transition_target` via `wasm-bindgen`. Uses `serde-wasm-bindgen` for JsValue conversion.

**Canonical EventType→Transition mapping**: `src/lifecycle/mapping.rs` provides `event_type_to_transition()`, `transition_to_display()`, `transition_target()`. Previously consumer-defined; now canonical in the crate.

**Variant→EventType constants**: Each protocol module exports `INSTRUCTION_EVENT_TYPES`, `EVENT_EVENT_TYPES`, `CLOSED_VARIANTS` static arrays mapping variant names to `EventType` values. Tests verify these match actual classify/resolve outputs.

**Compile-time guardrails**: Each protocol has a `#[cfg(all(test, feature = "native"))]` `classify_decoded()` function with exhaustive match on the Carbon instruction enum. When upstream adds new variants, tests break at compile time.

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

## Features

```toml
[features]
default = ["native"]
native = ["solana-pubkey", "carbon-*-decoder"]  # Full Solana/Carbon deps for production
wasm = ["wasm-bindgen", "serde-wasm-bindgen"]   # WASM target for docs-site
```

## Commands

```bash
cargo test                  # 140 tests (113 unit + 27 integration) — native feature
cargo test --features wasm  # 150 tests (123 unit + 27 integration) — native+wasm
cargo clippy                # pedantic + deny(unwrap_used, expect_used, panic, ...)
cargo fmt                   # format
cargo llvm-cov              # coverage
cargo llvm-cov --html       # HTML report → target/llvm-cov/html/

# WASM build (for docs-site)
wasm-pack build --target bundler --release --out-dir docs-site/src/wasm-pkg -- --features wasm --no-default-features

# Docs-site
cd docs-site && npm run dev   # includes wasm build
cd docs-site && npm run build # production build
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
- `EventType → LifecycleTransition` mapping is canonical in `src/lifecycle/mapping.rs`. End-to-end lifecycle tests use it directly.
- WASM feature uses `cdylib` + `rlib` crate types — `cdylib` required for wasm-pack, `rlib` required for `cargo test`.
- `EventType::as_ref()` returns snake_case (strum). WASM API uses `event_type_to_pascal()` helper for PascalCase output matching TS conventions.
- Docs-site uses `vite-plugin-wasm` + `vite-plugin-top-level-await` for transparent WASM imports (no manual `init()`).
- Limit V1 instruction fixtures contain only `CancelOrder` records (793 from real data). V1 event fixtures are synthetic.
- Pre-commit hooks via `cargo-husky`: runs test, clippy, fmt on commit.
