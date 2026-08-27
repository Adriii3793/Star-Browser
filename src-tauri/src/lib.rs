mod commands;
mod db;
mod error;
mod state;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tauri::Emitter;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use state::AppState;

fn shortcut_bindings() -> Vec<(Shortcut, &'static str)> {
    vec! [
        ("ctrl+shift+t".parse().unwrap(), "newtab"),
        ("ctrl+shift+h".parse().unwrap(), "history"),
        ("ctrl+shift+s".parse().unwrap(), "chat"),
    ]
}

#[cfg(windows)]
fn strip_system_frame(window: &tauri::WebviewWindow) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{
        DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE,
        DWMWCP_DONOTROUND, DWM_WINDOW_CORNER_PREFERENCE,
    };

    const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;

    let Ok(raw) = window.hwnd() else {
        return;
    };
    let hwnd = HWND(raw.0 as *mut core::ffi::c_void);

    unsafe {
        let corner: DWM_WINDOW_CORNER_PREFERENCE = DWMWCP_DONOTROUND;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &corner as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );

        let border = DWMWA_COLOR_NONE;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_BORDER_COLOR,
            &border as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = dotenvy::from_filename("../env");
    let _ = dotenvy::dotenv();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
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
        last_tab_urls: Arc::new(Mutex::new(HashMap::new())),
    });
    app.manage(commands::webview::PermissionRegistry::default());
    commands::webview::grant_main_window_media(app.handle());

    if let Some(win) = app.get_webview_window("main") {
        #[cfg(windows)]
        strip_system_frame(&win);
        let _ = win.show();
    }
    let handle = app.handle().clone();
    for (shortcut, action) in shortcut_bindings() {
        let action = action.to_string();
        let handle_for_event = handle.clone();
        let _ = app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, event| {
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
        commands::webview::warm_overlay_webview,
        commands::webview::close_overlay_webview,
        commands::ai::ai_chat,
        commands::page::fetch_page_context,
        commands::webview::read_tab_page,
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
        commands::webview::set_tab_muted,
        commands::webview::tab_media_toggle,
        commands::webview::tab_stop_media,
        commands::webview::set_adblock,
        commands::webview::pending_permission,
        commands::webview::current_permission,
        commands::tabs::save_tab_session,
        commands::tabs::load_tab_session,
    ])
    .run(tauri::generate_context!())
    .expect("failed to start star");
}