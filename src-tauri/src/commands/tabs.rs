use serde::{Deserialize, Serialize};
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

const KEY: &str = "tab_session";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabSessionEntry {
    pub url: String,
    pub title: String,
    pub search_text: Option<String>,
    pub has_navigated: bool,
    pub hist: Vec<String>,
    pub cursor: i32,
    pub zoom: f64,
    pub group_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabSessionGroup {
    pub id: String,
    pub name: String,
    pub color: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TabSession {
    pub tabs: Vec<TabSessionEntry>,
    pub groups: Vec<TabSessionGroup>,
    pub active_index: i32,
}

#[tauri::command]
pub async fn save_tab_session(
    state: State<'_, AppState>,
    session: TabSession,
) -> Result<(), AppError> {
    let json = serde_json::to_string(&session).map_err(|_| AppError::TabSessionSerialize)?;
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
pub async fn load_tab_session(
    state: State<'_, AppState>,
) -> Result<Option<TabSession>, AppError> {
    let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?1")
        .bind(KEY)
        .fetch_optional(&state.db)
        .await?;
    Ok(row.and_then(|(v,)| serde_json::from_str(&v).ok()))
}
