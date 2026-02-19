#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("parse error: {reason}")]
    Parse { reason: String },

    #[error("protocol error: {reason}")]
    Protocol { reason: String },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}
