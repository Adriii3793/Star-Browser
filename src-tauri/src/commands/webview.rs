use tauri::webview::{WebviewBuilder};
use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, State, WebviewUrl};

use crate::error::AppError;
use crate::state::AppState;

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
struct TabShortcut {
    tab_id: String,
    action: String,
}

const SHORTCUT_TITLE_PREFIX: &str = "@@star-shortcut@@:";

const SHORTCUT_FORWARD_SCRIPT: &str = r#"(function () {
  var PREFIX = "@@star-shortcut@@:";

  function resolveAction(e) {
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
    return null;
  }

  function signalViaTitle(signal) {
    var original = document.title;
    document.title = signal;
    Promise.resolve().then(function () {
      if (document.title === signal) document.title = original;
    });
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
            return Ok(());
        }
    }

    let nav_app = app.clone();
    let nav_tab_id = tab_id.clone();
    let shortcut_app = app.clone();
    let shortcut_tab_id = tab_id.clone();
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed))
        .zoom_hotkeys_enabled(true)
        .initialization_script_for_all_frames(SHORTCUT_FORWARD_SCRIPT)
        .on_navigation(move |url| {
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
            if let Some(action) = title.strip_prefix(SHORTCUT_TITLE_PREFIX) {
                let _ = shortcut_app.emit(
                    "tab-shortcut",
                    TabShortcut {
                        tab_id: shortcut_tab_id.clone(),
                        action: action.to_string(),
                    },
                );
            }
        });

    #[cfg(target_os = "macos")]
    let builder = builder.user_agent(
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 \
         (KHTML, like Gecko) Version/17.6 Safari/605.1.15",
    );

    let main = app
    .get_window("main")
    .ok_or(AppError::WindowNotFound)?;

    let webview = main.add_child(
        builder,
        LogicalPosition::new(x, y),
        LogicalSize::new(width, height),
    )?;
    webview.show()?;

    state.views.lock().unwrap().insert(label, webview);
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

) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    let views = state.views.lock().unwrap();

    if let Some(webview) = views.get(&label) {
        webview.set_position(LogicalPosition::new(x, y))?;
        webview.set_size(LogicalSize::new(width, height))?;
    }

    Ok(())
}

#[tauri::command]
pub async fn show_tab_webview(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError>  {
    let target = label_for(&tab_id);
    let views = state.views.lock().unwrap();

    for(label, webview) in views.iter() {
        if *label == MENU_LABEL {
            continue;
        }
        if *label == target {
            webview.show()?;
        }
        else {
            webview.hide()?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn hide_tab_webview(
    state: State<'_, AppState>,
    tab_id: String,
) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.hide()?;
    }
    Ok(())
}

#[tauri::command]
pub async fn close_tab_webview(
    state: State<'_, AppState>,
    tab_id: String,

) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().remove(&label) {
        webview.close()?;
    }
    Ok(())
}

const MENU_LABEL: &str = "__menu_overlay__";

#[tauri::command]
pub async fn open_menu_webview(
    app: AppHandle,
    state: State<'_, AppState>,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
) -> Result<(), AppError> {
    {
        let views = state.views.lock().unwrap();
        if let Some(webview) = views.get(MENU_LABEL) {
            webview.set_position(LogicalPosition::new(x, y))?;
            webview.set_size(LogicalSize::new(width, height))?;
            webview.show()?;
            let _ = webview.set_focus();
            return Ok(());
        }
    }

    let builder = WebviewBuilder::new(MENU_LABEL, WebviewUrl::App("menu".into()))
        .transparent(true);

    let main = app.get_window("main").ok_or(AppError::WindowNotFound)?;

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
pub async fn close_menu_webview(state: State<'_, AppState>) -> Result<(), AppError> {
    if let Some(webview) = state.views.lock().unwrap().remove(MENU_LABEL) {
        webview.close()?;
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
    let label= label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
        webview.eval("history.forward()")?;
    }
    Ok(())
}

#[tauri::command]
pub async fn tab_reload(state: State<'_, AppState>, tab_id: String) -> Result<(), AppError> {
    let label = label_for(&tab_id);
    if let Some(webview) = state.views.lock().unwrap().get(&label) {
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
