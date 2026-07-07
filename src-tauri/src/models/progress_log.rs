//! 进度日志模型。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

/// 工单下的单条进度记录。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLog {
    pub id: Option<i64>,
    pub work_order_id: i64,
    pub content: String,
    pub created_at: NaiveDateTime,
}
