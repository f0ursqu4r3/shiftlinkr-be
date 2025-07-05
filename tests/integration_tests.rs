use actix_web::{App, http::StatusCode, test, web};
use be::AppState;
use be::handlers::auth;
use serde_json::json;

mod common;

#[actix_web::test]
async fn test_register_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
        "name": "Test User",
        "role": "employee"
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
    assert_eq!(body["user"]["role"], "employee");
}

#[actix_web::test]
async fn test_login_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
        "name": "Login User",
        "role": "manager"
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
    assert_eq!(body["user"]["role"], "manager");
}

#[actix_web::test]
async fn test_me_endpoint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
        "name": "Me User",
        "role": "admin"
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
    assert_eq!(body["user"]["role"], "admin");
}

#[actix_web::test]
async fn test_me_endpoint_without_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Missing or invalid authorization header")
    );
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Email already exists")
    );
}

#[actix_web::test]
async fn test_login_wrong_password() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
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
    assert!(
        body["error"]
            .as_str()
            .unwrap()
            .contains("Invalid email or password")
    );
}
