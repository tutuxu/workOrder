//! 图片附件模型。

use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum OwnerType {
    WorkOrder,
    ProgressLog,
}

impl OwnerType {
    pub fn as_str(self) -> &'static str {
        match self {
            OwnerType::WorkOrder => "work_order",
            OwnerType::ProgressLog => "progress_log",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "work_order" => Some(OwnerType::WorkOrder),
            "progress_log" => Some(OwnerType::ProgressLog),
            _ => None,
        }
    }

    pub fn dir_name(self) -> &'static str {
        self.as_str()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: Option<i64>,
    pub owner_type: OwnerType,
    pub owner_id: i64,
    pub file_name: String,
    pub original_name: Option<String>,
    pub mime_type: String,
    pub file_size: i64,
    pub created_at: NaiveDateTime,
    /// 完整磁盘路径，查询时由 service 填充，不存 DB。
    pub file_path: String,
}
