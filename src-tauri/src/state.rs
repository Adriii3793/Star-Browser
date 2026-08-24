use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sqlx::SqlitePool;
use tauri::Webview;

pub struct AppState {
    pub db: SqlitePool,
    pub views: Mutex<HashMap<String, Webview>>,
    pub last_tab_urls: Arc<Mutex<HashMap<String, String>>>,
}