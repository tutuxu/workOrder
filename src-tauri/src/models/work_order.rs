use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

use super::work_order_status::WorkOrderStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkOrderInput {
    pub title: String,
    pub description: Option<String>,
    pub status: WorkOrderStatus,
    pub waiting_for: Option<String>,
    pub waiting_reason: Option<String>,
    pub due_date: Option<NaiveDateTime>,
}
