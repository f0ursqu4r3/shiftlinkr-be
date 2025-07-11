#![allow(dead_code)]

use actix_web::web;
use anyhow::Result;
use be::auth::AuthService;
use be::config::Config;
use be::database::init_database;
use be::database::repositories::location_repository::LocationRepository;
use be::database::repositories::password_reset_repository::PasswordResetTokenRepository;
use be::database::repositories::user_repository::UserRepository;
use be::AppState;
use sqlx::SqlitePool;
use std::env;
use tempfile::TempDir;

pub struct TestContext {
    pub pool: SqlitePool,
    pub config: Config,
    pub auth_service: AuthService,
    pub _temp_dir: TempDir, // Keep for cleanup but don't expose
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let database_url = format!("sqlite:{}/test.db", temp_dir.path().display());

        let config = Config {
            database_url: database_url.clone(),
            jwt_secret: "test-jwt-secret-key".to_string(),
            jwt_expiration_days: 1,
            host: "127.0.0.1".to_string(),
            port: 0,
            environment: "test".to_string(),
        };

        let pool = init_database(&database_url).await?;
        let user_repository = UserRepository::new(pool.clone());
        let password_reset_repository = PasswordResetTokenRepository::new(pool.clone());
        let auth_service =
            AuthService::new(user_repository, password_reset_repository, config.clone());

        Ok(TestContext {
            pool,
            config,
            auth_service,
            _temp_dir: temp_dir,
        })
    }
}

pub fn setup_test_env() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    let _ = env_logger::builder().is_test(true).try_init();
}

pub async fn create_admin_app_data() -> (
    web::Data<AppState>,
    web::Data<LocationRepository>,
    web::Data<Config>,
    TestContext,
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let location_repo_data = web::Data::new(LocationRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (app_state, location_repo_data, config_data, ctx)
}
