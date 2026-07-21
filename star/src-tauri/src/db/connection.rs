use std::path::Path;
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn init(path: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
    .filename(path)
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal)
    .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
    .max_connections(4)
    .connect_with(options)
    .await?;

    sqlx::migrate!("./src/db/migrations").run(&pool).await?;

    Ok(pool)
    
}