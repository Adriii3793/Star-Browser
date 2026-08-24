use crate::error::AppError;
use crate::state::AppState;
use std::path::Path;
use tauri::webview::WebviewBuilder;
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl};

fn label_for(tab_id: &str) -> String {
    format!("tab-{tab_id}")
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct TabUrlChanged {
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
const PERMISSION_LABEL: &str = "__permission_overlay__";
const MENU_LABEL: &str = "__menu_overlay__";
const OVERLAY_LABEL: &str = "__panel_overlay__";

const OFFLINE_SCRIPT: &str = include_str!("../scripts/offline.js");
const COSMETIC_SCRIPT: &str = include_str!("../scripts/cosmetic.js");
const TAB_CORNER_RADIUS: f64 = 12.0;

static COSMETIC_ON: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);

fn cosmetic_enabled() -> bool {
    COSMETIC_ON.load(std::sync::atomic::Ordering::Relaxed)
}
const STOP_MEDIA_SCRIPT: &str = r#"(function () {
    try {
        var els = Array.prototype.slice.call(document.querySelectorAll('video,audio'));
        els.forEach(function (el) {
            var stream = el.srcObject;
            if (stream && typeof stream.getTracks === 'function') {
            stream.getTracks().forEach(function (t) { t.stop(); });
            }
        });
    } catch (e) {}
})();"#;

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

#[cfg(windows)]
pub use win_permissions::PermissionRegistry;

#[cfg(not(windows))]
#[derive(Default)]
pub struct PermissionRegistry;

fn hide_permission_overlay(app: &AppHandle) -> Result<(), AppError> {
    let state = app.state::<AppState>();
    let webview = state.views.lock().unwrap().get(PERMISSION_LABEL).cloned();
    if let Some(webview) = webview {
        webview.hide()?;
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

    let existing = state.views.lock().unwrap().get(PERMISSION_LABEL).cloned();
    if let Some(webview) = existing {
        webview.set_position(LogicalPosition::new(0.0, 0.0))?;
        webview.set_size(LogicalSize::new(size.width, size.height))?;
        let _ = webview.reparent(&main);
        webview.show()?;
        let _ = webview.set_focus();
        let _ = app.emit("permission-requested", request);
        return Ok(());
    }

    let builder = WebviewBuilder::new(PERMISSION_LABEL, WebviewUrl::App("permission".into()))
        .transparent(true);
    let webview = main.add_child(
        builder,
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(size.width, size.height),
    )?;
    webview.show()?;
    let _ = webview.set_focus();

    state
        .views
        .lock()
        .unwrap()
        .insert(PERMISSION_LABEL.to_string(), webview);

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

  function resolveAction(e) {
    if (e.key === 'F11' && !e.ctrlKey && !e.metaKey && !e.altKey && !e.shiftKey) return 'fullscreen';
    if (!(e.ctrlKey || e.metaKey) || e.altKey) return null;
    if (e.shiftKey) {
      return (e.key === 'Delete' || e.key === 'Backspace') ? 'cleardata' : null;
    }
    var key = e.key.toLowerCase();
    if (e.key === '=' || e.key === '+') return 'zoomin';
    if (e.key === '-' || e.key === '_') return 'zoomout';
    if (e.key === '0') return 'zoomreset';
    if (key === 't') return 'newtab';
    if (key === 'w') return 'closetab';
    if (key === 'h') return 'history';
    if (key === 'p') return 'print';
        if (key === 'k') return 'search';
    return null;
  }

  var titleGen = 0;
  var titleBeforeSignals = null;

  function signalViaTitle(signal) {
    if (titleBeforeSignals === null) titleBeforeSignals = document.title;
    var myGen = ++titleGen;
    document.title = signal;
    setTimeout(function () {
      if (myGen !== titleGen) return;
      var original = titleBeforeSignals;
      titleBeforeSignals = null;
      document.title = original;
    }, 0);
  }

  function signalUrl() {
    signalViaTitle(URL_PREFIX + window.location.href);
  }

  function observeHistory(method) {
    var original = window.history[method];
    if (typeof original !== 'function') return;
    window.history[method] = function () {
      var result = original.apply(this, arguments);
      signalUrl();
      return result;
    };
  }

  observeHistory('pushState');
  observeHistory('replaceState');
  window.addEventListener('popstate', signalUrl);
  window.addEventListener('hashchange', signalUrl);

  if (window.top === window) {
    var lastHref = location.href;
    setInterval(function () {
      if (location.href !== lastHref) {
        lastHref = location.href;
        signalUrl();
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
    let webview = state.views.lock().unwrap().get(&label).cloned();
    let Some(webview) = webview else {
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
    let parsed = url.parse().map_err(|_| AppError::InvalidUrl)?;

    {
        let views = state.views.lock().unwrap();
        if let Some(webview) = views.get(&label) {
            webview.navigate(parsed)?;
            webview.set_position(LogicalPosition::new(x, y))?;
            webview.set_size(LogicalSize::new(width, height))?;
            webview.show()?;
            #[cfg(windows)]
            {
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
            return Ok(());
        }
    }

    let nav_app = app.clone();
    let nav_tab_id = tab_id.clone();
    let title_app = app.clone();
    let title_tab_id = tab_id.clone();
    let shortcut_app = app.clone();
    let shortcut_tab_id = tab_id.clone();
    let last_urls = state.last_tab_urls.clone();    let mut builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed))
        .zoom_hotkeys_enabled(false)
        .initialization_script_for_all_frames(SHORTCUT_FORWARD_SCRIPT)
        .initialization_script(CONTEXT_MENU_SCRIPT)
        .initialization_script(OFFLINE_SCRIPT);
    if cosmetic_enabled() {
        builder = builder.initialization_script_for_all_frames(COSMETIC_SCRIPT);
    }
    let builder = builder
        .on_navigation(move |url| {
            if url.as_str().starts_with("http") {
                last_urls
                    .lock()
                    .unwrap()
                    .insert(nav_tab_id.clone(), url.to_string());
            }
            let _ = nav_app.emit(
                "tab-url-changed",
                TabUrlChanged {
                    tab_id: nav_tab_id.clone(),
                    url: url.to_string(),
                },
            );
            true
        })
        .on_document_title_changed(move |_webview, title| {
            if let Some(url) = title.strip_prefix(URL_TITLE_PREFIX) {
                let _ = title_app.emit(
                    "tab-url-changed",
                    TabUrlChanged {
                        tab_id: title_tab_id.clone(),
                        url: url.to_string(),
                    },
                );
            } else if let Some(action) = title.strip_prefix(SHORTCUT_TITLE_PREFIX) {
                let _ = shortcut_app.emit(
                    "tab-shortcut",
                    TabShortcut {
                        tab_id: shortcut_tab_id.clone(),
                        action: action.to_string(),
                    },
                );
            } else {
                let clean_title = title.trim();
                if !clean_title.is_empty() {
                    let _ = title_app.emit(
                        "tab-title-changed",
                        TabTitleChanged {
                            tab_id: title_tab_id.clone(),
                            title: clean_title.to_string(),
                        },
                    );
                }
            }
        });

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
    webview.show()?;

    #[cfg(windows)]
    {
        win_permissions::register(&app, tab_id.clone(), &webview);
        let scale = webview.window().scale_factor().unwrap_or(1.0);
        win_permissions::round_corners(
            &webview,
            (width * scale).round() as i32,
            (height * scale).round() as i32,
            (radius.unwrap_or(TAB_CORNER_RADIUS) * scale).round() as i32,
            round_bottom_left.unwrap_or(true),
            round_bottom_right.unwrap_or(true),
        );
    }

    state.views.lock().unwrap().insert(label, webview);
    state
        .last_tab_urls
        .lock()
        .unwrap()
        .insert(tab_id.clone(), url);
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

    let views = state.views.lock().unwrap();
    let webview = views.get(&label).ok_or(AppError::WindowNotFound)?;
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
    let views = state.views.lock().unwrap();

    if let Some(webview) = views.get(&label) {
        webview.set_position(LogicalPosition::new(x, y))?;
        webview.set_size(LogicalSize::new(width, height))?;
        #[cfg(windows)]
        {
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

    Ok(())
}

#[tauri::command]
pub async fn show_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let target = label_for(&tab_id);
    let views = state.views.lock().unwrap();

    for (label, webview) in views.iter() {
        if *label == MENU_LABEL || *label == OVERLAY_LABEL || *label == PERMISSION_LABEL {
            continue;
        }
        if *label == target {
            webview.show()?;
        } else {
            webview.hide()?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.hide()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    state.last_tab_urls.lock().unwrap().remove(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().remove(&label) {
        webview.eval(STOP_MEDIA_SCRIPT).ok();
        #[cfg(windows)]
        win_permissions::force_close(&webview);
        let _ = webview.close();
    }
    Ok(())
}

#[tauri::command]
pub async fn open_menu_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;

    {
        let views = state.views.lock().unwrap();
        if let Some(webview) = views.get(MENU_LABEL) {
            webview.set_position(LogicalPosition::new(x, y))?;
            webview.set_size(LogicalSize::new(width, height))?;
            let _ = webview.reparent(&main);
            webview.show()?;
            let _ = webview.set_focus();
            return Ok(());
        }
    }

    let builder = WebviewBuilder::new(MENU_LABEL, WebviewUrl::App("menu".into())).transparent(true);

    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    webview.show()?;
    let _ = webview.set_focus();

    state
        .views
        .lock()
        .unwrap()
        .insert(MENU_LABEL.to_string(), webview);
    Ok(())
}

#[tauri::command]
pub async fn warm_overlay_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;

    {
        let views = state.views.lock().unwrap();
        if views.contains_key(OVERLAY_LABEL) {
            return Ok(());
        }
    }

    let builder =
        WebviewBuilder::new(OVERLAY_LABEL, WebviewUrl::App("overlay".into())).transparent(true);

    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    webview.hide()?;

    state
        .views
        .lock()
        .unwrap()
        .insert(OVERLAY_LABEL.to_string(), webview);
    Ok(())
}

#[tauri::command]
pub async fn open_overlay_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;

    {
        let views = state.views.lock().unwrap();
        if let Some(webview) = views.get(OVERLAY_LABEL) {
            webview.set_position(LogicalPosition::new(x, y))?;
            webview.set_size(LogicalSize::new(width, height))?;
            let _ = webview.reparent(&main);
            webview.show()?;
            let _ = webview.set_focus();
            return Ok(());
        }
    }

    let builder =
        WebviewBuilder::new(OVERLAY_LABEL, WebviewUrl::App("overlay".into())).transparent(true);

    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    webview.show()?;
    let _ = webview.set_focus();

    state
        .views
        .lock()
        .unwrap()
        .insert(OVERLAY_LABEL.to_string(), webview);
    Ok(())
}

#[tauri::command]
pub async fn close_overlay_webview(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    if let Some(webview) = state.views.lock().unwrap().get(OVERLAY_LABEL) {
        webview.hide()?;
    }
    if let Some(main) = app.get_window("main") {
        let _ = main.set_focus();
    }
    Ok(())
}

#[tauri::command]
pub async fn close_menu_webview(app: AppHandle, state: State<'_, AppState>) -> Result<(), AppError> {
    if let Some(webview) = state.views.lock().unwrap().get(MENU_LABEL) {
        webview.hide()?;
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
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.set_zoom(factor)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_back(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval("history.back()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_forward(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval("history.forward()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_reload(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let webview = state.views.lock().unwrap().get(&label).cloned();
    if let Some(webview) = webview {
        let is_offline_page = webview
            .url()
            .map(|u| u.as_str().starts_with("data:") || u.as_str() == "about:blank")
            .unwrap_or(false);
        if is_offline_page {
            if let Some(url) = state
                .last_tab_urls
                .lock()
                .unwrap()
                .get(&tab_id)
                .cloned()
                .and_then(|u| u.parse().ok())
            {
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
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval("window.print()")?;
    }
    Ok(())
}

const MEDIA_TOGGLE_SCRIPT: &str = r#"(function () {
  var els = Array.prototype.slice.call(document.querySelectorAll('video,audio'));
  var playing = els.filter(function (m) { return !m.paused; });
  if (playing.length) {
    playing.forEach(function (m) { m.pause(); });
  } else if (els.length) {
    els[0].play().catch(function () {});
  }
})();"#;

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

#[tauri::command]
pub async fn tab_media_toggle(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval(MEDIA_TOGGLE_SCRIPT)?;
    }
    Ok(())
}
#[tauri::command]
pub async fn tab_stop_media(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval(STOP_MEDIA_SCRIPT)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn set_tab_muted(
    state: State<'_, AppState>,
    tab_id: String,
    muted: bool,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let webview = state.views.lock().unwrap().get(&label).cloned();
    if let Some(webview) = webview {
        #[cfg(windows)]
        let _ = webview.with_webview(move |platform| {
            win_permissions::apply_muted(&platform, muted);
        });
        #[cfg(not(windows))]
        let _ = (webview, muted);
    }
    Ok(())
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
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Mutex;

    use tauri::webview::WebviewBuilder;
    use tauri::{
        AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Webview, WebviewUrl,
    };
    use webview2_com::Microsoft::Web::WebView2::Win32::{
        ICoreWebView2, ICoreWebView2Deferral, ICoreWebView2NewWindowRequestedEventArgs,
        ICoreWebView2PermissionRequestedEventArgs, ICoreWebView2_2, ICoreWebView2_4,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT, COREWEBVIEW2_WEB_RESOURCE_CONTEXT_ALL,
        COREWEBVIEW2_WEB_RESOURCE_CONTEXT_DOCUMENT, COREWEBVIEW2_DOWNLOAD_STATE,
        COREWEBVIEW2_DOWNLOAD_STATE_COMPLETED, COREWEBVIEW2_DOWNLOAD_STATE_IN_PROGRESS,
        COREWEBVIEW2_PERMISSION_KIND, COREWEBVIEW2_PERMISSION_KIND_CAMERA,
        COREWEBVIEW2_PERMISSION_KIND_GEOLOCATION, COREWEBVIEW2_PERMISSION_KIND_MICROPHONE,
        COREWEBVIEW2_PERMISSION_KIND_NOTIFICATIONS, COREWEBVIEW2_PERMISSION_STATE_ALLOW,
        COREWEBVIEW2_PERMISSION_STATE_DENY, ICoreWebView2_8, COREWEBVIEW2_WEB_ERROR_STATUS,
        COREWEBVIEW2_WEB_ERROR_STATUS_CANNOT_CONNECT, COREWEBVIEW2_WEB_ERROR_STATUS_DISCONNECTED,
        COREWEBVIEW2_WEB_ERROR_STATUS_HOST_NAME_NOT_RESOLVED,
        COREWEBVIEW2_WEB_ERROR_STATUS_SERVER_UNREACHABLE, COREWEBVIEW2_WEB_ERROR_STATUS_TIMEOUT,
    };
    use webview2_com::PermissionRequestedEventHandler;
    use webview2_com::{
        take_pwstr, DownloadStartingEventHandler, ExecuteScriptCompletedHandler,
        IsDocumentPlayingAudioChangedEventHandler, IsMutedChangedEventHandler,
        NavigationCompletedEventHandler, NewWindowRequestedEventHandler, StateChangedEventHandler,
        WebResourceRequestedEventHandler,
    };
    use windows::core::BOOL;
    use windows::core::{Interface, HSTRING, PWSTR};

    use super::{
        file_name_for, hide_permission_overlay, show_permission_overlay, DownloadFinished,
        DownloadStarted, PermissionRequested, TabAudioChanged, TabPopup,
    };

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

    static NEXT_ID: AtomicU64 = AtomicU64::new(1);

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
        let base = unsafe {
            windows::Win32::Graphics::Gdi::CreateRoundRectRgn(0, 0, w, h, r * 2, r * 2)
        };
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

    pub async fn execute_script(webview: &Webview, script: &'static str) -> Option<String> {
        let (tx, rx) = tokio::sync::oneshot::channel::<Option<String>>();
        let sender = std::sync::Arc::new(Mutex::new(Some(tx)));
        let dispatch_sender = sender.clone();

        let dispatched = webview.with_webview(move |platform| {
            let finish = |slot: &std::sync::Arc<Mutex<Option<tokio::sync::oneshot::Sender<Option<String>>>>>,
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
                        registry
                            .pending
                            .lock()
                            .unwrap()
                            .insert(request_id.clone(), PendingNative { args, deferral });
                        registry
                            .queue
                            .lock()
                            .unwrap()
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

            attach_offline_page(&core);
            attach_audio_tracking(&app, tab_id.clone(), &core);
            attach_ad_blocking(&core);
            attach_download_tracking(&app, tab_id.clone(), &core);
        });
    }

    pub static ADBLOCK: AtomicBool = AtomicBool::new(true);

    pub fn set_adblock_enabled(enabled: bool) {
        ADBLOCK.store(enabled, Ordering::Relaxed);
    }

    const AD_HOSTS: &[&str] = &[
        "doubleclick.net",
        "googlesyndication.com",
        "googleadservices.com",
        "googletagservices.com",
        "google-analytics.com",
        "googletagmanager.com",
        "amazon-adsystem.com",
        "adnxs.com",
        "adnxs-simple.com",
        "criteo.com",
        "criteo.net",
        "taboola.com",
        "taboolasyndication.com",
        "outbrain.com",
        "zemanta.com",
        "pubmatic.com",
        "rubiconproject.com",
        "openx.net",
        "adsafeprotected.com",
        "moatads.com",
        "scorecardresearch.com",
        "adsrvr.org",
        "smartadserver.com",
        "yieldmo.com",
        "casalemedia.com",
        "indexww.com",
        "sharethrough.com",
        "teads.tv",
        "3lift.com",
        "bidswitch.net",
        "sovrn.com",
        "lijit.com",
        "media.net",
        "adform.net",
        "adcolony.com",
        "applovin.com",
        "unityads.unity3d.com",
        "inmobi.com",
        "mopub.com",
        "chartbeat.com",
        "quantserve.com",
        "quantcount.com",
        "krxd.net",
        "bluekai.com",
        "demdex.net",
        "everesttech.net",
        "omtrdc.net",
        "branch.io",
        "segment.io",
        "segment.com",
        "amplitude.com",
        "mixpanel.com",
        "hotjar.com",
        "fullstory.com",
        "mouseflow.com",
        "clarity.ms",
        "newrelic.com",
        "nr-data.net",
        "optimizely.com",
        "onesignal.com",
        "pushwoosh.com",
        "adroll.com",
        "servedbyadbutler.com",
        "revcontent.com",
        "mgid.com",
        "propellerads.com",
        "popads.net",
        "poperblocker.com",
        "exoclick.com",
        "juicyads.com",
        "trafficjunky.net",
        "zedo.com",
        "adhigh.net",
        "adskeeper.com",
        "yieldlab.net",
        "improvedigital.com",
        "districtm.io",
        "gumgum.com",
        "spotxchange.com",
        "springserve.com",
        "innovid.com",
        "flashtalking.com",
        "sizmek.com",
        "serving-sys.com",
        "advertising.com",
        "adtechus.com",
        "bidr.io",
        "agkn.com",
        "rlcdn.com",
        "crwdcntrl.net",
        "tapad.com",
        "id5-sync.com",
        "pippio.com",
        "ads.yahoo.com",
        "ads.linkedin.com",
        "analytics.tiktok.com",
        "ads-twitter.com",
        "ads.pinterest.com",
        "mc.yandex.ru",
        "mc.yandex.com",
        "an.yandex.ru",
        "yandexadexchange.net",
        "adfox.ru",
        "adriver.ru",
        "counter.yadro.ru",
        "top-fwz1.mail.ru",
        "rs.mail.ru",
        "ad.mail.ru",
        "top100.rambler.ru",
        "connect.facebook.net",
        "matomo.cloud",
        "piwik.pro",
        "statcounter.com",
        "histats.com",
        "sitemeter.com",
        "addthis.com",
        "sharethis.com",
        "crazyegg.com",
        "luckyorange.com",
        "inspectlet.com",
        "smartlook.com",
        "heapanalytics.com",
        "kissmetrics.com",
        "keen.io",
        "loggly.com",
        "bugsnag.com",
        "sentry-cdn.com",
        "trackjs.com",
        "bugsnag.com",
        "d2wy8f7a9ursnm.cloudfront.net",
        "clicktale.net",
        "tealiumiq.com",
        "ensighten.com",
        "adobedtm.com",
        "2o7.net",
        "coremetrics.com",
        "webtrends.com",
        "comscore.com",
        "nielsen.com",
        "imrworldwide.com",
        "effectivemeasure.net",
        "gemius.pl",
        "wt-safetag.com",
        "yieldoptimizer.com",
        "contextweb.com",
        "adtelligent.com",
        "betweendigital.com",
        "buysellads.com",
        "carbonads.com",
        "ezoic.net",
        "ezojs.com",
        "playwire.com",
        "primis.tech",
        "vidoomy.com",
        "onetag-sys.com",
        "smaato.net",
        "tremorhub.com",
        "yieldmo.net",
        "amazonaax.com",
        "assoc-amazon.com",
        "criteo.com",
        "bidswitch.net",
        "magnite.com",
        "spotxchange.com",
        "rtbhouse.com",
        "smartadserver.com",
        "adition.com",
        "adscale.de",
        "mediaimpact.de",
        "gg.gg",
        "tiktok.com",
        "tiktokcdn.com",
        "ads.tiktok.com",
        "business-api.tiktok.com",
        "redditinc.com",
        "ads.reddit.com",
        "events.redditmedia.com",
        "snap-adkit.com",
        "ads.linkedin.com",
        "adx.linkedin.com",
        "ads-twitter.com",
        "static.ads-twitter.com",
        "ads.pinterest.com",
        "logs.pinterest.com",
        "tr.snapchat.com",
        "app-measurement.com",
        "firebaseinstallations.googleapis.com",
        "doubleverify.com",
        "dvtps.com",
        "adsafeprotected.com",
        "moatpixel.com",
        "iasds01.com",
        "screencore.io",
        "openweb.com",
        "spot.im",
        "arc.io",
        "permutive.com",
        "permutive.app",
        "1dmp.io",
        "id5-sync.com",
        "uidapi.com",
        "adhigh.net",
        "yieldmo.com",
        "sonobi.com",
        "gamoshi.com",
        "smilewanted.com",
        "improvedigital.com",
        "videoheroes.tv",
        "connatix.com",
        "cdn.connatix.com",
        "jwplayer.com",
        "aniview.com",
        "mediavine.com",
        "raptive.com",
        "adthrive.com",
        "monumetric.com",
        "sheknows.com",
        "playground.xyz",
        "freestar.com",
        "freestar.io",
        "ad.plus",
        "galaksion.com",
        "hilltopads.net",
        "adcash.com",
        "clickadu.com",
        "onclickalgo.com",
        "onclckpro.com",
        "onclickperformance.com",
        "push-mania.com",
        "propellerclick.com",
        "propelleradsystem.com",
        "adsterra.com",
        "adsterratech.com",
        "cashtrafic.com",
        "trafficstars.com",
        "tsyndicate.com",
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

    fn is_ad_url(uri: &str) -> bool {
        if !uri.starts_with("http") {
            return false;
        }
        match host_of(uri) {
            Some(host) => host_blocked(&host) || path_blocked(uri),
            None => false,
        }
    }

    fn attach_ad_blocking(core: &ICoreWebView2) {
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
            if !is_ad_url(&uri) {
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

    fn attach_audio_tracking(app: &AppHandle, tab_id: String, core: &ICoreWebView2) {
        let Ok(core8) = core.cast::<ICoreWebView2_8>() else {
            return;
        };

        let audio_app = app.clone();
        let audio_tab = tab_id.clone();
        let audio_core = core8.clone();
        let audio_handler = IsDocumentPlayingAudioChangedEventHandler::create(Box::new(
            move |_, _| {
                emit_audio_state(&audio_app, &audio_tab, &audio_core);
                Ok(())
            },
        ));
        let mut audio_token: i64 = 0;
        let _ =
            unsafe { core8.add_IsDocumentPlayingAudioChanged(&audio_handler, &mut audio_token) };

        let mute_app = app.clone();
        let mute_core = core8.clone();
        let mute_handler = IsMutedChangedEventHandler::create(Box::new(move |_, _| {
            emit_audio_state(&mute_app, &tab_id, &mute_core);
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
                    tab_id: tab_id.clone(),
                    url: target_uri,
                },
            );
        }

        let Ok(deferral) = (unsafe { args.GetDeferral() }) else {
            return;
        };
        let pending = PendingNewWindow { args, deferral };
        let spawn_app = app.clone();
        let _ = app.run_on_main_thread(move || {
            let pending = pending;
            let (Some(main), Ok(blank_url)) = (spawn_app.get_window("main"), "about:blank".parse())
            else {
                let _ = unsafe { pending.deferral.Complete() };
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
                let _ = ghost.close();
                return;
            }

            let cleanup = ghost.clone();
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_secs(30));
                let close_target = cleanup.clone();
                let _ = cleanup.run_on_main_thread(move || {
                    force_close(&close_target);
                    let _ = close_target.close();
                });
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
        if let Err(e) = unsafe { core4.add_DownloadStarting(&download_handler, &mut download_token) }
        {
            eprintln!(
                "[star] downloads: failed to register DownloadStarting handler for tab {tab_id}: {e:?}"
            );
        }
    }

    pub fn current(app: &AppHandle) -> Option<PermissionRequested> {
        app.state::<PermissionRegistry>()
            .current
            .lock()
            .unwrap()
            .clone()
    }

    fn pump(app: &AppHandle) {
        let next = {
            let registry = app.state::<PermissionRegistry>();
            let mut current = registry.current.lock().unwrap();
            if current.is_some() {
                return;
            }
            let next = registry.queue.lock().unwrap().pop_front();
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

    pub fn resolve(app: &AppHandle, request_id: &str, granted: bool) {
        let pending = {
            let registry = app.state::<PermissionRegistry>();
            let mut current = registry.current.lock().unwrap();
            if current.as_ref().map(|c| c.request_id.as_str()) == Some(request_id) {
                *current = None;
            }
            let pending = registry.pending.lock().unwrap().remove(request_id);
            pending
        };

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

        pump(app);
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
    }
}
