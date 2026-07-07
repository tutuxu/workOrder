//! 代办状态配置 Tauri Command。

use std::path::Path;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::models::status_config::{ExportStatusConfigResult, StatusConfig};
use crate::services::status_config_service;
use crate::AppState;

fn map_err(err: crate::error::ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_status_config(state: State<'_, AppState>) -> Result<StatusConfig, String> {
    status_config_service::load_config(&state.data_dir).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn save_status_config(
    state: State<'_, AppState>,
    config: StatusConfig,
) -> Result<(), String> {
    status_config_service::save_config(&state.data_dir, &config).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn pick_status_config_save_path(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("状态配置", &["json"])
        .set_file_name(status_config_service::default_export_filename())
        .blocking_save_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn pick_status_config_file(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("状态配置", &["json"])
        .blocking_pick_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn export_status_config(
    state: State<'_, AppState>,
    save_path: String,
) -> Result<ExportStatusConfigResult, String> {
    let path = status_config_service::export_config(&state.data_dir, Path::new(&save_path))
        .map_err(map_err)?;
    Ok(ExportStatusConfigResult {
        success: true,
        file_path: path.display().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn import_status_config(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<StatusConfig, String> {
    status_config_service::import_config(&state.data_dir, Path::new(&file_path)).map_err(map_err)
}
