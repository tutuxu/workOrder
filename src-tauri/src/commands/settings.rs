use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

use crate::error::ServiceError;
use crate::models::settings::{ChangeDataDirResult, SettingsInfo};
use crate::services::settings_service;
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
pub fn restart_app(app: AppHandle) -> Result<(), String> {
    app.restart();
}
