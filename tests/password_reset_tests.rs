use be::database::models::CreateUserRequest;
use chrono::Utc;

mod common;

#[tokio::test]
async fn test_forgot_password_valid_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // First create a user
    let register_request = CreateUserRequest {
        email: "forgot@example.com".to_string(),
        password: "password123".to_string(),
        name: "Forgot User".to_string(),
    };

    let registration = ctx.auth_service.register(register_request).await;
    assert!(registration.is_ok());

    // Test forgot password
    let result = ctx.auth_service.forgot_password("forgot@example.com").await;
    assert!(result.is_ok());

    let token = result.unwrap();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36); // UUID length
}

#[tokio::test]
async fn test_forgot_password_invalid_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Test forgot password with non-existent email
    let result = ctx
        .auth_service
        .forgot_password("nonexistent@example.com")
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("User not found"));
}

#[tokio::test]
async fn test_reset_password_valid_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserRequest {
        email: "reset@example.com".to_string(),
        password: "oldpassword123".to_string(),
        name: "Reset User".to_string(),
    };

    let registration = ctx.auth_service.register(register_request).await;
    assert!(registration.is_ok());

    // Request password reset
    let token = ctx
        .auth_service
        .forgot_password("reset@example.com")
        .await
        .unwrap();

    // Reset password with valid token
    let result = ctx
        .auth_service
        .reset_password(&token, "newpassword123")
        .await;
    assert!(result.is_ok());

    // Verify old password doesn't work
    let old_login = ctx
        .auth_service
        .login(be::database::models::LoginRequest {
            email: "reset@example.com".to_string(),
            password: "oldpassword123".to_string(),
        })
        .await;
    assert!(old_login.is_err());

    // Verify new password works
    let new_login = ctx
        .auth_service
        .login(be::database::models::LoginRequest {
            email: "reset@example.com".to_string(),
            password: "newpassword123".to_string(),
        })
        .await;
    assert!(new_login.is_ok());
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Test reset with invalid token
    let result = ctx
        .auth_service
        .reset_password("invalid-token", "newpassword123")
        .await;
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Invalid or expired reset token"));
}

#[tokio::test]
async fn test_reset_password_used_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserRequest {
        email: "used@example.com".to_string(),
        password: "password123".to_string(),
        name: "Used Token User".to_string(),
    };

    let registration = ctx.auth_service.register(register_request).await;
    assert!(registration.is_ok());

    // Request password reset
    let token = ctx
        .auth_service
        .forgot_password("used@example.com")
        .await
        .unwrap();

    // Use the token once
    let first_reset = ctx
        .auth_service
        .reset_password(&token, "newpassword123")
        .await;
    assert!(first_reset.is_ok());

    // Try to use the same token again
    let second_reset = ctx
        .auth_service
        .reset_password(&token, "anotherpassword123")
        .await;
    assert!(second_reset.is_err());
    assert!(second_reset
        .unwrap_err()
        .to_string()
        .contains("Invalid or expired reset token"));
}

#[tokio::test]
async fn test_multiple_reset_tokens_invalidated() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserRequest {
        email: "multiple@example.com".to_string(),
        password: "password123".to_string(),
        name: "Multiple Tokens User".to_string(),
    };

    let registration = ctx.auth_service.register(register_request).await;
    assert!(registration.is_ok());

    // Request multiple password resets
    let token1 = ctx
        .auth_service
        .forgot_password("multiple@example.com")
        .await
        .unwrap();
    let token2 = ctx
        .auth_service
        .forgot_password("multiple@example.com")
        .await
        .unwrap();

    // Use the second token
    let reset_result = ctx
        .auth_service
        .reset_password(&token2, "newpassword123")
        .await;
    assert!(reset_result.is_ok());

    // First token should now be invalid (all tokens for user should be invalidated)
    let old_token_result = ctx
        .auth_service
        .reset_password(&token1, "anotherpassword123")
        .await;
    assert!(old_token_result.is_err());
}

#[tokio::test]
async fn test_password_reset_repository_create_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user first
    let user = be::database::models::User::new(
        "repo-test@example.com".to_string(),
        "hashed_password".to_string(),
        "Repo Test User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    // Create password reset token repository
    let reset_repo =
        be::database::repositories::password_reset_repository::PasswordResetTokenRepository::new(
            ctx.pool.clone(),
        );

    // Create a token
    let token = reset_repo.create_token(&user.id).await.unwrap();

    assert!(!token.id.is_empty());
    assert!(!token.token.is_empty());
    assert_eq!(token.user_id, user.id);
    assert!(token.used_at.is_none());
    assert!(token.expires_at > Utc::now().naive_utc());
}

#[tokio::test]
async fn test_password_reset_repository_find_valid_token() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "find-test@example.com".to_string(),
        "hashed_password".to_string(),
        "Find Test User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    let reset_repo =
        be::database::repositories::password_reset_repository::PasswordResetTokenRepository::new(
            ctx.pool.clone(),
        );

    // Create a token
    let created_token = reset_repo.create_token(&user.id).await.unwrap();

    // Find the token
    let found_token = reset_repo
        .find_valid_token(&created_token.token)
        .await
        .unwrap();
    assert!(found_token.is_some());

    let found = found_token.unwrap();
    assert_eq!(found.id, created_token.id);
    assert_eq!(found.token, created_token.token);
    assert_eq!(found.user_id, user.id);
    assert!(found.used_at.is_none());
}

#[tokio::test]
async fn test_password_reset_repository_mark_token_used() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "mark-used@example.com".to_string(),
        "hashed_password".to_string(),
        "Mark Used User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    let reset_repo =
        be::database::repositories::password_reset_repository::PasswordResetTokenRepository::new(
            ctx.pool.clone(),
        );

    // Create a token
    let created_token = reset_repo.create_token(&user.id).await.unwrap();

    // Mark as used
    let mark_result = reset_repo.mark_token_used(&created_token.token).await;
    assert!(mark_result.is_ok());

    // Token should no longer be valid
    let found_token = reset_repo
        .find_valid_token(&created_token.token)
        .await
        .unwrap();
    assert!(found_token.is_none());
}

#[tokio::test]
async fn test_password_reset_repository_cleanup_expired() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "cleanup@example.com".to_string(),
        "hashed_password".to_string(),
        "Cleanup User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    let reset_repo =
        be::database::repositories::password_reset_repository::PasswordResetTokenRepository::new(
            ctx.pool.clone(),
        );

    // Create a token
    let _token = reset_repo.create_token(&user.id).await.unwrap();

    // Cleanup should not remove the fresh token (it's not expired)
    let cleaned = reset_repo.cleanup_expired_tokens().await.unwrap();
    assert_eq!(cleaned, 0);
}

#[tokio::test]
async fn test_password_reset_repository_invalidate_user_tokens() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "invalidate@example.com".to_string(),
        "hashed_password".to_string(),
        "Invalidate User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    let reset_repo =
        be::database::repositories::password_reset_repository::PasswordResetTokenRepository::new(
            ctx.pool.clone(),
        );

    // Create multiple tokens
    let token1 = reset_repo.create_token(&user.id).await.unwrap();
    let token2 = reset_repo.create_token(&user.id).await.unwrap();

    // Both should be valid initially
    assert!(reset_repo
        .find_valid_token(&token1.token)
        .await
        .unwrap()
        .is_some());
    assert!(reset_repo
        .find_valid_token(&token2.token)
        .await
        .unwrap()
        .is_some());

    // Invalidate all tokens for user
    let invalidate_result = reset_repo.invalidate_user_tokens(&user.id).await;
    assert!(invalidate_result.is_ok());

    // Both tokens should now be invalid
    assert!(reset_repo
        .find_valid_token(&token1.token)
        .await
        .unwrap()
        .is_none());
    assert!(reset_repo
        .find_valid_token(&token2.token)
        .await
        .unwrap()
        .is_none());
}

#[tokio::test]
async fn test_user_repository_update_password() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "update-pwd@example.com".to_string(),
        "old_hashed_password".to_string(),
        "Update Password User".to_string(),
    );

    let user_repo =
        be::database::repositories::user_repository::UserRepository::new(ctx.pool.clone());
    user_repo.create_user(&user).await.unwrap();

    // Update password
    let new_password_hash = "new_hashed_password";
    let update_result = user_repo.update_password(&user.id, new_password_hash).await;
    assert!(update_result.is_ok());

    // Verify password was updated
    let updated_user = user_repo.find_by_id(&user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.password_hash, new_password_hash);
    assert!(updated_user.updated_at > user.updated_at);
}
