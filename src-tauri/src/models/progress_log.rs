//! 进度日志模型。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

use crate::models::work_order_status::WorkOrderStatus;

/// 工单下的单条进度记录。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLog {
    pub id: Option<i64>,
    pub work_order_id: i64,
    pub title: String,
    pub content: Option<String>,
    pub status: WorkOrderStatus,
    pub created_at: NaiveDateTime,
}

/// 创建或更新进度记录时的输入。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ProgressLogInput {
    pub title: String,
    pub content: Option<String>,
    pub status: WorkOrderStatus,
}
