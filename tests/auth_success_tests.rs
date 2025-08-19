use actix_web::{App, http::StatusCode, test, web};
use be::handlers::auth;
use be::middleware::CacheLayer;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_auth_register_and_login_workflow() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);
    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
            web::scope("/api/v1").service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/me", web::get().to(auth::me)),
            ),
        ),
    )
    .await;

    // Test 1: Register a new user
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

    let register_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(register_body["token"].is_string());
    assert!(register_body["user"]["id"].is_string());
    assert_eq!(register_body["user"]["email"], "test@example.com");
    assert_eq!(register_body["user"]["name"], "Test User");

    // Test 2: Login with the registered user
    let login_data = json!({
        "email": "test@example.com",
        "password": "password123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let login_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(login_body["token"].is_string());
    assert!(login_body["user"]["id"].is_string());
    assert_eq!(login_body["user"]["email"], "test@example.com");
    assert_eq!(login_body["user"]["name"], "Test User");

    // Test 3: Use token to access /me endpoint
    let auth_token = login_body["token"].as_str().unwrap();
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/me")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let me_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(me_body["user"]["email"], "test@example.com");
    assert_eq!(me_body["user"]["name"], "Test User");
}

#[actix_web::test]
#[serial]
async fn test_auth_invalid_credentials() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);
    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
            web::scope("/api/v1")
                .service(web::scope("/auth").route("/login", web::post().to(auth::login))),
        ),
    )
    .await;

    // Test login with invalid credentials
    let login_data = json!({
        "email": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_auth_duplicate_email_registration() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);
    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
            web::scope("/api/v1")
                .service(web::scope("/auth").route("/register", web::post().to(auth::register))),
        ),
    )
    .await;

    // Register first user
    let register_data = json!({
        "email": "duplicate@example.com",
        "password": "password123",
    "name": "First User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to register with same email
    let duplicate_data = json!({
        "email": "duplicate@example.com",
        "password": "different123",
    "name": "Second User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&duplicate_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["error"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_auth_invite_workflow() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);
    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
            web::scope("/api/v1").service(
                web::scope("/auth")
                    .route("/register", web::post().to(auth::register))
                    .route("/login", web::post().to(auth::login))
                    .route("/invite", web::post().to(auth::create_invite))
                    .route("/invite/{token}", web::get().to(auth::get_invite))
                    .route(
                        "/invite/{token}/accept",
                        web::post().to(auth::accept_invite),
                    )
                    .route("/invites", web::get().to(auth::get_my_invites)),
            ),
        ),
    )
    .await;

    // Create an admin user in a company
    let (admin_user_id, admin_token, company_id) =
        common::create_test_user_with_token("admin@example.com", "password123", "Admin User")
            .await
            .unwrap();
    // Promote to admin
    common::make_user_admin_of_company(admin_user_id, company_id)
        .await
        .unwrap();

    // Test 1: Create an invite
    let invite_data = json!({
        "email": "invitee@example.com",
        "role": "employee"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/invite")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&invite_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let invite_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(invite_body["invite_link"].is_string());
    assert!(invite_body["expires_at"].is_string());

    // Extract token from invite_link
    let invite_link = invite_body["invite_link"].as_str().unwrap();
    let invite_token = invite_link.split('/').last().unwrap();

    // Test 2: Get invite details
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/auth/invite/{}", invite_token))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let get_invite_body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(get_invite_body["email"], "invitee@example.com");
    assert_eq!(get_invite_body["role"], "employee");

    // Register the invitee account so they can accept the invite
    let invitee_register = json!({
        "email": "invitee@example.com",
        "password": "newpassword123",
        "name": "Invited User"
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&invitee_register)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let invitee_body: serde_json::Value = test::read_body_json(resp).await;
    let invitee_token = invitee_body["token"].as_str().unwrap();

    // Test 3: Accept the invite with invitee's token and token in path
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/auth/invite/{}/accept", invite_token))
        .insert_header(("Authorization", format!("Bearer {}", invitee_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let accept_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(accept_body["token"].is_string());
    assert_eq!(accept_body["user"]["email"], "invitee@example.com");
    assert_eq!(accept_body["user"]["name"], "Invited User");

    // Test 4: Check admin's invites
    let req = test::TestRequest::get()
        .uri("/api/v1/auth/invites")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let invites_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(invites_body["invites"].is_array());
    assert!(invites_body["invites"].as_array().unwrap().len() >= 1);
}

#[actix_web::test]
#[serial]
async fn test_auth_password_reset_workflow() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);
    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
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

    // First, register a user
    let register_data = json!({
        "email": "resetuser@example.com",
        "password": "oldpassword123",
    "name": "Reset User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Test 1: Request password reset
    let forgot_data = json!({
        "email": "resetuser@example.com"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/forgot-password")
        .set_json(&forgot_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let forgot_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(forgot_body["message"].is_string());
    assert!(forgot_body["token"].is_string()); // For testing purposes

    let reset_token = forgot_body["token"].as_str().unwrap();

    // Test 2: Reset password using token
    let reset_data = json!({
        "token": reset_token,
        "new_password": "newpassword123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/reset-password")
        .set_json(&reset_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let reset_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(reset_body["message"].is_string());

    // Test 3: Login with new password
    let login_data = json!({
        "email": "resetuser@example.com",
        "password": "newpassword123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let login_body: serde_json::Value = test::read_body_json(resp).await;
    assert!(login_body["token"].is_string());
    assert_eq!(login_body["user"]["email"], "resetuser@example.com");

    // Test 4: Verify old password no longer works
    let old_login_data = json!({
        "email": "resetuser@example.com",
        "password": "oldpassword123"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&old_login_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
