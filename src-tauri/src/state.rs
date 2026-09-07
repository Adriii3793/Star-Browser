use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use sqlx::SqlitePool;
use tauri::Webview;

pub struct AppState {
    pub db: SqlitePool,
    pub views: Mutex<HashMap<String, Webview>>,
    pub last_tab_urls: Arc<Mutex<HashMap<String, String>>>,
    pub history_write_error: Mutex<Option<String>>,
}

pub fn lock_recover<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl AppState {
    pub fn views(&self) -> MutexGuard<'_, HashMap<String, Webview>> {
        lock_recover(&self.views)
    }

    pub fn view(&self, label: &str) -> Option<Webview> {
        self.views().get(label).cloned()
    }

    pub fn views_snapshot(&self) -> Vec<(String, Webview)> {
        self.views()
            .iter()
            .map(|(label, webview)| (label.clone(), webview.clone()))
            .collect()
    }

    pub fn last_tab_url(&self, tab_id: &str) -> Option<String> {
        lock_recover(&self.last_tab_urls).get(tab_id).cloned()
    }

    pub fn set_last_tab_url(&self, tab_id: String, url: String) {
        lock_recover(&self.last_tab_urls).insert(tab_id, url);
    }

    pub fn forget_last_tab_url(&self, tab_id: &str) {
        lock_recover(&self.last_tab_urls).remove(tab_id);
    }

    pub fn set_history_write_error(&self, error: Option<String>) {
        *lock_recover(&self.history_write_error) = error;
    }

    pub fn history_write_error(&self) -> Option<String> {
        lock_recover(&self.history_write_error).clone()
    }
}
