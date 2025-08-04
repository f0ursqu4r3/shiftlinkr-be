#![allow(dead_code)]

use actix_web::{http::StatusCode, test, web};
use anyhow::Result;
use be::config::Config;
use be::database::init_database;
use be::database::models::{AddEmployeeToCompanyInput, CompanyRole, CreateCompanyInput, User};
use be::database::repositories::activity::ActivityRepository;
use be::database::repositories::company::CompanyRepository;
use be::database::repositories::password_reset::PasswordResetTokenRepository;
use be::database::repositories::pto_balance::PtoBalanceRepository;
use be::database::repositories::time_off::TimeOffRepository;
use be::database::repositories::user::UserRepository;
use be::services::auth::AuthService;
use be::services::ActivityLogger;
use chrono::Utc;
use serde_json::json;
use sqlx::PgPool;
use std::env;
use uuid::Uuid;

pub struct TestContext {
    pub pool: PgPool,
    pub config: Config,
    pub auth_service: AuthService,
    pub time_off_repo: TimeOffRepository,
    pub pto_balance_repo: PtoBalanceRepository,
    pub activity_logger: ActivityLogger,
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

        let pool = init_database(&database_url, true).await?;
        let user_repository = UserRepository::new(pool.clone());
        let password_reset_repository = PasswordResetTokenRepository::new(pool.clone());
        let company_repository = CompanyRepository::new(pool.clone());
        let activity_repository = ActivityRepository::new(pool.clone());

        let auth_service = AuthService::new(
            config.clone(),
            user_repository,
            company_repository,
            password_reset_repository,
        );

        let time_off_repo = TimeOffRepository::new(pool.clone());
        let pto_balance_repo = PtoBalanceRepository::new(pool.clone());
        let activity_logger = ActivityLogger::new(activity_repository);

        Ok(TestContext {
            pool,
            config,
            auth_service,
            time_off_repo,
            pto_balance_repo,
            activity_logger,
        })
    }
}

pub fn setup_test_env() {
    unsafe {
        env::set_var("RUST_LOG", "debug");
    }
    let _ = env_logger::builder().is_test(true).try_init();
}

pub async fn create_test_repositories(
    ctx: &TestContext,
) -> (
    web::Data<TimeOffRepository>,
    web::Data<PtoBalanceRepository>,
    web::Data<ActivityLogger>,
    web::Data<Config>,
) {
    let time_off_repo_data = web::Data::new(ctx.time_off_repo.clone());
    let pto_balance_repo_data = web::Data::new(ctx.pto_balance_repo.clone());
    let activity_logger_data = web::Data::new(ctx.activity_logger.clone());
    let config_data = web::Data::new(ctx.config.clone());

    (
        time_off_repo_data,
        pto_balance_repo_data,
        activity_logger_data,
        config_data,
    )
}

pub async fn create_admin_user(ctx: &TestContext) -> String {
    // Create a test company first
    let company_repo = CompanyRepository::new(ctx.pool.clone());
    let create_company_input = CreateCompanyInput {
        name: "Test Company".to_string(),
        description: None,
        website: None,
        phone: None,
        email: Some("test@company.com".to_string()),
        address: None,
        logo_url: None,
        timezone: Some("UTC".to_string()),
    };
    let company = company_repo
        .create_company(&create_company_input)
        .await
        .expect("Failed to create test company");

    // Create admin user
    let user = User {
        id: Uuid::new_v4(),
        email: "admin@test.com".to_string(),
        password_hash: bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap(),
        name: "Admin User".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let user_repo = UserRepository::new(ctx.pool.clone());
    let admin_user = user_repo
        .create_user(&user)
        .await
        .expect("Failed to create admin user");

    let user_id = admin_user.id;

    // Add user to company as admin
    let add_employee_request = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(true),
        hire_date: None,
    };

    company_repo
        .add_employee_to_company(company.id, &add_employee_request)
        .await
        .expect("Failed to add admin to company");

    // Generate JWT token for this user
    ctx.auth_service
        .generate_company_token(admin_user.id, company.id)
        .await
        .expect("Failed to generate JWT token")
}

/// Helper function to make a user an admin of the default company for testing
pub async fn make_user_admin_of_default_company(
    company_repo: &CompanyRepository,
    user_id: Uuid,
    company_id: Uuid,
) -> Result<()> {
    let add_employee_request = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Admin),
        is_primary: Some(true),
        hire_date: None,
    };

    company_repo
        .add_employee_to_company(company_id, &add_employee_request)
        .await?;

    Ok(())
}

/// Helper function to register a user and get auth token
pub async fn register_test_user<S>(
    app: &S,
    email: &str,
    password: &str,
    name: &str,
) -> Result<(String, String)>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
{
    let register_data = json!({
        "email": email,
        "password": password,
        "name": name
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_srv_request();

    let resp = test::call_service(app, req).await;
    if resp.status() != StatusCode::OK {
        let body: serde_json::Value = test::read_body_json(resp).await;
        return Err(anyhow::anyhow!("Registration failed: {:?}", body));
    }

    let body: serde_json::Value = test::read_body_json(resp).await;
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().to_string();

    Ok((token, user_id))
}

/// Helper function to create an admin user for testing
pub async fn create_admin_user_and_token<S>(
    app: &S,
    company_repo: &CompanyRepository,
    email: &str,
    password: &str,
    name: &str,
) -> Result<(String, String)>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
{
    let (token, user_id_str) = register_test_user(app, email, password, name).await?;
    let user_id = Uuid::parse_str(&user_id_str)?;

    // Create a company for testing
    let create_company_input = CreateCompanyInput {
        name: "Test Company".to_string(),
        description: None,
        website: None,
        phone: None,
        email: Some("test@company.com".to_string()),
        address: None,
        logo_url: None,
        timezone: Some("UTC".to_string()),
    };
    let company = company_repo.create_company(&create_company_input).await?;

    make_user_admin_of_default_company(company_repo, user_id, company.id).await?;
    Ok((token, user_id_str))
}

/// Helper to create a manager user for testing
pub async fn create_manager_user_and_token<S>(
    app: &S,
    company_repo: &CompanyRepository,
    email: &str,
    password: &str,
    name: &str,
) -> Result<(String, String)>
where
    S: actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse<actix_web::body::BoxBody>,
        Error = actix_web::Error,
    >,
{
    let (token, user_id_str) = register_test_user(app, email, password, name).await?;
    let user_id = Uuid::parse_str(&user_id_str)?;

    // Create a company for testing
    let create_company_input = CreateCompanyInput {
        name: "Test Company".to_string(),
        description: None,
        website: None,
        phone: None,
        email: Some("test@company.com".to_string()),
        address: None,
        logo_url: None,
        timezone: Some("UTC".to_string()),
    };
    let company = company_repo.create_company(&create_company_input).await?;

    let add_employee_request = AddEmployeeToCompanyInput {
        user_id,
        role: Some(CompanyRole::Manager),
        is_primary: Some(true),
        hire_date: None,
    };

    company_repo
        .add_employee_to_company(company.id, &add_employee_request)
        .await?;

    Ok((token, user_id_str))
}
