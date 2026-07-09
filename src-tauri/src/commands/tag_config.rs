//! 代办标签配置 Tauri Command。

use std::path::Path;

use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::models::tag_config::{ExportTagConfigResult, TagConfig};
use crate::services::tag_config_service;
use crate::AppState;

fn map_err(err: crate::error::ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_tag_config(state: State<'_, AppState>) -> Result<TagConfig, String> {
    tag_config_service::load_config(&state.data_dir).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn save_tag_config(state: State<'_, AppState>, config: TagConfig) -> Result<(), String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    tag_config_service::save_config_with_cleanup(&state.data_dir, &conn, &config).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn count_work_orders_by_tag(
    state: State<'_, AppState>,
    tag_id: String,
) -> Result<i64, String> {
    let conn = state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())?;
    tag_config_service::count_work_orders_with_tag(&conn, &tag_id).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn pick_tag_config_save_path(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("标签配置", &["json"])
        .set_file_name(tag_config_service::default_export_filename())
        .blocking_save_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn pick_tag_config_file(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("标签配置", &["json"])
        .blocking_pick_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn export_tag_config(
    state: State<'_, AppState>,
    save_path: String,
) -> Result<ExportTagConfigResult, String> {
    let path = tag_config_service::export_config(&state.data_dir, Path::new(&save_path))
        .map_err(map_err)?;
    Ok(ExportTagConfigResult {
        success: true,
        file_path: path.display().to_string(),
    })
}

#[tauri::command]
#[specta::specta]
pub fn import_tag_config(
    state: State<'_, AppState>,
    file_path: String,
) -> Result<TagConfig, String> {
    tag_config_service::import_config(&state.data_dir, Path::new(&file_path)).map_err(map_err)
}
