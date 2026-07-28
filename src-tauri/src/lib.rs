mod commands;
mod db;
mod error;
mod state;

use std::collections::HashMap;
use std::sync::Mutex;
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use state::AppState;

fn shortcut_bindings() -> Vec<(Shortcut, &'static str)> {
    vec![
        ("ctrl+shift+t".parse().unwrap(), "new-tab"),        
        ("ctrl+shift+h".parse().unwrap(), "show-history"),
        ("ctrl+shift+s".parse().unwrap(), "open-ai-chat"),
    ]
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Dev convenience only: this relative path resolves when running from src-tauri/.
    // Installed builds get the key from the value build.rs embedded (see commands::ai).
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::dotenv();

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
    .setup(|app| {
        let dir = app.path().app_data_dir()?;
        std::fs::create_dir_all(&dir)?;
        let db_path = dir.join("glow.db");
    
    let pool = tauri::async_runtime::block_on(db::connection::init(&db_path))?;
    app.manage(AppState {
        db: pool,
        views: Mutex::new(HashMap::new()),
    });
    let handle = app.handle().clone();
    for (shortcut, action) in shortcut_bindings() {
        let action = action.to_string();
        let shortcut_for_handler = shortcut.clone();
        let handle_for_event = handle.clone();
        let _ = app.global_shortcut().on_shortcut(shortcut_for_handler, move |_app, _shortcut, event| {
            if event.state != ShortcutState::Pressed {
                return;
            }
            let _ = handle_for_event.emit("global-shortcut", action.as_str());
        });
        let _ = app.global_shortcut().register(shortcut);
    }

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
        commands::webview::open_overlay_webview,
        commands::webview::close_overlay_webview,
        commands::ai::usage_status,
        commands::ai::ai_chat,
        commands::page::fetch_page_context,
        commands::setup::is_setup_complete,
        commands::setup::save_setup,
        commands::setup::load_setup,
        commands::setup::reset_setup,
        commands::setup::set_default_browser,
        commands::setup::data_dir,
        commands::setup::open_data_dir,
        commands::webview::tab_back,
        commands::webview::tab_forward,
        commands::webview::tab_reload,
        commands::webview::tab_print,
        commands::webview::set_tab_zoom,
    ])
    .run(tauri::generate_context!())
    .expect("Errore Avvio")
}