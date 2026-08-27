use serde::{Deserialize, Serialize};
use tauri::State;

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
    Ok(row.and_then(|(v,)| serde_json::from_str(&v).ok()))
}
