pub mod config;
pub mod database;
pub mod error;
pub mod handlers;
pub mod middleware;
pub mod routes;
pub mod services;

// Re-export commonly used types
pub use config::Config;
pub use services::AuthService;

// Re-export database types
pub use database::{init_database, repositories};

// Re-export middleware
pub use middleware::*;

// Re-export services
pub use services::*;
