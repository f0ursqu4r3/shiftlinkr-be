use anyhow::Result;
use sqlx::PgPool;
use std::sync::OnceLock;

pub mod models;
pub mod repositories;
pub mod utils;

static DB_POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn init_database(database_url: &str, run_migrations: bool) -> Result<PgPool> {
    // Create connection pool
    let new_pool = PgPool::connect(database_url).await?;

    // Run migrations
    if run_migrations {
        println!("Running database migrations...");
        sqlx::migrate!("./migrations").run(&new_pool).await?;
        println!("Migrations completed successfully");
    }

    DB_POOL
        .set(new_pool.clone())
        .expect("DB_POOL can only be set once");

    Ok(new_pool)
}

pub fn pool() -> &'static PgPool {
    DB_POOL.get().expect("DB_POOL not initialized")
}
