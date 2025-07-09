use actix_web::test;
use actix_web::web;
use actix_web::App;
use anyhow::Result;
use chrono::{NaiveDateTime, Utc};
use fake::faker::chrono::en::*;
use fake::faker::internet::en::*;
use fake::faker::name::en::*;
use fake::Fake;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::SqlitePool;
use tempfile::NamedTempFile;
use uuid::Uuid;

use crate::auth::AuthService;
use crate::config::Config;
use crate::database::invite_repository::InviteRepository;
use crate::database::location_repository::LocationRepository;
use crate::database::models::*;
use crate::database::password_reset_repository::PasswordResetTokenRepository;
use crate::database::shift_repository::ShiftRepository;
use crate::database::user_repository::UserRepository;
use crate::database::*;
use crate::handlers;
use crate::AppState;

/// Test database wrapper that provides isolated testing environment
pub struct TestDb {
    pub pool: SqlitePool,
    _temp_file: NamedTempFile,
}

impl TestDb {
    /// Create a new test database with fresh schema
    pub async fn new() -> Result<Self> {
        let temp_file = NamedTempFile::new()?;
        let database_url = format!("sqlite:{}", temp_file.path().display());

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        // Run migrations
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(TestDb {
            pool,
            _temp_file: temp_file,
        })
    }

    /// Get a reference to the database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

/// Test application factory that creates a configured Actix app for testing
pub struct TestApp {
    pub db: TestDb,
    pub config: Config,
}

impl TestApp {
    /// Create a new test application instance
    pub async fn new() -> Result<Self> {
        let db = TestDb::new().await?;
        let config = Config::test_config()?;

        Ok(TestApp { db, config })
    }

    /// Create an Actix web app configured for testing
    pub async fn create_app(
        &self,
    ) -> App<
        impl actix_web::dev::ServiceFactory<
            actix_web::dev::ServiceRequest,
            Config = (),
            Response = actix_web::dev::ServiceResponse,
            Error = actix_web::Error,
            InitError = (),
        >,
    > {
        // Initialize repositories
        let user_repository = UserRepository::new(self.db.pool.clone());
        let location_repository = LocationRepository::new(self.db.pool.clone());
        let shift_repository = ShiftRepository::new(self.db.pool.clone());
        let password_reset_repository = PasswordResetTokenRepository::new(self.db.pool.clone());
        let invite_repository = InviteRepository::new(self.db.pool.clone());
        let time_off_repository =
            crate::database::time_off_repository::TimeOffRepository::new(self.db.pool.clone());
        let swap_repository =
            crate::database::shift_swap_repository::ShiftSwapRepository::new(self.db.pool.clone());
        let stats_repository =
            crate::database::stats_repository::StatsRepository::new(self.db.pool.clone());
        let auth_service = AuthService::new(
            user_repository,
            password_reset_repository,
            self.config.clone(),
        );

        // Create app state and repository data
        let app_state = web::Data::new(AppState { auth_service });
        let location_repo_data = web::Data::new(location_repository);
        let shift_repo_data = web::Data::new(shift_repository);
        let invite_repo_data = web::Data::new(invite_repository);
        let time_off_repo_data = web::Data::new(time_off_repository);
        let swap_repo_data = web::Data::new(swap_repository);
        let stats_repo_data = web::Data::new(stats_repository);
        let config_data = web::Data::new(self.config.clone());

        App::new()
            .app_data(app_state)
            .app_data(location_repo_data)
            .app_data(shift_repo_data)
            .app_data(invite_repo_data)
            .app_data(time_off_repo_data)
            .app_data(swap_repo_data)
            .app_data(stats_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/me", web::get().to(handlers::auth::me))
                            .route(
                                "/forgot-password",
                                web::post().to(handlers::auth::forgot_password),
                            )
                            .route(
                                "/reset-password",
                                web::post().to(handlers::auth::reset_password),
                            )
                            .route("/invite", web::post().to(handlers::auth::create_invite))
                            .route("/invite/{token}", web::get().to(handlers::auth::get_invite))
                            .route(
                                "/invite/accept",
                                web::post().to(handlers::auth::accept_invite),
                            )
                            .route("/invites", web::get().to(handlers::auth::get_my_invites)),
                    )
                    .service(
                        web::scope("/admin")
                            .route(
                                "/locations",
                                web::post().to(handlers::admin::create_location),
                            )
                            .route("/locations", web::get().to(handlers::admin::get_locations))
                            .route(
                                "/locations/{id}",
                                web::get().to(handlers::admin::get_location),
                            )
                            .route(
                                "/locations/{id}",
                                web::put().to(handlers::admin::update_location),
                            )
                            .route(
                                "/locations/{id}",
                                web::delete().to(handlers::admin::delete_location),
                            )
                            .route("/teams", web::post().to(handlers::admin::create_team))
                            .route("/teams", web::get().to(handlers::admin::get_teams))
                            .route("/teams/{id}", web::get().to(handlers::admin::get_team))
                            .route("/teams/{id}", web::put().to(handlers::admin::update_team))
                            .route(
                                "/teams/{id}",
                                web::delete().to(handlers::admin::delete_team),
                            )
                            .route(
                                "/teams/{team_id}/members/{user_id}",
                                web::post().to(handlers::admin::add_team_member),
                            )
                            .route(
                                "/teams/{team_id}/members",
                                web::get().to(handlers::admin::get_team_members),
                            )
                            .route(
                                "/teams/{team_id}/members/{user_id}",
                                web::delete().to(handlers::admin::remove_team_member),
                            ),
                    )
                    .service(
                        web::scope("/shifts")
                            .route("", web::post().to(handlers::shifts::create_shift))
                            .route("", web::get().to(handlers::shifts::get_shifts))
                            .route("/{id}", web::get().to(handlers::shifts::get_shift))
                            .route("/{id}", web::put().to(handlers::shifts::update_shift))
                            .route("/{id}", web::delete().to(handlers::shifts::delete_shift))
                            .route(
                                "/{id}/assign",
                                web::post().to(handlers::shifts::assign_shift),
                            )
                            .route(
                                "/{id}/unassign",
                                web::post().to(handlers::shifts::unassign_shift),
                            )
                            .route(
                                "/{id}/status",
                                web::post().to(handlers::shifts::update_shift_status),
                            )
                            .route("/{id}/claim", web::post().to(handlers::shifts::claim_shift)),
                    )
                    .service(
                        web::scope("/time-off")
                            .route(
                                "",
                                web::post().to(handlers::time_off::create_time_off_request),
                            )
                            .route("", web::get().to(handlers::time_off::get_time_off_requests))
                            .route(
                                "/{id}/approve",
                                web::post().to(handlers::time_off::approve_time_off_request),
                            )
                            .route(
                                "/{id}/deny",
                                web::post().to(handlers::time_off::deny_time_off_request),
                            )
                            .route(
                                "/{id}",
                                web::put().to(handlers::time_off::update_time_off_request),
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
            )
    }
}

/// Authentication helper for tests
pub struct AuthHelper;

impl AuthHelper {
    /// Create a test JWT token for a user
    pub fn create_test_token(user: &User, config: &Config) -> Result<String> {
        use crate::auth::Claims;
        use jsonwebtoken::{encode, EncodingKey, Header};

        let claims = Claims {
            sub: user.id.clone(),
            email: user.email.clone(),
            exp: (Utc::now() + chrono::Duration::hours(24)).timestamp() as usize,
            role: user.role.to_string(),
        };

        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(config.jwt_secret.as_ref()),
        )
        .map_err(|e| anyhow::anyhow!("Failed to create test token: {}", e))
    }

    /// Create authorization header for requests
    pub fn auth_header(token: &str) -> (&'static str, String) {
        ("Authorization", format!("Bearer {}", token))
    }
}

/// Mock data generators using the fake crate
pub struct MockData;

impl MockData {
    /// Generate a random user for testing
    pub fn user() -> CreateUserRequest {
        CreateUserRequest {
            email: SafeEmail().fake(),
            password: "Test123!".to_string(), // Use consistent password for tests
            name: Name().fake(),
            role: Some(UserRole::Employee),
        }
    }

    /// Generate a random admin user
    pub fn admin_user() -> CreateUserRequest {
        CreateUserRequest {
            email: SafeEmail().fake(),
            password: "Test123!".to_string(),
            name: Name().fake(),
            role: Some(UserRole::Admin),
        }
    }

    /// Generate a random manager user
    pub fn manager_user() -> CreateUserRequest {
        CreateUserRequest {
            email: SafeEmail().fake(),
            password: "Test123!".to_string(),
            name: Name().fake(),
            role: Some(UserRole::Manager),
        }
    }

    /// Generate a random location
    pub fn location() -> LocationInput {
        LocationInput {
            name: format!("{} Office", Name().fake::<String>()),
            address: Some(format!("{} Main St", (1..9999).fake::<i32>())),
            phone: Some("+1234567890".to_string()),
            email: Some("test@location.com".to_string()),
        }
    }

    /// Generate a random team
    pub fn team(location_id: i64) -> TeamInput {
        TeamInput {
            name: format!("{} Team", Name().fake::<String>()),
            location_id,
            description: Some("Test team description".to_string()),
        }
    }

    /// Generate a random shift
    pub fn shift(location_id: i64, team_id: Option<i64>) -> ShiftInput {
        let start_time: NaiveDateTime = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();

        let end_time = start_time + chrono::Duration::hours(8);

        ShiftInput {
            title: "Test Shift".to_string(),
            description: Some("Test shift description".to_string()),
            location_id,
            team_id,
            assigned_user_id: None,
            start_time,
            end_time,
            hourly_rate: Some((15.0..50.0).fake()),
        }
    }

    /// Generate random time-off request data
    pub fn time_off_request(user_id: String) -> TimeOffRequestInput {
        let start_date = chrono::NaiveDate::from_ymd_opt(2025, 8, 1)
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap();

        let end_date = start_date + chrono::Duration::days((1..14).fake());

        TimeOffRequestInput {
            user_id,
            start_date,
            end_date,
            reason: "Test vacation request".to_string(),
            request_type: TimeOffType::Vacation,
        }
    }

    /// Generate random shift swap data
    pub fn shift_swap(original_shift_id: i64, requesting_user_id: String) -> ShiftSwapInput {
        ShiftSwapInput {
            original_shift_id,
            requesting_user_id,
            target_user_id: None,
            target_shift_id: None,
            notes: Some("Test swap request".to_string()),
            swap_type: ShiftSwapType::Open,
        }
    }
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that response is successful and contains expected data
    pub fn assert_success_response<T>(body: &str) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        let response: serde_json::Value =
            serde_json::from_str(body).expect("Response should be valid JSON");

        assert_eq!(response["success"], true, "Response should be successful");

        serde_json::from_value(response["data"].clone())
            .expect("Response data should deserialize correctly")
    }

    /// Assert that response is an error with expected message
    pub fn assert_error_response(body: &str, _expected_status: u16) {
        let response: serde_json::Value =
            serde_json::from_str(body).expect("Response should be valid JSON");

        assert_eq!(response["success"], false, "Response should be an error");
        assert!(response["message"].is_string(), "Error should have message");
    }

    /// Assert that database record exists
    pub async fn assert_record_exists<T>(pool: &SqlitePool, table: &str, id: &str)
    where
        T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
    {
        let query = format!("SELECT * FROM {} WHERE id = ?", table);
        let result: Option<T> = sqlx::query_as(&query)
            .bind(id)
            .fetch_optional(pool)
            .await
            .expect("Database query should succeed");

        assert!(result.is_some(), "Record should exist in database");
    }

    /// Assert that database record count matches expected
    pub async fn assert_record_count(pool: &SqlitePool, table: &str, expected_count: i64) {
        let query = format!("SELECT COUNT(*) as count FROM {}", table);
        let result: (i64,) = sqlx::query_as(&query)
            .fetch_one(pool)
            .await
            .expect("Count query should succeed");

        assert_eq!(
            result.0, expected_count,
            "Record count should match expected"
        );
    }
}

/// Configuration extension for testing
impl Config {
    /// Create a test configuration
    pub fn test_config() -> Result<Self> {
        Ok(Config {
            environment: "test".to_string(),
            database_url: ":memory:".to_string(), // Will be overridden by TestDb
            jwt_secret: "test_jwt_secret_key_for_testing_only".to_string(),
            jwt_expiration_days: 30,
            host: "127.0.0.1".to_string(),
            port: 0, // Let OS choose available port
        })
    }
}
