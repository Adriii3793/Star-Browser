use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;

use crate::error::AppError;
use crate::state::AppState;

const DAILY_LIMIT: i64 = 50;
const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;
const MODELS: &[&str] = &[
    "nvidia/nemotron-3-nano-omni-30b-a3b-reasoning:free",
    "google/gemma-4-26b-a4b-it:free",
];
const DEFAULT_MODEL: &str = MODELS[0];

fn resolve_model(requested: Option<&str>) -> &'static str {
    requested
        .map(str::trim)
        .and_then(|name| MODELS.iter().find(|known| **known == name).copied())
        .unwrap_or(DEFAULT_MODEL)
}

const AI_TIMEOUT: Duration = Duration::from_secs(90);
const AI_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);

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
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        if !key.trim().is_empty() {
            return Ok(key.trim().to_string());
        }
    }

    match option_env!("STAR_EMBEDDED_API_KEY") {
        Some(key) if !key.trim().is_empty() => Ok(key.trim().to_string()),
        _ => Err(AppError::MissingApiKey),
    }
}

fn now_millis() -> i64 {
    SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|elapsed| elapsed.as_millis() as i64)
    .unwrap_or(0)
}
async fn requests_in_window(state: &AppState) -> Result<i64, AppError> {
    let since = now_millis() - WINDOW_MS;
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM usage_log WHERE used_at >= ?1")
    .bind(since)
    .fetch_one(&state.db)
    .await?;
    Ok(count)
    
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

fn describe_failure(status: reqwest::StatusCode, body: &str) -> String {
    let detail = serde_json::from_str::<Value>(body)
        .ok()
        .and_then(|v| {
            v.get("error")
                .and_then(|e| e.get("message").or(Some(e)))
                .map(|m| m.as_str().map(str::to_string).unwrap_or_else(|| m.to_string()))
        })
        .unwrap_or_else(|| body.chars().take(300).collect());

    let detail = detail.trim();
    if detail.is_empty() {
        format!("AI request failed ({status})")
    } else {
        format!("AI request failed ({status}): {detail}")
    }
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
    model: Option<String>,
) -> Result<String, AppError> {
    let used = requests_in_window(&state).await?;
    if used >= DAILY_LIMIT {
        return Err(AppError::RateLimited);
    }

    let key = api_key()?;

    let body = ChatRequest {
        model: resolve_model(model.as_deref()),
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

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(AppError::AiFailed(describe_failure(status, &body)));
    }

    let raw = response.text().await.map_err(|_| AppError::AiRequest)?;
    let parsed: ChatResponse = serde_json::from_str(&raw)
        .map_err(|_| AppError::AiFailed(describe_failure(status, &raw)))?;

    let reply = parsed
        .choices
        .into_iter()
        .next()
        .map(|choice| content_to_text(&choice.message.content))
        .ok_or(AppError::AiRequest)?;

    let now = now_millis();
    sqlx::query("INSERT INTO usage_log (used_at) VALUES (?1)")
        .bind(now)
        .execute(&state.db)
        .await?;

    sqlx::query("DELETE FROM usage_log WHERE used_at < ?1")
        .bind(now - WINDOW_MS)
        .execute(&state.db)
        .await?;

    Ok(reply)
}