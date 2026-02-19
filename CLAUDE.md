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
    mod.rs                  # Protocol enum, EventType enum, program ID constants, account parsing helpers
    dca.rs                  # Jupiter DCA adapter
    limit_v1.rs             # Jupiter Limit V1 adapter
    limit_v2.rs             # Jupiter Limit V2 adapter
    kamino.rs               # Kamino adapter (context-dependent correlation)
tests/
  adapter_fixtures.rs       # Integration tests using real JSON fixtures
  fixtures/                 # dca_*.json, kamino_*.json, limit_v2_*.json
```

## Key Architecture

**Pipeline**: `RawInstruction/RawEvent → Protocol::from_program_id() → adapter_for() → classify_*() → EventType → resolve_event() → (CorrelationOutcome, EventPayload)`

**State machine**: `LifecycleEngine::decide_transition()` — non-terminal accepts all transitions; terminal (completed/cancelled/expired) only accepts `MetadataOnly`.

**Adapters are stateless** — zero-sized structs stored as statics. Each protocol implements `ProtocolAdapter` with `classify_instruction`, `classify_event`, and `resolve_event`.

## Protocol-Specific Notes

- **DCA**: ClosedEvent terminal status derived from `user_closed` + `unfilled_amount` fields (not a direct status code)
- **Kamino**: `OrderDisplayEvent` has no order PDA — requires `ResolveContext::pre_fetched_order_pdas` from instruction-level account parsing. Returns `Uncorrelated` if PDAs missing.
- **Limit V2**: Args may be nested in `{"params": {...}}` wrapper or flat — adapter handles both
- **Limit V1**: `CancelExpiredOrder` instruction maps to `Expired` EventType (distinct from V2 which has no expiry instruction)

## Conventions

- `thiserror` for all errors. `reason` field for string context (not `source`).
- `#[expect(clippy::unwrap_used, reason = "...")]` on test modules (not `#[allow(...)]`)
- `cfg_attr(not(test), deny(...))` in lib.rs for production-only denies
- Fixtures loaded via `env!("CARGO_MANIFEST_DIR")` + path
- Serde `default` behavior ignores unknown fields — fixtures from main crate (with extra fields) work directly

## Commands

```bash
cargo test                  # 48 tests (34 unit + 14 integration)
cargo clippy                # pedantic + deny(unwrap_used, expect_used, panic, ...)
cargo fmt                   # format
cargo llvm-cov              # coverage (~66% lines)
cargo llvm-cov --html       # HTML report → target/llvm-cov/html/
```

## Gotchas

- Global `~/.cargo/config.toml` sets `-D clippy::unwrap-used` and `-D clippy::allow-attributes` via RUSTFLAGS — overrides Cargo.toml and code-level `#[allow]`. Use `#[expect(...)]` instead.
- `protocols/mod.rs` helper functions (`parse_accounts`, `find_signer`, `value_to_pubkey`, `unwrap_named`) are shared across all adapters — changes affect all protocols.
- `SnapshotDelta::delta` is always `>= 0` even if snapshot regresses. Regression tracked as separate bool.
- Pre-commit hooks via `cargo-husky`: runs test, clippy, fmt on commit.
