use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SettingsInfo {
    pub data_dir: String,
    pub settings_path: String,
    pub env_override: bool,
    pub default_data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct ChangeDataDirResult {
    pub success: bool,
    pub restart_required: bool,
    pub new_data_dir: String,
}
