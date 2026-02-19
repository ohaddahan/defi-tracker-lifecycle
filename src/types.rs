#[derive(serde::Deserialize)]
pub struct RawInstruction {
    pub id: i64,
    pub signature: String,
    pub instruction_index: i32,
    pub program_id: String,
    pub inner_program_id: String,
    pub instruction_name: String,
    pub accounts: Option<serde_json::Value>,
    pub args: Option<serde_json::Value>,
    pub slot: i64,
}

#[derive(serde::Deserialize)]
pub struct RawEvent {
    pub id: i64,
    pub signature: String,
    pub event_index: i32,
    pub program_id: String,
    pub inner_program_id: String,
    pub event_name: String,
    pub fields: Option<serde_json::Value>,
    pub slot: i64,
}

pub struct ResolveContext {
    pub pre_fetched_order_pdas: Option<Vec<String>>,
}
