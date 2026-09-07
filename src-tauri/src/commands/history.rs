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

fn canonical_url(raw: &str) -> String {
    let trimmed = raw.trim();
    let Ok(parsed) = url::Url::parse(trimmed) else {
        return trimmed.to_string();
    };
    if !matches!(parsed.scheme(), "http" | "https") {
        return trimmed.to_string();
    }

    parsed.to_string()
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
    let url = canonical_url(&url);

    let outcome = sqlx::query(
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
    .await;

    match outcome {
        Ok(_) => {
            state.set_history_write_error(None);
            Ok(())
        }
        Err(cause) => {
            let error = AppError::from(cause);
            state.set_history_write_error(Some(error.to_string()));
            Err(error)
        }
    }
}

#[tauri::command]
pub async fn retitle_visit(
    state: State<'_, AppState>,
    url: String,
    title: String,
) -> Result<(), AppError> {
    if url.trim().is_empty() || title.trim().is_empty() {
        return Ok(());
    }
    sqlx::query("UPDATE history SET title = ?2 WHERE url = ?1")
        .bind(canonical_url(&url))
        .bind(title)
        .execute(&state.db)
        .await?;
    Ok(())
}

#[tauri::command]
pub async fn history_write_error(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    Ok(state.history_write_error())
}

#[tauri::command]
pub async fn recent_history(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<HistoryEntry>, AppError> {
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
    use super::{canonical_url, like_pattern};

    #[test]
    fn spellings_of_one_destination_collapse_to_a_single_key() {
        assert_eq!(canonical_url("https://example.com"), "https://example.com/");
        assert_eq!(canonical_url("https://example.com/"), "https://example.com/");
        assert_eq!(canonical_url("HTTPS://Example.COM/"), "https://example.com/");
        assert_eq!(canonical_url("https://example.com:443/"), "https://example.com/");
        assert_eq!(canonical_url("http://example.com:80/a"), "http://example.com/a");
        assert_eq!(canonical_url("  https://example.com/  "), "https://example.com/");
    }

    #[test]
    fn destinations_that_really_differ_stay_apart() {
        assert_ne!(
            canonical_url("https://example.com/watch?v=1"),
            canonical_url("https://example.com/watch?v=2")
        );
        assert_ne!(
            canonical_url("https://example.com/"),
            canonical_url("https://www.example.com/")
        );
        assert_ne!(
            canonical_url("http://example.com/"),
            canonical_url("https://example.com/")
        );
        assert_ne!(
            canonical_url("https://example.com/a"),
            canonical_url("https://example.com/b")
        );
        assert_ne!(
            canonical_url("https://example.com/app#/inbox"),
            canonical_url("https://example.com/app#/sent")
        );
    }

    #[test]
    fn anything_unparseable_or_non_http_is_stored_exactly_as_given() {
        assert_eq!(canonical_url("about:blank"), "about:blank");
        assert_eq!(canonical_url("file:///tmp/x.html"), "file:///tmp/x.html");
        assert_eq!(canonical_url("not a url"), "not a url");
    }

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
