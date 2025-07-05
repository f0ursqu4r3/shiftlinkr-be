use sqlx::SqlitePool;
use tempfile::TempDir;
use anyhow::Result;
use std::env;

// Import the modules we need to test
use be::config::Config;
use be::database::init_database;
use be::database::user_repository::UserRepository;
use be::auth::AuthService;

pub struct TestContext {
    pub pool: SqlitePool,
    pub config: Config,
    pub temp_dir: TempDir,
    pub auth_service: AuthService,
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
        let auth_service = AuthService::new(user_repository, config.clone());

        Ok(TestContext {
            pool,
            config,
            temp_dir,
            auth_service,
        })
    }
}

pub fn setup_test_env() {
    unsafe { env::set_var("RUST_LOG", "debug"); }
    let _ = env_logger::builder().is_test(true).try_init();
}
