mod commands;
mod db;
mod error;
mod state;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use state::AppState;

/// Default chrome colour, matching `--bg-chrome` of the light theme in `app.css`.
/// Used until the frontend reports the active theme via `set_caption_color`.
#[cfg(windows)]
const DEFAULT_CAPTION_COLOR: &str = "#faf7f7";

/// `#rrggbb` (or `#rgb`) to a Win32 `COLORREF`, which is `0x00bbggrr`.
#[cfg(windows)]
fn colorref_from_hex(hex: &str) -> Option<u32> {
    let raw = hex.trim().trim_start_matches('#');
    let full = match raw.len() {
        3 => raw.chars().flat_map(|c| [c, c]).collect::<String>(),
        6 => raw.to_owned(),
        _ => return None,
    };
    let r = u32::from_str_radix(&full[0..2], 16).ok()?;
    let g = u32::from_str_radix(&full[2..4], 16).ok()?;
    let b = u32::from_str_radix(&full[4..6], 16).ok()?;
    Some(r | (g << 8) | (b << 16))
}

/// Paints the 1px non-client strip that Windows 11 keeps at the top of an
/// undecorated window with `shadow: true`. tao inserts that inset in
/// `WM_NCCALCSIZE` so DWM still draws the drop shadow; DWM fills it with the
/// caption colour, which otherwise reads as a hard line above our chrome.
#[cfg(windows)]
fn apply_caption_color(window: &tauri::WebviewWindow, hex: &str) {
    use windows::Win32::Foundation::HWND;
    use windows::Win32::Graphics::Dwm::{DwmSetWindowAttribute, DWMWA_CAPTION_COLOR};

    let (Ok(raw), Some(color)) = (window.hwnd(), colorref_from_hex(hex)) else {
        return;
    };
    let hwnd = HWND(raw.0 as *mut core::ffi::c_void);

    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_CAPTION_COLOR,
            &color as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<u32>() as u32,
        );
    }
}

#[tauri::command]
fn set_caption_color(window: tauri::WebviewWindow, color: String) {
    #[cfg(windows)]
    apply_caption_color(&window, &color);
    #[cfg(not(windows))]
    let _ = (window, color);
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

    apply_caption_color(window, DEFAULT_CAPTION_COLOR);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::dotenv();

    tauri::Builder::default()
        .plugin(tauri_plugin_os::init())
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

    Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        set_caption_color,
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
        commands::webview::warm_overlay_webview,
        commands::webview::close_overlay_webview,
        commands::ai::ai_chat,
        commands::page::fetch_page_context,
        commands::webview::read_tab_page,
        commands::setup::is_setup_complete,
        commands::setup::save_setup,
        commands::setup::load_setup,
        commands::files::save_text_file,
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
    .expect("failed to start star")
}