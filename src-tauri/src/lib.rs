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
    let _ = dotenvy::from_filename("../.env");

    tauri::Builder::default()
    .plugin(tauri_plugin_window_state::Builder::default().build())
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
        commands::webview::hide_tab_webview,
        commands::webview::close_tab_webview,
        commands::webview::open_menu_webview,
        commands::webview::close_menu_webview,
        commands::ai::usage_status,
        commands::ai::ai_chat,
        commands::webview::tab_back,
        commands::webview::tab_forward,
        commands::webview::tab_reload,
        commands::webview::tab_print,
        commands::webview::set_tab_zoom,
    ])
    .run(tauri::generate_context!())
    .expect("Errore Avvio")
}