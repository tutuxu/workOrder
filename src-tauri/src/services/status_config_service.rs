//! 代办状态配置的读写与校验。

use std::fs;
use std::path::Path;

use crate::error::ServiceError;
use crate::models::status_config::{StatusConfig, DEFAULT_GLASS_COLORS};

pub const CONFIG_FILE_NAME: &str = "status_config.json";

fn config_path(data_dir: &Path) -> std::path::PathBuf {
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

/// 读取配置；文件不存在时返回默认配置（不写盘）。
pub fn load_config(data_dir: &Path) -> Result<StatusConfig, ServiceError> {
    let path = config_path(data_dir);
    if !path.exists() {
        return Ok(StatusConfig::default_config());
    }
    let content = fs::read_to_string(&path)?;
    let config: StatusConfig = serde_json::from_str(&content)
        .map_err(|e| ServiceError::Validation(format!("状态配置格式错误：{e}")))?;
    let config = normalize_colors(config);
    validate_config(&config)?;
    Ok(config)
}

fn normalize_colors(mut config: StatusConfig) -> StatusConfig {
    for (index, status) in config.statuses.iter_mut().enumerate() {
        if status.color.trim().is_empty() || !is_valid_color(&status.color) {
            status.color = DEFAULT_GLASS_COLORS[index % DEFAULT_GLASS_COLORS.len()].into();
        }
    }
    config
}

/// 若配置文件不存在则写入默认配置。
pub fn ensure_default_config(data_dir: &Path) -> Result<(), ServiceError> {
    let path = config_path(data_dir);
    if path.exists() {
        return Ok(());
    }
    save_config(data_dir, &StatusConfig::default_config())
}

pub fn save_config(data_dir: &Path, config: &StatusConfig) -> Result<(), ServiceError> {
    let config = normalize_colors(config.clone());
    validate_config(&config)?;
    fs::create_dir_all(data_dir)?;
    let json = serde_json::to_string_pretty(&config)
        .map_err(|e| ServiceError::Validation(format!("serialize config: {e}")))?;
    fs::write(config_path(data_dir), json)?;
    Ok(())
}

pub fn validate_config(config: &StatusConfig) -> Result<(), ServiceError> {
    if config.statuses.is_empty() {
        return Err(ServiceError::Validation("至少保留一个状态".into()));
    }
    let mut seen_ids = std::collections::HashSet::new();
    for status in &config.statuses {
        if !is_valid_id(&status.id) {
            return Err(ServiceError::Validation(format!(
                "状态 ID 无效：{}（仅允许字母、数字、下划线）",
                status.id
            )));
        }
        if status.label.trim().is_empty() {
            return Err(ServiceError::Validation(format!(
                "状态 {} 的显示名不能为空",
                status.id
            )));
        }
        if !is_valid_color(&status.color) {
            return Err(ServiceError::Validation(format!(
                "状态 {} 的颜色格式无效",
                status.id
            )));
        }
        if !seen_ids.insert(status.id.clone()) {
            return Err(ServiceError::Validation(format!(
                "状态 ID 重复：{}",
                status.id
            )));
        }
        let mut seen_keys = std::collections::HashSet::new();
        for field in &status.fields {
            if !is_valid_id(&field.key) {
                return Err(ServiceError::Validation(format!(
                    "字段 key 无效：{}",
                    field.key
                )));
            }
            if field.label.trim().is_empty() {
                return Err(ServiceError::Validation(format!(
                    "状态 {} 的字段 {} 标签不能为空",
                    status.id, field.key
                )));
            }
            if !seen_keys.insert(field.key.clone()) {
                return Err(ServiceError::Validation(format!(
                    "状态 {} 的字段 key 重复：{}",
                    status.id, field.key
                )));
            }
        }
    }
    Ok(())
}

/// 按当前配置校验工单 extra_fields 必填项。
pub fn validate_extra_fields(
    config: &StatusConfig,
    status_id: &str,
    extra_fields: &std::collections::HashMap<String, String>,
) -> Result<(), ServiceError> {
    let Some(status) = config.status_by_id(status_id) else {
        return Ok(());
    };
    for field in &status.fields {
        if !field.required {
            continue;
        }
        let value = extra_fields
            .get(&field.key)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty());
        if value.is_none() {
            return Err(ServiceError::Validation(format!(
                "请填写{}",
                field.label
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::status_config::{StatusDefinition, StatusField, StatusFieldType};
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
        let dir = temp_dir("status-config");
        ensure_default_config(&dir).unwrap();
        assert!(config_path(&dir).exists());
        let loaded = load_config(&dir).unwrap();
        assert_eq!(loaded.statuses.len(), 4);
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn rejects_duplicate_status_id() {
        let config = StatusConfig {
            version: 1,
            statuses: vec![
                StatusDefinition {
                    id: "A".into(),
                    label: "A".into(),
                    order: 0,
                    color: "rgba(203, 213, 225, 0.55)".into(),
                    fields: vec![],
                },
                StatusDefinition {
                    id: "A".into(),
                    label: "B".into(),
                    order: 1,
                    color: "rgba(147, 197, 253, 0.50)".into(),
                    fields: vec![],
                },
            ],
        };
        assert!(validate_config(&config).is_err());
    }

    #[test]
    fn validate_required_fields() {
        let config = StatusConfig::default_config();
        let mut fields = std::collections::HashMap::new();
        let err = validate_extra_fields(&config, "WAITING_REPLY", &fields).unwrap_err();
        assert!(matches!(err, ServiceError::Validation(_)));
        fields.insert("waitingFor".into(), "联调方".into());
        validate_extra_fields(&config, "WAITING_REPLY", &fields).unwrap();
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = temp_dir("status-config-rt");
        let mut config = StatusConfig::default_config();
        config.statuses.push(StatusDefinition {
            id: "CUSTOM".into(),
            label: "自定义".into(),
            order: 4,
            color: "rgba(196, 181, 253, 0.45)".into(),
            fields: vec![StatusField {
                key: "note".into(),
                label: "备注".into(),
                field_type: StatusFieldType::Text,
                required: false,
            }],
        });
        save_config(&dir, &config).unwrap();
        let loaded = load_config(&dir).unwrap();
        assert_eq!(loaded.statuses.len(), 5);
        let _ = fs::remove_dir_all(&dir);
    }
}
