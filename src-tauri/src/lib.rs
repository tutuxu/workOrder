//! workOrder 桌面应用 Rust 后端入口。
//!
//! 分层：commands → services → db，共享 models 与 `ServiceError`。
//! 前端通过 Tauri `invoke` 调用 Command；类型绑定由 tauri-specta 导出至 `src/bindings.ts`。

mod commands;
pub mod db;
mod error;
mod models;
pub mod services;
mod settings;

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder, ErrorHandlingMode};

pub struct AppState {
    pub db: Mutex<Connection>,
    pub data_dir: PathBuf,
    pub settings_path: PathBuf,
}

fn specta_builder() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .dangerously_cast_bigints_to_number()
        .error_handling(ErrorHandlingMode::Throw)
        .commands(collect_commands![
        commands::work_order::list_work_orders,
        commands::work_order::get_work_order,
        commands::work_order::create_work_order,
        commands::work_order::update_work_order,
        commands::work_order::delete_work_order,
        commands::work_order::update_priorities,
        commands::work_order::is_work_order_overdue,
        commands::status_config::get_status_config,
        commands::status_config::save_status_config,
        commands::status_config::pick_status_config_save_path,
        commands::status_config::pick_status_config_file,
        commands::status_config::export_status_config,
        commands::status_config::import_status_config,
        commands::tag_config::get_tag_config,
        commands::tag_config::save_tag_config,
        commands::tag_config::count_work_orders_by_tag,
        commands::tag_config::pick_tag_config_save_path,
        commands::tag_config::pick_tag_config_file,
        commands::tag_config::export_tag_config,
        commands::tag_config::import_tag_config,
        commands::progress_log::list_progress_logs,
        commands::progress_log::add_progress_log,
        commands::progress_log::update_progress_log,
        commands::progress_log::delete_progress_log,
        commands::settings::get_settings,
        commands::settings::pick_data_dir,
        commands::settings::change_data_dir,
        commands::settings::pick_backup_save_path,
        commands::settings::pick_backup_file,
        commands::settings::export_backup,
        commands::settings::import_backup,
        commands::settings::restart_app,
        commands::settings::get_shortcut_bindings,
        commands::settings::save_shortcut_bindings,
        commands::attachment::list_attachments,
        commands::attachment::add_attachment_from_file,
        commands::attachment::add_attachment_from_bytes,
        commands::attachment::delete_attachment,
        commands::attachment::pick_attachment_file,
    ])
}

fn typescript_exporter() -> Typescript {
    Typescript::default()
}

/// 将 TypeScript 绑定导出到 `src/bindings.ts`（debug 构建与单元测试时调用）。
pub fn export_typescript_bindings(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    specta_builder()
        .export(typescript_exporter(), path)
        .map_err(Into::into)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let builder = specta_builder();

    #[cfg(debug_assertions)]
    builder
        .export(typescript_exporter(), "../src/bindings.ts")
        .expect("Failed to export typescript bindings");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let settings_path = settings::settings_path()
                .map_err(|e| format!("failed to resolve settings path: {e}"))?;
            let loaded = settings::load(&settings_path)
                .map_err(|e| format!("failed to load settings: {e}"))?;
            let settings_data_dir = loaded.as_ref().map(|s| s.data_dir.as_str());
            let data_dir = db::connection::resolve_data_dir(settings_data_dir, None);

            if settings_data_dir.is_some() {
                std::fs::create_dir_all(&data_dir)
                    .map_err(|e| format!("failed to create configured data dir: {e}"))?;
            }

            services::settings_service::apply_pending_restore(&settings_path, &data_dir)
                .map_err(|e| format!("failed to apply pending restore: {e}"))?;

            services::migration_service::run_data_migrations(&data_dir)
                .map_err(|e| format!("failed to migrate data: {e}"))?;

            let conn = db::connection::open_connection(&data_dir)
                .map_err(|e| format!("failed to open database: {e}"))?;

            app.manage(AppState {
                db: Mutex::new(conn),
                data_dir,
                settings_path,
            });
            Ok(())
        })
        .invoke_handler(builder.invoke_handler())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_bindings() {
        export_typescript_bindings("../src/bindings.ts").expect("export bindings");
    }
}
