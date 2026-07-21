// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
mod commands;
mod db;
mod error;
mod state;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
    .setup(|app| {
        let dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("glow.db");
    
    let pool = tauri::async_runtime::block_on(db::connection::init(&db_path))?;
    app.manage(AppState {
        db: pool,
        views: Mutex::new(HashMap::new()),
    });

    Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        commands::history::record_visit,
        commands::history::recent_history,
        commands::history::search_history,
        commands::history::clear_history,
        commands::webview::open_tab_webview,
        commands::webview::navigate_tab_webview,
        commands::webview::set_tab_bounds,
        commands::webview::show_tab_webview,
        commands::webview::close_tab_webview,
    ])
    .run(tauri::generate_context!())
    .expect("Errore Avvio")
}