use be::database::models::{CreateUserRequest, LoginRequest};
use chrono::Utc;

mod common;

#[tokio::test]
async fn test_user_registration() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let request = CreateUserRequest {
        email: "register@example.com".to_string(),
        password: "password123".to_string(),
        name: "Register User".to_string(),
    };

    let result = ctx.auth_service.register(request).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, "register@example.com");
    assert_eq!(response.user.name, "Register User");
}

#[tokio::test]
async fn test_duplicate_email_registration() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let request = CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "password123".to_string(),
        name: "First User".to_string(),
    };

    // First registration should succeed
    let result1 = ctx.auth_service.register(request).await;
    assert!(result1.is_ok());

    // Second registration with same email should fail
    let request2 = CreateUserRequest {
        email: "duplicate@example.com".to_string(),
        password: "different_password".to_string(),
        name: "Second User".to_string(),
    };

    let result2 = ctx.auth_service.register(request2).await;
    assert!(result2.is_err());
    assert!(result2
        .unwrap_err()
        .to_string()
        .contains("Email already exists"));
}

#[tokio::test]
async fn test_user_login() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // First register a user
    let register_request = CreateUserRequest {
        email: "login@example.com".to_string(),
        password: "password123".to_string(),
        name: "Login User".to_string(),
    };

    ctx.auth_service.register(register_request).await.unwrap();

    // Now try to login
    let login_request = LoginRequest {
        email: "login@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = ctx.auth_service.login(login_request).await;
    assert!(result.is_ok());

    let response = result.unwrap();
    assert!(!response.token.is_empty());
    assert_eq!(response.user.email, "login@example.com");
    assert_eq!(response.user.name, "Login User");
}

#[tokio::test]
async fn test_login_with_wrong_password() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Register a user
    let register_request = CreateUserRequest {
        email: "wrongpass@example.com".to_string(),
        password: "correct_password".to_string(),
        name: "Wrong Pass User".to_string(),
    };

    ctx.auth_service.register(register_request).await.unwrap();

    // Try to login with wrong password
    let login_request = LoginRequest {
        email: "wrongpass@example.com".to_string(),
        password: "wrong_password".to_string(),
    };

    let result = ctx.auth_service.login(login_request).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid email or password"));
}

#[tokio::test]
async fn test_login_with_nonexistent_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let login_request = LoginRequest {
        email: "nonexistent@example.com".to_string(),
        password: "password123".to_string(),
    };

    let result = ctx.auth_service.login(login_request).await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid email or password"));
}

#[tokio::test]
async fn test_jwt_token_verification() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Register and get token
    let register_request = CreateUserRequest {
        email: "jwt@example.com".to_string(),
        password: "password123".to_string(),
        name: "JWT User".to_string(),
    };

    let auth_response = ctx.auth_service.register(register_request).await.unwrap();
    let token = auth_response.token;

    // Verify token
    let claims_result = ctx.auth_service.verify_token(&token);
    assert!(claims_result.is_ok());

    let claims = claims_result.unwrap();
    assert_eq!(claims.email, "jwt@example.com");
    assert_eq!(claims.role, "employee"); // All users get "employee" role in JWT since roles are now company-specific
    assert_eq!(claims.sub, auth_response.user.id);

    // Check expiration is in the future
    let now = Utc::now().timestamp() as usize;
    assert!(claims.exp > now);
}

#[tokio::test]
async fn test_invalid_jwt_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let invalid_token = "invalid.jwt.token";
    let result = ctx.auth_service.verify_token(invalid_token);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_user_from_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Register a user
    let register_request = CreateUserRequest {
        email: "tokenuser@example.com".to_string(),
        password: "password123".to_string(),
        name: "Token User".to_string(),
    };

    let auth_response = ctx.auth_service.register(register_request).await.unwrap();
    let token = auth_response.token;

    // Get user from token
    let user_result = ctx.auth_service.get_user_from_token(&token).await;
    assert!(user_result.is_ok());

    let user = user_result.unwrap();
    assert_eq!(user.email, "tokenuser@example.com");
    assert_eq!(user.name, "Token User");
}

#[tokio::test]
async fn test_get_user_from_invalid_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let invalid_token = "invalid.jwt.token";
    let result = ctx.auth_service.get_user_from_token(invalid_token).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_jwt_token_expiration_configuration() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Register a user
    let register_request = CreateUserRequest {
        email: "expiry@example.com".to_string(),
        password: "password123".to_string(),
        name: "Expiry User".to_string(),
    };

    let auth_response = ctx.auth_service.register(register_request).await.unwrap();
    let token = auth_response.token;

    // Verify token and check expiration
    let claims = ctx.auth_service.verify_token(&token).unwrap();

    // Should expire in 1 day (as configured in test context)
    let now = Utc::now().timestamp() as usize;
    let expected_expiry = now + (24 * 60 * 60); // 1 day in seconds

    // Allow for some time difference in test execution
    assert!(claims.exp > now);
    assert!(claims.exp <= expected_expiry + 60); // Allow 1 minute buffer
}
