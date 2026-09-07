use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
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

const OPENROUTER_URL: &str = "https://openrouter.ai/api/v1/chat/completions";

enum Route {
    Direct { key: String },
    Proxy { url: &'static str },
}

const KEY_SETTING: &str = "openrouter_api_key";

async fn stored_key(db: &sqlx::SqlitePool) -> Option<String> {
    sqlx::query_as::<_, (String,)>("SELECT value FROM settings WHERE key = ?1")
        .bind(KEY_SETTING)
        .fetch_optional(db)
        .await
        .ok()
        .flatten()
        .map(|(value,)| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KeySource {
    User,
    Environment,
    Embedded,
}

impl KeySource {
    fn name(self) -> &'static str {
        match self {
            KeySource::User => "user",
            KeySource::Environment => "environment",
            KeySource::Embedded => "embedded",
        }
    }
}

async fn local_key(db: &sqlx::SqlitePool) -> Option<(KeySource, String)> {
    if let Some(key) = stored_key(db).await {
        return Some((KeySource::User, key));
    }
    if let Ok(key) = std::env::var("OPENROUTER_API_KEY") {
        if !key.trim().is_empty() {
            return Some((KeySource::Environment, key.trim().to_owned()));
        }
    }
    option_env!("STAR_EMBEDDED_API_KEY")
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(|key| (KeySource::Embedded, key.to_owned()))
}

async fn route(db: &sqlx::SqlitePool) -> Result<Route, AppError> {
    if let Some((_, key)) = local_key(db).await {
        return Ok(Route::Direct { key });
    }
    match option_env!("STAR_AI_PROXY") {
        Some(url) if !url.trim().is_empty() => Ok(Route::Proxy { url: url.trim() }),
        _ => Err(AppError::MissingApiKey),
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiKeyStatus {
    pub source: &'static str,
    pub hint: Option<String>,
}

fn hint_of(key: &str) -> String {
    let tail: String = key.chars().rev().take(4).collect::<Vec<_>>().into_iter().rev().collect();
    format!("····{tail}")
}

#[tauri::command]
pub async fn ai_key_status(state: State<'_, AppState>) -> Result<AiKeyStatus, AppError> {
    if let Some((source, key)) = local_key(&state.db).await {
        return Ok(AiKeyStatus {
            source: source.name(),
            hint: Some(hint_of(&key)),
        });
    }
    let proxied = option_env!("STAR_AI_PROXY").is_some_and(|url| !url.trim().is_empty());
    Ok(AiKeyStatus {
        source: if proxied { "proxy" } else { "none" },
        hint: None,
    })
}

#[tauri::command]
pub async fn set_ai_key(state: State<'_, AppState>, key: String) -> Result<AiKeyStatus, AppError> {
    let key = key.trim();
    if key.is_empty() {
        sqlx::query("DELETE FROM settings WHERE key = ?1")
            .bind(KEY_SETTING)
            .execute(&state.db)
            .await?;
    } else {
        sqlx::query(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        )
        .bind(KEY_SETTING)
        .bind(key)
        .execute(&state.db)
        .await?;
    }
    ai_key_status(state).await
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
struct ChatRequest<'a> {
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
                .map(|m| {
                    m.as_str()
                        .map(str::to_string)
                        .unwrap_or_else(|| m.to_string())
                })
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

    let route = route(&state.db).await?;

    let body = ChatRequest {
        model: resolve_model(model.as_deref()),
        messages,
    };

    let request = match &route {
        Route::Direct { key } => http_client()?
            .post(OPENROUTER_URL)
            .header("Authorization", format!("Bearer {key}")),
        Route::Proxy { url } => http_client()?.post(*url),
    };

    let response = request
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| {
            if e.is_timeout() {
                AppError::AiTimeout
            } else {
                AppError::AiRequest
            }
        })?;

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
