#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use crate::database::models::*;
    use crate::database::user_repository::UserRepository;
    use crate::database::location_repository::LocationRepository;
    use crate::database::shift_repository::ShiftRepository;
    use crate::database::time_off_repository::TimeOffRepository;
    use actix_web::{test, web, http::StatusCode};
    use pretty_assertions::assert_eq;
    use serial_test::serial;
    use sqlx::SqlitePool;

    #[tokio::test]
    #[serial]
    async fn test_get_dashboard_stats_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test user
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;
        let token = AuthHelper::create_test_token(&user, &test_app.config)
            .expect("Failed to create test token");

        // Create test data
        let location = create_test_location(&test_app.db.pool).await;
        create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;
        create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;
        create_test_time_off_request(&test_app.db.pool, &user.id).await;

        // Act
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/dashboard")
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let stats: DashboardStats = TestAssertions::assert_success_response(body_str);

        assert_eq!(stats.total_shifts, 2);
        assert_eq!(stats.pending_time_off_requests, 1);
        assert!(stats.total_hours >= 0.0);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_shift_stats_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test user
        let manager_data = MockData::manager_user();
        let manager = create_test_user(&test_app.db.pool, &manager_data).await;
        let token = AuthHelper::create_test_token(&manager, &test_app.config)
            .expect("Failed to create test token");

        // Create test data
        let location = create_test_location(&test_app.db.pool).await;
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;

        // Create shifts with different statuses
        create_test_shift_with_status(&test_app.db.pool, location.id, Some(user.id.clone()), ShiftStatus::Assigned).await;
        create_test_shift_with_status(&test_app.db.pool, location.id, None, ShiftStatus::Open).await;
        create_test_shift_with_status(&test_app.db.pool, location.id, Some(user.id.clone()), ShiftStatus::Completed).await;

        // Act
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/shifts")
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let stats: ShiftStats = TestAssertions::assert_success_response(body_str);

        assert_eq!(stats.total_shifts, 3);
        assert_eq!(stats.assigned_shifts, 2);
        assert_eq!(stats.unassigned_shifts, 1);
        assert_eq!(stats.completed_shifts, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_time_off_stats_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let manager_data = MockData::manager_user();
        let manager = create_test_user(&test_app.db.pool, &manager_data).await;
        let token = AuthHelper::create_test_token(&manager, &test_app.config)
            .expect("Failed to create test token");

        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;

        // Create time-off requests with different statuses
        create_test_time_off_request_with_status(&test_app.db.pool, &user.id, TimeOffStatus::Pending).await;
        create_test_time_off_request_with_status(&test_app.db.pool, &user.id, TimeOffStatus::Approved).await;
        create_test_time_off_request_with_status(&test_app.db.pool, &user.id, TimeOffStatus::Denied).await;

        // Act
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/time-off")
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let stats: TimeOffStats = TestAssertions::assert_success_response(body_str);

        assert_eq!(stats.total_requests, 3);
        assert_eq!(stats.pending_requests, 1);
        assert_eq!(stats.approved_requests, 1);
        assert_eq!(stats.denied_requests, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_dashboard_stats_with_date_filter() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test user
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;
        let token = AuthHelper::create_test_token(&user, &test_app.config)
            .expect("Failed to create test token");

        // Create test data
        let location = create_test_location(&test_app.db.pool).await;
        
        // Create shifts for different date ranges
        create_test_shift_for_date(&test_app.db.pool, location.id, Some(user.id.clone()), 
            chrono::NaiveDate::from_ymd_opt(2025, 8, 1).unwrap().and_hms_opt(9, 0, 0).unwrap()).await;
        create_test_shift_for_date(&test_app.db.pool, location.id, Some(user.id.clone()), 
            chrono::NaiveDate::from_ymd_opt(2025, 9, 1).unwrap().and_hms_opt(9, 0, 0).unwrap()).await;

        // Act - Request stats for August 2025
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/dashboard?start_date=2025-08-01&end_date=2025-08-31")
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let stats: DashboardStats = TestAssertions::assert_success_response(body_str);

        // Should only include shifts from August
        assert_eq!(stats.total_shifts, 1);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_unauthorized_access() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Act & Assert - Dashboard stats without auth
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/dashboard")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Act & Assert - Shift stats without auth
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/shifts")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Act & Assert - Time-off stats without auth
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/time-off")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_stats_employee_access_restriction() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create employee user (not manager/admin)
        let employee_data = MockData::user(); // Regular employee
        let employee = create_test_user(&test_app.db.pool, &employee_data).await;
        let token = AuthHelper::create_test_token(&employee, &test_app.config)
            .expect("Failed to create test token");

        // Act & Assert - Admin-level stats should be forbidden for employees
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/shifts") // Organization-wide stats
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    #[serial]
    async fn test_get_user_specific_dashboard_stats() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let user1_data = MockData::user();
        let user1 = create_test_user(&test_app.db.pool, &user1_data).await;
        let user1_token = AuthHelper::create_test_token(&user1, &test_app.config)
            .expect("Failed to create test token");

        let user2_data = MockData::user();
        let user2 = create_test_user(&test_app.db.pool, &user2_data).await;

        let location = create_test_location(&test_app.db.pool).await;

        // Create shifts for both users
        create_test_shift(&test_app.db.pool, location.id, Some(user1.id.clone())).await;
        create_test_shift(&test_app.db.pool, location.id, Some(user1.id.clone())).await;
        create_test_shift(&test_app.db.pool, location.id, Some(user2.id.clone())).await;

        // Act - Get dashboard stats for user1
        let req = test::TestRequest::get()
            .uri("/api/v1/stats/dashboard")
            .insert_header(AuthHelper::auth_header(&user1_token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let stats: DashboardStats = TestAssertions::assert_success_response(body_str);

        // Should only include user1's shifts (2), not user2's (1)
        assert_eq!(stats.total_shifts, 2);
    }

    // Helper functions for test data creation
    async fn create_test_user(pool: &SqlitePool, user_data: &CreateUserRequest) -> User {
        let user_repo = UserRepository::new(pool.clone());
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            email: user_data.email.clone(),
            password_hash: "test_hash".to_string(),
            name: user_data.name.clone(),
            role: user_data.role.clone().unwrap_or(UserRole::Employee),
            created_at: chrono::Utc::now().naive_utc(),
            updated_at: chrono::Utc::now().naive_utc(),
        };
        user_repo.create_user(&user).await
            .expect("Failed to create test user");
        user
    }

    async fn create_test_location(pool: &SqlitePool) -> Location {
        let location_repo = LocationRepository::new(pool.clone());
        let location_data = MockData::location();
        location_repo.create_location(location_data).await
            .expect("Failed to create test location")
    }

    async fn create_test_shift(pool: &SqlitePool, location_id: i64, assigned_user_id: Option<String>) -> Shift {
        let shift_repo = ShiftRepository::new(pool.clone());
        let mut shift_data = MockData::shift(location_id, None);
        // Convert string user ID to i64 for the database
        shift_data.assigned_user_id = assigned_user_id.and_then(|id| id.parse::<i64>().ok());
        shift_repo.create_shift(shift_data).await
            .expect("Failed to create test shift")
    }

    async fn create_test_shift_with_status(pool: &SqlitePool, location_id: i64, assigned_user_id: Option<String>, status: ShiftStatus) -> Shift {
        let mut shift = create_test_shift(pool, location_id, assigned_user_id).await;
        
        // Update shift status
        let query = "UPDATE shifts SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?";
        sqlx::query(query)
            .bind(status.to_string())
            .bind(shift.id)
            .execute(pool)
            .await
            .expect("Failed to update shift status");
            
        shift.status = status;
        shift
    }

    async fn create_test_shift_for_date(pool: &SqlitePool, location_id: i64, assigned_user_id: Option<String>, start_time: chrono::NaiveDateTime) -> Shift {
        let shift_repo = ShiftRepository::new(pool.clone());
        let shift_data = ShiftInput {
            title: "Test Shift".to_string(),
            description: Some("Test shift".to_string()),
            location_id,
            team_id: None,
            assigned_user_id: assigned_user_id.and_then(|id| id.parse::<i64>().ok()),
            start_time,
            end_time: start_time + chrono::Duration::hours(8),
            hourly_rate: Some(20.0),
        };
        shift_repo.create_shift(shift_data).await
            .expect("Failed to create test shift")
    }

    async fn create_test_time_off_request(pool: &SqlitePool, user_id: &str) -> TimeOffRequest {
        let time_off_repo = TimeOffRepository::new(pool.clone());
        let request_data = MockData::time_off_request(user_id.to_string());
        time_off_repo.create_request(request_data).await
            .expect("Failed to create test time-off request")
    }

    async fn create_test_time_off_request_with_status(pool: &SqlitePool, user_id: &str, status: TimeOffStatus) -> TimeOffRequest {
        let mut request = create_test_time_off_request(pool, user_id).await;
        
        // Update request status
        let query = "UPDATE time_off_requests SET status = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?";
        sqlx::query(query)
            .bind(status.to_string())
            .bind(request.id)
            .execute(pool)
            .await
            .expect("Failed to update time-off request status");
            
        request.status = status;
        request
    }
}
