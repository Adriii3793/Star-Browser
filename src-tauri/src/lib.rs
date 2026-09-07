mod commands;
mod db;
mod error;
#[cfg(gtk)]
mod gtk_layout;
#[cfg(target_os = "linux")]
mod shutdown;
mod state;

use state::AppState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

#[cfg(windows)]
const DEFAULT_CAPTION_COLOR: &str = "#faf7f7";

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

#[tauri::command]
fn window_tiled() -> Option<bool> {
    #[cfg(gtk)]
    {
        Some(gtk_layout::is_tiled())
    }
    #[cfg(not(gtk))]
    {
        None
    }
}

fn shortcut_bindings() -> Vec<(Shortcut, &'static str)> {
    vec![
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

    apply_caption_color(window, DEFAULT_CAPTION_COLOR);
}

#[cfg(gtk)]
fn configure_webkit_renderer() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some() {
        return;
    }
    let opted_out = matches!(
        std::env::var("STAR_WEBKIT_FORCE_NODMABUF")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes"
    );
    if opted_out {
        std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
    }
}

fn teardown(app: &tauri::AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    for (_, webview) in state.views_snapshot() {
        let _ = webview.eval("(function(){var m=window.__starMedia;if(m)m.stop();})();");
        let _ = webview.close();
    }
    state.views().clear();
}

fn load_env_beside_executable() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let mut dir = exe.parent();
    for _ in 0..4 {
        let Some(here) = dir else { return };
        let _ = dotenvy::from_path(here.join(".env"));
        dir = here.parent();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = dotenvy::from_filename("../.env");
    let _ = dotenvy::dotenv();
    load_env_beside_executable();
    #[cfg(gtk)]
    configure_webkit_renderer();

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
            let setup_pending =
                tauri::async_runtime::block_on(commands::setup::setup_pending(&pool));
            app.manage(AppState {
                db: pool,
                views: Mutex::new(HashMap::new()),
                last_tab_urls: Arc::new(Mutex::new(HashMap::new())),
                history_write_error: Mutex::new(None),
            });
            app.manage(commands::webview::PermissionRegistry::default());
            commands::webview::grant_main_window_media(app.handle());

            if let Some(win) = app.get_webview_window("main") {
                #[cfg(windows)]
                strip_system_frame(&win);

                #[cfg(gtk)]
                if let Ok(vbox) = win.default_vbox() {
                    gtk_layout::install(&vbox);
                }

                #[cfg(gtk)]
                if let Ok(gtk_window) = win.gtk_window() {
                    gtk_layout::track_window_state(&gtk_window);
                }

                if setup_pending {
                    let _ = win.set_size(tauri::LogicalSize::new(880.0, 640.0));
                    let _ = win.center();
                }

                if let Err(error) = win.show() {
                    eprintln!("[star] window: failed to show the main window: {error:?}");
                }
            }

            let handle = app.handle().clone();
            for (shortcut, action) in shortcut_bindings() {
                let action = action.to_string();
                let handle_for_event = handle.clone();
                let _ =
                    app.global_shortcut()
                        .on_shortcut(shortcut, move |_app, _shortcut, event| {
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
            set_caption_color,
            window_tiled,
            commands::history::record_visit,
            commands::history::recent_history,
            commands::history::search_history,
            commands::history::clear_history,
            commands::history::history_write_error,
            commands::webview::open_tab_webview,
            commands::webview::navigate_tab_webview,
            commands::webview::set_tab_bounds,
            commands::webview::show_tab_webview,
            commands::webview::hide_tab_webview,
            commands::webview::close_tab_webview,
            commands::webview::open_menu_webview,
            commands::webview::close_menu_webview,
            commands::webview::open_overlay_webview,
            commands::webview::warm_menu_webview,
            commands::webview::warm_overlay_webview,
            commands::webview::close_overlay_webview,
            commands::webview::set_surface_clip,
            commands::webview::set_fullscreen_restore,
            commands::webview::take_fullscreen_restore,
            commands::ai::ai_chat,
            commands::ai::ai_key_status,
            commands::ai::set_ai_key,
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
            commands::history::retitle_visit,
            commands::tabs::save_tab_session,
            commands::tabs::load_tab_session,
        ])
        .build(tauri::generate_context!())
        .expect("failed to start star")
        .run(|handle, event| match event {
            tauri::RunEvent::ExitRequested { .. } => teardown(handle),
            tauri::RunEvent::WindowEvent { label, event, .. } if label == "main" => match event {
                tauri::WindowEvent::Resized(size) => {
                    commands::webview::window_resized(handle, size)
                }
                tauri::WindowEvent::ScaleFactorChanged {
                    scale_factor,
                    new_inner_size,
                    ..
                } => commands::webview::window_resized_with_scale(
                    handle,
                    new_inner_size,
                    scale_factor,
                ),
                tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed => {
                    teardown(handle)
                }
                _ => {}
            },
            tauri::RunEvent::Exit => {
                #[cfg(target_os = "linux")]
                shutdown::reap_children();
            }
            _ => {}
        });
}
