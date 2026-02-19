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

pub use error::Error;
pub use lifecycle::adapters::{
    CorrelationOutcome, EventPayload, ProtocolAdapter, adapter_for, dca_closed_terminal_status,
    kamino_display_terminal_status,
};
pub use lifecycle::{
    LifecycleEngine, LifecycleTransition, SnapshotDelta, TerminalStatus, TransitionDecision,
};
pub use protocols::{EventType, Protocol};
pub use types::{RawEvent, RawInstruction, ResolveContext};
