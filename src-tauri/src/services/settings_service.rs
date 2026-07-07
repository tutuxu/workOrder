use std::path::PathBuf;

use crate::db::connection::resolve_data_dir;
use crate::error::ServiceError;
use crate::models::settings::{ChangeDataDirResult, SettingsInfo};
use crate::settings::{self, Settings};
use crate::settings::migration::{copy_data_dir, validate_target_dir, verify_migration};

pub fn env_override_active() -> bool {
    std::env::var("WORKORDER_DATA_DIR").is_ok()
}

pub fn get_settings_info(current_data_dir: &PathBuf, settings_path: &PathBuf) -> Result<SettingsInfo, ServiceError> {
    let default_data_dir = resolve_data_dir(None, None);
    Ok(SettingsInfo {
        data_dir: current_data_dir.display().to_string(),
        settings_path: settings_path.display().to_string(),
        env_override: env_override_active(),
        default_data_dir: default_data_dir.display().to_string(),
    })
}

pub fn change_data_dir(
    current_data_dir: &PathBuf,
    settings_path: &PathBuf,
    new_path: &str,
) -> Result<ChangeDataDirResult, ServiceError> {
    if env_override_active() {
        return Err(ServiceError::Validation(
            "当前由环境变量 WORKORDER_DATA_DIR 指定，无法在应用内修改".into(),
        ));
    }

    let new_dir = PathBuf::from(new_path.trim());
    if new_path.trim().is_empty() {
        return Err(ServiceError::Validation("请选择有效的目录路径".into()));
    }

    validate_target_dir(current_data_dir, &new_dir)?;
    copy_data_dir(current_data_dir, &new_dir)?;
    verify_migration(current_data_dir, &new_dir)?;

    let settings = Settings::new(new_dir.display().to_string());
    settings::save(settings_path, &settings)?;

    Ok(ChangeDataDirResult {
        success: true,
        restart_required: true,
        new_data_dir: new_dir.display().to_string(),
    })
}
