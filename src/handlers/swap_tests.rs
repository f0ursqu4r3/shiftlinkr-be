#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use actix_web::{test, web, http::StatusCode};
    use pretty_assertions::assert_eq;
    use serial_test::serial;

    #[tokio::test]
    #[serial]
    async fn test_create_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test data
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;
        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;
        
        let token = AuthHelper::create_test_token(&user, &test_app.config)
            .expect("Failed to create test token");

        let swap_data = MockData::shift_swap(shift.id, user.id.clone());

        // Act
        let req = test::TestRequest::post()
            .uri("/api/v1/swaps")
            .insert_header(AuthHelper::auth_header(&token))
            .set_json(&swap_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let created_swap: ShiftSwap = TestAssertions::assert_success_response(body_str);

        assert_eq!(created_swap.original_shift_id, shift.id);
        assert_eq!(created_swap.requesting_user_id, user.id);
        assert_eq!(created_swap.status, ShiftSwapStatus::Open);

        // Verify in database
        TestAssertions::assert_record_count(&test_app.db.pool, "shift_swaps", 1).await;
    }

    #[tokio::test]
    #[serial]
    async fn test_create_targeted_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let requesting_user_data = MockData::user();
        let requesting_user = create_test_user(&test_app.db.pool, &requesting_user_data).await;
        
        let target_user_data = MockData::user();
        let target_user = create_test_user(&test_app.db.pool, &target_user_data).await;

        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(requesting_user.id.clone())).await;
        
        let token = AuthHelper::create_test_token(&requesting_user, &test_app.config)
            .expect("Failed to create test token");

        let swap_data = ShiftSwapInput {
            original_shift_id: shift.id,
            requesting_user_id: requesting_user.id.clone(),
            target_user_id: Some(target_user.id.clone()),
            notes: Some("Need to swap due to appointment".to_string()),
            swap_type: ShiftSwapType::Targeted,
        };

        // Act
        let req = test::TestRequest::post()
            .uri("/api/v1/swaps")
            .insert_header(AuthHelper::auth_header(&token))
            .set_json(&swap_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::CREATED);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let created_swap: ShiftSwap = TestAssertions::assert_success_response(body_str);

        assert_eq!(created_swap.swap_type, ShiftSwapType::Targeted);
        assert_eq!(created_swap.target_user_id, Some(target_user.id));
        assert_eq!(created_swap.status, ShiftSwapStatus::Pending);
    }

    #[tokio::test]
    #[serial]
    async fn test_respond_to_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let requesting_user_data = MockData::user();
        let requesting_user = create_test_user(&test_app.db.pool, &requesting_user_data).await;
        
        let responding_user_data = MockData::user();
        let responding_user = create_test_user(&test_app.db.pool, &responding_user_data).await;
        let responding_token = AuthHelper::create_test_token(&responding_user, &test_app.config)
            .expect("Failed to create test token");

        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(requesting_user.id.clone())).await;
        let swap_request = create_test_shift_swap(&test_app.db.pool, shift.id, &requesting_user.id).await;

        let response_data = serde_json::json!({
            "action": "accept",
            "notes": "I can take this shift"
        });

        // Act
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/swaps/{}/respond", swap_request.id))
            .insert_header(AuthHelper::auth_header(&responding_token))
            .set_json(&response_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let updated_swap: ShiftSwap = TestAssertions::assert_success_response(body_str);

        assert_eq!(updated_swap.status, ShiftSwapStatus::Pending);
        assert_eq!(updated_swap.target_user_id, Some(responding_user.id));
    }

    #[tokio::test]
    #[serial]
    async fn test_approve_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let requesting_user_data = MockData::user();
        let requesting_user = create_test_user(&test_app.db.pool, &requesting_user_data).await;
        
        let target_user_data = MockData::user();
        let target_user = create_test_user(&test_app.db.pool, &target_user_data).await;

        let manager_data = MockData::manager_user();
        let manager = create_test_user(&test_app.db.pool, &manager_data).await;
        let manager_token = AuthHelper::create_test_token(&manager, &test_app.config)
            .expect("Failed to create test token");

        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(requesting_user.id.clone())).await;
        let mut swap_request = create_test_shift_swap(&test_app.db.pool, shift.id, &requesting_user.id).await;
        
        // Update swap to pending with target user
        update_swap_status(&test_app.db.pool, swap_request.id, ShiftSwapStatus::Pending, Some(&target_user.id)).await;

        let approval_data = ApprovalRequest {
            notes: Some("Swap approved - adequate coverage maintained".to_string()),
        };

        // Act
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/swaps/{}/approve", swap_request.id))
            .insert_header(AuthHelper::auth_header(&manager_token))
            .set_json(&approval_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let updated_swap: ShiftSwap = TestAssertions::assert_success_response(body_str);

        assert_eq!(updated_swap.status, ShiftSwapStatus::Approved);
        assert_eq!(updated_swap.approved_by, Some(manager.id));
    }

    #[tokio::test]
    #[serial]
    async fn test_deny_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test users
        let requesting_user_data = MockData::user();
        let requesting_user = create_test_user(&test_app.db.pool, &requesting_user_data).await;

        let manager_data = MockData::manager_user();
        let manager = create_test_user(&test_app.db.pool, &manager_data).await;
        let manager_token = AuthHelper::create_test_token(&manager, &test_app.config)
            .expect("Failed to create test token");

        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(requesting_user.id.clone())).await;
        let swap_request = create_test_shift_swap(&test_app.db.pool, shift.id, &requesting_user.id).await;

        let denial_data = DenialRequest {
            notes: "Cannot approve swap - would leave shift uncovered".to_string(),
        };

        // Act
        let req = test::TestRequest::post()
            .uri(&format!("/api/v1/swaps/{}/deny", swap_request.id))
            .insert_header(AuthHelper::auth_header(&manager_token))
            .set_json(&denial_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let updated_swap: ShiftSwap = TestAssertions::assert_success_response(body_str);

        assert_eq!(updated_swap.status, ShiftSwapStatus::Denied);
        assert_eq!(updated_swap.approved_by, Some(manager.id));
        assert_eq!(updated_swap.approval_notes, Some(denial_data.notes));
    }

    #[tokio::test]
    #[serial]
    async fn test_get_shift_swap_requests_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test user
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;
        let token = AuthHelper::create_test_token(&user, &test_app.config)
            .expect("Failed to create test token");

        let location = create_test_location(&test_app.db.pool).await;
        let shift1 = create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;
        let shift2 = create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;

        // Create multiple swap requests
        create_test_shift_swap(&test_app.db.pool, shift1.id, &user.id).await;
        create_test_shift_swap(&test_app.db.pool, shift2.id, &user.id).await;

        // Act
        let req = test::TestRequest::get()
            .uri("/api/v1/swaps")
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::OK);

        let body = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body).unwrap();
        let swaps: Vec<ShiftSwap> = TestAssertions::assert_success_response(body_str);
        
        assert_eq!(swaps.len(), 2);
    }

    #[tokio::test]
    #[serial]
    async fn test_cancel_shift_swap_request_success() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        // Create test user
        let user_data = MockData::user();
        let user = create_test_user(&test_app.db.pool, &user_data).await;
        let token = AuthHelper::create_test_token(&user, &test_app.config)
            .expect("Failed to create test token");

        let location = create_test_location(&test_app.db.pool).await;
        let shift = create_test_shift(&test_app.db.pool, location.id, Some(user.id.clone())).await;
        let swap_request = create_test_shift_swap(&test_app.db.pool, shift.id, &user.id).await;

        // Act
        let req = test::TestRequest::delete()
            .uri(&format!("/api/v1/swaps/{}", swap_request.id))
            .insert_header(AuthHelper::auth_header(&token))
            .to_request();

        let resp = test::call_service(&mut app, req).await;

        // Assert
        assert_eq!(resp.status(), StatusCode::NO_CONTENT);

        // Verify status updated to cancelled in database
        let updated_swap = get_swap_by_id(&test_app.db.pool, swap_request.id).await;
        assert_eq!(updated_swap.status, ShiftSwapStatus::Cancelled);
    }

    #[tokio::test]
    #[serial]
    async fn test_unauthorized_access_to_swap_endpoints() {
        // Arrange
        let test_app = TestApp::new().await.expect("Failed to create test app");
        let app = test_app.create_app().await;
        let mut app = test::init_service(app).await;

        let swap_data = serde_json::json!({
            "original_shift_id": 1,
            "requesting_user_id": "user123",
            "swap_type": "open"
        });

        // Act & Assert - Create swap without auth
        let req = test::TestRequest::post()
            .uri("/api/v1/swaps")
            .set_json(&swap_data)
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

        // Act & Assert - Get swaps without auth
        let req = test::TestRequest::get()
            .uri("/api/v1/swaps")
            .to_request();

        let resp = test::call_service(&mut app, req).await;
        assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
    }

    // Helper functions for test data creation
    async fn create_test_user(pool: &SqlitePool, user_data: &CreateUserRequest) -> User {
        let user_repo = UserRepository::new(pool.clone());
        user_repo.create_user(user_data).await
            .expect("Failed to create test user")
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
        shift_data.assigned_user_id = assigned_user_id;
        shift_repo.create_shift(shift_data).await
            .expect("Failed to create test shift")
    }

    async fn create_test_shift_swap(pool: &SqlitePool, shift_id: i64, user_id: &str) -> ShiftSwap {
        let swap_repo = ShiftSwapRepository::new(pool.clone());
        let swap_data = MockData::shift_swap(shift_id, user_id.to_string());
        swap_repo.create_swap_request(swap_data).await
            .expect("Failed to create test shift swap")
    }

    async fn update_swap_status(pool: &SqlitePool, swap_id: i64, status: ShiftSwapStatus, target_user_id: Option<&str>) {
        let query = "UPDATE shift_swaps SET status = ?, target_user_id = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?";
        sqlx::query(query)
            .bind(status.to_string())
            .bind(target_user_id)
            .bind(swap_id)
            .execute(pool)
            .await
            .expect("Failed to update swap status");
    }

    async fn get_swap_by_id(pool: &SqlitePool, swap_id: i64) -> ShiftSwap {
        let query = "SELECT * FROM shift_swaps WHERE id = ?";
        sqlx::query_as(query)
            .bind(swap_id)
            .fetch_one(pool)
            .await
            .expect("Failed to get swap by id")
    }
}
