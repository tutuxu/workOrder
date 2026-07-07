//! workOrder 桌面应用 Rust 后端入口。
//!
//! 分层：commands → services → db，共享 models 与 `ServiceError`。
//! 前端通过 Tauri `invoke` 调用 Command；类型绑定由 tauri-specta 导出至 `src/bindings.ts`。

mod commands;
mod db;
mod error;
mod models;
mod services;

use std::sync::Mutex;

use rusqlite::Connection;
use specta_typescript::Typescript;
use tauri::Manager;
use tauri_specta::{collect_commands, Builder, ErrorHandlingMode};

pub struct AppState {
    pub db: Mutex<Connection>,
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
        commands::progress_log::list_progress_logs,
        commands::progress_log::add_progress_log,
        commands::progress_log::update_progress_log,
        commands::progress_log::delete_progress_log,
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
        .setup(|app| {
            let data_dir = db::connection::resolve_data_dir(None);
            let conn = db::connection::open_connection(&data_dir)
                .map_err(|e| format!("failed to open database: {e}"))?;
            app.manage(AppState {
                db: Mutex::new(conn),
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
