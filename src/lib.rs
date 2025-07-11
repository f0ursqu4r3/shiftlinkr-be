pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;

pub use auth::AuthService;
pub use config::Config;
use database::repositories::CompanyRepository;

pub struct AppState {
    pub auth_service: AuthService,
    pub company_repository: CompanyRepository,
}
