use tauri::webview::WebviewBuilder;
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
    let builder = WebviewBuilder::new(&label, WebviewUrl::External(parsed))
        .on_navigation(move |url| {
            let _ = nav_app.emit(
                "tab-url-changed",
                TabUrlChanged {
                    tab_id: nav_tab_id.clone(),
                    url: url.to_string(),
                },
            );
            true
        });

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