/// A decoded Solana instruction row as produced by the upstream indexer.
#[derive(serde::Deserialize)]
pub struct RawInstruction {
    /// Database row id.
    pub id: i64,
    /// Transaction signature (base58).
    pub signature: String,
    /// Position of this instruction within the transaction.
    pub instruction_index: i32,
    /// Top-level program that was invoked.
    pub program_id: String,
    /// Innermost program if this is a CPI; equals `program_id` otherwise.
    pub inner_program_id: String,
    /// Carbon-decoded instruction discriminator name (e.g. `"OpenDca"`).
    pub instruction_name: String,
    /// Parsed account list, if available.
    pub accounts: Option<serde_json::Value>,
    /// Parsed instruction arguments, if available.
    pub args: Option<serde_json::Value>,
    /// Solana slot in which the transaction landed.
    pub slot: i64,
}

/// A decoded Solana event (log) row as produced by the upstream indexer.
#[derive(serde::Deserialize)]
pub struct RawEvent {
    /// Database row id.
    pub id: i64,
    /// Transaction signature (base58).
    pub signature: String,
    /// Position of this event within the transaction logs.
    pub event_index: i32,
    /// Top-level program that was invoked.
    pub program_id: String,
    /// Innermost program if this is a CPI; equals `program_id` otherwise.
    pub inner_program_id: String,
    /// Carbon-decoded event discriminator name (e.g. `"FilledEvent"`).
    pub event_name: String,
    /// Parsed event fields as `{"EventName": {..}}` JSON.
    pub fields: Option<serde_json::Value>,
    /// Solana slot in which the transaction landed.
    pub slot: i64,
}

/// Caller-supplied context needed to resolve certain events.
///
/// Kamino `OrderDisplayEvent` carries no order PDA in its payload.
/// The caller must pre-fetch order PDAs from the instruction-level account
/// list and pass them here so the adapter can correlate the event.
pub struct ResolveContext {
    /// Order PDAs extracted from instruction accounts for the same transaction.
    /// Required for Kamino `OrderDisplayEvent`; `None` causes `Uncorrelated`.
    pub pre_fetched_order_pdas: Option<Vec<String>>,
}
