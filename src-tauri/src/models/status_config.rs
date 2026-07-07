//! 代办状态与字段配置模型。

use serde::{Deserialize, Serialize};
use specta::Type;

/// 状态字段类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
pub enum StatusFieldType {
    Text,
    Textarea,
    Date,
}

/// 某状态下的一条可填字段。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusField {
    pub key: String,
    pub label: String,
    #[serde(rename = "type")]
    pub field_type: StatusFieldType,
    pub required: bool,
}

fn default_status_color() -> String {
    DEFAULT_GLASS_COLORS[0].into()
}

/// 毛玻璃风格默认色板（rgba，用于列表行背景）。
pub const DEFAULT_GLASS_COLORS: [&str; 8] = [
    "rgba(203, 213, 225, 0.55)", // 雾灰
    "rgba(147, 197, 253, 0.50)", // 天蓝
    "rgba(134, 239, 172, 0.45)", // 薄荷
    "rgba(252, 211, 77, 0.45)",  // 琥珀
    "rgba(251, 182, 206, 0.45)", // 玫瑰
    "rgba(196, 181, 253, 0.45)", // 薰衣草
    "rgba(94, 234, 212, 0.40)",  // 青绿
    "rgba(255, 255, 255, 0.65)", // 珍珠
];

/// 单个代办状态定义。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusDefinition {
    pub id: String,
    pub label: String,
    pub order: i32,
    #[serde(default = "default_status_color")]
    pub color: String,
    pub fields: Vec<StatusField>,
}

/// 数据目录中的 `status_config.json` 根结构。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct StatusConfig {
    pub version: u32,
    pub statuses: Vec<StatusDefinition>,
}

impl StatusConfig {
    pub fn default_config() -> Self {
        Self {
            version: 1,
            statuses: vec![
                StatusDefinition {
                    id: "NOT_STARTED".into(),
                    label: "未处置".into(),
                    order: 0,
                    color: DEFAULT_GLASS_COLORS[0].into(),
                    fields: vec![],
                },
                StatusDefinition {
                    id: "IN_PROGRESS".into(),
                    label: "处置中".into(),
                    order: 1,
                    color: DEFAULT_GLASS_COLORS[1].into(),
                    fields: vec![],
                },
                StatusDefinition {
                    id: "WAITING_REPLY".into(),
                    label: "待回复".into(),
                    order: 2,
                    color: DEFAULT_GLASS_COLORS[3].into(),
                    fields: vec![
                        StatusField {
                            key: "waitingFor".into(),
                            label: "等待对象".into(),
                            field_type: StatusFieldType::Text,
                            required: true,
                        },
                        StatusField {
                            key: "waitingReason".into(),
                            label: "等待原因".into(),
                            field_type: StatusFieldType::Textarea,
                            required: false,
                        },
                    ],
                },
                StatusDefinition {
                    id: "COMPLETED".into(),
                    label: "已完成".into(),
                    order: 3,
                    color: DEFAULT_GLASS_COLORS[2].into(),
                    fields: vec![],
                },
            ],
        }
    }

    pub fn status_by_id(&self, id: &str) -> Option<&StatusDefinition> {
        self.statuses.iter().find(|s| s.id == id)
    }

    pub fn label_for(&self, id: &str) -> String {
        self.status_by_id(id)
            .map(|s| s.label.clone())
            .unwrap_or_else(|| format!("未知状态 ({id})"))
    }
}
