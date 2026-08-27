use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct HistoryEntry {
    pub id: i64,
    pub url: String,
    pub title: String,
    pub query: Option<String>,
    pub visited_at: i64,
    pub visit_count: i64,
}

fn now_millis() -> i64 {
    SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| elapsed.as_millis() as i64)
    .unwrap_or(0)
}

#[tauri::command] 
pub async fn record_visit(
    state: State<'_, AppState>,
    url: String,
    title: String,
    query: Option<String>,
) -> Result<(), AppError> {
    if url.trim().is_empty() {
        return Ok(());
    }

    sqlx::query(
        "INSERT INTO history (url, title, query, visited_at, visit_count)
        VALUES (?1, ?2, ?3, ?4, 1)
        ON CONFLICT(url) DO UPDATE SET
        title = excluded.title,
        query = COALESCE(excluded.query, history.query),
        visited_at = excluded.visited_at,
        visit_count = history.visit_count + 1",
    )
    .bind(url)
    .bind(title)
    .bind(query)
    .bind(now_millis())
    .execute(&state.db)
    .await?;

    Ok(())
}

#[tauri::command]
pub async fn recent_history(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<HistoryEntry>, AppError>{
    let entries = sqlx::query_as::<_, HistoryEntry>(
        "SELECT id, url, title, query, visited_at, visit_count
        FROM history
        ORDER BY  visited_at DESC
        LIMIT ?1",
    )
    .bind(limit.clamp(1, 200))
    .fetch_all(&state.db)
    .await?;

    Ok(entries)
}

fn like_pattern(term: &str) -> String {
    let mut escaped = String::with_capacity(term.len() + 2);
    for c in term.trim().chars() {
        if matches!(c, '%' | '_' | '\\') {
            escaped.push('\\');
        }
        escaped.push(c);
    }
    format!("%{escaped}%")
}

#[tauri::command]
pub async fn search_history(
    state: State<'_, AppState>,
    term: String,
    limit: i64,
) -> Result<Vec<HistoryEntry>, AppError> {
    let pattern = like_pattern(&term);

    let entries = sqlx::query_as::<_, HistoryEntry>(
        "SELECT id, url, title, query, visited_at, visit_count
        FROM history
        WHERE query LIKE ?1 ESCAPE '\\' OR title LIKE ?1 ESCAPE '\\' OR url LIKE ?1 ESCAPE '\\'
        ORDER BY visited_at DESC
        LIMIT ?2",
    )
    .bind(pattern)
    .bind(limit.clamp(1, 200))
    .fetch_all(&state.db)
    .await?;

    Ok(entries)
}

#[tauri::command]
pub async fn clear_history(state: State<'_, AppState>) -> Result<(), AppError> {
    sqlx::query("DELETE FROM history")
    .execute(&state.db)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::like_pattern;

    #[test]
    fn escapes_like_metacharacters_so_they_match_literally() {
        assert_eq!(like_pattern("50%"), r"%50\%%");
        assert_eq!(like_pattern("my_file"), r"%my\_file%");
        assert_eq!(like_pattern(r"a\b"), r"%a\\b%");
    }

    #[test]
    fn leaves_ordinary_terms_alone_and_trims() {
        assert_eq!(like_pattern("  github  "), "%github%");
    }
}
