use anyhow::Result;
use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub async fn new_pool(database_url: &str) -> Result<SqlitePool> {
    // Use SQLite directly (simpler and primary database per constraints)
    let opts = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true);
    let pool = SqlitePool::connect_with(opts).await?;
    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    // Run migrations using the macro
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::sqlite::SqlitePoolOptions;

    #[tokio::test]
    async fn test_migrate_sqlite() -> Result<()> {
        // Use sqlite pool directly for in-memory testing
        let pool = SqlitePoolOptions::new()
            .connect("sqlite::memory:")
            .await?;

        // Run migrations directly on sqlite pool
        sqlx::migrate!("./migrations").run(&pool).await?;

        // Verify app_boot table exists
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM app_boot")
            .fetch_one(&pool)
            .await?;
        assert_eq!(count, 0);

        Ok(())
    }
}