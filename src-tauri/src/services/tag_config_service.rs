//! 代办标签配置的读写与校验。

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use chrono::Local;
use rusqlite::{params, Connection};

use crate::error::ServiceError;
use crate::models::status_config::DEFAULT_GLASS_COLORS;
use crate::models::tag_config::TagConfig;

pub const CONFIG_FILE_NAME: &str = "tag_config.json";

fn config_path(data_dir: &Path) -> PathBuf {
    data_dir.join(CONFIG_FILE_NAME)
}

fn is_valid_id(value: &str) -> bool {
    !value.is_empty()
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}

fn is_valid_color(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    if value.starts_with('#') {
        let hex = &value[1..];
        return matches!(hex.len(), 3 | 4 | 6 | 8)
            && hex.chars().all(|c| c.is_ascii_hexdigit());
    }
    if value.starts_with("rgba(") && value.ends_with(')') {
        return true;
    }
    if value.starts_with("rgb(") && value.ends_with(')') {
        return true;
    }
    false
}

pub fn load_config(data_dir: &Path) -> Result<TagConfig, ServiceError> {
    let path = config_path(data_dir);
    if !path.exists() {
        return Ok(TagConfig::default_config());
    }
    let content = fs::read_to_string(&path)?;
    let config: TagConfig = serde_json::from_str(&content)
        .map_err(|e| ServiceError::Validation(format!("标签配置格式错误：{e}")))?;
    let config = normalize_colors(config);
    validate_config(&config)?;
    Ok(config)
}

fn normalize_colors(mut config: TagConfig) -> TagConfig {
    for (index, tag) in config.tags.iter_mut().enumerate() {
        if tag.color.trim().is_empty() || !is_valid_color(&tag.color) {
            tag.color = DEFAULT_GLASS_COLORS[index % DEFAULT_GLASS_COLORS.len()].into();
        }
    }
    config
}

pub fn ensure_default_config(data_dir: &Path) -> Result<(), ServiceError> {
    let path = config_path(data_dir);
    if path.exists() {
        return Ok(());
    }
    save_config(data_dir, &TagConfig::default_config())
}

pub fn default_export_filename() -> String {
    Local::now()
        .format("tag-config-backup-%Y%m%d-%H%M%S.json")
        .to_string()
}

pub fn export_config(data_dir: &Path, save_path: &Path) -> Result<PathBuf, ServiceError> {
    if save_path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Err(ServiceError::Validation("备份文件须为 .json 格式".into()));
    }
    let config = load_config(data_dir)?;
    if let Some(parent) = save_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| ServiceError::Validation(format!("serialize config: {e}")))?;
    fs::write(save_path, json)?;
    Ok(save_path.to_path_buf())
}

pub fn import_config(data_dir: &Path, file_path: &Path) -> Result<TagConfig, ServiceError> {
    if !file_path.exists() {
        return Err(ServiceError::Validation("请选择配置文件".into()));
    }
    if file_path.extension().and_then(|e| e.to_str()) != Some("json") {
        return Err(ServiceError::Validation("配置文件须为 .json 格式".into()));
    }
    let content = fs::read_to_string(file_path)?;
    let config: TagConfig = serde_json::from_str(&content)
        .map_err(|e| ServiceError::Validation(format!("标签配置格式错误：{e}")))?;
    let config = normalize_colors(config);
    validate_config(&config)?;
    save_config(data_dir, &config)?;
    Ok(config)
}

pub fn save_config(data_dir: &Path, config: &TagConfig) -> Result<(), ServiceError> {
    let config = normalize_colors(config.clone());
    validate_config(&config)?;
    fs::create_dir_all(data_dir)?;
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| ServiceError::Validation(format!("serialize config: {e}")))?;
    fs::write(config_path(data_dir), json)?;
    Ok(())
}

pub fn validate_config(config: &TagConfig) -> Result<(), ServiceError> {
    let mut seen_ids = HashSet::new();
    for tag in &config.tags {
        if !is_valid_id(&tag.id) {
            return Err(ServiceError::Validation(format!(
                "标签 ID 无效：{}（仅允许字母、数字、下划线）",
                tag.id
            )));
        }
        if tag.label.trim().is_empty() {
            return Err(ServiceError::Validation(format!(
                "标签 {} 的显示名不能为空",
                tag.id
            )));
        }
        if !is_valid_color(&tag.color) {
            return Err(ServiceError::Validation(format!(
                "标签 {} 的颜色格式无效",
                tag.id
            )));
        }
        if !seen_ids.insert(tag.id.clone()) {
            return Err(ServiceError::Validation(format!("标签 ID 重复：{}", tag.id)));
        }
    }
    Ok(())
}

pub fn validate_tag_ids(config: &TagConfig, tag_ids: &[String]) -> Result<(), ServiceError> {
    for tag_id in tag_ids {
        if config.tag_by_id(tag_id).is_none() {
            return Err(ServiceError::Validation(format!("未知标签：{tag_id}")));
        }
    }
    Ok(())
}

pub fn save_config_with_cleanup(
    data_dir: &Path,
    conn: &Connection,
    config: &TagConfig,
) -> Result<(), ServiceError> {
    let old = load_config(data_dir).unwrap_or_else(|_| TagConfig::default_config());
    let new_ids: HashSet<_> = config.tags.iter().map(|t| t.id.as_str()).collect();
    let removed: Vec<_> = old
        .tags
        .iter()
        .filter(|t| !new_ids.contains(t.id.as_str()))
        .map(|t| t.id.clone())
        .collect();
    for tag_id in &removed {
        conn.execute(
            "DELETE FROM work_order_tag WHERE tag_id = ?1",
            params![tag_id],
        )?;
    }
    save_config(data_dir, config)
}

pub fn count_work_orders_with_tag(conn: &Connection, tag_id: &str) -> Result<i64, ServiceError> {
    conn.query_row(
        "SELECT COUNT(DISTINCT work_order_id) FROM work_order_tag WHERE tag_id = ?1",
        params![tag_id],
        |row| row.get(0),
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tag_config::TagDefinition;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(prefix: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{nanos}"))
    }

    #[test]
    fn ensure_default_writes_file() {
        let dir = temp_dir("tag-config");
        ensure_default_config(&dir).unwrap();
        assert!(config_path(&dir).exists());
        let loaded = load_config(&dir).unwrap();
        assert_eq!(loaded.tags.len(), 2);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_duplicate_tag_id() {
        let config = TagConfig {
            version: 1,
            tags: vec![
                TagDefinition {
                    id: "A".into(),
                    label: "A".into(),
                    order: 0,
                    color: "rgba(203,213,225,0.55)".into(),
                },
                TagDefinition {
                    id: "A".into(),
                    label: "B".into(),
                    order: 1,
                    color: "rgba(147,197,253,0.50)".into(),
                },
            ],
        };
        assert!(validate_config(&config).is_err());
    }
}
