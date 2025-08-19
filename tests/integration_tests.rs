use actix_web::{App, http::StatusCode, test, web};
use be::handlers::auth;
use be::middleware::CacheLayer;
use serde_json::json;

mod common;

#[actix_web::test]
async fn test_register_endpoint() {
    common::setup_test_env();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
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
        .set_json(&register_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["name"], "Test User");
}

#[actix_web::test]
async fn test_login_endpoint() {
    common::setup_test_env();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
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
        .set_json(&register_data)
        .to_request();

    test::call_service(&app, reg_req).await;

    // Now try to login
    let login_data = json!({
        "email": "login@example.com",
        "password": "password123"
    });

    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "login@example.com");
    assert_eq!(body["user"]["name"], "Login User");
}

#[actix_web::test]
async fn test_me_endpoint() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
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
        .set_json(&register_data)
        .to_request();

    let reg_resp = test::call_service(&app, reg_req).await;
    let reg_body: serde_json::Value = test::read_body_json(reg_resp).await;
    let token = reg_body["token"].as_str().unwrap();

    // Use token to access /me endpoint
    let me_req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, me_req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "me@example.com");
    assert_eq!(body["user"]["name"], "Me User");
}

#[actix_web::test]
async fn test_me_endpoint_without_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/me", web::get().to(auth::me))),
            ),
    )
    .await;

    let req = test::TestRequest::get().uri("/api/v1/auth/me").to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app =
        test::init_service(
            App::new()
                .app_data(web::Data::new(CacheLayer::new(1000, 60)))
                .service(web::scope("/api/v1").service(
                    web::scope("/auth").route("/register", web::post().to(auth::register)),
                )),
        )
        .await;

    let register_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
        "name": "First User"
    });

    // First registration
    let req1 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp1 = test::call_service(&app, req1).await;
    assert_eq!(resp1.status(), StatusCode::OK);

    // Second registration with same email
    let req2 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp2 = test::call_service(&app, req2).await;
    assert_eq!(resp2.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_login_wrong_password() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/login", web::post().to(auth::login)),
                ),
            ),
    )
    .await;

    // Register a user
    let register_data = json!({
        "email": "wrongpass@example.com",
        "password": "correct_password",
        "name": "Wrong Pass User"
    });
    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    test::call_service(&app, reg_req).await;

    // Try to login with wrong password
    let login_data = json!({
        "email": "wrongpass@example.com",
        "password": "wrong_password"
    });
    let login_req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data)
        .to_request();
    let resp = test::call_service(&app, login_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_forgot_password_endpoint() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(auth::register))
                        .route("/forgot-password", web::post().to(auth::forgot_password)),
                ),
            ),
    )
    .await;

    // First register a user
    let register_data = json!({
        "email": "forgot@example.com",
        "password": "password123",
        "name": "Forgot User"
    });
    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    test::call_service(&app, reg_req).await;

    // Test forgot password with existing email
    let forgot_data = json!({ "email": "forgot@example.com" });
    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data)
        .to_request();
    let resp = test::call_service(&app, forgot_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_forgot_password_nonexistent_email() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/forgot-password", web::post().to(auth::forgot_password)),
                ),
            ),
    )
    .await;

    // Test forgot password with non-existent email (still 200)
    let forgot_data = json!({ "email": "nonexistent@example.com" });
    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data)
        .to_request();
    let resp = test::call_service(&app, forgot_req).await;
    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_reset_password_invalid_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(web::scope("/api/v1").service(
                web::scope("/auth").route("/reset-password", web::post().to(auth::reset_password)),
            )),
    )
    .await;

    // Test reset password with invalid token
    let reset_data = json!({
        "token": "invalid-token",
        "new_password": "newpassword123"
    });
    let reset_req = test::TestRequest::post()
        .uri("/api/v1/auth/reset-password")
        .set_json(&reset_data)
        .to_request();
    let resp = test::call_service(&app, reset_req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn test_complete_password_reset_flow() {
    common::setup_test_env();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
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
        "name": "Complete User"
    });
    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    test::call_service(&app, reg_req).await;

    // 2. Request password reset (this generates a token)
    let forgot_data = json!({ "email": "complete@example.com" });
    let forgot_req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data)
        .to_request();
    let forgot_resp = test::call_service(&app, forgot_req).await;
    assert_eq!(forgot_resp.status(), StatusCode::OK);

    // 3. Simulate retrieving token via service call
    let token = be::services::auth::forgot_password("complete@example.com")
        .await
        .unwrap();

    // 4. Reset password with the token
    let reset_data = json!({
        "token": token,
        "new_password": "newpassword123"
    });
    let reset_req = test::TestRequest::post()
        .uri("/api/v1/auth/reset-password")
        .set_json(&reset_data)
        .to_request();
    let reset_resp = test::call_service(&app, reset_req).await;
    assert_eq!(reset_resp.status(), StatusCode::OK);

    // 5. Verify old password doesn't work and new one does
    let old_login = json!({ "email": "complete@example.com", "password": "oldpassword123" });
    let req_old = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&old_login)
        .to_request();
    let old_resp = test::call_service(&app, req_old).await;
    assert_eq!(old_resp.status(), StatusCode::BAD_REQUEST);

    let new_login = json!({ "email": "complete@example.com", "password": "newpassword123" });
    let req_new = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&new_login)
        .to_request();
    let new_resp = test::call_service(&app, req_new).await;
    assert_eq!(new_resp.status(), StatusCode::OK);
}
