use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkOrderStatus {
    NotStarted,
    InProgress,
    WaitingReply,
    Completed,
}

impl WorkOrderStatus {
    pub fn all() -> &'static [WorkOrderStatus] {
        &[
            WorkOrderStatus::NotStarted,
            WorkOrderStatus::InProgress,
            WorkOrderStatus::WaitingReply,
            WorkOrderStatus::Completed,
        ]
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            WorkOrderStatus::NotStarted => "未处置",
            WorkOrderStatus::InProgress => "处置中",
            WorkOrderStatus::WaitingReply => "待回复",
            WorkOrderStatus::Completed => "已完成",
        }
    }

    pub fn as_db_str(&self) -> &'static str {
        match self {
            WorkOrderStatus::NotStarted => "NOT_STARTED",
            WorkOrderStatus::InProgress => "IN_PROGRESS",
            WorkOrderStatus::WaitingReply => "WAITING_REPLY",
            WorkOrderStatus::Completed => "COMPLETED",
        }
    }

    pub fn from_db_str(value: &str) -> Option<Self> {
        match value {
            "NOT_STARTED" => Some(WorkOrderStatus::NotStarted),
            "IN_PROGRESS" => Some(WorkOrderStatus::InProgress),
            "WAITING_REPLY" => Some(WorkOrderStatus::WaitingReply),
            "COMPLETED" => Some(WorkOrderStatus::Completed),
            _ => None,
        }
    }
}
