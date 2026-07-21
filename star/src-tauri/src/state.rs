use std::collections::HashMap;
use std::sync::Mutex;

use sqlx::SqlitePool;
use tauri::Webview;

pub struct AppState {
    pub db: SqlitePool,
    pub views: Mutex<HashMap<String, Webview>>,
}