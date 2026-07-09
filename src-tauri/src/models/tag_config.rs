//! 代办标签配置模型。

use serde::{Deserialize, Serialize};
use specta::Type;

use crate::models::status_config::DEFAULT_GLASS_COLORS;

fn default_tag_color() -> String {
    DEFAULT_GLASS_COLORS[0].into()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum TagMatchMode {
    Any,
    All,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagDefinition {
    pub id: String,
    pub label: String,
    pub order: i32,
    #[serde(default = "default_tag_color")]
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ExportTagConfigResult {
    pub success: bool,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct TagConfig {
    pub version: u32,
    pub tags: Vec<TagDefinition>,
}

impl TagConfig {
    pub fn default_config() -> Self {
        Self {
            version: 1,
            tags: vec![
                TagDefinition {
                    id: "TAG_1".into(),
                    label: "标签一".into(),
                    order: 0,
                    color: DEFAULT_GLASS_COLORS[0].into(),
                },
                TagDefinition {
                    id: "TAG_2".into(),
                    label: "标签二".into(),
                    order: 1,
                    color: DEFAULT_GLASS_COLORS[1].into(),
                },
            ],
        }
    }

    pub fn tag_by_id(&self, id: &str) -> Option<&TagDefinition> {
        self.tags.iter().find(|t| t.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_two_tags() {
        let config = TagConfig::default_config();
        assert_eq!(config.version, 1);
        assert_eq!(config.tags.len(), 2);
        assert_eq!(config.tags[0].id, "TAG_1");
        assert_eq!(config.tags[0].label, "标签一");
        assert_eq!(config.tags[1].id, "TAG_2");
        assert_eq!(config.tags[1].label, "标签二");
    }
}
