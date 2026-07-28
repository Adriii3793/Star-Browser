use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

const DAILY_LIMIT: i64 = 50;
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
const MODEL: &str = "google/gemma-4-26b-a4b-it:free";

const AI_TIMEOUT: Duration = Duration::from_secs(90);
const AI_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

/// Shared client: reuses the connection pool instead of rebuilding TLS per call.
fn http_client() -> Result<&'static reqwest::Client, AppError> {
    static CLIENT: OnceLock<Option<reqwest::Client>> = OnceLock::new();
    CLIENT
        .get_or_init(|| {
            reqwest::Client::builder()
                .timeout(AI_TIMEOUT)
                .connect_timeout(AI_CONNECT_TIMEOUT)
                .build()
                .ok()
        })
        .as_ref()
        .ok_or(AppError::AiRequest)
}

fn api_key() -> Result<String, AppError> {
    std::env::var("OPENROUTER_API_KEY").map_err(|_| AppError::MissingApiKey)
}

fn now_millis() -> i64 {
    SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| elapsed.as_millis() as i64)
    .unwrap_or(0)
}
#[derive(Serialize)]
pub struct UsageStatus {
    pub used: i64,
    pub limit: i64,
}



async fn requests_in_window(state: &AppState) -> Result<i64, AppError> {
    let since = now_millis() - WINDOW_MS;
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM usage_log WHERE used_at >= ?1")
    .bind(since)
    .fetch_one(&state.db)
    .await?;
    Ok(count)
    
}
#[tauri::command]
pub async fn usage_status(state: State<'_, AppState>) -> Result<UsageStatus, AppError> {
    Ok(UsageStatus {
        used: requests_in_window(&state).await?,
        limit: DAILY_LIMIT,
    }) 
}

#[derive(Serialize)]
struct ChatRequest <'a> {
    model: &'a str,
    messages: Vec<ChatMessage>,
}
#[derive(Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: Value,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChatMessage,
}

fn content_to_text(content: &Value) -> String {
    match content {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts
            .iter()
            .filter_map(|p| p.get("text").and_then(Value::as_str))
            .collect::<Vec<_>>()
            .join("\n"),
        _ => String::new(),
    }
}

#[tauri::command]
pub async fn ai_chat(
    state: State<'_, AppState>,
    messages: Vec<ChatMessage>,

) -> Result<String, AppError> {
    let used = requests_in_window(&state).await?;
    if used >= DAILY_LIMIT {
        return Err(AppError::RateLimited);
    }

    let key = api_key()?;

    let body = ChatRequest {
        model: MODEL,
        messages,
    };

    let response = http_client()?
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| if e.is_timeout() { AppError::AiTimeout } else { AppError::AiRequest })?;

    if !response.status().is_success() {
        return Err(AppError::AiRequest);
    }

    let parsed: ChatResponse = response.json().await.map_err(|_| AppError::AiRequest)?;

    let reply = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| content_to_text(&choice.message.content))
        .ok_or(AppError::AiRequest)?;

    sqlx::query("INSERT INTO usage_log (used_at) VALUES (?1)")
        .bind(now_millis())
        .execute(&state.db)
        .await?;

    Ok(reply)
    }