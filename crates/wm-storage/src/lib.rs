use anyhow::{Context, Result};
use sqlx::{Any, AnyPool, any::AnyConnectOptions, ConnectOptions, migrate::Migrator};
use std::str::FromStr;
use std::path::Path;

static MIGRATOR: Migrator = Migrator::new(std::path::Path::new("./crates/wm-storage/migrations")).const_new();

pub async fn new_pool(database_url: &str) -> Result<AnyPool> {
    let mut opts = AnyConnectOptions::from_str(database_url)
        .with_context(|| format!("invalid DATABASE_URL: {}", database_url))?;
    // Reduce noisy logs by default
    opts.log_statements(log::LevelFilter::Off);
    let pool = AnyPool::connect_with(opts).await?;
    Ok(pool)
}

pub async fn migrate(pool: &AnyPool) -> Result<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

// Helpers for tests or offline prepare paths
pub fn migrations_dir() -> &'static Path {
    Path::new("./crates/wm-storage/migrations")
}