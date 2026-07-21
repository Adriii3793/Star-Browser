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
    AppDir,

    #[error("tauri error: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("url non valido")]
    InvalidUrl,

    #[error("finestra principale non trovata")]
    WindowNotFound,
}
impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}