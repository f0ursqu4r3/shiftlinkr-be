use actix_web::{test, web, App};
use serde_json::json;
use be::handlers::auth;
use be::AppState;

mod common;

async fn create_test_app() -> impl actix_web::dev::Service<
    actix_web::dev::ServiceRequest,
    Response = actix_web::dev::ServiceResponse,
    Error = actix_web::Error,
> {
    let ctx = common::TestContext::new().await.unwrap();
    
    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });

    test::init_service(
        App::new()
            .app_data(app_state)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(auth::register))
                            .route("/login", web::post().to(auth::login))
                            .route("/me", web::get().to(auth::me))
                    )
            )
    ).await
}

#[actix_web::test]
async fn test_register_endpoint() {
    common::setup_test_env();
    let app = create_test_app().await;

    let register_data = json!({
        "email": "test@example.com",
        "password": "password123",
        "name": "Test User",
        "role": "employee"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "test@example.com");
    assert_eq!(body["user"]["name"], "Test User");
    assert_eq!(body["user"]["role"], "employee");
}

#[actix_web::test]
async fn test_register_duplicate_email() {
    common::setup_test_env();
    let app = create_test_app().await;

    let register_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
        "name": "First User",
        "role": "employee"
    });

    // First registration
    let req1 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();

    let resp1 = test::call_service(&app, req1).await;
    assert!(resp1.status().is_success());

    // Second registration with same email
    let req2 = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    assert!(resp2.status().is_client_error());

    let body: serde_json::Value = test::read_body_json(resp2).await;
    assert!(body["error"].as_str().unwrap().contains("Email already exists"));
}

#[actix_web::test]
async fn test_login_endpoint() {
    common::setup_test_env();
    let app = create_test_app().await;

    // First register a user
    let register_data = json!({
        "email": "login@example.com",
        "password": "password123",
        "name": "Login User",
        "role": "manager"
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
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["email"], "login@example.com");
    assert_eq!(body["user"]["name"], "Login User");
    assert_eq!(body["user"]["role"], "manager");
}

#[actix_web::test]
async fn test_login_wrong_password() {
    common::setup_test_env();
    let app = create_test_app().await;

    // Register a user
    let register_data = json!({
        "email": "wrongpass@example.com",
        "password": "correct_password",
        "name": "Wrong Pass User",
        "role": "employee"
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
    assert!(resp.status().is_client_error());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Invalid email or password"));
}

#[actix_web::test]
async fn test_me_endpoint() {
    common::setup_test_env();
    let app = create_test_app().await;

    // Register a user and get token
    let register_data = json!({
        "email": "me@example.com",
        "password": "password123",
        "name": "Me User",
        "role": "admin"
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
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["email"], "me@example.com");
    assert_eq!(body["user"]["name"], "Me User");
    assert_eq!(body["user"]["role"], "admin");
}

#[actix_web::test]
async fn test_me_endpoint_without_token() {
    common::setup_test_env();
    let app = create_test_app().await;

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].as_str().unwrap().contains("Missing or invalid authorization header"));
}

#[actix_web::test]
async fn test_me_endpoint_with_invalid_token() {
    common::setup_test_env();
    let app = create_test_app().await;

    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", "Bearer invalid.jwt.token"))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].is_string());
}

#[actix_web::test]
async fn test_register_with_different_roles() {
    common::setup_test_env();
    let app = create_test_app().await;

    // Test admin role
    let admin_data = json!({
        "email": "admin@example.com",
        "password": "password123",
        "name": "Admin User",
        "role": "admin"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&admin_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["role"], "admin");

    // Test manager role
    let manager_data = json!({
        "email": "manager@example.com",
        "password": "password123",
        "name": "Manager User",
        "role": "manager"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&manager_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["role"], "manager");

    // Test employee role
    let employee_data = json!({
        "email": "employee@example.com",
        "password": "password123",
        "name": "Employee User",
        "role": "employee"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&employee_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["role"], "employee");
}

#[actix_web::test]
async fn test_register_with_missing_fields() {
    common::setup_test_env();
    let app = create_test_app().await;

    // Missing email
    let data = json!({
        "password": "password123",
        "name": "Test User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Missing password
    let data = json!({
        "email": "test@example.com",
        "name": "Test User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // Missing name
    let data = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn test_register_without_role_defaults_to_employee() {
    common::setup_test_env();
    let app = create_test_app().await;

    let data = json!({
        "email": "norole@example.com",
        "password": "password123",
        "name": "No Role User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["user"]["role"], "employee");
}
