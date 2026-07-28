use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),

    #[error("could not resolve the application data directory")]
    #[allow(dead_code)]
    AppDir,

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("url not valid")]
    InvalidUrl,

    #[error("ftab not found")]
    WindowNotFound,

    #[error("API key not configured ")]
    MissingApiKey,
    #[error("daily AI request limit reached")]
    RateLimited,

    #[error("error in the AI request")]
    AiRequest,

    #[error("the AI took too long to respond. Please try again")]
    AiTimeout,

    #[error("could not read this page")]
    PageFetch,

    #[error("could not save your setup")]
    SetupSerialize,

    #[error("could not open the default-apps settings")]
    DefaultBrowser,
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}