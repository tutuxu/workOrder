//! 代办状态配置 Tauri Command。

use tauri::State;

use crate::models::status_config::StatusConfig;
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
