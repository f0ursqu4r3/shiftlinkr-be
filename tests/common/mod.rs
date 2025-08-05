#![allow(dead_code)]

use actix_web::{http::StatusCode, test, web};
use anyhow::Result;
use be::config::Config;
use be::database::init_database;
use be::database::models::{AddEmployeeToCompanyInput, CompanyRole, CreateCompanyInput, User};
use be::database::repositories::{company as company_repo, user as user_repo};
use be::services::auth;
use chrono::Utc;
use sqlx::PgPool;
use std::env;
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

        let config = Config {
            database_url: database_url.clone(),
            run_migrations: true,
            jwt_secret: "test-jwt-secret-key".to_string(),
            jwt_expiration_days: 1,
            host: "127.0.0.1".to_string(),
            port: 0,
            environment: "test".to_string(),
            client_base_url: "http://localhost:3000".to_string(),
        };

        // Try to initialize the global pool, ignore errors if already set
        let pool = match init_database(&database_url, true).await {
            Ok(p) => p,
            Err(e) => {
                // If initialization fails (probably already set), create a new connection
                // but don't run migrations again
                eprintln!("init_database failed: {}, creating direct connection", e);
                let p = PgPool::connect(&database_url).await?;
                // Run migrations just in case they haven't been run
                let _ = sqlx::migrate!("./migrations").run(&p).await;
                p
            }
        };

        // Clean up any existing test data
        Self::cleanup_test_data(&pool).await?;

        Ok(TestContext { pool, config })
    }

    async fn cleanup_test_data(pool: &PgPool) -> Result<()> {
        // Clean up test data in reverse dependency order, ignoring missing tables
        let _ = sqlx::query("DELETE FROM user_companies")
            .execute(pool)
            .await;
        let _ = sqlx::query("DELETE FROM companies").execute(pool).await;
        let _ = sqlx::query("DELETE FROM users").execute(pool).await;
        Ok(())
    }
}

pub fn setup_test_env() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
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
) -> Result<(Uuid, String)> {
    let user = User {
        id: Uuid::new_v4(),
        name: name.to_string(),
        email: email.to_string(),
        password_hash: bcrypt::hash(password, 4).unwrap(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created_user = user_repo::create_user(&user).await?;
    let token = auth::generate_company_token(created_user.id, Uuid::new_v4()).await?;

    Ok((created_user.id, token))
}

/// Create a test company and return its ID
pub async fn create_test_company(name: &str, user_id: Uuid) -> Result<Uuid> {
    let company_input = CreateCompanyInput {
        name: name.to_string(),
        description: Some("Test company".to_string()),
        website: None,
        phone: None,
        email: None,
        address: None,
        logo_url: None,
        timezone: None,
    };

    let company = company_repo::create_company(&company_input).await?;

    // Add the user as an admin
    let add_employee_input = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(true),
        hire_date: None,
    };

    company_repo::add_employee_to_company(company.id, &add_employee_input).await?;

    Ok(company.id)
}

/// Make a user an admin of a company
pub async fn make_user_admin_of_company(user_id: Uuid, company_id: Uuid) -> Result<()> {
    let add_employee_input = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(false),
        hire_date: None,
    };

    company_repo::add_employee_to_company(company_id, &add_employee_input).await?;
    Ok(())
}

/// Create a test user with a default company and return user_id, company_id, and token
pub async fn create_user_with_company(
    email: &str,
    password: &str,
    name: &str,
    company_name: &str,
) -> Result<(Uuid, Uuid, String)> {
    let (user_id, _) = create_test_user_with_token(email, password, name).await?;
    let company_id = create_test_company(company_name, user_id).await?;
    let token = auth::generate_company_token(user_id, company_id).await?;

    Ok((user_id, company_id, token))
}

/// Helper to create a default test company with ID
pub async fn create_default_test_company() -> Result<Uuid> {
    let company_input = CreateCompanyInput {
        name: "Test Company".to_string(),
        description: Some("Default test company".to_string()),
        website: None,
        phone: None,
        email: None,
        address: None,
        logo_url: None,
        timezone: None,
    };

    let company = company_repo::create_company(&company_input).await?;
    Ok(company.id)
}

/// Helper to create a test user with the given email and make them admin of the default company
pub async fn create_admin_user_str(email: &str) -> Result<Uuid> {
    let user = User {
        id: Uuid::new_v4(),
        name: "Admin User".to_string(),
        email: email.to_string(),
        password_hash: bcrypt::hash("password", 4).unwrap(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let created_user = user_repo::create_user(&user).await?;
    Ok(created_user.id)
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
