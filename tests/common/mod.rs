#![allow(dead_code)]

use actix_web::{http::StatusCode, test, web};
use anyhow::Result;
use be::config::Config;
use be::database::init_database;
use be::database::models::{AddEmployeeToCompanyRequest, CompanyRole};
use be::database::repositories::company::CompanyRepository;
use be::database::repositories::location::LocationRepository;
use be::database::repositories::password_reset::PasswordResetTokenRepository;
use be::database::repositories::user::UserRepository;
use be::services::auth::AuthService;
use be::{ActivityLogger, ActivityRepository, AppState};
use serde_json::json;
use sqlx::SqlitePool;
use std::env;
use tempfile::TempDir;

pub struct TestContext {
    pub pool: SqlitePool,
    pub config: Config,
    pub auth_service: AuthService,
    pub _temp_dir: TempDir, // Keep temp dir alive
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
    web::Data<CompanyRepository>,
    web::Data<Config>,
    web::Data<ActivityLogger>,
    TestContext,
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let activity_logger = ActivityLogger::new(ActivityRepository::new(ctx.pool.clone()));
    let activity_logger_data = web::Data::new(activity_logger.clone());

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger,
    });
    let location_repo_data = web::Data::new(LocationRepository::new(ctx.pool.clone()));
    let company_repo_data = web::Data::new(CompanyRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (
        app_state,
        location_repo_data,
        company_repo_data,
        config_data,
        activity_logger_data,
        ctx,
    )
}

/// Helper function to make a user an admin of the default company (ID 1) for testing
pub async fn make_user_admin_of_default_company(
    company_repo: &CompanyRepository,
    user_id: &str,
) -> Result<()> {
    let add_employee_request = AddEmployeeToCompanyRequest {
        user_id: user_id.to_string(),
        role: CompanyRole::Admin,
        is_primary: Some(true),
        hire_date: None,
    };

    company_repo
        .add_employee_to_company(1, &add_employee_request)
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
    let (token, user_id) = register_test_user(app, email, password, name).await?;
    make_user_admin_of_default_company(company_repo, &user_id).await?;
    Ok((token, user_id))
}
