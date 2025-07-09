use actix_web::{http::StatusCode, test, web};
use anyhow::Result;
use be::auth::AuthService;
use be::config::Config;
use be::database::init_database;
use be::database::invite_repository::InviteRepository;
use be::database::location_repository::LocationRepository;
use be::database::password_reset_repository::PasswordResetTokenRepository;
use be::database::shift_repository::ShiftRepository;
use be::database::shift_swap_repository::ShiftSwapRepository;
use be::database::stats_repository::StatsRepository;
use be::database::time_off_repository::TimeOffRepository;
use be::database::user_repository::UserRepository;
use be::AppState;
use serde_json::json;
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

// Helper to create a user and get their auth token
pub async fn create_authenticated_user(
    app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    role: &str,
) -> String {
    let register_data = json!({
        "email": format!("{}@example.com", role),
        "password": "password123",
        "name": format!("{} User", role),
        "role": role
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_srv_request();

    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    body["token"].as_str().unwrap().to_string()
}

// Helper to make an authenticated request
pub async fn make_authenticated_request(
    app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    method: &str,
    uri: &str,
    token: &str,
    body: Option<serde_json::Value>,
) -> actix_web::dev::ServiceResponse {
    let mut req = match method {
        "GET" => test::TestRequest::get().uri(uri),
        "POST" => test::TestRequest::post().uri(uri),
        "PUT" => test::TestRequest::put().uri(uri),
        "DELETE" => test::TestRequest::delete().uri(uri),
        _ => panic!("Unsupported method: {}", method),
    };

    req = req.insert_header(("Authorization", format!("Bearer {}", token)));

    if let Some(body) = body {
        req = req.set_json(&body);
    }

    test::call_service(app, req.to_srv_request()).await
}

// Helper to make an unauthenticated request
pub async fn make_unauthenticated_request(
    app: &impl actix_web::dev::Service<
        actix_web::dev::ServiceRequest,
        Response = actix_web::dev::ServiceResponse,
        Error = actix_web::Error,
    >,
    method: &str,
    uri: &str,
    body: Option<serde_json::Value>,
) -> actix_web::dev::ServiceResponse {
    let mut req = match method {
        "GET" => test::TestRequest::get().uri(uri),
        "POST" => test::TestRequest::post().uri(uri),
        "PUT" => test::TestRequest::put().uri(uri),
        "DELETE" => test::TestRequest::delete().uri(uri),
        _ => panic!("Unsupported method: {}", method),
    };

    if let Some(body) = body {
        req = req.set_json(&body);
    }

    test::call_service(app, req.to_srv_request()).await
}

// Helper to create admin app setup dependencies
pub async fn create_admin_app_data() -> (
    web::Data<AppState>,
    web::Data<LocationRepository>,
    web::Data<Config>,
    TestContext, // Return the context to keep it alive
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

// Helper to create auth app setup dependencies
pub async fn create_auth_app_data() -> (
    web::Data<AppState>,
    web::Data<InviteRepository>,
    web::Data<be::Config>,
    TestContext, // Return the context to keep it alive
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let invite_repo_data = web::Data::new(InviteRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (app_state, invite_repo_data, config_data, ctx)
}

// Helper to create shifts app setup dependencies
pub async fn create_shifts_app_data() -> (
    web::Data<AppState>,
    web::Data<ShiftRepository>,
    web::Data<LocationRepository>,
    web::Data<be::Config>,
    TestContext, // Return the context to keep it alive
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let shift_repo_data = web::Data::new(ShiftRepository::new(ctx.pool.clone()));
    let location_repo_data = web::Data::new(LocationRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (
        app_state,
        shift_repo_data,
        location_repo_data,
        config_data,
        ctx,
    )
}

// Helper to create time-off app setup dependencies
pub async fn create_time_off_app_data() -> (
    web::Data<AppState>,
    web::Data<TimeOffRepository>,
    web::Data<be::Config>,
    TestContext, // Return the context to keep it alive
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (app_state, time_off_repo_data, config_data, ctx)
}

// Helper to create swaps app setup dependencies
pub async fn create_swaps_app_data() -> (
    web::Data<AppState>,
    web::Data<ShiftSwapRepository>,
    web::Data<be::Config>,
    TestContext, // Return the context to keep it alive
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let swap_repo_data = web::Data::new(ShiftSwapRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (app_state, swap_repo_data, config_data, ctx)
}

// Helper to create stats app setup dependencies
pub async fn create_stats_app_data() -> (
    web::Data<AppState>,
    web::Data<StatsRepository>,
    web::Data<be::Config>,
    TestContext, // Return the context to keep it alive
) {
    setup_test_env();
    let ctx = TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
    });
    let stats_repo_data = web::Data::new(StatsRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (app_state, stats_repo_data, config_data, ctx)
}
