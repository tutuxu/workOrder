use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::ServiceError;
use std::collections::HashMap;

use crate::models::settings::{
    ChangeDataDirResult, ExportBackupResult, ImportBackupResult, SettingsInfo,
    ShortcutBindingsPayload,
};
use crate::services::settings_service;
use crate::settings::backup;
use crate::AppState;

fn map_err(err: ServiceError) -> String {
    err.to_string()
}

#[tauri::command]
#[specta::specta]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsInfo, String> {
    settings_service::get_settings_info(&state.data_dir, &state.settings_path).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn pick_data_dir(app: AppHandle) -> Result<Option<String>, String> {
    let folder = app
        .dialog()
        .file()
        .blocking_pick_folder();
    Ok(folder.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn change_data_dir(
    state: State<'_, AppState>,
    new_path: String,
) -> Result<ChangeDataDirResult, String> {
    settings_service::change_data_dir(&state.data_dir, &state.settings_path, &new_path)
        .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn pick_backup_save_path(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("备份文件", &["zip"])
        .set_file_name(backup::default_backup_filename())
        .blocking_save_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn pick_backup_file(app: AppHandle) -> Result<Option<String>, String> {
    let file = app
        .dialog()
        .file()
        .add_filter("备份文件", &["zip"])
        .blocking_pick_file();
    Ok(file.map(|p| p.to_string()))
}

#[tauri::command]
#[specta::specta]
pub fn export_backup(
    state: State<'_, AppState>,
    save_path: String,
) -> Result<ExportBackupResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    settings_service::export_backup(&conn, &state.data_dir, &save_path).map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn import_backup(
    state: State<'_, AppState>,
    zip_path: String,
) -> Result<ImportBackupResult, String> {
    settings_service::import_backup(&state.data_dir, &state.settings_path, &zip_path)
        .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}

#[tauri::command]
#[specta::specta]
pub fn get_shortcut_bindings(state: State<'_, AppState>) -> Result<ShortcutBindingsPayload, String> {
    settings_service::get_shortcut_bindings(&state.settings_path)
        .map(|bindings| ShortcutBindingsPayload { bindings })
        .map_err(map_err)
}

#[tauri::command]
#[specta::specta]
pub fn save_shortcut_bindings(
    state: State<'_, AppState>,
    bindings: HashMap<String, String>,
) -> Result<(), String> {
    settings_service::save_shortcut_bindings(&state.settings_path, &state.data_dir, bindings)
        .map_err(map_err)
}
