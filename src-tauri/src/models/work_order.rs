//! 工单与进度日志的数据模型，供 Service 层与 Tauri Command 序列化使用。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

use super::work_order_status::WorkOrderStatus;

/// 工单完整记录（含 id、priority 与时间戳）。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrder {
    pub id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub status: WorkOrderStatus,
    pub priority: i32,
    pub waiting_for: Option<String>,
    pub waiting_reason: Option<String>,
    pub due_date: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

/// 创建或更新工单时的输入（不含 priority 与时间戳，由 Service 层填充）。
#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderInput {
    pub title: String,
    pub description: Option<String>,
    pub status: WorkOrderStatus,
    pub waiting_for: Option<String>,
    pub waiting_reason: Option<String>,
    pub due_date: Option<NaiveDateTime>,
}
