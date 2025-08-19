use be::database::models::CreateUserInput;
use be::database::repositories::{password_reset as reset_repo, user as user_repo};
use be::database::transaction::DatabaseTransaction;
use be::services::auth as auth_service;
use chrono::Utc;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_forgot_password_valid_email() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // First create a user
    let register_request = CreateUserInput {
        email: "forgot@example.com".to_string(),
        password: "password123".to_string(),
        name: "Forgot User".to_string(),
    };

    let registration = auth_service::register(register_request).await;
    assert!(registration.is_ok());

    // Test forgot password
    let result = auth_service::forgot_password("forgot@example.com").await;
    assert!(result.is_ok());

    let token = result.unwrap();
    assert!(!token.is_empty());
    assert_eq!(token.len(), 36); // UUID length
}

#[tokio::test]
async fn test_forgot_password_invalid_email() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Test forgot password with non-existent email
    let result = auth_service::forgot_password("nonexistent@example.com").await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("User not found"));
}

#[tokio::test]
async fn test_reset_password_valid_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserInput {
        email: "reset@example.com".to_string(),
        password: "oldpassword123".to_string(),
        name: "Reset User".to_string(),
    };

    let registration = auth_service::register(register_request).await;
    assert!(registration.is_ok());

    // Request password reset
    let token = auth_service::forgot_password("reset@example.com")
        .await
        .unwrap();

    // Reset password with valid token
    let result = auth_service::reset_password(&token, "newpassword123").await;
    assert!(result.is_ok());

    // Verify old password doesn't work
    let old_login = auth_service::login(be::database::models::LoginInput {
        email: "reset@example.com".to_string(),
        password: "oldpassword123".to_string(),
    })
    .await;
    assert!(old_login.is_err());

    // Verify new password works
    let new_login = auth_service::login(be::database::models::LoginInput {
        email: "reset@example.com".to_string(),
        password: "newpassword123".to_string(),
    })
    .await;
    assert!(new_login.is_ok());
}

#[tokio::test]
async fn test_reset_password_invalid_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Test reset with invalid token
    let result = auth_service::reset_password("invalid-token", "newpassword123").await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("Invalid or expired reset token")
    );
}

#[tokio::test]
async fn test_reset_password_used_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserInput {
        email: "used@example.com".to_string(),
        password: "password123".to_string(),
        name: "Used Token User".to_string(),
    };

    let registration = auth_service::register(register_request).await;
    assert!(registration.is_ok());

    // Request password reset
    let token = auth_service::forgot_password("used@example.com")
        .await
        .unwrap();

    // Use the token once
    let first_reset = auth_service::reset_password(&token, "newpassword123").await;
    assert!(first_reset.is_ok());

    // Try to use the same token again
    let second_reset = auth_service::reset_password(&token, "anotherpassword123").await;
    assert!(second_reset.is_err());
    assert!(
        second_reset
            .unwrap_err()
            .to_string()
            .contains("Invalid or expired reset token")
    );
}

#[tokio::test]
async fn test_multiple_reset_tokens_invalidated() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let register_request = CreateUserInput {
        email: "multiple@example.com".to_string(),
        password: "password123".to_string(),
        name: "Multiple Tokens User".to_string(),
    };

    let registration = auth_service::register(register_request).await;
    assert!(registration.is_ok());

    // Request multiple password resets
    let token1 = auth_service::forgot_password("multiple@example.com")
        .await
        .unwrap();
    let token2 = auth_service::forgot_password("multiple@example.com")
        .await
        .unwrap();

    // Use the second token
    let reset_result = auth_service::reset_password(&token2, "newpassword123").await;
    assert!(reset_result.is_ok());

    // First token should now be invalid (all tokens for user should be invalidated)
    let old_token_result = auth_service::reset_password(&token1, "anotherpassword123").await;
    assert!(old_token_result.is_err());
}

#[tokio::test]
async fn test_password_reset_repository_create_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user first
    let user = be::database::models::User::new(
        "repo-test@example.com".to_string(),
        "hashed_password".to_string(),
        "Repo Test User".to_string(),
    );

    // Create user and token transactionally
    let token = DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            let token = reset_repo::create_token(tx, u.id).await?;
            Ok::<_, be::error::AppError>(token)
        })
    })
    .await
    .unwrap();

    assert_ne!(token.id, Uuid::nil());
    assert!(!token.token.is_empty());
    assert_eq!(token.user_id, user.id);
    assert!(token.used_at.is_none());
    assert!(token.expires_at > Utc::now());
}

#[tokio::test]
async fn test_password_reset_repository_find_valid_token() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "find-test@example.com".to_string(),
        "hashed_password".to_string(),
        "Find Test User".to_string(),
    );

    let created_token = DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            let token = reset_repo::create_token(tx, u.id).await?;
            Ok::<_, be::error::AppError>(token)
        })
    })
    .await
    .unwrap();

    // Find the token
    let found_token = reset_repo::find_valid_token(&created_token.token)
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
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "mark-used@example.com".to_string(),
        "hashed_password".to_string(),
        "Mark Used User".to_string(),
    );

    let created_token = DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            let token = reset_repo::create_token(tx, u.id).await?;
            Ok::<_, be::error::AppError>(token)
        })
    })
    .await
    .unwrap();

    // Mark as used
    let mark_result = DatabaseTransaction::run(|tx| {
        let t = created_token.token.clone();
        Box::pin(
            async move { Ok::<_, be::error::AppError>(reset_repo::mark_token_used(tx, &t).await?) },
        )
    })
    .await;
    assert!(mark_result.is_ok());

    // Token should no longer be valid
    let found_token = reset_repo::find_valid_token(&created_token.token)
        .await
        .unwrap();
    assert!(found_token.is_none());
}

#[tokio::test]
async fn test_password_reset_repository_cleanup_expired() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "cleanup@example.com".to_string(),
        "hashed_password".to_string(),
        "Cleanup User".to_string(),
    );

    DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            let _token = reset_repo::create_token(tx, u.id).await?;
            Ok::<_, be::error::AppError>(())
        })
    })
    .await
    .unwrap();

    // Cleanup should not remove the fresh token (it's not expired)
    let cleaned = DatabaseTransaction::run(|tx| {
        Box::pin(async move {
            Ok::<_, be::error::AppError>(reset_repo::cleanup_expired_tokens(tx).await?)
        })
    })
    .await
    .unwrap();
    assert_eq!(cleaned, 0);
}

#[tokio::test]
async fn test_password_reset_repository_invalidate_user_tokens() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "invalidate@example.com".to_string(),
        "hashed_password".to_string(),
        "Invalidate User".to_string(),
    );

    // Create multiple tokens
    let (token1, token2) = DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move {
            user_repo::create_user(tx, &u).await?;
            let t1 = reset_repo::create_token(tx, u.id).await?;
            let t2 = reset_repo::create_token(tx, u.id).await?;
            Ok::<_, be::error::AppError>((t1, t2))
        })
    })
    .await
    .unwrap();

    // Both should be valid initially
    assert!(
        reset_repo::find_valid_token(&token1.token)
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        reset_repo::find_valid_token(&token2.token)
            .await
            .unwrap()
            .is_some()
    );

    // Invalidate all tokens for user
    let invalidate_result = DatabaseTransaction::run(|tx| {
        let uid = user.id;
        Box::pin(async move {
            Ok::<_, be::error::AppError>(reset_repo::invalidate_user_tokens(tx, uid).await?)
        })
    })
    .await;
    assert!(invalidate_result.is_ok());

    // Both tokens should now be invalid
    assert!(
        reset_repo::find_valid_token(&token1.token)
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        reset_repo::find_valid_token(&token2.token)
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn test_user_repository_update_password() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a user
    let user = be::database::models::User::new(
        "update-pwd@example.com".to_string(),
        "old_hashed_password".to_string(),
        "Update Password User".to_string(),
    );

    DatabaseTransaction::run(|tx| {
        let u = user.clone();
        Box::pin(async move { Ok::<_, be::error::AppError>(user_repo::create_user(tx, &u).await?) })
    })
    .await
    .unwrap();

    // Update password
    let new_password_hash = "new_hashed_password";
    let update_result = DatabaseTransaction::run(|tx| {
        let uid = user.id;
        let nh = new_password_hash.to_string();
        Box::pin(async move {
            Ok::<_, be::error::AppError>(user_repo::update_password(tx, uid, &nh).await?)
        })
    })
    .await;
    assert!(update_result.is_ok());

    // Verify password was updated
    let updated_user = user_repo::find_by_id(user.id).await.unwrap().unwrap();
    assert_eq!(updated_user.password_hash, new_password_hash);
    assert!(updated_user.updated_at > user.updated_at);
}
