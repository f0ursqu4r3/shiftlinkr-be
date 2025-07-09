use anyhow::Result;
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePool, Sqlite};

pub mod invite_repository;
pub mod location_repository;
pub mod models;
pub mod password_reset_repository;
pub mod pto_balance_repository;
pub mod shift_claim_repository;
pub mod shift_repository;
pub mod shift_swap_repository;
pub mod stats_repository;
pub mod time_off_repository;
pub mod types;
pub mod user_repository;

pub async fn init_database(database_url: &str) -> Result<SqlitePool> {
    // Create database if it doesn't exist
    if !Sqlite::database_exists(database_url).await.unwrap_or(false) {
        println!("Creating database {}", database_url);
        match Sqlite::create_database(database_url).await {
            Ok(_) => println!("Database created successfully"),
            Err(error) => panic!("Error creating database: {}", error),
        }
    } else {
        println!("Database already exists");
    }

    // Create connection pool
    let pool = SqlitePool::connect(database_url).await?;

    // Run migrations
    println!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    println!("Migrations completed successfully");

    Ok(pool)
}
