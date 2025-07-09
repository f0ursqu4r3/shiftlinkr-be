use actix_web::{test, web, App};
use anyhow::Result;
use chrono::Utc;
use sqlx::SqlitePool;
use std::env;
use tempfile::TempDir;

// Import the modules we need to test
use be::auth::AuthService;
use be::config::Config;
use be::database::init_database;
use be::database::models::*;
use be::database::password_reset_repository::PasswordResetTokenRepository;
use be::database::shift_swap_repository::ShiftSwapRepository;
use be::database::stats_repository::StatsRepository;
use be::database::time_off_repository::TimeOffRepository;
use be::database::user_repository::UserRepository;
use be::handlers;
use be::handlers::admin::ApiResponse;
use be::AppState;

// Test database wrapper
pub struct TestDb {
    pub pool: SqlitePool,
    _temp_dir: TempDir,
}

impl TestDb {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let database_url = format!("sqlite:{}/test.db", temp_dir.path().display());
        let pool = init_database(&database_url).await?;

        Ok(TestDb {
            pool,
            _temp_dir: temp_dir,
        })
    }
}

// Test application wrapper
pub struct TestApp {
    pub db: TestDb,
    pub config: Config,
}

impl TestApp {
    pub async fn new() -> Result<Self> {
        let db = TestDb::new().await?;

        let config = Config {
            database_url: "sqlite::memory:".to_string(),
            jwt_secret: "test-jwt-secret-key-that-is-long-enough".to_string(),
            jwt_expiration_days: 1,
            host: "127.0.0.1".to_string(),
            port: 0,
            environment: "test".to_string(),
        };

        // Create repositories
        let user_repository = UserRepository::new(db.pool.clone());
        let password_reset_repository = PasswordResetTokenRepository::new(db.pool.clone());
        let time_off_repository = TimeOffRepository::new(db.pool.clone());
        let shift_swap_repository = ShiftSwapRepository::new(db.pool.clone());
        let stats_repository = StatsRepository::new(db.pool.clone());
        let auth_service =
            AuthService::new(user_repository, password_reset_repository, config.clone());

        // Create app state and repository data
        let app_state = web::Data::new(AppState { auth_service });
        let time_off_repo_data = web::Data::new(time_off_repository);
        let shift_swap_repo_data = web::Data::new(shift_swap_repository);
        let stats_repo_data = web::Data::new(stats_repository);
        let config_data = web::Data::new(config.clone());

        let app = test::init_service(
            App::new()
                .app_data(app_state)
                .app_data(time_off_repo_data)
                .app_data(shift_swap_repo_data)
                .app_data(stats_repo_data)
                .app_data(config_data)
                .service(
                    web::scope("/api/v1")
                        .service(
                            web::scope("/time-off")
                                .route(
                                    "",
                                    web::post().to(handlers::time_off::create_time_off_request),
                                )
                                .route("", web::get().to(handlers::time_off::get_time_off_requests))
                                .route(
                                    "/{id}",
                                    web::get().to(handlers::time_off::get_time_off_request),
                                )
                                .route(
                                    "/{id}",
                                    web::put().to(handlers::time_off::update_time_off_request),
                                )
                                .route(
                                    "/{id}",
                                    web::delete().to(handlers::time_off::delete_time_off_request),
                                )
                                .route(
                                    "/{id}/approve",
                                    web::post().to(handlers::time_off::approve_time_off_request),
                                )
                                .route(
                                    "/{id}/deny",
                                    web::post().to(handlers::time_off::deny_time_off_request),
                                ),
                        )
                        .service(
                            web::scope("/swaps")
                                .route("", web::post().to(handlers::swaps::create_swap_request))
                                .route("", web::get().to(handlers::swaps::get_swap_requests))
                                .route("/{id}", web::get().to(handlers::swaps::get_swap_request))
                                .route(
                                    "/{id}/respond",
                                    web::post().to(handlers::swaps::respond_to_swap),
                                )
                                .route(
                                    "/{id}/approve",
                                    web::post().to(handlers::swaps::approve_swap_request),
                                )
                                .route(
                                    "/{id}/deny",
                                    web::post().to(handlers::swaps::deny_swap_request),
                                ),
                        )
                        .service(
                            web::scope("/stats")
                                .route(
                                    "/dashboard",
                                    web::get().to(handlers::stats::get_dashboard_stats),
                                )
                                .route("/shifts", web::get().to(handlers::stats::get_shift_stats))
                                .route(
                                    "/time-off",
                                    web::get().to(handlers::stats::get_time_off_stats),
                                ),
                        ),
                ),
        )
        .await;

        Ok(TestApp { db, config })
    }
}

// Mock data generators
pub struct MockData;

impl MockData {
    pub fn user() -> CreateUserRequest {
        CreateUserRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test User".to_string(),
            role: Some(UserRole::Employee),
        }
    }

    pub fn manager() -> CreateUserRequest {
        CreateUserRequest {
            email: "manager@example.com".to_string(),
            password: "password123".to_string(),
            name: "Test Manager".to_string(),
            role: Some(UserRole::Manager),
        }
    }

    pub fn time_off_request(user_id: String) -> TimeOffRequestInput {
        let start_date = Utc::now().naive_utc() + chrono::Duration::days(7);
        let end_date = start_date + chrono::Duration::days(3);

        TimeOffRequestInput {
            user_id,
            start_date,
            end_date,
            reason: "Vacation".to_string(),
            request_type: TimeOffType::Vacation,
        }
    }
}

// Authentication helpers
pub struct AuthHelper;

impl AuthHelper {
    pub fn create_test_token(user: &User, config: &Config) -> Result<String> {
        let claims = be::auth::Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            role: user.role.to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::days(config.jwt_expiration_days))
                .timestamp() as usize,
        };

        // For now return a dummy token - we'll need to implement proper JWT creation
        Ok("dummy_jwt_token".to_string())
    }

    pub fn auth_header(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {}", token))
    }
}

// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    pub fn assert_success_response<T>(body: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let response: ApiResponse<T> =
            serde_json::from_str(body).expect("Failed to parse JSON response");

        assert!(
            response.success,
            "Expected successful response but got error: {:?}",
            response.message
        );
        response.data.expect("Expected data in successful response")
    }

    pub async fn assert_record_count(pool: &SqlitePool, table: &str, expected_count: i64) {
        let query = format!("SELECT COUNT(*) as count FROM {}", table);
        let result = sqlx::query_scalar::<_, i64>(&query)
            .fetch_one(pool)
            .await
            .expect("Failed to count records");

        assert_eq!(
            result, expected_count,
            "Expected {} records in {} table, but found {}",
            expected_count, table, result
        );
    }
}

// Helper functions
pub async fn create_test_user(pool: &SqlitePool, user_data: &CreateUserRequest) -> User {
    // Create a user directly in the database for testing
    // This bypasses the normal registration flow
    let user_id = uuid::Uuid::new_v4().to_string();
    let password_hash = bcrypt::hash(&user_data.password, 12).expect("Failed to hash password");
    let role_str = user_data
        .role
        .as_ref()
        .map(|r| r.to_string())
        .unwrap_or_else(|| "employee".to_string());
    let now = chrono::Utc::now().naive_utc();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, password_hash, name, role, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        user_id,
        user_data.email,
        password_hash,
        user_data.name,
        role_str,
        now,
        now
    )
    .execute(pool)
    .await
    .expect("Failed to insert test user");

    User {
        id: user_id,
        email: user_data.email.clone(),
        password_hash,
        name: user_data.name.clone(),
        role: user_data.role.clone().unwrap_or(UserRole::Employee),
        created_at: now,
        updated_at: now,
    }
}

pub async fn create_test_time_off_request(pool: &SqlitePool, user_id: &str) -> TimeOffRequest {
    let time_off_repo = TimeOffRepository::new(pool.clone());
    let request_data = MockData::time_off_request(user_id.to_string());
    time_off_repo
        .create_request(request_data)
        .await
        .expect("Failed to create test time off request")
}

#[allow(dead_code)]
pub struct TestContext {
    pub pool: SqlitePool,
    pub config: Config,
    pub temp_dir: TempDir,
    pub auth_service: AuthService,
}

impl TestContext {
    #[allow(dead_code)]
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
            temp_dir,
            auth_service,
        })
    }
}

pub fn setup_test_env() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    let _ = env_logger::builder().is_test(true).try_init();
}
