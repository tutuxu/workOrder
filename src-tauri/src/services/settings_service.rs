use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::db::connection::resolve_data_dir;
use crate::error::ServiceError;
use crate::models::settings::{
    ChangeDataDirResult, ExportBackupResult, ImportBackupResult, SettingsInfo,
};
use crate::settings::{self, backup, Settings};
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

pub fn export_backup(
    conn: &Connection,
    data_dir: &Path,
    save_path: &str,
) -> Result<ExportBackupResult, ServiceError> {
    let path = PathBuf::from(save_path.trim());
    if save_path.trim().is_empty() {
        return Err(ServiceError::Validation("请选择保存位置".into()));
    }
    if path.extension().and_then(|e| e.to_str()) != Some("zip") {
        return Err(ServiceError::Validation("备份文件须为 .zip 格式".into()));
    }

    backup::checkpoint_connection(conn)?;
    backup::create_backup_zip(data_dir, &path)?;

    Ok(ExportBackupResult {
        success: true,
        file_path: path.display().to_string(),
    })
}

pub fn import_backup(
    current_data_dir: &Path,
    settings_path: &Path,
    zip_path: &str,
) -> Result<ImportBackupResult, ServiceError> {
    let zip = PathBuf::from(zip_path.trim());
    if zip_path.trim().is_empty() {
        return Err(ServiceError::Validation("请选择备份文件".into()));
    }
    if !zip.is_file() {
        return Err(ServiceError::Validation("备份文件不存在".into()));
    }

    let temp_dir = backup::temp_restore_dir();
    backup::extract_backup_zip(&zip, &temp_dir)?;
    backup::validate_backup_contents(&temp_dir)?;

    let mut settings = settings::load(settings_path)?.unwrap_or_else(|| {
        Settings::new(current_data_dir.display().to_string())
    });
    settings.pending_restore_from = Some(temp_dir.display().to_string());
    settings::save(settings_path, &settings)?;

    Ok(ImportBackupResult {
        success: true,
        restart_required: true,
    })
}

pub fn apply_pending_restore(
    settings_path: &Path,
    data_dir: &Path,
) -> Result<(), ServiceError> {
    let Some(mut settings) = settings::load(settings_path)? else {
        return Ok(());
    };
    let Some(pending) = settings.pending_restore_from.take() else {
        return Ok(());
    };

    let pending_path = PathBuf::from(&pending);
    backup::apply_restore(&pending_path, data_dir)?;
    settings::save(settings_path, &settings)?;
    let _ = std::fs::remove_dir_all(&pending_path);
    Ok(())
}
