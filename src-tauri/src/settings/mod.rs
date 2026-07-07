//! 应用设置：exe 旁 settings.json 读写。

pub mod backup;
pub mod migration;

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::ServiceError;

const SETTINGS_VERSION: u32 = 1;
const SETTINGS_FILENAME: &str = "settings.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Settings {
    pub version: u32,
    pub data_dir: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pending_restore_from: Option<String>,
}

impl Settings {
    pub fn new(data_dir: impl Into<String>) -> Self {
        Self {
            version: SETTINGS_VERSION,
            data_dir: data_dir.into(),
            pending_restore_from: None,
        }
    }
}

/// `{executable_parent}/settings.json`
pub fn settings_path() -> Result<PathBuf, ServiceError> {
    let exe = std::env::current_exe()?;
    let parent = exe
        .parent()
        .ok_or_else(|| ServiceError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "executable has no parent directory",
        )))?;
    Ok(parent.join(SETTINGS_FILENAME))
}

pub fn load(path: &Path) -> Result<Option<Settings>, ServiceError> {
    if !path.exists() {
        return Ok(None);
    }
    let content = fs::read_to_string(path)?;
    let settings: Settings = serde_json::from_str(&content)
        .map_err(|e| ServiceError::Validation(format!("invalid settings.json: {e}")))?;
    Ok(Some(settings))
}

pub fn save(path: &Path, settings: &Settings) -> Result<(), ServiceError> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| ServiceError::Validation(format!("serialize settings: {e}")))?;
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_settings_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("workorder-settings-test-{nanos}.json"))
    }

    #[test]
    fn load_returns_none_when_missing() {
        let path = temp_settings_path();
        assert!(load(&path).unwrap().is_none());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let path = temp_settings_path();
        let settings = Settings::new(r"D:\workorder-data");
        save(&path, &settings).unwrap();
        let loaded = load(&path).unwrap().expect("settings exist");
        assert_eq!(loaded, settings);
        let _ = fs::remove_file(&path);
    }
}
