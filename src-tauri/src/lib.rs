mod commands;
mod db;
mod error;
mod models;
mod services;

use std::sync::Mutex;

use rusqlite::Connection;
use tauri::Manager;

pub struct AppState {
    pub db: Mutex<Connection>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
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
        .invoke_handler(tauri::generate_handler![
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
