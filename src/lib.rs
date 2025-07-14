pub mod config;
pub mod database;
pub mod handlers;
pub mod middleware;
pub mod services;

pub use config::Config;
pub use database::repositories::{ActivityRepository, CompanyRepository};
pub use services::{ActivityLogger, AuthService};

pub struct AppState {
    pub auth_service: AuthService,
    pub company_repository: CompanyRepository,
    pub activity_repository: ActivityRepository,
    pub activity_logger: ActivityLogger,
}
