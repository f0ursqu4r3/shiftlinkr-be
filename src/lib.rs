pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;

#[cfg(test)]
pub mod test_utils;

pub use auth::AuthService;
pub use config::Config;

pub struct AppState {
    pub auth_service: AuthService,
}
