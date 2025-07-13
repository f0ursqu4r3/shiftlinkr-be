use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::company::CompanyRepository;
use be::handlers::auth;
use be::{ActivityLogger, ActivityRepository, AppState};
use serde_json::json;

mod common;

#[actix_web::test]
async fn test_register_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    let register_data = json!({
        "email": "test@example.com",
        "password": "password123",
        "name": "Test User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let resp = test::call_service(&app, req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["name"], "Test User");
}

#[actix_web::test]
async fn test_login_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    // First register a user
    let register_data = json!({
        "email": "login@example.com",
        "password": "password123",
        "name": "Login User"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    test::call_service(&app, reg_req.to_request()).await;

    // Now try to login
    let login_data = json!({
        "email": "login@example.com",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data);

    let resp = test::call_service(&app, login_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "login@example.com");
    assert_eq!(body["user"]["name"], "Login User");
}

#[actix_web::test]
async fn test_me_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    // Register a user and get token
    let register_data = json!({
        "email": "me@example.com",
        "password": "password123",
        "name": "Me User"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let reg_resp = test::call_service(&app, reg_req.to_request()).await;
    let reg_body: serde_json::Value = test::read_body_json(reg_resp).await;
    let token = reg_body["token"].as_str().unwrap();

    // Use token to access /me endpoint
    let me_req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", token)));

    let resp = test::call_service(&app, me_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "me@example.com");
    assert_eq!(body["user"]["name"], "Me User");
}

#[actix_web::test]
async fn test_me_endpoint_without_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/auth/me");

    let resp = test::call_service(&app, req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Missing or invalid authorization header"));
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    let register_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
        "name": "First User",
        "role": "employee"
    });

    // First registration
    let req1 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let resp1 = test::call_service(&app, req1.to_request()).await;
    assert_eq!(resp1.status(), StatusCode::OK);

    // Second registration with same email
    let req2 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let resp2 = test::call_service(&app, req2.to_request()).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Email already exists"));
}

#[actix_web::test]
async fn test_login_wrong_password() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me)),
                ),
            ),
    )
    .await;

    // Register a user
    let register_data = json!({
        "email": "wrongpass@example.com",
        "password": "correct_password",
        "name": "Wrong Pass User",
        "role": "employee"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    test::call_service(&app, reg_req.to_request()).await;

    // Try to login with wrong password
    let login_data = json!({
        "email": "wrongpass@example.com",
        "password": "wrong_password"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data);

    let resp = test::call_service(&app, login_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Invalid email or password"));
}

#[actix_web::test]
async fn test_forgot_password_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/me", web::get().to(auth::me))
                        .route("/forgot-password", web::post().to(auth::forgot_password))
                        .route("/reset-password", web::post().to(auth::reset_password)),
                ),
            ),
    )
    .await;

    // First register a user
    let register_data = json!({
        "email": "forgot@example.com",
        "password": "password123",
        "name": "Forgot User",
        "role": "employee"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    test::call_service(&app, reg_req.to_request()).await;

    // Test forgot password with existing email
    let forgot_data = json!({
        "email": "forgot@example.com"
    });

    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data);

    let resp = test::call_service(&app, forgot_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("password reset link has been sent"));
}

#[actix_web::test]
async fn test_forgot_password_nonexistent_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/forgot-password", web::post().to(auth::forgot_password)),
                ),
            ),
    )
    .await;

    // Test forgot password with non-existent email
    let forgot_data = json!({
        "email": "nonexistent@example.com"
    });

    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data);

    let resp = test::call_service(&app, forgot_req.to_request()).await;
    // Should still return 200 for security (don't reveal if email exists)
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("password reset link has been sent"));
}

#[actix_web::test]
async fn test_reset_password_invalid_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(web::scope("/api/v1").service(
                web::scope("/auth").route("/reset-password", web::post().to(auth::reset_password)),
            )),
    )
    .await;

    // Test reset password with invalid token
    let reset_data = json!({
        "token": "invalid-token-12345",
        "new_password": "newpassword123"
    });

    let reset_req = test::TestRequest::post()
        .uri("/api/v1/auth/reset-password")
        .set_json(&reset_data);

    let resp = test::call_service(&app, reset_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"]
        .as_str()
        .unwrap()
        .contains("Invalid or expired reset token"));
}

#[actix_web::test]
async fn test_complete_password_reset_flow() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login))
                        .route("/forgot-password", web::post().to(auth::forgot_password))
                        .route("/reset-password", web::post().to(auth::reset_password)),
                ),
            ),
    )
    .await;

    // 1. Register a user
    let register_data = json!({
        "email": "complete@example.com",
        "password": "oldpassword123",
        "name": "Complete User",
        "role": "employee"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    test::call_service(&app, reg_req.to_request()).await;

    // 2. Request password reset (this generates a token)
    let forgot_data = json!({
        "email": "complete@example.com"
    });

    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data);

    let forgot_resp = test::call_service(&app, forgot_req.to_request()).await;
    assert_eq!(forgot_resp.status(), StatusCode::OK);

    // 3. Get the token from the auth service (simulating getting it from email)
    let token = ctx
        .auth_service
        .forgot_password("complete@example.com")
        .await
        .unwrap();

    // 4. Reset password with the token
    let reset_data = json!({
        "token": token,
        "new_password": "newpassword123"
    });

    let reset_req = test::TestRequest::post()
        .uri("/api/v1/auth/reset-password")
        .set_json(&reset_data);

    let reset_resp = test::call_service(&app, reset_req.to_request()).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);

    let reset_body: serde_json::Value = test::read_body_json(reset_resp).await;
    assert!(reset_body["message"]
        .as_str()
        .unwrap()
        .contains("Password has been reset successfully"));

    // 5. Verify old password doesn't work
    let old_login_data = json!({
        "email": "complete@example.com",
        "password": "oldpassword123"
    });

    let old_login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&old_login_data);

    let old_login_resp = test::call_service(&app, old_login_req.to_request()).await;
    assert_eq!(old_login_resp.status(), StatusCode::BAD_REQUEST);

    // 6. Verify new password works
    let new_login_data = json!({
        "email": "complete@example.com",
        "password": "newpassword123"
    });

    let new_login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&new_login_data);

    let new_login_resp = test::call_service(&app, new_login_req.to_request()).await;
    assert_eq!(new_login_resp.status(), StatusCode::OK);

    let login_body: serde_json::Value = test::read_body_json(new_login_resp).await;
    assert!(login_body["token"].is_string());
    assert_eq!(login_body["user"]["email"], "complete@example.com");
}
