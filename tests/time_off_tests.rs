mod common;

use actix_web::{http::StatusCode, test};
use be::database::models::*;
use be::handlers::time_off::{ApprovalRequest, DenialRequest};
use common::*;
use pretty_assertions::assert_eq;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_time_off_request_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test user
    let user_data = MockData::user();
    let user = create_test_user(&test_app.db.pool, &user_data).await;
    let token = AuthHelper::create_test_token(&user, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request data
    let time_off_data = MockData::time_off_request(user.id.clone());

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/time-off")
        .insert_header(AuthHelper::auth_header(&token))
        .set_json(&time_off_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let _created_request: TimeOffRequest = TestAssertions::assert_success_response(body_str);

    // Verify in database
    TestAssertions::assert_record_count(&test_app.db.pool, "time_off_requests", 1).await;
}

#[tokio::test]
#[serial]
async fn test_create_time_off_request_unauthorized() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    let time_off_data = MockData::time_off_request("user123".to_string());

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/time-off")
        .set_json(&time_off_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
#[serial]
async fn test_get_time_off_requests_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test user and time-off requests
    let user_data = MockData::user();
    let user = create_test_user(&test_app.db.pool, &user_data).await;
    let token = AuthHelper::create_test_token(&user, &test_app.config)
        .expect("Failed to create test token");

    // Create multiple time-off requests
    create_test_time_off_request(&test_app.db.pool, &user.id).await;
    create_test_time_off_request(&test_app.db.pool, &user.id).await;

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/time-off")
        .insert_header(AuthHelper::auth_header(&token))
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let requests: Vec<TimeOffRequest> = TestAssertions::assert_success_response(body_str);

    assert_eq!(requests.len(), 2);
}

#[tokio::test]
#[serial]
async fn test_approve_time_off_request_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test users
    let employee_data = MockData::user();
    let employee = create_test_user(&test_app.db.pool, &employee_data).await;

    let manager_data = MockData::manager_user();
    let manager = create_test_user(&test_app.db.pool, &manager_data).await;
    let manager_token = AuthHelper::create_test_token(&manager, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request
    let time_off_request = create_test_time_off_request(&test_app.db.pool, &employee.id).await;

    let approval_data = ApprovalRequest {
        notes: Some("Approved for vacation".to_string()),
    };

    // Act
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/time-off/{}/approve", time_off_request.id))
        .insert_header(AuthHelper::auth_header(&manager_token))
        .set_json(&approval_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let updated_request: TimeOffRequest = TestAssertions::assert_success_response(body_str);

    assert_eq!(updated_request.status, TimeOffStatus::Approved);
    assert_eq!(updated_request.approved_by, Some(manager.id));
}

#[tokio::test]
#[serial]
async fn test_deny_time_off_request_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test users
    let employee_data = MockData::user();
    let employee = create_test_user(&test_app.db.pool, &employee_data).await;

    let manager_data = MockData::manager_user();
    let manager = create_test_user(&test_app.db.pool, &manager_data).await;
    let manager_token = AuthHelper::create_test_token(&manager, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request
    let time_off_request = create_test_time_off_request(&test_app.db.pool, &employee.id).await;

    let denial_data = DenialRequest {
        notes: "Insufficient coverage during requested period".to_string(),
    };

    // Act
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/time-off/{}/deny", time_off_request.id))
        .insert_header(AuthHelper::auth_header(&manager_token))
        .set_json(&denial_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let updated_request: TimeOffRequest = TestAssertions::assert_success_response(body_str);

    assert_eq!(updated_request.status, TimeOffStatus::Denied);
    assert_eq!(updated_request.approved_by, Some(manager.id));
    assert_eq!(updated_request.approval_notes, Some(denial_data.notes));
}

#[tokio::test]
#[serial]
async fn test_approve_time_off_request_forbidden_for_employee() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test users
    let employee_data = MockData::user();
    let employee = create_test_user(&test_app.db.pool, &employee_data).await;
    let employee_token = AuthHelper::create_test_token(&employee, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request
    let time_off_request = create_test_time_off_request(&test_app.db.pool, &employee.id).await;

    let approval_data = ApprovalRequest {
        notes: Some("Self-approval attempt".to_string()),
    };

    // Act
    let req = test::TestRequest::post()
        .uri(&format!("/api/v1/time-off/{}/approve", time_off_request.id))
        .insert_header(AuthHelper::auth_header(&employee_token))
        .set_json(&approval_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
#[serial]
async fn test_update_time_off_request_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test user
    let user_data = MockData::user();
    let user = create_test_user(&test_app.db.pool, &user_data).await;
    let token = AuthHelper::create_test_token(&user, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request
    let time_off_request = create_test_time_off_request(&test_app.db.pool, &user.id).await;

    let update_data = TimeOffRequestInput {
        user_id: user.id.clone(),
        start_date: time_off_request.start_date + chrono::Duration::days(1),
        end_date: time_off_request.end_date + chrono::Duration::days(1),
        reason: "Updated vacation request".to_string(),
        request_type: TimeOffType::Vacation,
    };

    // Act
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/time-off/{}", time_off_request.id))
        .insert_header(AuthHelper::auth_header(&token))
        .set_json(&update_data)
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::OK);

    let body = test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body).unwrap();
    let updated_request: TimeOffRequest = TestAssertions::assert_success_response(body_str);

    assert_eq!(updated_request.reason, update_data.reason);
}

#[tokio::test]
#[serial]
async fn test_delete_time_off_request_success() {
    // Arrange
    let test_app = TestApp::new().await.expect("Failed to create test app");
    let app = test_app.create_app().await;
    let mut app = test::init_service(app).await;

    // Create test user
    let user_data = MockData::user();
    let user = create_test_user(&test_app.db.pool, &user_data).await;
    let token = AuthHelper::create_test_token(&user, &test_app.config)
        .expect("Failed to create test token");

    // Create time-off request
    let time_off_request = create_test_time_off_request(&test_app.db.pool, &user.id).await;

    // Act
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/time-off/{}", time_off_request.id))
        .insert_header(AuthHelper::auth_header(&token))
        .to_request();

    let resp = test::call_service(&mut app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);

    // Verify deleted from database
    TestAssertions::assert_record_count(&test_app.db.pool, "time_off_requests", 0).await;
}

// Helper functions for test data creation
async fn create_test_user(pool: &SqlitePool, user_data: &CreateUserRequest) -> User {
    let user_repo = UserRepository::new(pool.clone());
    user_repo
        .create_user(user_data)
        .await
        .expect("Failed to create test user")
}

async fn create_test_time_off_request(pool: &SqlitePool, user_id: &str) -> TimeOffRequest {
    let time_off_repo = TimeOffRepository::new(pool.clone());
    let request_data = MockData::time_off_request(user_id.to_string());
    time_off_repo
        .create_request(request_data)
        .await
        .expect("Failed to create test time-off request")
}
