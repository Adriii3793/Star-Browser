use crate::error::AppError;
use crate::state::lock_recover;
use crate::state::AppState;
use std::path::Path;
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl};

fn label_for(tab_id: &str) -> String {
    format!("tab-{tab_id}")
}

type Rect = (f64, f64, f64, f64);

static TAB_RECTS: std::sync::Mutex<std::collections::BTreeMap<String, Rect>> =
    std::sync::Mutex::new(std::collections::BTreeMap::new());
static WINDOW_SIZE: std::sync::Mutex<Option<(f64, f64)>> = std::sync::Mutex::new(None);

#[derive(Clone, Copy, PartialEq)]
struct Corners {
    radius: f64,
    bottom_left: bool,
    bottom_right: bool,
}

impl Corners {
    fn from_args(
        radius: Option<f64>,
        bottom_left: Option<bool>,
        bottom_right: Option<bool>,
    ) -> Self {
        Self {
            radius: radius.unwrap_or(TAB_CORNER_RADIUS),
            bottom_left: bottom_left.unwrap_or(true),
            bottom_right: bottom_right.unwrap_or(true),
        }
    }
}

fn place_tab_view(
    label: Option<&str>,
    webview: &tauri::Webview,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    follow_window: bool,
    restore_load: bool,
    corners: Option<Corners>,
) -> Result<(), AppError> {
    #[cfg(not(gtk))]
    let _ = (restore_load, corners);

    if follow_window {
        if let Some(label) = label {
            lock_recover(&TAB_RECTS).insert(label.to_string(), (x, y, width, height));
        }
    }

    #[cfg(gtk)]
    {
        crate::gtk_layout::place(
            webview,
            x,
            y,
            width,
            height,
            follow_window,
            restore_load,
            corners.map(|corners| crate::gtk_layout::Corners {
                radius: corners.radius.round() as i32,
                bottom_left: corners.bottom_left,
                bottom_right: corners.bottom_right,
            }),
        );
    }
    #[cfg(not(gtk))]
    {
        webview.set_position(LogicalPosition::new(x, y))?;
        webview.set_size(LogicalSize::new(width, height))?;
    }
    Ok(())
}

fn forget_tab_rect(label: &str) {
    lock_recover(&TAB_RECTS).remove(label);
}

fn full_window_size(main: &tauri::Window) -> Result<(f64, f64), AppError> {
    let scale = main.scale_factor()?;
    let size = main.inner_size()?.to_logical::<f64>(scale);
    Ok((size.width, size.height))
}

fn is_full_window_surface(label: &str) -> bool {
    matches!(label, MENU_LABEL | OVERLAY_LABEL | PERMISSION_LABEL)
}

fn resized_surface_rect(
    label: &str,
    (x, y, width, height): Rect,
    next: (f64, f64),
    delta: Option<(f64, f64)>,
) -> Option<Rect> {
    if is_full_window_surface(label) {
        Some((0.0, 0.0, next.0.max(1.0), next.1.max(1.0)))
    } else if let Some((dw, dh)) = delta {
        Some((x, y, (width + dw).max(1.0), (height + dh).max(1.0)))
    } else {
        None
    }
}

#[cfg(test)]
mod layout_tests {
    use super::*;

    #[test]
    fn full_window_surfaces_fill_the_new_client_area_without_a_delta() {
        assert_eq!(
            resized_surface_rect(MENU_LABEL, (0.0, 0.0, 800.0, 600.0), (1920.0, 1080.0), None),
            Some((0.0, 0.0, 1920.0, 1080.0))
        );
        assert_eq!(
            resized_surface_rect(
                PERMISSION_LABEL,
                (0.0, 0.0, 800.0, 600.0),
                (1920.0, 1080.0),
                None
            ),
            Some((0.0, 0.0, 1920.0, 1080.0))
        );
    }

    #[test]
    fn page_surfaces_preserve_origin_and_apply_window_delta() {
        assert_eq!(
            resized_surface_rect(
                "tab-example",
                (10.0, 80.0, 790.0, 520.0),
                (1000.0, 800.0),
                Some((200.0, 100.0))
            ),
            Some((10.0, 80.0, 990.0, 620.0))
        );
        assert_eq!(
            resized_surface_rect("tab-example", (10.0, 80.0, 790.0, 520.0), (1000.0, 800.0), None),
            None
        );
    }
}

#[cfg(not(gtk))]
fn resize_surfaces(app: &AppHandle, size: tauri::PhysicalSize<u32>, scale: f64) {
    let scale = scale.max(f64::EPSILON);
    let next = (size.width as f64 / scale, size.height as f64 / scale);
    let previous = lock_recover(&WINDOW_SIZE).replace(next);
    let delta = previous
        .filter(|previous| previous.0 > 1.0 && previous.1 > 1.0)
        .map(|previous| (next.0 - previous.0, next.1 - previous.1));

    if previous == Some(next) {
        return;
    }

    let state = app.state::<AppState>();
    let tracked: Vec<(String, Rect)> = lock_recover(&TAB_RECTS)
        .iter()
        .map(|(label, rect)| (label.clone(), *rect))
        .collect();

    for (label, (x, y, width, height)) in tracked {
        let Some(webview) = state.view(&label) else {
            forget_tab_rect(&label);
            continue;
        };

        let Some((x, y, width, height)) = resized_surface_rect(
            &label,
            (x, y, width, height),
            next,
            delta,
        ) else {
            continue;
        };

        let _ = place_tab_view(
            Some(&label),
            &webview,
            x,
            y,
            width,
            height,
            true,
            false,
            None,
        );
    }
}

pub fn window_resized(app: &AppHandle, size: tauri::PhysicalSize<u32>) {
    #[cfg(gtk)]
    {
        let _ = (app, size);
        return;
    }

    #[cfg(not(gtk))]
    {
        let Some(main) = app.get_window("main") else {
            return;
        };
        resize_surfaces(app, size, main.scale_factor().unwrap_or(1.0));
    }
}

pub fn window_resized_with_scale(
    app: &AppHandle,
    size: tauri::PhysicalSize<u32>,
    scale: f64,
) {
    #[cfg(gtk)]
    {
        let _ = (app, size, scale);
    }

    #[cfg(not(gtk))]
    resize_surfaces(app, size, scale);
}

#[cfg(windows)]
const CLIPPED_SURFACE_SCRIPT: &str = "window.__starClippedSurface = true;";

const SURFACE_NEEDS_CLIP: bool = cfg!(windows);

#[derive(Clone, Copy, Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClipPart {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    #[serde(default)]
    pub radius: f64,
}

type ClipRect = (i32, i32, i32, i32, i32);


#[cfg_attr(not(windows), allow(dead_code))]
fn clip_rects(parts: &[ClipPart], scale: f64) -> Vec<ClipRect> {
    parts
        .iter()
        .map(|part| {
            let left = (part.x * scale).floor();
            let top = (part.y * scale).floor();
            let right = ((part.x + part.width) * scale).ceil();
            let bottom = ((part.y + part.height) * scale).ceil();
            (
                left as i32,
                top as i32,
                ((right - left) as i32).max(1),
                ((bottom - top) as i32).max(1),
                (part.radius * scale).round().max(0.0) as i32,
            )
        })
        .collect()
}

#[allow(unused_variables)]
fn clip_surface(webview: &tauri::Webview, parts: &[ClipPart], scale: f64) {
    #[cfg(windows)]
    win_permissions::clip_to_rects(webview, &clip_rects(parts, scale));
}

#[cfg(test)]
mod clip_tests {
    use super::*;

    fn part(x: f64, y: f64, width: f64, height: f64, radius: f64) -> ClipPart {
        ClipPart {
            x,
            y,
            width,
            height,
            radius,
        }
    }

    #[test]
    fn each_part_keeps_its_own_radius_rather_than_a_shared_one() {
        let rects = clip_rects(&[part(0.0, 0.0, 100.0, 50.0, 14.0), part(0.0, 0.0, 10.0, 10.0, 0.0)], 1.0);
        assert_eq!(rects[0].4, 14);
        assert_eq!(rects[1].4, 0);
    }

    #[test]
    fn scaling_rounds_outward_so_a_card_is_never_shaved() {
        let rects = clip_rects(&[part(10.4, 20.6, 100.2, 50.7, 8.0)], 1.0);
        let (x, y, width, height, radius) = rects[0];
        assert_eq!((x, y), (10, 20));
        assert!(x as f64 <= 10.4 && y as f64 <= 20.6);
        assert!((x + width) as f64 >= 10.4 + 100.2);
        assert!((y + height) as f64 >= 20.6 + 50.7);
        assert_eq!(radius, 8);
    }

    #[test]
    fn a_radius_scales_with_the_display() {
        let rects = clip_rects(&[part(0.0, 0.0, 100.0, 100.0, 14.0)], 2.0);
        assert_eq!(rects[0].4, 28);
        assert_eq!((rects[0].2, rects[0].3), (200, 200));
    }

    #[test]
    fn nothing_painted_is_nothing_clipped_to() {
        assert!(clip_rects(&[], 1.0).is_empty());
    }

}

fn create_overlay_view(
    main: &tauri::Window,
    label: &str,
    page: &str,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<tauri::Webview, AppError> {
    let builder = WebviewBuilder::new(label, WebviewUrl::App(page.into())).transparent(true);
    #[cfg(windows)]
    let builder = if clippable_surface(label) {
        builder.initialization_script(CLIPPED_SURFACE_SCRIPT)
    } else {
        builder
    };
    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    place_tab_view(Some(label), &webview, x, y, width, height, true, true, None)?;
    if SURFACE_NEEDS_CLIP && clippable_surface(label) {
        clip_surface(&webview, &[], 1.0);
    }
    Ok(webview)
}

fn blank_reshown_surface(label: &str, webview: &tauri::Webview) {
    if SURFACE_NEEDS_CLIP && clippable_surface(label) {
        clip_surface(webview, &[], 1.0);
    }
}

fn clippable_surface(label: &str) -> bool {
    matches!(label, MENU_LABEL | OVERLAY_LABEL)
}

fn show_tab_view(webview: &tauri::Webview) -> Result<(), AppError> {
    webview.show()?;
    Ok(())
}

fn hide_tab_view(webview: &tauri::Webview) -> Result<(), AppError> {
    webview.hide()?;
    Ok(())
}

fn close_tab_view(webview: &tauri::Webview) {
    #[cfg(gtk)]
    crate::gtk_layout::forget(webview);
    let _ = webview.close();
}

type TabUrls = std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, String>>>;

fn report_main_frame_url(app: &AppHandle, tab_id: &str, last_urls: &TabUrls, url: &str) {
    if url.starts_with("http") {
        lock_recover(last_urls).insert(tab_id.to_string(), url.to_string());
    }
    let _ = app.emit(
        "tab-url-changed",
        TabUrlChanged {
            tab_id: tab_id.to_string(),
            url: url.to_string(),
            replaced: false,
        },
    );
}

const NAVIGATION_IS_MAIN_FRAME_ONLY: bool = cfg!(any(windows, target_os = "macos"));

fn handle_navigation(app: &AppHandle, tab_id: &str, last_urls: &TabUrls, url: &url::Url) -> bool {
    if NAVIGATION_IS_MAIN_FRAME_ONLY {
        report_main_frame_url(app, tab_id, last_urls, url.as_str());
    }
    true
}

fn handle_title(app: &AppHandle, tab_id: &str, last_urls: &TabUrls, title: &str) {
    let addressed = title
        .strip_prefix(URL_TITLE_PREFIX)
        .map(|url| (url, false))
        .or_else(|| {
            title
                .strip_prefix(REPLACED_URL_TITLE_PREFIX)
                .map(|url| (url, true))
        });
    if let Some((url, replaced)) = addressed {
        let current = lock_recover(last_urls).get(tab_id).cloned();
        if url_sentinel_allowed(current.as_deref(), url) {
            let _ = app.emit(
                "tab-url-changed",
                TabUrlChanged {
                    tab_id: tab_id.to_string(),
                    url: url.to_string(),
                    replaced,
                },
            );
        }
    } else if let Some(icon) = title.strip_prefix(ICON_TITLE_PREFIX) {
        if icon.len() <= 2048 && (icon.starts_with("https://") || icon.starts_with("http://")) {
            let _ = app.emit(
                "tab-icon-changed",
                TabIconChanged {
                    tab_id: tab_id.to_string(),
                    url: icon.to_string(),
                },
            );
        }
    } else if let Some(action) = title.strip_prefix(SHORTCUT_TITLE_PREFIX) {
        #[cfg(windows)]
        let _ = action;
        #[cfg(not(windows))]
        if shortcut_is_forgeable_safely(action) {
            let _ = app.emit(
                "tab-shortcut",
                TabShortcut {
                    tab_id: tab_id.to_string(),
                    action: action.to_string(),
                },
            );
        }
    } else if let Some(payload) = title.strip_prefix(AUDIO_TITLE_PREFIX) {
        #[cfg(not(windows))]
        {
            let mut parts = payload.split(',');
            let audible = parts.next() == Some("1");
            let muted = parts.next() == Some("1");
            let _ = app.emit(
                "tab-audio-changed",
                TabAudioChanged {
                    tab_id: tab_id.to_string(),
                    audible,
                    muted,
                },
            );
        }
        #[cfg(windows)]
        let _ = payload;
    } else {
        let clean_title = title.trim();
        if !clean_title.is_empty() {
            let _ = app.emit(
                "tab-title-changed",
                TabTitleChanged {
                    tab_id: tab_id.to_string(),
                    title: clean_title.to_string(),
                },
            );
        }
    }
}

#[cfg(not(windows))]
fn handle_download(
    app: &AppHandle,
    tab_id: &str,
    event: tauri::webview::DownloadEvent<'_>,
) -> bool {
    match event {
        tauri::webview::DownloadEvent::Requested { url, destination } => {
            let suggested = destination.to_string_lossy().into_owned();
            let file_name = file_name_for(&suggested, url.as_str());
            let target = unique_download_path(app, &file_name);
            let file_name = target
                .file_name()
                .and_then(|name| name.to_str())
                .map(str::to_owned)
                .unwrap_or(file_name);
            *destination = target;

            if let Err(e) = app.emit(
                "download-started",
                DownloadStarted {
                    tab_id: tab_id.to_string(),
                    file_name,
                },
            ) {
                eprintln!("[star] downloads: failed to emit download-started: {e:?}");
            }
        }
        tauri::webview::DownloadEvent::Finished { url, path, success } => {
            let resolved = path
                .as_ref()
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_default();
            let file_name = file_name_for(&resolved, url.as_str());

            if let Err(e) = app.emit(
                "download-finished",
                DownloadFinished {
                    tab_id: tab_id.to_string(),
                    file_name,
                    success,
                },
            ) {
                eprintln!("[star] downloads: failed to emit download-finished: {e:?}");
            }
        }
        _ => {}
    }
    true
}

mod keyval {
    pub const BACK_SPACE: u32 = 0xff08;
    pub const DELETE: u32 = 0xffff;
    pub const F5: u32 = 0xffc2;
    pub const F11: u32 = 0xffc8;
    pub const ZERO: u32 = b'0' as u32;
    pub const PLUS: u32 = b'+' as u32;
    pub const MINUS: u32 = b'-' as u32;
    pub const EQUAL: u32 = b'=' as u32;
    pub const UNDERSCORE: u32 = b'_' as u32;
    pub const H: u32 = b'h' as u32;
    pub const K: u32 = b'k' as u32;
    pub const P: u32 = b'p' as u32;
    pub const R: u32 = b'r' as u32;
    pub const S: u32 = b's' as u32;
    pub const T: u32 = b't' as u32;
    pub const W: u32 = b'w' as u32;
}

fn ascii_lower_keyval(keyval: u32) -> u32 {
    if (b'A' as u32..=b'Z' as u32).contains(&keyval) {
        keyval + 32
    } else {
        keyval
    }
}

#[cfg_attr(not(gtk), allow(dead_code))]
fn gtk_accelerator_action(keyval: u32, ctrl: bool, shift: bool, alt: bool) -> Option<&'static str> {
    if keyval == keyval::F11 {
        return (!ctrl && !shift && !alt).then_some("fullscreen");
    }
    if keyval == keyval::F5 {
        return (!ctrl && !shift && !alt).then_some("reload");
    }
    if !ctrl || alt {
        return None;
    }
    let key = ascii_lower_keyval(keyval);
    if shift {
        return match key {
            keyval::DELETE | keyval::BACK_SPACE => Some("cleardata"),
            keyval::S => Some("chat"),
            _ => None,
        };
    }
    match key {
        keyval::EQUAL | keyval::PLUS => Some("zoomin"),
        keyval::MINUS | keyval::UNDERSCORE => Some("zoomout"),
        keyval::ZERO => Some("zoomreset"),
        keyval::T => Some("newtab"),
        keyval::W => Some("closetab"),
        keyval::H => Some("history"),
        keyval::P => Some("print"),
        keyval::R => Some("reload"),
        keyval::K => Some("search"),
        _ => None,
    }
}

#[cfg(test)]
mod gtk_accelerator_tests {
    use super::{gtk_accelerator_action as action, keyval};

    const NONE: (bool, bool, bool) = (false, false, false);
    const CTRL: (bool, bool, bool) = (true, false, false);
    const CTRL_SHIFT: (bool, bool, bool) = (true, true, false);

    fn act(k: u32, (c, s, a): (bool, bool, bool)) -> Option<&'static str> {
        action(k, c, s, a)
    }

    #[test]
    fn the_actions_the_title_channel_must_refuse_are_reachable_from_a_real_key() {
        assert_eq!(act(keyval::P, CTRL), Some("print"));
        assert_eq!(act(keyval::W, CTRL), Some("closetab"));
        assert_eq!(act(keyval::F11, NONE), Some("fullscreen"));
        assert_eq!(act(keyval::DELETE, CTRL_SHIFT), Some("cleardata"));
    }

    #[test]
    fn it_matches_the_injected_script() {
        assert_eq!(act(keyval::R, CTRL), Some("reload"));
        assert_eq!(act(keyval::F5, NONE), Some("reload"));
        assert_eq!(act(keyval::T, CTRL), Some("newtab"));
        assert_eq!(act(keyval::H, CTRL), Some("history"));
        assert_eq!(act(keyval::K, CTRL), Some("search"));
        assert_eq!(act(keyval::ZERO, CTRL), Some("zoomreset"));
        assert_eq!(act(keyval::EQUAL, CTRL), Some("zoomin"));
        assert_eq!(act(keyval::MINUS, CTRL), Some("zoomout"));
        assert_eq!(act(keyval::S, CTRL_SHIFT), Some("chat"));
        assert_eq!(act(keyval::BACK_SPACE, CTRL_SHIFT), Some("cleardata"));
    }

    #[test]
    fn shift_reaches_the_binding_through_the_capital_it_produces() {
        assert_eq!(act(b'S' as u32, CTRL_SHIFT), Some("chat"));
    }

    #[test]
    fn a_cache_ignoring_reload_stays_unbound() {
        assert_eq!(act(keyval::R, CTRL_SHIFT), None);
    }

    #[test]
    fn text_editing_keys_are_left_to_the_page() {
        for key in [b'a', b'c', b'v', b'x', b'z', b'f'] {
            assert_eq!(act(key as u32, CTRL), None, "ctrl+{} must reach the page", key as char);
        }
    }

    #[test]
    fn a_bare_or_alt_combination_is_not_a_shortcut() {
        assert_eq!(act(keyval::P, NONE), None);
        assert_eq!(act(keyval::P, (true, false, true)), None);
        assert_eq!(act(keyval::F11, CTRL), None);
        assert_eq!(act(keyval::F5, CTRL), None);
    }
}

#[allow(unused_variables)]
fn finish_tab_setup(
    app: &AppHandle,
    tab_id: &str,
    webview: &tauri::Webview,
    width: f64,
    height: f64,
    radius: Option<f64>,
    round_bottom_left: Option<bool>,
    round_bottom_right: Option<bool>,
) {
    #[cfg(gtk)]
    {
        let app = app.clone();
        let tab_id = tab_id.to_string();
        crate::gtk_layout::on_key_press(webview, move |keyval, ctrl, shift, alt| {
            let Some(action) = gtk_accelerator_action(keyval, ctrl, shift, alt) else {
                return false;
            };
            let _ = app.emit(
                "tab-shortcut",
                TabShortcut {
                    tab_id: tab_id.clone(),
                    action: action.to_string(),
                },
            );
            true
        });
    }

    #[cfg(windows)]
    {
        win_permissions::register(app, tab_id.to_string(), webview);
        let scale = webview.window().scale_factor().unwrap_or(1.0);
        win_permissions::round_corners(
            webview,
            (width * scale).round() as i32,
            (height * scale).round() as i32,
            (radius.unwrap_or(TAB_CORNER_RADIUS) * scale).round() as i32,
            round_bottom_left.unwrap_or(true),
            round_bottom_right.unwrap_or(true),
        );
    }
}

#[cfg_attr(windows, allow(dead_code))]
fn shortcut_is_forgeable_safely(action: &str) -> bool {
    matches!(
        action,
        "zoomin" | "zoomout" | "zoomreset" | "newtab" | "history" | "search" | "chat" | "reload"
    )
}

#[cfg(test)]
mod shortcut_trust_tests {
    use super::shortcut_is_forgeable_safely;

    #[test]
    fn the_actions_a_hostile_page_could_abuse_are_refused() {
        for action in ["fullscreen", "print", "closetab", "cleardata"] {
            assert!(
                !shortcut_is_forgeable_safely(action),
                "{action} must not be reachable from a page-written title"
            );
        }
    }

    #[test]
    fn ordinary_navigation_shortcuts_still_pass() {
        for action in [
            "zoomin",
            "zoomout",
            "zoomreset",
            "newtab",
            "history",
            "search",
            "chat",
        ] {
            assert!(shortcut_is_forgeable_safely(action));
        }
    }

    #[test]
    fn an_unknown_action_is_refused() {
        assert!(!shortcut_is_forgeable_safely("quit"));
        assert!(!shortcut_is_forgeable_safely(""));
    }
}

fn url_sentinel_allowed(current: Option<&str>, incoming: &str) -> bool {
    let incoming_is_http = incoming.starts_with("https://") || incoming.starts_with("http://");
    match current {
        None => incoming_is_http,
        Some(page) => same_origin(page, incoming) || (incoming_is_http && page == incoming),
    }
}

fn same_origin(a: &str, b: &str) -> bool {
    match (url::Url::parse(a), url::Url::parse(b)) {
        (Ok(a), Ok(b)) => a.origin().is_tuple() && a.origin() == b.origin(),
        _ => false,
    }
}

#[cfg(all(test, windows))]
mod accelerator_tests {
    use super::win_permissions::accelerator_action;

    const CTRL: (bool, bool, bool) = (true, false, false);
    const CTRL_SHIFT: (bool, bool, bool) = (true, true, false);
    const NONE: (bool, bool, bool) = (false, false, false);

    fn action(vk: u32, (c, s, a): (bool, bool, bool)) -> Option<&'static str> {
        accelerator_action(vk, c, s, a)
    }

    #[test]
    fn maps_the_shortcuts_the_injected_script_used_to_forward() {
        assert_eq!(action(0x54, CTRL), Some("newtab"));
        assert_eq!(action(0x57, CTRL), Some("closetab"));
        assert_eq!(action(0x48, CTRL), Some("history"));
        assert_eq!(action(0x50, CTRL), Some("print"));
        assert_eq!(action(0x4B, CTRL), Some("search"));
        assert_eq!(action(0x52, CTRL), Some("reload"));
        assert_eq!(action(0x74, NONE), Some("reload"));
        assert_eq!(action(0x30, CTRL), Some("zoomreset"));
        assert_eq!(action(0xBB, CTRL), Some("zoomin"));
        assert_eq!(action(0xBD, CTRL), Some("zoomout"));
        assert_eq!(action(0x53, CTRL_SHIFT), Some("chat"));
        assert_eq!(action(0x2E, CTRL_SHIFT), Some("cleardata"));
        assert_eq!(action(0x08, CTRL_SHIFT), Some("cleardata"));
        assert_eq!(action(0x7A, NONE), Some("fullscreen"));
    }

    #[test]
    fn ignores_keys_that_are_not_ours() {
        assert_eq!(action(0x54, NONE), None);
        assert_eq!(action(0x41, CTRL), None); 
        assert_eq!(action(0x43, CTRL), None);
        assert_eq!(action(0x56, CTRL), None);
        assert_eq!(action(0x54, (true, false, true)), None);
        assert_eq!(action(0x7A, CTRL), None);
        assert_eq!(action(0x74, CTRL), None);
        assert_eq!(action(0x52, CTRL_SHIFT), None);
        assert_eq!(action(0x54, CTRL_SHIFT), None);
    }
}

#[cfg(test)]
mod origin_tests {
    use super::{same_origin, url_sentinel_allowed};

    #[test]
    fn accepts_same_document_history_changes() {
        assert!(same_origin(
            "https://app.example/a",
            "https://app.example/b?q=1"
        ));
        assert!(same_origin(
            "https://app.example:8443/a",
            "https://app.example:8443/b"
        ));
    }

    #[test]
    fn rejects_a_page_claiming_another_sites_address() {
        assert!(!same_origin(
            "https://evil.example/",
            "https://bank.example/login"
        ));
        assert!(!same_origin("https://app.example/", "http://app.example/"));
        assert!(!same_origin(
            "https://app.example/",
            "https://app.example:8443/"
        ));
        assert!(!same_origin(
            "https://app.example/",
            "https://sub.app.example/"
        ));
    }

    #[test]
    fn rejects_opaque_and_unparseable_urls() {
        assert!(!same_origin("data:text/html,a", "data:text/html,a"));
        assert!(!same_origin("about:blank", "about:blank"));
        assert!(!same_origin("https://app.example/", "not a url"));
        assert!(!same_origin("", ""));
    }

    #[test]
    fn sentinel_allows_a_first_http_url_when_nothing_has_been_recorded_yet() {
        assert!(url_sentinel_allowed(None, "https://youtube.com/watch?v=x"));
        assert!(url_sentinel_allowed(None, "http://local.example/"));
    }

    #[test]
    fn sentinel_bootstrapping_refuses_non_http_schemes_out_of_nothing() {
        assert!(!url_sentinel_allowed(None, "data:text/html,<h1>Bank</h1>"));
        assert!(!url_sentinel_allowed(None, "javascript:stealCookies()"));
        assert!(!url_sentinel_allowed(None, "about:config"));
        assert!(!url_sentinel_allowed(None, ""));
    }

    #[test]
    fn sentinel_allows_classic_same_document_history_changes() {
        assert!(url_sentinel_allowed(
            Some("https://app.example/a"),
            "https://app.example/b?q=1"
        ));
    }

    #[test]
    fn sentinel_allows_a_byte_identical_retransmission_of_the_committed_url() {
        assert!(url_sentinel_allowed(
            Some("https://youtube.com/watch?v=x"),
            "https://youtube.com/watch?v=x"
        ));
    }

    #[test]
    fn sentinel_retransmission_does_not_widen_scheme_or_port() {
        assert!(!url_sentinel_allowed(
            Some("https://app.example/"),
            "http://app.example/"
        ));
        assert!(!url_sentinel_allowed(
            Some("https://app.example/"),
            "https://app.example:8443/"
        ));
    }

    #[test]
    fn sentinel_refuses_the_post_redirect_stale_poll_and_any_spoof() {
        assert!(!url_sentinel_allowed(
            Some("https://www.youtube.com/"),
            "https://accounts.google.com/ServiceLogin?service=youtube&continue=..."
        ));
        assert!(!url_sentinel_allowed(
            Some("https://evil.example/"),
            "https://bank.example/login"
        ));
    }

    #[test]
    fn sentinel_retransmission_still_refuses_non_http_payloads() {
        assert!(!url_sentinel_allowed(
            Some("data:text/html,x"),
            "data:text/html,x"
        ));
    }
}

fn raise_to_front(webview: &tauri::Webview, main: &tauri::Window) {
    #[cfg(gtk)]
    {
        let _ = main;
        crate::gtk_layout::raise(webview);
    }
    #[cfg(not(gtk))]
    let _ = webview.reparent(main);
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabUrlChanged {
    tab_id: String,
    url: String,
    replaced: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabLoadChanged {
    tab_id: String,
    loading: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabIconChanged {
    tab_id: String,
    url: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabTitleChanged {
    tab_id: String,
    title: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabShortcut {
    tab_id: String,
    action: String,
}

const SHORTCUT_TITLE_PREFIX: &str = "@@star-shortcut@@:";
const URL_TITLE_PREFIX: &str = "@@star-url@@:";
const REPLACED_URL_TITLE_PREFIX: &str = "@@star-url-replaced@@:";
const AUDIO_TITLE_PREFIX: &str = "@@star-audio@@:";
const ICON_TITLE_PREFIX: &str = "@@star-icon@@:";
const PERMISSION_LABEL: &str = "__permission_overlay__";
const MENU_LABEL: &str = "__menu_overlay__";
const OVERLAY_LABEL: &str = "__panel_overlay__";

const OFFLINE_SCRIPT: &str = include_str!("../scripts/offline.js");
const COSMETIC_SCRIPT: &str = include_str!("../scripts/cosmetic.js");
const MEDIA_SCRIPT: &str = include_str!("../scripts/media.js");
const TAB_CORNER_RADIUS: f64 = 12.0;

static COSMETIC_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
static WAS_MAXIMIZED_BEFORE_FULLSCREEN: std::sync::Mutex<Option<bool>> = std::sync::Mutex::new(None);

fn cosmetic_enabled() -> bool {
    COSMETIC_ON.load(std::sync::atomic::Ordering::Relaxed)
}
#[tauri::command]
pub async fn set_fullscreen_restore(maximized: bool) -> Result<(), AppError> {
    *lock_recover(&WAS_MAXIMIZED_BEFORE_FULLSCREEN) = Some(maximized);
    Ok(())
}

#[tauri::command]
pub async fn take_fullscreen_restore() -> Result<bool, AppError> {
    Ok(lock_recover(&WAS_MAXIMIZED_BEFORE_FULLSCREEN)
        .take()
        .unwrap_or(false))
}

#[cfg(windows)]
fn apply_fullscreen_correctly(main: &tauri::Window, want_fullscreen: bool) -> Result<(), AppError> {
    use std::thread::sleep;
    use std::time::Duration;

    const UNMAX_DELAY: Duration = Duration::from_millis(120);

    if want_fullscreen {
        let was_maximized = main.is_maximized().unwrap_or(false);
        {
            let mut guard = lock_recover(&WAS_MAXIMIZED_BEFORE_FULLSCREEN);
            *guard = Some(was_maximized);
        }
        if was_maximized {
            if let Err(e) = main.unmaximize() {
                eprintln!(
                    "[star] fullscreen: unmaximize before fullscreen failed, continuing: {e:?}"
                );
            }
            sleep(UNMAX_DELAY);
        }
        main.set_fullscreen(true)?;
    } else {
        let restore_max = lock_recover(&WAS_MAXIMIZED_BEFORE_FULLSCREEN).take();
        main.set_fullscreen(false)?;
        if matches!(restore_max, Some(true)) {
            sleep(UNMAX_DELAY);
            if let Err(e) = main.maximize() {
                eprintln!(
                    "[star] fullscreen: could not restore maximized state after exit: {e:?}"
                );
            }
        }
    }
    Ok(())
}
#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionRequested {
    tab_id: String,
    request_id: String,
    kind: String,
    uri: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabAudioChanged {
    tab_id: String,
    audible: bool,
    muted: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadStarted {
    tab_id: String,
    file_name: String,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadFinished {
    tab_id: String,
    file_name: String,
    success: bool,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabPopup {
    tab_id: String,
    url: String,
}

fn file_name_for(path: &str, uri: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            uri.split('?')
                .next()
                .and_then(|value| value.rsplit('/').next())
                .filter(|name| !name.is_empty())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "download".to_string())
}

#[cfg(not(windows))]
use crate::commands::files::unique_download_path;

#[cfg(windows)]
pub use win_permissions::PermissionRegistry;

#[cfg(not(windows))]
#[derive(Default)]
pub struct PermissionRegistry;

fn hide_permission_overlay(app: &AppHandle) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let webview = state.view(PERMISSION_LABEL);
    if let Some(webview) = webview {
        hide_tab_view(&webview)?;
    }
    if let Some(main) = app.get_window("main") {
        let _ = main.set_focus();
    }
    Ok(())
}

fn show_permission_overlay(app: &AppHandle, request: PermissionRequested) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;
    let scale = main.scale_factor()?;
    let size = main.inner_size()?.to_logical::<f64>(scale);
    let state = app.state::<AppState>();

    let existing = state.view(PERMISSION_LABEL);
    if let Some(webview) = existing {
        place_tab_view(
            Some(PERMISSION_LABEL),
            &webview,
            0.0,
            0.0,
            size.width,
            size.height,
            true,
            true,
            None,
        )?;
        raise_to_front(&webview, &main);
        show_tab_view(&webview)?;
        let _ = webview.set_focus();
        let _ = app.emit("permission-requested", request);
        return Ok(());
    }

    let webview = create_overlay_view(
        &main,
        PERMISSION_LABEL,
        "permission",
        0.0,
        0.0,
        size.width,
        size.height,
    )?;
    show_tab_view(&webview)?;
    let _ = webview.set_focus();

    state.views().insert(PERMISSION_LABEL.to_string(), webview);

    let _ = app.emit("permission-requested", request);
    Ok(())
}

const CONTEXT_MENU_SCRIPT: &str = r#"(function () {
  if (window.__starContextMenu) return;
  window.__starContextMenu = true;
  document.addEventListener('contextmenu', function (e) {
    var t = e.target;
    if (t && t.closest && t.closest('video,audio')) return;
    e.preventDefault();
  }, true);
})();"#;

const SHORTCUT_FORWARD_SCRIPT: &str = r#"(function () {
  var PREFIX = "@@star-shortcut@@:";
  var URL_PREFIX = "@@star-url@@:";
  var REPLACE_PREFIX = "@@star-url-replaced@@:";
  var ICON_PREFIX = "@@star-icon@@:";

  var lastHref = location.href;

  function resolveAction(e) {
    if (e.key === 'F11' && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) return 'fullscreen';
    if (e.key === 'F5' && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) return 'reload';
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return null;
    if (e.shiftKey) {
      if (e.key === 'Delete' || e.key === 'Backspace') return 'cleardata';
      if (e.key.toLowerCase() === 's') return 'chat';
      return null;
    }
    var key = e.key.toLowerCase();
    if (e.key === '=' || e.key === '+') return 'zoomin';
    if (e.key === '-' || e.key === '_') return 'zoomout';
    if (e.key === '0') return 'zoomreset';
    if (key === 't') return 'newtab';
    if (key === 'w') return 'closetab';
    if (key === 'h') return 'history';
    if (key === 'p') return 'print';
    if (key === 'r') return 'reload';
        if (key === 'k') return 'search';
    return null;
  }

  function signalViaTitle(signal) {
    var host = window.__starTitleSignal;
    if (!host) {
      host = window.__starTitleSignal = {
        gen: 0,
        original: null,
        send: function (text) {
          if (this.original === null) this.original = document.title;
          var self = this;
          var mine = ++this.gen;
          document.title = text;
          setTimeout(function () {
            if (mine !== self.gen) return;
            var restore = self.original;
            self.original = null;
            document.title = restore;
          }, 0);
        }
      };
    }
    host.send(signal);
  }

  function signalUrl(replaced) {
    lastHref = window.location.href;
    signalViaTitle((replaced ? REPLACE_PREFIX : URL_PREFIX) + window.location.href);
  }

  var lastIcon = null;
  function signalIcon() {
    try {
      var links = document.querySelectorAll('link[rel~="icon"],link[rel="apple-touch-icon"]');
      var best = null;
      for (var i = 0; i < links.length; i++) {
        var href = links[i].href;
        if (href && /^https?:/i.test(href)) best = href;
      }
      if (best && best !== lastIcon) {
        lastIcon = best;
        signalViaTitle(ICON_PREFIX + best);
      }
    } catch (e) {}
  }

  function observeHistory(method, replaced) {
    var original = window.history[method];
    if (typeof original !== 'function') return;
    window.history[method] = function () {
      var result = original.apply(this, arguments);
      signalUrl(replaced);
      return result;
    };
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', signalIcon, { once: true });
  } else {
    signalIcon();
  }
  window.addEventListener('load', signalIcon);

  observeHistory('pushState', false);
  observeHistory('replaceState', true);
  window.addEventListener('popstate', function () { signalUrl(false); signalIcon(); });
  window.addEventListener('hashchange', function () { signalUrl(false); });

  if (window.top === window) {
    setInterval(function () {
      if (location.href !== lastHref) {
        signalUrl(false);
        signalIcon();
      }
    }, 800);
  }

  document.addEventListener('keydown', function (e) {
    var action = resolveAction(e);
    if (!action) return;
    e.preventDefault();
    e.stopPropagation();
    if (window.top === window) {
      signalViaTitle(PREFIX + action);
    } else {
      window.top.postMessage(PREFIX + action, '*');
    }
  }, true);

  if (window.top === window) {
    window.addEventListener('message', function (e) {
      if (typeof e.data === 'string' && e.data.indexOf(PREFIX) === 0) {
        signalViaTitle(e.data);
      }
    });
  }
})();"#;

const EXTRACT_PAGE_SCRIPT: &str = r#"(function () {
  try {
    var body = document.body;
    if (!body) return null;
    var text = (body.innerText || body.textContent || '').replace(/\s+/g, ' ').trim();
    var images = [];
    var imgs = document.querySelectorAll('img[src]');
    for (var i = 0; i < imgs.length && images.length < 12; i++) {
      var src = imgs[i].currentSrc || imgs[i].src;
      if (src && src.indexOf('data:') !== 0) images.push(src);
    }
    var videos = [];
    var vids = document.querySelectorAll('video[src], video source[src]');
    for (var j = 0; j < vids.length && videos.length < 6; j++) {
      if (vids[j].src) videos.push(vids[j].src);
    }
    var MAX = 16000;
    return {
      url: location.href,
      title: document.title || '',
      text: text.slice(0, MAX),
      images: images,
      videos: videos,
      truncated: text.length > MAX
    };
  } catch (e) {
    return null;
  }
})()"#;

#[tauri::command]
pub async fn read_tab_page(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<Option<crate::commands::page::PageContext>, AppError> {
    let label = label_for(&tab_id);
    let Some(webview) = state.view(&label) else {
        return Ok(None);
    };
    Ok(read_live_page(&webview).await)
}

#[cfg(windows)]
async fn read_live_page(webview: &tauri::Webview) -> Option<crate::commands::page::PageContext> {
    let raw = win_permissions::execute_script(webview, EXTRACT_PAGE_SCRIPT).await?;
    if raw.trim().is_empty() || raw.trim() == "null" {
        return None;
    }
    serde_json::from_str::<crate::commands::page::PageContext>(&raw).ok()
}

#[cfg(not(windows))]
async fn read_live_page(_webview: &tauri::Webview) -> Option<crate::commands::page::PageContext> {
    None
}

#[tauri::command]
pub async fn open_tab_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
    url: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: Option<f64>,
    round_bottom_left: Option<bool>,
    round_bottom_right: Option<bool>,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let parsed: url::Url = url.parse().map_err(|_| AppError::InvalidUrl)?;
    let corners = Corners::from_args(radius, round_bottom_left, round_bottom_right);

    {
        if let Some(webview) = state.view(&label) {
            let webview = &webview;
            webview.navigate(parsed.clone())?;
            place_tab_view(Some(&label), webview, x, y, width, height, true, false, Some(corners))?;
            show_tab_view(webview)?;
            #[cfg(windows)]
            {
                let scale = webview.window().scale_factor().unwrap_or(1.0);
                win_permissions::round_corners(
                    webview,
                    (width * scale).round() as i32,
                    (height * scale).round() as i32,
                    (corners.radius * scale).round() as i32,
                    corners.bottom_left,
                    corners.bottom_right,
                );
            }
            return Ok(());
        }
    }

    let nav_app = app.clone();
    let nav_tab_id = tab_id.clone();
    let nav_urls = state.last_tab_urls.clone();
    let title_app = app.clone();
    let title_tab_id = tab_id.clone();
    let title_urls = state.last_tab_urls.clone();
    #[cfg(not(any(windows, target_os = "macos")))]
    let (load_app, load_tab_id, load_urls) =
        (app.clone(), tab_id.clone(), state.last_tab_urls.clone());
    let (page_app, page_tab_id) = (app.clone(), tab_id.clone());
    #[cfg(gtk)]
    let initial_url = {
        let blank = "about:blank".parse().map_err(|_| AppError::InvalidUrl)?;
        WebviewUrl::External(blank)
    };
    #[cfg(not(gtk))]
    let initial_url = WebviewUrl::External(parsed.clone());

    let mut builder = WebviewBuilder::new(&label, initial_url)
        .zoom_hotkeys_enabled(false)
        .initialization_script_for_all_frames(SHORTCUT_FORWARD_SCRIPT)
        .initialization_script_for_all_frames(MEDIA_SCRIPT)
        .initialization_script(CONTEXT_MENU_SCRIPT)
        .initialization_script(OFFLINE_SCRIPT);
    if cosmetic_enabled() {
        builder = builder.initialization_script_for_all_frames(COSMETIC_SCRIPT);
    }
    let builder = builder
        .on_navigation(move |url| handle_navigation(&nav_app, &nav_tab_id, &nav_urls, url))
        .on_document_title_changed(move |_webview, title| {
            handle_title(&title_app, &title_tab_id, &title_urls, &title);
        });

    let builder = builder.on_page_load(move |_webview, payload| {
        #[cfg(not(any(windows, target_os = "macos")))]
        report_main_frame_url(&load_app, &load_tab_id, &load_urls, payload.url().as_str());
        if payload.url().scheme() == "about" {
            return;
        }
        let loading = matches!(payload.event(), tauri::webview::PageLoadEvent::Started);
        let _ = page_app.emit(
            "tab-load-changed",
            TabLoadChanged {
                tab_id: page_tab_id.clone(),
                loading,
            },
        );
    });

    #[cfg(not(windows))]
    let builder = {
        let download_app = app.clone();
        let download_tab_id = tab_id.clone();
        builder.on_download(move |_webview, event| {
            handle_download(&download_app, &download_tab_id, event)
        })
    };

    #[cfg(target_os = "macos")]
    let builder = builder.user_agent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
    );

    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;

    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    place_tab_view(Some(&label), &webview, x, y, width, height, true, false, Some(corners))?;

    #[cfg(gtk)]
    webview.navigate(parsed)?;

    webview.show()?;

    finish_tab_setup(
        &app,
        &tab_id,
        &webview,
        width,
        height,
        radius,
        round_bottom_left,
        round_bottom_right,
    );

    state.views().insert(label, webview);
    state.set_last_tab_url(tab_id.clone(), url);
    Ok(())
}

#[tauri::command]
pub async fn navigate_tab_webview(
    state: State<'_, AppState>,
    tab_id: String,
    url: String,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let parsed = url.parse().map_err(|_| AppError::InvalidUrl)?;

    let webview = state.view(&label).ok_or(AppError::WindowNotFound)?;
    webview.navigate(parsed)?;
    Ok(())
}

#[tauri::command]
pub async fn set_tab_bounds(
    state: State<'_, AppState>,
    tab_id: String,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: Option<f64>,
    round_bottom_left: Option<bool>,
    round_bottom_right: Option<bool>,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);

    if let Some(webview) = state.view(&label) {
        let webview = &webview;
        let corners = Corners::from_args(radius, round_bottom_left, round_bottom_right);
        place_tab_view(Some(&label), webview, x, y, width, height, true, false, Some(corners))?;
        #[cfg(windows)]
        {
            let scale = webview.window().scale_factor().unwrap_or(1.0);
            win_permissions::round_corners(
                webview,
                (width * scale).round() as i32,
                (height * scale).round() as i32,
                (corners.radius * scale).round() as i32,
                corners.bottom_left,
                corners.bottom_right,
            );
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn show_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let target = label_for(&tab_id);
    for (label, webview) in state.views_snapshot() {
        if label == MENU_LABEL || label == OVERLAY_LABEL || label == PERMISSION_LABEL {
            continue;
        }
        if label == target {
            show_tab_view(&webview)?;
        } else {
            hide_tab_view(&webview)?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.view(&label) {
        hide_tab_view(&webview)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_tab_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    state.forget_last_tab_url(&tab_id);
    forget_tab_rect(&label);
    #[cfg(windows)]
    win_permissions::forget_tab(&app, &tab_id);
    #[cfg(not(windows))]
    let _ = &app;
    let removed = state.views().remove(&label);
    if let Some(webview) = removed {
        webview
            .eval("(function(){var m=window.__starMedia;if(m)m.stop();})();")
            .ok();
        #[cfg(windows)]
        win_permissions::force_close(&webview);
        close_tab_view(&webview);
    }
    Ok(())
}

#[tauri::command]
pub async fn open_menu_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;
    let (x, y) = (0.0, 0.0);
    let (width, height) = full_window_size(&main)?;

    if let Some(webview) = state.view(MENU_LABEL) {
        place_tab_view(
            Some(MENU_LABEL),
            &webview,
            x,
            y,
            width,
            height,
            true,
            true,
            None,
        )?;
        raise_to_front(&webview, &main);
        blank_reshown_surface(MENU_LABEL, &webview);
        show_tab_view(&webview)?;
        let _ = webview.set_focus();
        return Ok(());
    }

    let webview = match create_overlay_view(&main, MENU_LABEL, "menu", x, y, width, height) {
        Ok(webview) => webview,
        Err(e) => {
            let Some(existing) = state.view(MENU_LABEL) else {
                return Err(e);
            };
            place_tab_view(
                Some(MENU_LABEL),
                &existing,
                x,
                y,
                width,
                height,
                true,
                true,
                None,
            )?;
            raise_to_front(&existing, &main);
            blank_reshown_surface(MENU_LABEL, &existing);
            show_tab_view(&existing)?;
            let _ = existing.set_focus();
            return Ok(());
        }
    };
    show_tab_view(&webview)?;
    let _ = webview.set_focus();

    state.views().insert(MENU_LABEL.to_string(), webview);
    Ok(())
}

#[tauri::command]
pub async fn warm_menu_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;
    let (x, y) = (0.0, 0.0);
    let (width, height) = full_window_size(&main)?;

    if state.views().contains_key(MENU_LABEL) {
        return Ok(());
    }

    let Ok(webview) = create_overlay_view(&main, MENU_LABEL, "menu", x, y, width, height) else {
        return Ok(());
    };
    state.views().insert(MENU_LABEL.to_string(), webview.clone());
    hide_tab_view(&webview)?;
    Ok(())
}

#[tauri::command]
pub async fn warm_overlay_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;
    let (x, y) = (0.0, 0.0);
    let (width, height) = full_window_size(&main)?;

    if state.views().contains_key(OVERLAY_LABEL) {
        return Ok(());
    }

    let Ok(webview) = create_overlay_view(&main, OVERLAY_LABEL, "overlay", x, y, width, height)
    else {
        return Ok(());
    };
    state.views().insert(OVERLAY_LABEL.to_string(), webview.clone());
    hide_tab_view(&webview)?;
    Ok(())
}

#[tauri::command]
pub async fn open_overlay_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;
    let (x, y) = (0.0, 0.0);
    let (width, height) = full_window_size(&main)?;

    if let Some(webview) = state.view(OVERLAY_LABEL) {
        place_tab_view(
            Some(OVERLAY_LABEL),
            &webview,
            x,
            y,
            width,
            height,
            true,
            true,
            None,
        )?;
        raise_to_front(&webview, &main);
        blank_reshown_surface(OVERLAY_LABEL, &webview);
        show_tab_view(&webview)?;
        let _ = webview.set_focus();
        return Ok(());
    }

    let webview = match create_overlay_view(&main, OVERLAY_LABEL, "overlay", x, y, width, height) {
        Ok(webview) => webview,
        Err(e) => {
            let existing = state.view(OVERLAY_LABEL);
            let Some(existing) = existing else {
                return Err(e.into());
            };
            place_tab_view(
                Some(OVERLAY_LABEL),
                &existing,
                x,
                y,
                width,
                height,
                true,
                true,
                None,
            )?;
            raise_to_front(&existing, &main);
            blank_reshown_surface(OVERLAY_LABEL, &existing);
            show_tab_view(&existing)?;
            let _ = existing.set_focus();
            return Ok(());
        }
    };
    show_tab_view(&webview)?;
    let _ = webview.set_focus();

    state.views().insert(OVERLAY_LABEL.to_string(), webview);
    Ok(())
}

#[tauri::command]
pub async fn set_surface_clip(
    state: State<'_, AppState>,
    surface: String,
    parts: Vec<ClipPart>,
) -> Result<(), AppError> {
    if !SURFACE_NEEDS_CLIP {
        return Ok(());
    }
    let label = match surface.as_str() {
        "menu" => MENU_LABEL,
        "overlay" => OVERLAY_LABEL,
        _ => return Ok(()),
    };
    let Some(webview) = state.view(label) else {
        return Ok(());
    };

    let parts: Vec<ClipPart> = parts
        .into_iter()
        .filter(|part| part.width > 0.0 && part.height > 0.0)
        .collect();
    let scale = webview.window().scale_factor().unwrap_or(1.0);
    clip_surface(&webview, &parts, scale);
    Ok(())
}

#[tauri::command]
pub async fn close_overlay_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if let Some(webview) = state.view(OVERLAY_LABEL) {
        hide_tab_view(&webview)?;
    }
    if let Some(main) = app.get_window("main") {
        let _ = main.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn close_menu_webview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    if let Some(webview) = state.view(MENU_LABEL) {
        hide_tab_view(&webview)?;
    }
    if let Some(main) = app.get_window("main") {
        let _ = main.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn set_tab_zoom(
    state: State<'_, AppState>,
    tab_id: String,
    factor: f64,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.view(&label) {
        webview.set_zoom(factor)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_back(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.view(&label) {
        webview.eval("history.back()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_forward(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.view(&label) {
        webview.eval("history.forward()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_reload(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let webview = state.view(&label);
    if let Some(webview) = webview {
        let is_offline_page = webview
            .url()
            .map(|u| u.as_str().starts_with("data:") || u.as_str() == "about:blank")
            .unwrap_or(false);
        if is_offline_page {
            let last_url = state.last_tab_url(&tab_id).and_then(|u| u.parse().ok());
            if let Some(url) = last_url {
                webview.navigate(url)?;
                return Ok(());
            }
        }
        webview.eval("location.reload()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_print(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.view(&label) {
        let _ = webview.set_focus();
        #[cfg(target_os = "macos")]
        {
            webview.eval(
                r#"(function () { setTimeout(function () {
                    try { if (window.focus) window.focus(); window.print(); }
                    catch (e) { console.warn('print failed', e); }
                }, 60); })()"#,
            )?;
        }
        #[cfg(not(target_os = "macos"))]
        {
            webview.eval(
                r#"(function () { try { if (window.focus) window.focus(); window.print(); }
                    catch (e) { console.warn('print failed', e); } })()"#,
            )?;
        }
    }
    Ok(())
}

pub fn grant_main_window_media(app: &AppHandle) {
    #[cfg(windows)]
    {
        if let Some(window) = app.get_webview_window("main") {
            win_permissions::grant_app_media(window.as_ref());
        }
    }
    #[cfg(not(windows))]
    let _ = app;
}

#[tauri::command]
pub async fn set_adblock(enabled: bool) -> Result<(), AppError> {
    COSMETIC_ON.store(enabled, std::sync::atomic::Ordering::Relaxed);
    #[cfg(windows)]
    win_permissions::set_adblock_enabled(enabled);
    Ok(())
}

fn eval_media(state: &State<'_, AppState>, tab_id: &str, call: &str) -> Result<(), AppError> {
    let label = label_for(tab_id);
    let webview = state.view(&label);
    if let Some(webview) = webview {
        webview.eval(&format!(
            "(function(){{var m=window.__starMedia;if(m)m.{call};}})();"
        ))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_media_toggle(
    state: State<'_, AppState>,
    tab_id: String,
    _playing: Option<bool>,
) -> Result<(), AppError> {
    eval_media(&state, &tab_id, "toggle()")
}

#[tauri::command]
pub async fn tab_stop_media(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    eval_media(&state, &tab_id, "stop()")
}

#[tauri::command]
pub async fn set_tab_muted(
    state: State<'_, AppState>,
    tab_id: String,
    muted: bool,
) -> Result<(), AppError> {
    #[cfg(windows)]
    {
        let label = label_for(&tab_id);
        let webview = state.view(&label);
        if let Some(webview) = webview {
            let _ = webview.with_webview(move |platform| {
                win_permissions::apply_muted(&platform, muted);
            });
        }
        Ok(())
    }

    #[cfg(not(windows))]
    eval_media(&state, &tab_id, &format!("setMuted({muted})"))
}

#[tauri::command]
pub async fn pending_permission(
    app: AppHandle,
    request_id: String,
    granted: bool,
) -> Result<(), AppError> {
    #[cfg(windows)]
    win_permissions::resolve(&app, &request_id, granted);
    #[cfg(not(windows))]
    {
        let _ = (&app, &request_id, granted);
        hide_permission_overlay(&app)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn current_permission(app: AppHandle) -> Result<Option<PermissionRequested>, AppError> {
    #[cfg(windows)]
    return Ok(win_permissions::current(&app));
    #[cfg(not(windows))]
    {
        let _ = &app;
        Ok(None)
    }
}

#[cfg(windows)]
mod win_permissions {
    use std::collections::{HashMap, VecDeque};
    use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
    use std::sync::Mutex;

    use tauri::webview::WebviewBuilder;
    use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl};
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2Controller, ICoreWebView2Deferral,
        ICoreWebView2NewWindowRequestedEventArgs, ICoreWebView2PermissionRequestedEventArgs,
        ICoreWebView2_2, ICoreWebView2_4, ICoreWebView2_8, COREWEBVIEW2_DOWNLOAD_STATE,
        COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED, COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS,
        COREWEBVIEW2_KEY_EVENT_KIND, COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN,
        COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN, COREWEBVIEW2_PERMISSION_KIND,
        COREWEBVIEW2_PERMISSION_KIND_CAMERA, COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION,
        COREWEBVIEW2_PERMISSION_KIND_MICROPHONE, COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS,
        COREWEBVIEW2_PERMISSION_STATE_ALLOW, COREWEBVIEW2_PERMISSION_STATE_DENY,
        COREWEBVIEW2_WEB_ERROR_STATUS, COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT,
        COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED,
        COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED,
        COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE, COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT,
    };
    use webview2_com::PermissionRequestedEventHandler;
    use webview2_com::{
        take_pwstr, AcceleratorKeyPressedEventHandler, DownloadStartingEventHandler,
        ExecuteScriptCompletedHandler, IsDocumentPlayingAudioChangedEventHandler,
        IsMutedChangedEventHandler, NavigationCompletedEventHandler,
        NewWindowRequestedEventHandler, StateChangedEventHandler, WebResourceRequestedEventHandler,
    };
    use windows::core::BOOL;
    use windows::core::{Interface, HSTRING, PWSTR};

    use super::{
        file_name_for, hide_permission_overlay, show_permission_overlay, DownloadFinished,
        DownloadStarted, PermissionRequested, TabAudioChanged, TabPopup, TabShortcut,
    };
    use crate::state::lock_recover;

    struct PendingNative {
        args: ICoreWebView2PermissionRequestedEventArgs,
        deferral: ICoreWebView2Deferral,
    }

    unsafe impl Send for PendingNative {}

    #[derive(Default)]
    pub struct PermissionRegistry {
        pending: Mutex<HashMap<String, PendingNative>>,
        queue: Mutex<VecDeque<PermissionRequested>>,
        current: Mutex<Option<PermissionRequested>>,
    }

    impl PermissionRegistry {
        fn current_cloned(&self) -> Option<PermissionRequested> {
            lock_recover(&self.current).clone()
        }
    }

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

    static LIVE_GHOSTS: AtomicUsize = AtomicUsize::new(0);
    const MAX_LIVE_GHOSTS: usize = 4;

    fn kind_name(kind: COREWEBVIEW2_PERMISSION_KIND) -> Option<&'static str> {
        match kind {
            COREWEBVIEW2_PERMISSION_KIND_CAMERA => Some("camera"),
            COREWEBVIEW2_PERMISSION_KIND_MICROPHONE => Some("microphone"),
            COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION => Some("geolocation"),
            COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS => Some("notifications"),
            _ => None,
        }
    }

    fn square_off(base: windows::Win32::Graphics::Gdi::HRGN, x1: i32, y1: i32, x2: i32, y2: i32) {
        let patch = unsafe { windows::Win32::Graphics::Gdi::CreateRectRgn(x1, y1, x2, y2) };
        if patch.is_invalid() {
            return;
        }
        unsafe {
            windows::Win32::Graphics::Gdi::CombineRgn(
                Some(base),
                Some(base),
                Some(patch),
                windows::Win32::Graphics::Gdi::RGN_OR,
            );
            let _ = windows::Win32::Graphics::Gdi::DeleteObject(patch.into());
        }
    }

    fn corner_region(
        width: i32,
        height: i32,
        radius: i32,
        bottom_left: bool,
        bottom_right: bool,
    ) -> Option<windows::Win32::Graphics::Gdi::HRGN> {
        if radius <= 0 || (!bottom_left && !bottom_right) {
            return None;
        }
        let w = width + 1;
        let h = height + 1;
        let r = radius.min(w / 2).min(h / 2);
        if r <= 0 {
            return None;
        }
        let base =
            unsafe { windows::Win32::Graphics::Gdi::CreateRoundRectRgn(0, 0, w, h, r * 2, r * 2) };
        if base.is_invalid() {
            return None;
        }
        square_off(base, 0, 0, w, r);
        if !bottom_left {
            square_off(base, 0, h - r, r, h);
        }
        if !bottom_right {
            square_off(base, w - r, h - r, w, h);
        }
        Some(base)
    }

    pub fn round_corners(
        webview: &Webview,
        width: i32,
        height: i32,
        radius: i32,
        bottom_left: bool,
        bottom_right: bool,
    ) {
        if width <= 0 || height <= 0 {
            return;
        }
        let _ = webview.with_webview(move |platform| {
            let controller = platform.controller();
            let mut host = windows::Win32::Foundation::HWND::default();
            if unsafe { controller.ParentWindow(&mut host) }.is_err() || host.is_invalid() {
                return;
            }
            let Some(region) = corner_region(width, height, radius, bottom_left, bottom_right)
            else {
                unsafe { windows::Win32::Graphics::Gdi::SetWindowRgn(host, None, true) };
                return;
            };
            let applied =
                unsafe { windows::Win32::Graphics::Gdi::SetWindowRgn(host, Some(region), true) };
            if applied == 0 {
                let _ = unsafe { windows::Win32::Graphics::Gdi::DeleteObject(region.into()) };
            }
        });
    }

    pub fn clip_to_rects(webview: &Webview, rects: &[(i32, i32, i32, i32, i32)]) {
        let rects = rects.to_vec();
        let _ = webview.with_webview(move |platform| {
            let controller = platform.controller();
            let mut host = windows::Win32::Foundation::HWND::default();
            if unsafe { controller.ParentWindow(&mut host) }.is_err() || host.is_invalid() {
                return;
            }

            let region = unsafe { windows::Win32::Graphics::Gdi::CreateRectRgn(0, 0, 0, 0) };
            if region.is_invalid() {
                return;
            }
            for (x, y, width, height, radius) in rects {
                let part = if radius > 0 {
                    unsafe {
                        windows::Win32::Graphics::Gdi::CreateRoundRectRgn(
                            x,
                            y,
                            x + width + 1,
                            y + height + 1,
                            radius * 2,
                            radius * 2,
                        )
                    }
                } else {
                    unsafe {
                        windows::Win32::Graphics::Gdi::CreateRectRgn(
                            x,
                            y,
                            x + width,
                            y + height,
                        )
                    }
                };
                if part.is_invalid() {
                    continue;
                }
                unsafe {
                    windows::Win32::Graphics::Gdi::CombineRgn(
                        Some(region),
                        Some(region),
                        Some(part),
                        windows::Win32::Graphics::Gdi::RGN_OR,
                    );
                    let _ = windows::Win32::Graphics::Gdi::DeleteObject(part.into());
                }
            }

            let applied =
                unsafe { windows::Win32::Graphics::Gdi::SetWindowRgn(host, Some(region), true) };
            if applied == 0 {
                let _ = unsafe { windows::Win32::Graphics::Gdi::DeleteObject(region.into()) };
            }
        });
    }

    pub async fn execute_script(webview: &Webview, script: &'static str) -> Option<String> {
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
        let sender = std::sync::Arc::new(Mutex::new(Some(tx)));
        let dispatch_sender = sender.clone();

        let dispatched = webview.with_webview(move |platform| {
            let finish = |slot: &std::sync::Arc<
                Mutex<Option<tokio::sync::oneshot::Sender<Option<String>>>>,
            >,
                          value: Option<String>| {
                if let Ok(mut guard) = slot.lock() {
                    if let Some(tx) = guard.take() {
                        let _ = tx.send(value);
                    }
                }
            };

            let core = match unsafe { platform.controller().CoreWebView2() } {
                Ok(core) => core,
                Err(_) => {
                    finish(&dispatch_sender, None);
                    return;
                }
            };

            let handler_sender = dispatch_sender.clone();
            let handler = ExecuteScriptCompletedHandler::create(Box::new(move |result, json| {
                let value = if result.is_ok() { Some(json) } else { None };
                if let Ok(mut guard) = handler_sender.lock() {
                    if let Some(tx) = guard.take() {
                        let _ = tx.send(value);
                    }
                }
                Ok(())
            }));

            if unsafe { core.ExecuteScript(&HSTRING::from(script), &handler) }.is_err() {
                finish(&dispatch_sender, None);
            }
        });

        if dispatched.is_err() {
            if let Ok(mut guard) = sender.lock() {
                guard.take();
            }
            return None;
        }

        match tokio::time::timeout(std::time::Duration::from_secs(3), rx).await {
            Ok(Ok(value)) => value,
            _ => None,
        }
    }

    pub fn grant_app_media(webview: &Webview) {
        let _ = webview.with_webview(move |platform| {
            let controller = platform.controller();
            let core = match unsafe { controller.CoreWebView2() } {
                Ok(core) => core,
                Err(_) => return,
            };

            let handler =
                PermissionRequestedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args): Option<ICoreWebView2PermissionRequestedEventArgs> = args else {
                        return Ok(());
                    };
                    let mut kind = COREWEBVIEW2_PERMISSION_KIND(0);
                    unsafe { args.PermissionKind(&mut kind)? };
                    if kind == COREWEBVIEW2_PERMISSION_KIND_MICROPHONE
                        || kind == COREWEBVIEW2_PERMISSION_KIND_CAMERA
                    {
                        unsafe { args.SetState(COREWEBVIEW2_PERMISSION_STATE_ALLOW)? };
                    }
                    Ok(())
                }));

            let mut token: i64 = 0;
            let _ = unsafe { core.add_PermissionRequested(&handler, &mut token) };
        });
    }

    pub fn register(app: &AppHandle, tab_id: String, webview: &Webview) {
        let app = app.clone();
        let _ = webview.with_webview(move |platform| {
            let controller = platform.controller();
            let core = match unsafe { controller.CoreWebView2() } {
                Ok(core) => core,
                Err(_) => return,
            };

            let handler_app = app.clone();
            let permission_tab_id = tab_id.clone();
            let handler =
                PermissionRequestedEventHandler::create(Box::new(move |_sender, args| {
                    let Some(args): Option<ICoreWebView2PermissionRequestedEventArgs> = args else {
                        return Ok(());
                    };

                    let mut kind = COREWEBVIEW2_PERMISSION_KIND(0);
                    unsafe { args.PermissionKind(&mut kind)? };

                    let Some(kind_str) = kind_name(kind) else {
                        return Ok(());
                    };

                    let uri = {
                        let mut uri = PWSTR::null();
                        match unsafe { args.Uri(&mut uri) } {
                            Ok(()) => take_pwstr(uri),
                            Err(_) => String::new(),
                        }
                    };

                    let deferral = unsafe { args.GetDeferral()? };
                    let request_id = format!("p{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));

                    {
                        let registry = handler_app.state::<PermissionRegistry>();
                        lock_recover(&registry.pending)
                            .insert(request_id.clone(), PendingNative { args, deferral });
                        lock_recover(&registry.queue)
                            .push_back(PermissionRequested {
                                tab_id: permission_tab_id.clone(),
                                request_id,
                                kind: kind_str.to_string(),
                                uri,
                            });
                    }

                    let pump_app = handler_app.clone();
                    tauri::async_runtime::spawn(async move {
                        pump(&pump_app);
                    });

                    Ok(())
                }));

            let mut token: i64 = 0;
            let _ = unsafe { core.add_PermissionRequested(&handler, &mut token) };

            let popup_app = app.clone();
            let popup_tab_id = tab_id.clone();
            let new_window_handler =
                NewWindowRequestedEventHandler::create(Box::new(move |_, args| {
                    let Some(args): Option<ICoreWebView2NewWindowRequestedEventArgs> = args
                    else {
                        return Ok(());
                    };
                    handle_new_window_requested(&popup_app, popup_tab_id.clone(), args);
                    Ok(())
                }));
            let mut new_window_token: i64 = 0;
            if let Err(e) =
                unsafe { core.add_NewWindowRequested(&new_window_handler, &mut new_window_token) }
            {
                eprintln!(
                    "[star] popups: failed to register NewWindowRequested handler for tab {tab_id}: {e:?}"
                );
            }

            attach_keyboard_shortcuts(&app, tab_id.clone(), &controller);
            attach_offline_page(&core);
            attach_audio_tracking(&app, tab_id.clone(), &core);
            attach_ad_blocking(&app, tab_id.clone(), &core);
            attach_download_tracking(&app, tab_id.clone(), &core);
        });
    }

    pub static ADBLOCK: AtomicBool = AtomicBool::new(true);

    pub fn set_adblock_enabled(enabled: bool) {
        ADBLOCK.store(enabled, Ordering::Relaxed);
    }

    const AD_HOSTS: &[&str] = &[
        "1dmp.io",
        "2o7.net",
        "3lift.com",
        "ad.mail.ru",
        "ad.plus",
        "adcash.com",
        "adcolony.com",
        "addthis.com",
        "adform.net",
        "adfox.ru",
        "adhigh.net",
        "adition.com",
        "adnxs-simple.com",
        "adnxs.com",
        "adobedtm.com",
        "adriver.ru",
        "adroll.com",
        "ads-twitter.com",
        "ads.linkedin.com",
        "ads.pinterest.com",
        "ads.reddit.com",
        "ads.tiktok.com",
        "ads.yahoo.com",
        "adsafeprotected.com",
        "adscale.de",
        "adskeeper.com",
        "adsrvr.org",
        "adsterra.com",
        "adsterratech.com",
        "adtechus.com",
        "adtelligent.com",
        "adthrive.com",
        "advertising.com",
        "adx.linkedin.com",
        "agkn.com",
        "amazon-adsystem.com",
        "amazonaax.com",
        "amplitude.com",
        "an.yandex.ru",
        "analytics.tiktok.com",
        "aniview.com",
        "app-measurement.com",
        "applovin.com",
        "arc.io",
        "assoc-amazon.com",
        "betweendigital.com",
        "bidr.io",
        "bidswitch.net",
        "bluekai.com",
        "branch.io",
        "bugsnag.com",
        "buysellads.com",
        "carbonads.com",
        "casalemedia.com",
        "cashtrafic.com",
        "cdn.connatix.com",
        "chartbeat.com",
        "clarity.ms",
        "clickadu.com",
        "clicktale.net",
        "comscore.com",
        "connatix.com",
        "connect.facebook.net",
        "contextweb.com",
        "coremetrics.com",
        "counter.yadro.ru",
        "crazyegg.com",
        "criteo.com",
        "criteo.net",
        "crwdcntrl.net",
        "demdex.net",
        "districtm.io",
        "doubleclick.net",
        "doubleverify.com",
        "dvtps.com",
        "effectivemeasure.net",
        "ensighten.com",
        "events.redditmedia.com",
        "everesttech.net",
        "exoclick.com",
        "ezoic.net",
        "ezojs.com",
        "flashtalking.com",
        "freestar.com",
        "freestar.io",
        "fullstory.com",
        "galaksion.com",
        "gamoshi.com",
        "gemius.pl",
        "google-analytics.com",
        "googleadservices.com",
        "googlesyndication.com",
        "googletagmanager.com",
        "googletagservices.com",
        "gumgum.com",
        "heapanalytics.com",
        "hilltopads.net",
        "histats.com",
        "hotjar.com",
        "iasds01.com",
        "id5-sync.com",
        "improvedigital.com",
        "imrworldwide.com",
        "indexww.com",
        "inmobi.com",
        "innovid.com",
        "inspectlet.com",
        "juicyads.com",
        "keen.io",
        "kissmetrics.com",
        "krxd.net",
        "lijit.com",
        "loggly.com",
        "logs.pinterest.com",
        "luckyorange.com",
        "magnite.com",
        "matomo.cloud",
        "mc.yandex.com",
        "mc.yandex.ru",
        "media.net",
        "mediaimpact.de",
        "mediavine.com",
        "mgid.com",
        "mixpanel.com",
        "moatads.com",
        "moatpixel.com",
        "monumetric.com",
        "mopub.com",
        "mouseflow.com",
        "newrelic.com",
        "nielsen.com",
        "nr-data.net",
        "omtrdc.net",
        "onclckpro.com",
        "onclickalgo.com",
        "onclickperformance.com",
        "onesignal.com",
        "onetag-sys.com",
        "openx.net",
        "optimizely.com",
        "outbrain.com",
        "permutive.app",
        "permutive.com",
        "pippio.com",
        "piwik.pro",
        "playground.xyz",
        "playwire.com",
        "popads.net",
        "poperblocker.com",
        "primis.tech",
        "propellerads.com",
        "propelleradsystem.com",
        "propellerclick.com",
        "pubmatic.com",
        "push-mania.com",
        "pushwoosh.com",
        "quantcount.com",
        "quantserve.com",
        "raptive.com",
        "revcontent.com",
        "rlcdn.com",
        "rs.mail.ru",
        "rtbhouse.com",
        "rubiconproject.com",
        "scorecardresearch.com",
        "screencore.io",
        "segment.com",
        "segment.io",
        "sentry-cdn.com",
        "servedbyadbutler.com",
        "serving-sys.com",
        "sharethis.com",
        "sharethrough.com",
        "sitemeter.com",
        "sizmek.com",
        "smaato.net",
        "smartadserver.com",
        "smartlook.com",
        "smilewanted.com",
        "snap-adkit.com",
        "sonobi.com",
        "sovrn.com",
        "spot.im",
        "spotxchange.com",
        "springserve.com",
        "statcounter.com",
        "static.ads-twitter.com",
        "taboola.com",
        "taboolasyndication.com",
        "tapad.com",
        "teads.tv",
        "tealiumiq.com",
        "top-fwz1.mail.ru",
        "top100.rambler.ru",
        "tr.snapchat.com",
        "trackjs.com",
        "trafficjunky.net",
        "trafficstars.com",
        "tremorhub.com",
        "tsyndicate.com",
        "uidapi.com",
        "unityads.unity3d.com",
        "videoheroes.tv",
        "vidoomy.com",
        "webtrends.com",
        "wt-safetag.com",
        "yandexadexchange.net",
        "yieldlab.net",
        "yieldmo.com",
        "yieldmo.net",
        "yieldoptimizer.com",
        "zedo.com",
        "zemanta.com",
    ];

    const AD_PATH_MARKERS: &[&str] = &[
        "/pagead/",
        "/adserver",
        "/adservice",
        "/ad_server",
        "/adframe",
        "/adhandler",
        "/adrequest",
        "/advertisement",
        "/bannerad",
        "/banner_ad",
        "/popunder",
        "/prebid",
        "/openrtb",
        "/usersync",
        "/cookiesync",
        "/getad",
        "/showad",
        "/trackad",
        "/click_track",
        "/beacon",
        "/collect?",
        "/gtm.js",
        "/analytics.js",
        "/gtag/js",
        "/ads/",
        "/adv/",
        "/advert",
        "/adimage",
        "/adbanner",
        "/banners/",
        "/banner_",
        "/banner-",
        "-banner.",
        "_banner.",
        "/sponsor",
        "/promoted",
        "468x60",
        "728x90",
        "300x250",
        "336x280",
        "160x600",
        "120x600",
        "320x50",
        "970x250",
        "300x600",
    ];

    const AD_FILE_MARKERS: &[&str] = &[
        "/ads.js",
        "/ad.js",
        "/adsbygoogle.js",
        "/prebid.js",
        "/gpt.js",
        "/pubads",
        "/analytics.min.js",
        "/fbevents.js",
        "/hotjar",
        "/ads.min.js",
    ];

    fn host_of(uri: &str) -> Option<String> {
        url::Url::parse(uri)
            .ok()
            .and_then(|parsed| parsed.host_str().map(|h| h.to_ascii_lowercase()))
    }

    fn host_blocked(host: &str) -> bool {
        AD_HOSTS.iter().any(|rule| {
            host == *rule || (host.len() > rule.len() && host.ends_with(&format!(".{rule}")))
        })
    }

    const MULTI_PART_SUFFIXES: &[&str] = &[
        "co.uk", "org.uk", "ac.uk", "gov.uk", "co.jp", "ne.jp", "or.jp", "ac.jp", "com.au",
        "net.au", "org.au", "com.br", "com.mx", "com.ar", "com.tr", "com.cn", "com.tw", "co.kr",
        "co.in", "co.za", "co.nz", "com.sg", "com.hk", "com.pl", "com.ua",
    ];

    fn base_domain(host: &str) -> &str {
        let labels: Vec<&str> = host.split('.').collect();
        if labels.len() <= 2 {
            return host;
        }
        let last_two = &host
            [host.len() - (labels[labels.len() - 2].len() + 1 + labels[labels.len() - 1].len())..];
        let take = if MULTI_PART_SUFFIXES.contains(&last_two) {
            3
        } else {
            2
        };
        if labels.len() <= take {
            return host;
        }
        let keep: usize = labels[labels.len() - take..]
            .iter()
            .map(|l| l.len() + 1)
            .sum::<usize>()
            - 1;
        &host[host.len() - keep..]
    }

    fn same_site(a: &str, b: &str) -> bool {
        base_domain(a) == base_domain(b)
    }

    fn path_blocked(uri: &str) -> bool {
        let lower = uri.to_ascii_lowercase();
        let after_host = match lower.find("://") {
            Some(i) => match lower[i + 3..].find('/') {
                Some(j) => &lower[i + 3 + j..],
                None => return false,
            },
            None => lower.as_str(),
        };
        AD_PATH_MARKERS
            .iter()
            .chain(AD_FILE_MARKERS.iter())
            .any(|marker| after_host.contains(marker))
    }

    fn is_ad_url(uri: &str, page_host: Option<&str>) -> bool {
        if !uri.starts_with("http") {
            return false;
        }
        let Some(host) = host_of(uri) else {
            return false;
        };
        if let Some(page) = page_host {
            if same_site(&host, page) {
                return false;
            }
        }
        host_blocked(&host) || path_blocked(uri)
    }

    fn attach_ad_blocking(app: &AppHandle, tab_id: String, core: &ICoreWebView2) {
        let Ok(core2) = core.cast::<ICoreWebView2_2>() else {
            return;
        };
        let Ok(env) = (unsafe { core2.Environment() }) else {
            return;
        };
        let _ = unsafe {
            core.AddWebResourceRequestedFilter(
                &HSTRING::from("*"),
                COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
            )
        };
        let last_urls = app.state::<crate::state::AppState>().last_tab_urls.clone();
        let handler = WebResourceRequestedEventHandler::create(Box::new(move |_, args| {
            if !ADBLOCK.load(Ordering::Relaxed) {
                return Ok(());
            }
            let Some(args) = args else {
                return Ok(());
            };
            let mut context = COREWEBVIEW2_WEB_RESOURCE_CONTEXT::default();
            if unsafe { args.ResourceContext(&mut context) }.is_ok()
                && context == COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT
            {
                return Ok(());
            }
            let request = unsafe { args.Request()? };
            let uri = {
                let mut uri = PWSTR::null();
                unsafe { request.Uri(&mut uri)? };
                take_pwstr(uri)
            };
            let page = lock_recover(&last_urls).get(&tab_id).cloned();
            let page_host = page.as_deref().and_then(host_of);
            if !is_ad_url(&uri, page_host.as_deref()) {
                return Ok(());
            }
            if let Ok(response) = unsafe {
                env.CreateWebResourceResponse(
                    None,
                    403,
                    &HSTRING::from("Blocked"),
                    &HSTRING::from(""),
                )
            } {
                let _ = unsafe { args.SetResponse(&response) };
            }
            Ok(())
        }));
        let mut token: i64 = 0;
        let _ = unsafe { core.add_WebResourceRequested(&handler, &mut token) };
    }

    fn emit_audio_state(app: &AppHandle, tab_id: &str, core8: &ICoreWebView2_8) {
        let mut audible = BOOL::default();
        let mut muted = BOOL::default();
        let _ = unsafe { core8.IsDocumentPlayingAudio(&mut audible) };
        let _ = unsafe { core8.IsMuted(&mut muted) };
        let _ = app.emit(
            "tab-audio-changed",
            TabAudioChanged {
                tab_id: tab_id.to_string(),
                audible: audible.as_bool(),
                muted: muted.as_bool(),
            },
        );
    }

    pub(super) fn accelerator_action(
        virtual_key: u32,
        ctrl: bool,
        shift: bool,
        alt: bool,
    ) -> Option<&'static str> {
        const VK_BACK: u32 = 0x08;
        const VK_DELETE: u32 = 0x2E;
        const VK_F5: u32 = 0x74;
        const VK_F11: u32 = 0x7A;
        const VK_0: u32 = 0x30;
        const VK_H: u32 = 0x48;
        const VK_K: u32 = 0x4B;
        const VK_P: u32 = 0x50;
        const VK_R: u32 = 0x52;
        const VK_S: u32 = 0x53;
        const VK_T: u32 = 0x54;
        const VK_W: u32 = 0x57;
        const VK_OEM_PLUS: u32 = 0xBB;
        const VK_OEM_MINUS: u32 = 0xBD;

        if virtual_key == VK_F11 {
            return (!ctrl && !shift && !alt).then_some("fullscreen");
        }
        if virtual_key == VK_F5 {
            return (!ctrl && !shift && !alt).then_some("reload");
        }
        if !ctrl || alt {
            return None;
        }
        if shift {
            return match virtual_key {
                VK_DELETE | VK_BACK => Some("cleardata"),
                VK_S => Some("chat"),
                _ => None,
            };
        }
        match virtual_key {
            VK_OEM_PLUS => Some("zoomin"),
            VK_OEM_MINUS => Some("zoomout"),
            VK_0 => Some("zoomreset"),
            VK_T => Some("newtab"),
            VK_W => Some("closetab"),
            VK_H => Some("history"),
            VK_P => Some("print"),
            VK_R => Some("reload"),
            VK_K => Some("search"),
            _ => None,
        }
    }

    fn attach_keyboard_shortcuts(
        app: &AppHandle,
        tab_id: String,
        controller: &ICoreWebView2Controller,
    ) {
        use windows::Win32::UI::Input::KeyboardAndMouse::{
            GetKeyState, VK_CONTROL, VK_MENU, VK_SHIFT,
        };

        let shortcut_app = app.clone();
        let handler = AcceleratorKeyPressedEventHandler::create(Box::new(
            move |_controller, args| {
                let Some(args) = args else {
                    return Ok(());
                };

                let mut kind = COREWEBVIEW2_KEY_EVENT_KIND(0);
                unsafe { args.KeyEventKind(&mut kind)? };
                if kind != COREWEBVIEW2_KEY_EVENT_KIND_KEY_DOWN
                    && kind != COREWEBVIEW2_KEY_EVENT_KIND_SYSTEM_KEY_DOWN
                {
                    return Ok(());
                }

                let mut virtual_key = 0u32;
                unsafe { args.VirtualKey(&mut virtual_key)? };

                let held = |key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY| {
                    (unsafe { GetKeyState(key.0 as i32) } as u16 & 0x8000) != 0
                };
                let Some(action) = accelerator_action(
                    virtual_key,
                    held(VK_CONTROL),
                    held(VK_SHIFT),
                    held(VK_MENU),
                ) else {
                    return Ok(());
                };

                unsafe { args.SetHandled(true)? };

                if action == "fullscreen" {
                    if let Some(main) = shortcut_app.get_window("main") {
                        match main.is_fullscreen() {
                            Ok(fullscreen) => {
                                let want = !fullscreen;
                                #[cfg(windows)]
                                {
                                    if let Err(error) = super::apply_fullscreen_correctly(&main, want) {
                                        eprintln!(
                                            "[star] fullscreen: apply_fullscreen_correctly failed: {error:?}"
                                        );
                                        if let Err(fallback) = main.set_fullscreen(want) {
                                            eprintln!(
                                                "[star] fullscreen: fallback set_fullscreen also failed: {fallback:?}"
                                            );
                                        }
                                    }
                                }
                                #[cfg(not(windows))]
                                {
                                    if want {
                                        let was_max = main.is_maximized().unwrap_or(false);
                                        if was_max {
                                            let _ = main.unmaximize();
                                            std::thread::sleep(std::time::Duration::from_millis(120));
                                        }
                                    }
                                    if let Err(error) = main.set_fullscreen(want) {
                                        eprintln!("[star] fullscreen: failed to change window state: {error:?}");
                                    }
                                }
                            }
                            Err(error) => {
                                eprintln!(
                                    "[star] fullscreen: failed to read window state: {error:?}"
                                );
                            }
                        }
                    }
                    return Ok(());
                }

                let _ = shortcut_app.emit(
                    "tab-shortcut",
                    TabShortcut {
                        tab_id: tab_id.clone(),
                        action: action.to_string(),
                    },
                );
                Ok(())
            },
        ));

        let mut token: i64 = 0;
        if let Err(e) = unsafe { controller.add_AcceleratorKeyPressed(&handler, &mut token) } {
            eprintln!("[star] shortcuts: failed to register AcceleratorKeyPressed: {e:?}");
        }
    }

    fn attach_audio_tracking(app: &AppHandle, tab_id: String, core: &ICoreWebView2) {
        let Ok(core8) = core.cast::<ICoreWebView2_8>() else {
            return;
        };

        let audio_app = app.clone();
        let audio_tab = tab_id.clone();
        let audio_handler =
            IsDocumentPlayingAudioChangedEventHandler::create(Box::new(move |sender, _| {
                if let Some(core8) = sender.and_then(|s| s.cast::<ICoreWebView2_8>().ok()) {
                    emit_audio_state(&audio_app, &audio_tab, &core8);
                }
                Ok(())
            }));
        let mut audio_token: i64 = 0;
        let _ =
            unsafe { core8.add_IsDocumentPlayingAudioChanged(&audio_handler, &mut audio_token) };

        let mute_app = app.clone();
        let mute_handler = IsMutedChangedEventHandler::create(Box::new(move |sender, _| {
            if let Some(core8) = sender.and_then(|s| s.cast::<ICoreWebView2_8>().ok()) {
                emit_audio_state(&mute_app, &tab_id, &core8);
            }
            Ok(())
        }));
        let mut mute_token: i64 = 0;
        let _ = unsafe { core8.add_IsMutedChanged(&mute_handler, &mut mute_token) };
    }

    pub fn force_close(webview: &Webview) {
        let _ = webview.with_webview(|platform| {
            let _ = unsafe { platform.controller().Close() };
        });
    }

    pub fn apply_muted(platform: &tauri::webview::PlatformWebview, muted: bool) {
        let Ok(core) = (unsafe { platform.controller().CoreWebView2() }) else {
            return;
        };
        let Ok(core8) = core.cast::<ICoreWebView2_8>() else {
            return;
        };
        let _ = unsafe { core8.SetIsMuted(muted) };
    }

    fn percent_encode(input: &str) -> String {
        let mut out = String::with_capacity(input.len() * 3);
        for b in input.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{b:02X}")),
            }
        }
        out
    }

    fn error_page_html(original_url: &str, heading: &str, message: &str, detail: &str) -> String {
        let encoded = percent_encode(original_url);
        format!(
            r#"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{heading}</title>
<style>
:root {{
  color-scheme: light dark;
  --page: #ffffff;
  --ink: #1f2328;
  --muted: #5c636a;
  --line: rgba(31, 35, 40, .14);
  --accent: #4a3a2e;
}}
@media (prefers-color-scheme: dark) {{
  :root {{
    --page: #1c1917;
    --ink: #f2efec;
    --muted: #a5a09b;
    --line: rgba(255, 255, 255, .16);
    --accent: #f2efec;
  }}
}}
* {{ box-sizing: border-box; }}
html, body {{ height: 100%; }}
body {{
  margin: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 32px;
  background: var(--page);
  color: var(--ink);
  font: 400 15px/1.6 -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
}}
main {{ width: 100%; max-width: 460px; }}
h1 {{
  margin: 0 0 12px;
  font-size: 22px;
  font-weight: 600;
  letter-spacing: -.01em;
}}
p {{ margin: 0 0 8px; color: var(--muted); }}
.detail {{
  margin: 20px 0 0;
  padding-top: 16px;
  border-top: 1px solid var(--line);
  font-size: 12px;
  font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas, monospace;
  color: var(--muted);
  overflow-wrap: anywhere;
}}
button {{
  margin-top: 24px;
  padding: 9px 18px;
  border: 1px solid var(--line);
  border-radius: 8px;
  background: transparent;
  color: var(--accent);
  font: inherit;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color .15s ease, border-color .15s ease;
}}
button:hover {{ background: color-mix(in srgb, currentColor 8%, transparent); border-color: currentColor; }}
</style></head>
<body>
<main>
<h1>{heading}</h1>
<p>{message}</p>
<button id="star-retry" type="button">Reload</button>
<p class="detail">{detail}</p>
</main>
<script>
var target = decodeURIComponent("{encoded}");
function retry() {{ if (target) location.replace(target); else location.reload(); }}
document.getElementById('star-retry').addEventListener('click', retry);
window.addEventListener('online', retry);
</script></body></html>"#
        )
    }

    fn escape_html(value: &str) -> String {
        value
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;")
    }

    fn offline_page_html(original_url: &str) -> String {
        error_page_html(
            original_url,
            "No internet connection",
            "Star cannot reach the network right now. Check your connection, then reload the page.",
            &escape_html(original_url),
        )
    }

    fn site_error_page_html(original_url: &str) -> String {
        let host = url::Url::parse(original_url)
            .ok()
            .and_then(|u| u.host_str().map(|h| h.to_string()))
            .unwrap_or_default();
        let subject = if host.is_empty() {
            "This site".to_string()
        } else {
            escape_html(&host)
        };
        error_page_html(
            original_url,
            "This site can&#39;t be reached",
            &format!(
                "{subject} took too long to respond, or the address does not exist. Check the address, then reload the page."
            ),
            &escape_html(original_url),
        )
    }

    fn attach_offline_page(core: &ICoreWebView2) {
        let handler = NavigationCompletedEventHandler::create(Box::new(move |sender, args| {
            let (Some(sender), Some(args)) = (sender, args) else {
                return Ok(());
            };
            let mut success = BOOL::default();
            unsafe { args.IsSuccess(&mut success)? };
            if success.as_bool() {
                return Ok(());
            }
            let mut status = COREWEBVIEW2_WEB_ERROR_STATUS::default();
            unsafe { args.WebErrorStatus(&mut status)? };
            let offline = status == COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED
                || status == COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT;
            let site_unreachable = status == COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED
                || status == COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT
                || status == COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE;
            if !offline && !site_unreachable {
                return Ok(());
            }

            let original = {
                let mut source = PWSTR::null();
                match unsafe { sender.Source(&mut source) } {
                    Ok(()) => take_pwstr(source),
                    Err(_) => String::new(),
                }
            };
            let html = HSTRING::from(if offline {
                offline_page_html(&original)
            } else {
                site_error_page_html(&original)
            });
            let _ = unsafe { sender.NavigateToString(&html) };
            Ok(())
        }));
        let mut token: i64 = 0;
        let _ = unsafe { core.add_NavigationCompleted(&handler, &mut token) };
    }

    struct PendingNewWindow {
        args: ICoreWebView2NewWindowRequestedEventArgs,
        deferral: ICoreWebView2Deferral,
    }
    unsafe impl Send for PendingNewWindow {}

    fn handle_new_window_requested(
        app: &AppHandle,
        tab_id: String,
        args: ICoreWebView2NewWindowRequestedEventArgs,
    ) {
        let target_uri = {
            let mut uri = PWSTR::null();
            match unsafe { args.Uri(&mut uri) } {
                Ok(()) => take_pwstr(uri),
                Err(_) => String::new(),
            }
        };
        if !target_uri.is_empty() {
            let _ = app.emit(
                "tab-popup",
                TabPopup {
                    tab_id,
                    url: target_uri,
                },
            );
            unsafe {
                let _ = args.SetHandled(true);
            }
            return;
        }

        let reserved = LIVE_GHOSTS
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |live| {
                (live < MAX_LIVE_GHOSTS).then_some(live + 1)
            })
            .is_ok();
        if !reserved {
            unsafe {
                let _ = args.SetHandled(true);
            }
            return;
        }

        let Ok(deferral) = (unsafe { args.GetDeferral() }) else {
            LIVE_GHOSTS.fetch_sub(1, Ordering::SeqCst);
            return;
        };
        let pending = PendingNewWindow { args, deferral };
        let spawn_app = app.clone();
        let _ = app.run_on_main_thread(move || {
            let pending = pending;
            let (Some(main), Ok(blank_url)) = (spawn_app.get_window("main"), "about:blank".parse())
            else {
                let _ = unsafe { pending.deferral.Complete() };
                LIVE_GHOSTS.fetch_sub(1, Ordering::SeqCst);
                return;
            };

            let label = format!("popup-{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
            let builder = WebviewBuilder::new(&label, WebviewUrl::External(blank_url));
            let Ok(ghost) = main.add_child(
                builder,
                LogicalPosition::new(-2000.0, -2000.0),
                LogicalSize::new(1.0, 1.0),
            ) else {
                let _ = unsafe { pending.deferral.Complete() };
                LIVE_GHOSTS.fetch_sub(1, Ordering::SeqCst);
                return;
            };
            let _ = ghost.hide();

            let tracking_app = spawn_app.clone();
            let attach = ghost.with_webview(move |platform| {
                let pending = pending;
                let Ok(core) = (unsafe { platform.controller().CoreWebView2() }) else {
                    let _ = unsafe { pending.deferral.Complete() };
                    return;
                };
                attach_download_tracking(&tracking_app, tab_id, &core);
                unsafe {
                    let _ = pending.args.SetNewWindow(&core);
                    let _ = pending.args.SetHandled(true);
                    let _ = pending.deferral.Complete();
                }
            });
            if attach.is_err() {
                LIVE_GHOSTS.fetch_sub(1, Ordering::SeqCst);
                let _ = ghost.close();
                return;
            }

            let cleanup = ghost.clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let close_target = cleanup.clone();
                let _ = cleanup.run_on_main_thread(move || {
                    force_close(&close_target);
                    let _ = close_target.close();
                });
                LIVE_GHOSTS.fetch_sub(1, Ordering::SeqCst);
            });
        });
    }

    fn attach_download_tracking(app: &AppHandle, tab_id: String, core: &ICoreWebView2) {
        let Ok(core4) = core.cast::<ICoreWebView2_4>() else {
            eprintln!(
                "[star] downloads: WebView2 runtime does not support ICoreWebView2_4 for tab {tab_id}; download tracking disabled for this tab"
            );
            return;
        };
        let download_app = app.clone();
        let download_tab_id = tab_id.clone();
        let download_handler = DownloadStartingEventHandler::create(Box::new(move |_, args| {
            let Some(args) = args else {
                return Ok(());
            };

            let operation = match unsafe { args.DownloadOperation() } {
                Ok(op) => op,
                Err(e) => {
                    eprintln!("[star] downloads: DownloadOperation() failed: {e:?}");
                    return Err(e);
                }
            };
            let uri = {
                let mut uri = PWSTR::null();
                if let Err(e) = unsafe { operation.Uri(&mut uri) } {
                    eprintln!("[star] downloads: DownloadOperation::Uri() failed: {e:?}");
                    return Err(e);
                }
                take_pwstr(uri)
            };
            let destination = {
                let mut destination = PWSTR::null();
                if let Err(e) = unsafe { args.ResultFilePath(&mut destination) } {
                    eprintln!("[star] downloads: ResultFilePath() failed: {e:?}");
                    return Err(e);
                }
                take_pwstr(destination)
            };
            let file_name = file_name_for(&destination, &uri);

            if !destination.is_empty() {
                let destination = HSTRING::from(destination.as_str());
                unsafe {
                    if let Err(e) = args.SetResultFilePath(&destination) {
                        eprintln!("[star] downloads: SetResultFilePath() failed: {e:?}");
                        return Err(e);
                    }
                    if let Err(e) = args.SetHandled(true) {
                        eprintln!("[star] downloads: SetHandled() failed: {e:?}");
                        return Err(e);
                    }
                }
            }

            if let Err(e) = download_app.emit(
                "download-started",
                DownloadStarted {
                    tab_id: download_tab_id.clone(),
                    file_name: file_name.clone(),
                },
            ) {
                eprintln!("[star] downloads: failed to emit download-started: {e:?}");
            }

            let completed_app = download_app.clone();
            let completed_tab_id = download_tab_id.clone();
            let completed_file_name = file_name.clone();
            let state_handler = StateChangedEventHandler::create(Box::new(move |operation, _| {
                let Some(operation) = operation else {
                    return Ok(());
                };
                let mut state = COREWEBVIEW2_DOWNLOAD_STATE::default();
                if let Err(e) = unsafe { operation.State(&mut state) } {
                    eprintln!("[star] downloads: DownloadOperation::State() failed: {e:?}");
                    return Err(e);
                }
                if state != COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS {
                    if let Err(e) = completed_app.emit(
                        "download-finished",
                        DownloadFinished {
                            tab_id: completed_tab_id.clone(),
                            file_name: completed_file_name.clone(),
                            success: state == COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED,
                        },
                    ) {
                        eprintln!("[star] downloads: failed to emit download-finished: {e:?}");
                    }
                }
                Ok(())
            }));
            let mut state_token: i64 = 0;
            if let Err(e) = unsafe { operation.add_StateChanged(&state_handler, &mut state_token) }
            {
                eprintln!("[star] downloads: add_StateChanged() failed: {e:?}");
                return Err(e);
            }

            Ok(())
        }));
        let mut download_token: i64 = 0;
        if let Err(e) =
            unsafe { core4.add_DownloadStarting(&download_handler, &mut download_token) }
        {
            eprintln!(
                "[star] downloads: failed to register DownloadStarting handler for tab {tab_id}: {e:?}"
            );
        }
    }

    pub fn current(app: &AppHandle) -> Option<PermissionRequested> {
        app.state::<PermissionRegistry>().current_cloned()
    }

    fn pump(app: &AppHandle) {
        let next = {
            let registry = app.state::<PermissionRegistry>();
            let mut current = lock_recover(&registry.current);
            if current.is_some() {
                return;
            }
            let next = lock_recover(&registry.queue).pop_front();
            current.clone_from(&next);
            next
        };

        match next {
            Some(request) => {
                if show_permission_overlay(app, request.clone()).is_err() {
                    resolve(app, &request.request_id, false);
                }
            }
            None => {
                let _ = hide_permission_overlay(app);
            }
        }
    }

    fn complete_pending(app: &AppHandle, request_id: &str, granted: bool) {
        let pending = lock_recover(&app.state::<PermissionRegistry>().pending).remove(request_id);
        if let Some(pending) = pending {
            let _ = app.run_on_main_thread(move || {
                let pending = pending;
                let state = if granted {
                    COREWEBVIEW2_PERMISSION_STATE_ALLOW
                } else {
                    COREWEBVIEW2_PERMISSION_STATE_DENY
                };
                let _ = unsafe { pending.args.SetState(state) };
                let _ = unsafe { pending.deferral.Complete() };
            });
        }
    }

    pub fn resolve(app: &AppHandle, request_id: &str, granted: bool) {
        {
            let registry = app.state::<PermissionRegistry>();
            let mut current = lock_recover(&registry.current);
            if current.as_ref().map(|c| c.request_id.as_str()) == Some(request_id) {
                *current = None;
            }
        }

        complete_pending(app, request_id, granted);
        pump(app);
    }

    pub fn forget_tab(app: &AppHandle, tab_id: &str) {
        let registry = app.state::<PermissionRegistry>();

        let mut orphaned: Vec<String> = Vec::new();
        {
            let mut queue = lock_recover(&registry.queue);
            queue.retain(|request| {
                let ours = request.tab_id == tab_id;
                if ours {
                    orphaned.push(request.request_id.clone());
                }
                !ours
            });
        }

        let showing = {
            let current = lock_recover(&registry.current);
            current
                .as_ref()
                .filter(|request| request.tab_id == tab_id)
                .map(|request| request.request_id.clone())
        };

        for request_id in orphaned {
            complete_pending(app, &request_id, false);
        }

        if let Some(request_id) = showing {
            resolve(app, &request_id, false);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use webview2_com::Microsoft::Web::WebView2::Win32::{
            COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ,
            COREWEBVIEW2_PERMISSION_KIND_UNKNOWN_PERMISSION,
        };

        #[test]
        fn maps_each_supported_kind_to_its_own_label() {
            assert_eq!(
                kind_name(COREWEBVIEW2_PERMISSION_KIND_CAMERA),
                Some("camera")
            );
            assert_eq!(
                kind_name(COREWEBVIEW2_PERMISSION_KIND_MICROPHONE),
                Some("microphone")
            );
            assert_eq!(
                kind_name(COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION),
                Some("geolocation")
            );
            assert_eq!(
                kind_name(COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS),
                Some("notifications")
            );
        }

        #[test]
        fn leaves_unhandled_kinds_to_the_engine() {
            assert_eq!(kind_name(COREWEBVIEW2_PERMISSION_KIND_CLIPBOARD_READ), None);
            assert_eq!(
                kind_name(COREWEBVIEW2_PERMISSION_KIND_UNKNOWN_PERMISSION),
                None
            );
        }

        #[test]
        fn ad_host_list_has_no_duplicates() {
            let mut seen = std::collections::HashSet::new();
            for host in AD_HOSTS {
                assert!(seen.insert(*host), "{host} appears twice in AD_HOSTS");
            }
        }

        #[test]
        fn base_domain_finds_the_registrable_name() {
            assert_eq!(base_domain("example.com"), "example.com");
            assert_eq!(base_domain("cdn.images.example.com"), "example.com");
            assert_eq!(base_domain("www.bbc.co.uk"), "bbc.co.uk");
            assert_eq!(base_domain("a.b.c.example.co.jp"), "example.co.jp");
            assert_eq!(base_domain("localhost"), "localhost");
        }

        #[test]
        fn first_party_requests_are_never_blocked() {
            for (page, resource) in [
                ("www.tiktok.com", "https://www.tiktok.com/api/item/list"),
                ("mixpanel.com", "https://cdn.mixpanel.com/app.js"),
                ("www.hotjar.com", "https://static.hotjar.com/c/hotjar.js"),
                (
                    "news.example.com",
                    "https://img.example.com/banners/hero.jpg",
                ),
            ] {
                assert!(
                    !is_ad_url(resource, Some(page)),
                    "{resource} should be allowed as first-party on {page}"
                );
            }
        }

        #[test]
        fn third_party_trackers_are_still_blocked() {
            for resource in [
                "https://www.google-analytics.com/collect?v=1",
                "https://securepubads.g.doubleclick.net/tag/js/gpt.js",
                "https://static.hotjar.com/c/hotjar.js",
                "https://cdn.taboola.com/libtrc/loader.js",
            ] {
                assert!(
                    is_ad_url(resource, Some("news.example.com")),
                    "{resource} should be blocked as third-party"
                );
            }
        }

        #[test]
        fn ordinary_third_party_assets_are_left_alone() {
            for resource in [
                "https://cdn.jwplayer.com/libraries/abc.js",
                "https://fonts.gstatic.com/s/inter.woff2",
                "https://p16-sign.tiktokcdn-us.com/video.mp4",
                "https://unpkg.com/react@18/umd/react.production.min.js",
            ] {
                assert!(
                    !is_ad_url(resource, Some("news.example.com")),
                    "{resource} is a normal asset and must not be blocked"
                );
            }
        }

        #[test]
        fn non_http_schemes_are_ignored() {
            assert!(!is_ad_url(
                "data:image/png;base64,AAAA",
                Some("example.com")
            ));
            assert!(!is_ad_url(
                "blob:https://example.com/abc",
                Some("example.com")
            ));
        }
    }
}
