use anyhow::Result;
use sqlx::PgPool;

pub mod models;
pub mod repositories;
pub mod utils;

pub async fn init_database(database_url: &str) -> Result<PgPool> {
    // Create connection pool
    let pool = PgPool::connect(database_url).await?;

    // Run migrations
    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("Migrations completed successfully");

    Ok(pool)
}
