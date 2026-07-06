use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLog {
    pub id: Option<i64>,
    pub work_order_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}
