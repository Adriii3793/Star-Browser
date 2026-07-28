//! Onboarding persistence.
//!
//! The completed profile lives in the `settings` table of the app database, in
//! the OS app-data directory. That location is what makes onboarding behave
//! correctly across installs: a fresh machine (or a user whose app-data was
//! removed on uninstall) has no row, so setup runs; an upgrade keeps the row,
//! so returning users are not asked again.

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use crate::error::AppError;
use crate::state::AppState;

const KEY: &str = "setup";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetupData {
    pub name: String,
    pub avatar: Option<String>,
    pub search_engine: String,
    pub theme: String,
    pub background: Option<String>,
    #[serde(default)]
    pub custom_bg: Option<String>,
    #[serde(default)]
    pub custom_surface: Option<String>,
    #[serde(default)]
    pub custom_accent: Option<String>,
}

#[tauri::command]
pub async fn is_setup_complete(state: State<'_, AppState>) -> Result<bool, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?1")
        .bind(KEY)
        .fetch_optional(&state.db)
        .await?;
    Ok(row.is_some())
}

#[tauri::command]
pub async fn save_setup(state: State<'_, AppState>, data: SetupData) -> Result<(), AppError> {
    let json = serde_json::to_string(&data).map_err(|_| AppError::SetupSerialize)?;
    sqlx::query(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(KEY)
    .bind(json)
    .execute(&state.db)
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn load_setup(state: State<'_, AppState>) -> Result<Option<SetupData>, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?1")
        .bind(KEY)
        .fetch_optional(&state.db)
        .await?;
    // A row that fails to parse (e.g. written by an older version) is treated as
    // absent so the user gets a clean onboarding instead of a hard error.
    Ok(row.and_then(|(v,)| serde_json::from_str(&v).ok()))
}

#[tauri::command]
pub async fn reset_setup(state: State<'_, AppState>) -> Result<(), AppError> {
    sqlx::query("DELETE FROM settings WHERE key = ?1")
        .bind(KEY)
        .execute(&state.db)
        .await?;
    Ok(())
}

/// Absolute path of the folder holding the database and window state.
///
/// Uninstallers deliberately leave this directory in place so upgrades keep
/// user settings, which is why a reinstall alone does not re-trigger
/// onboarding. Surfacing the path lets the user inspect or clear it.
#[tauri::command]
pub async fn data_dir(app: AppHandle) -> Result<String, AppError> {
    let dir = app.path().app_data_dir().map_err(|_| AppError::AppDir)?;
    Ok(dir.to_string_lossy().into_owned())
}

/// Reveals the data folder in the OS file manager.
#[tauri::command]
pub async fn open_data_dir(app: AppHandle) -> Result<(), AppError> {
    let dir = app.path().app_data_dir().map_err(|_| AppError::AppDir)?;

    #[cfg(target_os = "windows")]
    let mut cmd = {
        let mut c = std::process::Command::new("explorer");
        c.arg(&dir);
        c
    };

    #[cfg(target_os = "macos")]
    let mut cmd = {
        let mut c = std::process::Command::new("open");
        c.arg(&dir);
        c
    };

    #[cfg(all(unix, not(target_os = "macos")))]
    let mut cmd = {
        let mut c = std::process::Command::new("xdg-open");
        c.arg(&dir);
        c
    };

    // explorer.exe returns a non-zero exit code even on success, so only a
    // spawn failure is treated as an error here.
    cmd.spawn().map_err(|_| AppError::DefaultBrowser)?;
    Ok(())
}

/// Makes this app the default browser where the OS still allows it.
///
/// Windows 10+ and macOS both removed the ability for an application to claim
/// the default-browser role without explicit user action, so on those platforms
/// the correct behaviour is to open the relevant settings panel and return
/// `false` (meaning "not applied automatically"). Linux still honours
/// `xdg-settings`, so there it can return `true`.
#[tauri::command]
pub async fn set_default_browser() -> Result<bool, AppError> {
    #[cfg(target_os = "windows")]
    {
        // `start` is a cmd.exe builtin. The empty "" is the window-title
        // argument; without it a quoted target would be parsed as the title.
        std::process::Command::new("cmd")
            .args(["/C", "start", "", "ms-settings:defaultapps"])
            .spawn()
            .map_err(|_| AppError::DefaultBrowser)?;
        Ok(false)
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.general")
            .spawn()
            .map_err(|_| AppError::DefaultBrowser)?;
        Ok(false)
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let applied = std::process::Command::new("xdg-settings")
            .args(["set", "default-web-browser", "star.desktop"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        Ok(applied)
    }
}
