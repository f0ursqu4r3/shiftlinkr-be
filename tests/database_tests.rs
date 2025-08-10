use be::database::models::User;
use be::database::repositories::user as user_repo;
use chrono::Utc;
use uuid::Uuid;

mod common;

#[tokio::test]
async fn test_create_user() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let user = User {
        id: Uuid::new_v4(),
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let result = user_repo::create_user(&user).await;
    if let Err(ref e) = result {
        eprintln!("Error creating user: {}", e);
    }
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_find_user_by_email() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let user = User {
        id: Uuid::new_v4(),
        name: "Find Me".to_string(),
        email: "findme@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    user_repo::create_user(&user).await.unwrap();

    let found_user = user_repo::find_by_email("findme@example.com")
        .await
        .unwrap();
    assert!(found_user.is_some());

    let found_user = found_user.unwrap();
    assert_eq!(found_user.email, "findme@example.com");
    assert_eq!(found_user.name, "Find Me");
}

#[tokio::test]
async fn test_find_user_by_id() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let user = User {
        id: Uuid::new_v4(),
        name: "Find By ID".to_string(),
        email: "findbyid@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    user_repo::create_user(&user).await.unwrap();

    let found_user = user_repo::find_by_id(user.id).await.unwrap();
    assert!(found_user.is_some());

    let found_user = found_user.unwrap();
    assert_eq!(found_user.id, user.id);
    assert_eq!(found_user.email, "findbyid@example.com");
}

#[tokio::test]
async fn test_email_exists() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Initially should not exist
    let exists = user_repo::email_exists("nonexistent@example.com")
        .await
        .unwrap();
    assert!(!exists);

    // Create user
    let user = User {
        id: Uuid::new_v4(),
        name: "Exists User".to_string(),
        email: "exists@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    user_repo::create_user(&user).await.unwrap();

    // Now should exist
    let exists = user_repo::email_exists("exists@example.com").await.unwrap();
    assert!(exists);
}

#[tokio::test]
async fn test_multiple_users() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    #[allow(unused_variables)]
    let _repo = ();

    // Test multiple users
    let admin_user = User::new(
        "admin@example.com".to_string(),
        "password".to_string(),
        "Admin User".to_string(),
    );

    let manager_user = User::new(
        "manager@example.com".to_string(),
        "password".to_string(),
        "Manager User".to_string(),
    );

    let employee_user = User::new(
        "employee@example.com".to_string(),
        "password".to_string(),
        "Employee User".to_string(),
    );

    // Create all users
    user_repo::create_user(&admin_user).await.unwrap();
    user_repo::create_user(&manager_user).await.unwrap();
    user_repo::create_user(&employee_user).await.unwrap();

    // Verify users can be found
    let found_admin = user_repo::find_by_email("admin@example.com")
        .await
        .unwrap()
        .unwrap();
    let found_manager = user_repo::find_by_email("manager@example.com")
        .await
        .unwrap()
        .unwrap();
    let found_employee = user_repo::find_by_email("employee@example.com")
        .await
        .unwrap()
        .unwrap();

    assert_eq!(found_admin.name, "Admin User");
    assert_eq!(found_manager.name, "Manager User");
    assert_eq!(found_employee.name, "Employee User");
}

#[tokio::test]
async fn test_user_creation_basic() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    #[allow(unused_variables)]
    let _repo = ();

    let user = User::new(
        "basic@example.com".to_string(),
        "password".to_string(),
        "Basic User".to_string(),
    );

    user_repo::create_user(&user).await.unwrap();

    let found_user = user_repo::find_by_email("basic@example.com")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(found_user.name, "Basic User");
}

#[tokio::test]
async fn test_duplicate_email_constraint() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    let user1 = User::new(
        "duplicate@example.com".to_string(),
        "password1".to_string(),
        "User One".to_string(),
    );

    let user2 = User::new(
        "duplicate@example.com".to_string(),
        "password2".to_string(),
        "User Two".to_string(),
    );

    // First user should succeed
    user_repo::create_user(&user1).await.unwrap();

    // Second user with same email should fail
    let result = user_repo::create_user(&user2).await;
    assert!(result.is_err());
}

#[test]
fn test_user_model_creation() {
    let user = User::new(
        "model@example.com".to_string(),
        "hashed_password".to_string(),
        "Model User".to_string(),
    );

    assert_eq!(user.email, "model@example.com");
    assert_eq!(user.name, "Model User");
    assert_eq!(user.password_hash, "hashed_password");
    assert!(!user.id.to_string().is_empty());

    // Check that timestamps are reasonable (within last minute)
    let now = Utc::now();
    assert!(user.created_at <= now);
    assert!(user.updated_at <= now);
    assert!(user.created_at > now - chrono::Duration::minutes(1));
}
