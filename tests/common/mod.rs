#![allow(dead_code)]

use actix_web::{http::StatusCode, test, web};
use anyhow::Result;
use be::config::Config;
use be::database::models::{AddEmployeeToCompanyInput, CompanyRole, CreateCompanyInput, User};
use be::database::repositories::{company as company_repo, user as user_repo};
use be::database::transaction::DatabaseTransaction;
use be::services::auth;
use chrono::Utc;
use sqlx::PgPool;
use std::env;
use std::sync::OnceLock;
use uuid::Uuid;

pub struct TestContext {
    pub pool: PgPool,
    pub config: Config,
}

impl TestContext {
    pub async fn new() -> Result<Self> {
        // Use test database URL from environment or default
        let database_url = env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/shiftlinkr_test".to_string());

        // Ensure global Config is initialized for handlers/services using config()
        // Map TEST_DATABASE_URL -> DATABASE_URL for the app config
        unsafe {
            env::set_var("DATABASE_URL", &database_url);
            env::set_var("RUN_MIGRATIONS", "true");
            env::set_var("JWT_SECRET", "test-jwt-secret-key");
            env::set_var("JWT_EXPIRATION_DAYS", "1");
            env::set_var("ENVIRONMENT", "test");
            env::set_var("BASE_URL", "http://localhost:3000");
            env::set_var("SKIP_ACTIVITY_LOG", "1");
        }

        // Initialize Config only once across tests
        static TEST_CONFIG_ONCE: OnceLock<()> = OnceLock::new();
        TEST_CONFIG_ONCE.get_or_init(|| {
            be::config::Config::from_env_only().expect("Failed to init test config");
        });
        let config = be::config::config().clone();

        // Initialize or reuse the global pool for tests (idempotent)
        let pool = be::database::init_database_for_tests(&database_url, true).await?;

        // Clean up any existing test data
        Self::cleanup_test_data(&pool).await?;

        Ok(TestContext { pool, config })
    }

    async fn cleanup_test_data(pool: &PgPool) -> Result<()> {
        // Fast cleanup between tests using TRUNCATE ... CASCADE
        let cleanup_sql = r#"
            TRUNCATE TABLE
                shift_claims,
                shift_swaps,
                time_off_requests,
                shifts,
                team_members,
                teams,
                locations,
                pto_balance_history,
                invite_tokens,
                company_activities,
                user_company,
                companies,
                password_reset_tokens,
                users
            RESTART IDENTITY CASCADE
        "#;
        let _ = sqlx::query(cleanup_sql).execute(pool).await;
        Ok(())
    }
}

pub fn setup_test_env() {
    unsafe {
        env::set_var("RUST_LOG", "warn");
    }
    let _ = env_logger::builder().is_test(true).try_init();
}

/// Create a simple configuration data for tests
pub async fn create_test_config() -> web::Data<Config> {
    let config = Config {
        database_url: env::var("TEST_DATABASE_URL")
            .unwrap_or_else(|_| "postgres://localhost:5432/shiftlinkr_test".to_string()),
        run_migrations: true,
        jwt_secret: "test-jwt-secret-key".to_string(),
        jwt_expiration_days: 1,
        host: "127.0.0.1".to_string(),
        port: 0,
        environment: "test".to_string(),
        client_base_url: "http://localhost:3000".to_string(),
    };
    web::Data::new(config)
}

/// Create a test user and return their ID and token
pub async fn create_test_user_with_token(
    email: &str,
    password: &str,
    name: &str,
) -> Result<(Uuid, String, Uuid)> {
    // Move owned strings into the transaction closure
    let email_s = email.to_string();
    let password_s = password.to_string();
    let name_s = name.to_string();

    let (user, company_id) = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Create user
            let user = User {
                id: Uuid::new_v4(),
                name: name_s,
                email: email_s.clone(),
                password_hash: bcrypt::hash(password_s, 4).unwrap(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let user = user_repo::create_user(tx, &user).await?;

            // Create a company
            let company = company_repo::create_company(
                tx,
                &CreateCompanyInput {
                    name: format!("Test Company for {}", email_s),
                    description: Some("Test company".to_string()),
                    website: None,
                    phone: None,
                    email: None,
                    address: None,
                    logo_url: None,
                    timezone: None,
                },
            )
            .await?;

            // Add user to company as employee (primary)
            let _emp = company_repo::add_employee_to_company(
                tx,
                company.id,
                &AddEmployeeToCompanyInput {
                    user_id: user.id,
                    role: Some(CompanyRole::Employee),
                    is_primary: Some(true),
                    hire_date: None,
                },
            )
            .await?;

            Ok::<_, be::error::AppError>((user, company.id))
        })
    })
    .await?;

    let token = auth::generate_company_token(user.id, company_id).await?;

    Ok((user.id, token, company_id))
}

/// Create a test company and return its ID
pub async fn create_test_company(name: &str, user_id: Uuid) -> Result<Uuid> {
    let name_s = name.to_string();
    let company_id = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let company = company_repo::create_company(
                tx,
                &CreateCompanyInput {
                    name: name_s,
                    description: Some("Test company".to_string()),
                    website: None,
                    phone: None,
                    email: None,
                    address: None,
                    logo_url: None,
                    timezone: None,
                },
            )
            .await?;

            // Add the user as an admin
            let add_employee_input = AddEmployeeToCompanyInput {
                user_id,
                role: Some(CompanyRole::Admin),
                is_primary: Some(true),
                hire_date: None,
            };

            let _ =
                company_repo::add_employee_to_company(tx, company.id, &add_employee_input).await?;

            Ok::<_, be::error::AppError>(company.id)
        })
    })
    .await?;

    Ok(company_id)
}

/// Make a user an admin of a company
pub async fn make_user_admin_of_company(user_id: Uuid, company_id: Uuid) -> Result<()> {
    DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            // Try to update existing role first
            if company_repo::update_employee_role(tx, company_id, user_id, &CompanyRole::Admin)
                .await?
                .is_none()
            {
                // No existing membership; add as non-primary admin
                let add_employee_input = AddEmployeeToCompanyInput {
                    user_id,
                    role: Some(CompanyRole::Admin),
                    is_primary: Some(false),
                    hire_date: None,
                };
                let _ = company_repo::add_employee_to_company(tx, company_id, &add_employee_input)
                    .await?;
            }
            Ok::<_, be::error::AppError>(())
        })
    })
    .await?;
    Ok(())
}

/// Create a test user with a default company and return user_id, company_id, and token
pub async fn create_user_with_company(
    email: &str,
    password: &str,
    name: &str,
    company_name: &str,
) -> Result<(Uuid, Uuid, String)> {
    let (user_id, _token_existing, _) = create_test_user_with_token(email, password, name).await?;
    let company_id = create_test_company(company_name, user_id).await?;
    let token = auth::generate_company_token(user_id, company_id).await?;

    Ok((user_id, company_id, token))
}

/// Helper to create a default test company with ID
pub async fn create_default_test_company() -> Result<Uuid> {
    let company_id = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let company = company_repo::create_company(
                tx,
                &CreateCompanyInput {
                    name: "Test Company".to_string(),
                    description: Some("Default test company".to_string()),
                    website: None,
                    phone: None,
                    email: None,
                    address: None,
                    logo_url: None,
                    timezone: None,
                },
            )
            .await?;
            Ok::<_, be::error::AppError>(company.id)
        })
    })
    .await?;
    Ok(company_id)
}

/// Helper to create a test user with the given email and make them admin of the default company
pub async fn create_admin_user_str(email: &str) -> Result<Uuid> {
    let email_s = email.to_string();
    let user_id = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            let user = User {
                id: Uuid::new_v4(),
                name: "Admin User".to_string(),
                email: email_s,
                password_hash: bcrypt::hash("password", 4).unwrap(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            let user = user_repo::create_user(tx, &user).await?;
            Ok::<_, be::error::AppError>(user.id)
        })
    })
    .await?;
    Ok(user_id)
}

/// Helper to make a user admin of the default company - updated signature
pub async fn make_user_admin_of_default_company_str(user_id: Uuid, company_id: Uuid) -> Result<()> {
    make_user_admin_of_company(user_id, company_id).await
}

/// Assert response status and extract JSON body
pub async fn assert_response_ok(response: actix_web::dev::ServiceResponse) -> serde_json::Value {
    assert_eq!(response.status(), StatusCode::OK);
    let body = test::read_body(response).await;
    serde_json::from_slice(&body).expect("Failed to parse response body as JSON")
}

/// Assert response has specific status code and extract JSON body
pub async fn assert_response_status(
    response: actix_web::dev::ServiceResponse,
    expected_status: StatusCode,
) -> serde_json::Value {
    assert_eq!(response.status(), expected_status);
    let body = test::read_body(response).await;
    serde_json::from_slice(&body).expect("Failed to parse response body as JSON")
}

/// Create a sample auth header with bearer token
pub fn create_auth_header(token: &str) -> (&'static str, String) {
    ("Authorization", format!("Bearer {}", token))
}

pub fn assert_response_unauthorized(response: actix_web::dev::ServiceResponse) {
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

pub fn assert_response_forbidden(response: actix_web::dev::ServiceResponse) {
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

pub fn assert_response_not_found(response: actix_web::dev::ServiceResponse) {
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

pub fn assert_response_bad_request(response: actix_web::dev::ServiceResponse) {
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
