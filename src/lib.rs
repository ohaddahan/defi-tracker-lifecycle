#![cfg_attr(
    not(test),
    deny(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::dbg_macro,
        clippy::print_stdout,
        clippy::print_stderr,
        clippy::panic,
    )
)]

pub mod error;
pub mod lifecycle;
pub mod protocols;
pub mod types;

#[cfg(feature = "wasm")]
pub mod wasm;

pub use error::Error;
pub use lifecycle::adapters::{
    CorrelationOutcome, EventPayload, ProtocolAdapter, adapter_for, dca_closed_terminal_status,
    kamino_display_terminal_status,
};
pub use lifecycle::mapping::{event_type_to_transition, transition_target, transition_to_display};
pub use lifecycle::{
    LifecycleEngine, LifecycleTransition, SnapshotDelta, TerminalStatus, TransitionDecision,
};
pub use protocols::{EventType, Protocol};
pub use types::{RawEvent, RawInstruction, ResolveContext};
