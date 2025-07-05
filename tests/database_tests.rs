use be::database::models::{User, UserRole};
use be::database::user_repository::UserRepository;
use chrono::Utc;

mod common;

#[tokio::test]
async fn test_create_user() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    let user = User::new(
        "test@example.com".to_string(),
        "hashed_password".to_string(),
        "Test User".to_string(),
        Some(UserRole::Employee),
    );

    let result = repo.create_user(&user).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_user_by_email() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    let user = User::new(
        "findme@example.com".to_string(),
        "hashed_password".to_string(),
        "Find Me".to_string(),
        Some(UserRole::Manager),
    );

    repo.create_user(&user).await.unwrap();

    let found_user = repo.find_by_email("findme@example.com").await.unwrap();
    assert!(found_user.is_some());
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.email, "findme@example.com");
    assert_eq!(found_user.name, "Find Me");
    assert!(matches!(found_user.role, UserRole::Manager));
}

#[tokio::test]
async fn test_find_user_by_id() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    let user = User::new(
        "findbyid@example.com".to_string(),
        "hashed_password".to_string(),
        "Find By ID".to_string(),
        Some(UserRole::Admin),
    );

    repo.create_user(&user).await.unwrap();

    let found_user = repo.find_by_id(&user.id).await.unwrap();
    assert!(found_user.is_some());
    
    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, "findbyid@example.com");
    assert!(matches!(found_user.role, UserRole::Admin));
}

#[tokio::test]
async fn test_email_exists() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    // Initially should not exist
    let exists = repo.email_exists("nonexistent@example.com").await.unwrap();
    assert!(!exists);

    // Create user
    let user = User::new(
        "exists@example.com".to_string(),
        "hashed_password".to_string(),
        "Exists User".to_string(),
        None,
    );

    repo.create_user(&user).await.unwrap();

    // Now should exist
    let exists = repo.email_exists("exists@example.com").await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_user_roles() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    // Test all roles
    let admin_user = User::new(
        "admin@example.com".to_string(),
        "password".to_string(),
        "Admin User".to_string(),
        Some(UserRole::Admin),
    );

    let manager_user = User::new(
        "manager@example.com".to_string(),
        "password".to_string(),
        "Manager User".to_string(),
        Some(UserRole::Manager),
    );

    let employee_user = User::new(
        "employee@example.com".to_string(),
        "password".to_string(),
        "Employee User".to_string(),
        Some(UserRole::Employee),
    );

    // Create all users
    repo.create_user(&admin_user).await.unwrap();
    repo.create_user(&manager_user).await.unwrap();
    repo.create_user(&employee_user).await.unwrap();

    // Verify roles
    let found_admin = repo.find_by_email("admin@example.com").await.unwrap().unwrap();
    let found_manager = repo.find_by_email("manager@example.com").await.unwrap().unwrap();
    let found_employee = repo.find_by_email("employee@example.com").await.unwrap().unwrap();

    assert!(matches!(found_admin.role, UserRole::Admin));
    assert!(matches!(found_manager.role, UserRole::Manager));
    assert!(matches!(found_employee.role, UserRole::Employee));
}

#[tokio::test]
async fn test_user_creation_with_default_role() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    let user = User::new(
        "default@example.com".to_string(),
        "password".to_string(),
        "Default User".to_string(),
        None, // No role specified
    );

    repo.create_user(&user).await.unwrap();

    let found_user = repo.find_by_email("default@example.com").await.unwrap().unwrap();
    assert!(matches!(found_user.role, UserRole::Employee));
}

#[tokio::test]
async fn test_duplicate_email_constraint() {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();
    let repo = UserRepository::new(ctx.pool.clone());

    let user1 = User::new(
        "duplicate@example.com".to_string(),
        "password1".to_string(),
        "User One".to_string(),
        Some(UserRole::Employee),
    );

    let user2 = User::new(
        "duplicate@example.com".to_string(),
        "password2".to_string(),
        "User Two".to_string(),
        Some(UserRole::Manager),
    );

    // First user should succeed
    repo.create_user(&user1).await.unwrap();

    // Second user with same email should fail
    let result = repo.create_user(&user2).await;
    assert!(result.is_err());
}

#[test]
fn test_user_model_creation() {
    let user = User::new(
        "model@example.com".to_string(),
        "hashed_password".to_string(),
        "Model User".to_string(),
        Some(UserRole::Admin),
    );

    assert_eq!(user.email, "model@example.com");
    assert_eq!(user.name, "Model User");
    assert_eq!(user.password_hash, "hashed_password");
    assert!(matches!(user.role, UserRole::Admin));
    assert!(!user.id.is_empty());
    
    // Check that timestamps are reasonable (within last minute)
    let now = Utc::now();
    assert!(user.created_at <= now);
    assert!(user.updated_at <= now);
    assert!(user.created_at > now - chrono::Duration::minutes(1));
}

#[test]
fn test_user_role_display() {
    assert_eq!(UserRole::Admin.to_string(), "admin");
    assert_eq!(UserRole::Manager.to_string(), "manager");
    assert_eq!(UserRole::Employee.to_string(), "employee");
}

#[test]
fn test_user_role_default() {
    let default_role = UserRole::default();
    assert!(matches!(default_role, UserRole::Employee));
}
