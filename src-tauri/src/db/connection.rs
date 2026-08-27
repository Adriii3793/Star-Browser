use std::path::Path;
use sqlx::sqlite::{
    SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous,
};
use sqlx::SqlitePool;

use crate::error::AppError;

pub async fn init(path: &Path) -> Result<SqlitePool, AppError> {
    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
    .max_connections(4)
    .connect_with(options)
    .await?;

    sqlx::migrate!("./src/db/migrations").run(&pool).await?;

    Ok(pool)
    
}
#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::migrate::Migrator;

    fn temp_db() -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("star-migrate-test-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn migrations_apply_cleanly_and_create_every_table_the_app_queries() {
        let path = temp_db();
        let pool = tauri::async_runtime::block_on(init(&path)).expect("migrations should apply");

        for table in ["settings", "history", "usage_log"] {
            let found: Option<(String,)> = tauri::async_runtime::block_on(
                sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' AND name = ?1")
                    .bind(table)
                    .fetch_optional(&pool),
            )
            .expect("query should succeed");
            assert!(found.is_some(), "table `{table}` is missing after migration");
        }

        tauri::async_runtime::block_on(pool.close());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn removing_an_applied_migration_stops_the_database_opening() {
        let mut path = std::env::temp_dir();
        path.push(format!("star-migrate-missing-{}.db", std::process::id()));
        let _ = std::fs::remove_file(&path);

        let pool = tauri::async_runtime::block_on(init(&path)).expect("first run should apply");
        tauri::async_runtime::block_on(pool.close());

        let full = sqlx::migrate!("./src/db/migrations");
        let without_003 = Migrator {
            migrations: std::borrow::Cow::Owned(
                full.iter().filter(|m| m.version != 3).cloned().collect(),
            ),
            ignore_missing: full.ignore_missing,
            locking: full.locking,
            no_tx: full.no_tx,
        };

        let result = tauri::async_runtime::block_on(async {
            let pool = sqlx::SqlitePool::connect(&format!("sqlite://{}", path.display())).await?;
            let outcome = without_003.run(&pool).await;
            pool.close().await;
            Ok::<_, sqlx::Error>(outcome)
        })
        .expect("connecting should work");

        match result {
            Err(sqlx::migrate::MigrateError::VersionMissing(3)) => {}
            other => panic!(
                "expected VersionMissing(3) when 003_usage.sql is absent, got {other:?}"
            ),
        }
        let _ = std::fs::remove_file(&path);
    }
}
