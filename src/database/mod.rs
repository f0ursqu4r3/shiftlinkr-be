use anyhow::Result;
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::OnceCell;

pub mod models;
pub mod repositories;
pub mod transaction;
pub mod utils;

pub type DbTransaction<'a> = Transaction<'a, Postgres>;
pub type DbPool = PgPool;
pub type DbExecutor<'a> = &'a mut DbTransaction<'a>;

// Use Arc<PgPool> wrapped in OnceCell for better lifetime management
static DB_POOL: OnceCell<Arc<PgPool>> = OnceCell::const_new();

pub async fn init_database(database_url: &str, run_migrations: bool) -> Result<PgPool> {
    // Create connection pool with sensible defaults for tests and dev
    let new_pool = PgPoolOptions::new()
        .max_connections(20)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(database_url)
        .await?;

    // Run migrations
    if run_migrations {
        println!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&new_pool).await?;
        println!("Migrations completed successfully");
    }

    let pool_arc = Arc::new(new_pool.clone());

    DB_POOL
        .set(pool_arc)
        .map_err(|_| anyhow::anyhow!("DB_POOL already initialized"))?;

    Ok(new_pool)
}

pub async fn init_database_for_tests(database_url: &str, run_migrations: bool) -> Result<PgPool> {
    // For tests, try to return existing pool if already set
    if let Some(existing_pool) = DB_POOL.get() {
        return Ok((**existing_pool).clone());
    }

    // Otherwise initialize normally
    init_database(database_url, run_migrations).await
}

// Return an owned PgPool - no lifetime issues
pub async fn get_pool() -> PgPool {
    let pool_arc = DB_POOL
        .get_or_init(|| async {
            panic!("Database pool not initialized. Call init_database first.")
        })
        .await;

    (**pool_arc).clone()
}

// Synchronous version for cases where you know the pool is initialized
pub fn get_pool_sync() -> PgPool {
    let pool_arc = DB_POOL
        .get()
        .expect("DB_POOL not initialized. Call init_database first.");

    (**pool_arc).clone()
}
