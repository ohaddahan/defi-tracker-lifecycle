/// Errors produced during instruction/event classification and resolution.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Input could not be parsed (e.g. malformed base58, missing fields).
    #[error("parse error: {reason}")]
    Parse { reason: String },

    /// Protocol-level invariant violation (e.g. overflow, unknown status code).
    #[error("protocol error: {reason}")]
    Protocol { reason: String },

    /// Transparent wrapper around [`serde_json::Error`].
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
