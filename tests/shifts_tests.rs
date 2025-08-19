use actix_web::{http::StatusCode, test, web, App};
use be::handlers::shifts;
use be::middleware::CacheLayer;
use serde_json::json;
use serial_test::serial;

mod common;

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();

            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(CacheLayer::new(1000, 60)))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/shifts")
                                .route("", web::post().to(shifts::create_shift))
                                .route("", web::get().to(shifts::get_shifts))
                                .route("/{id}", web::get().to(shifts::get_shift))
                                .route("/{id}", web::put().to(shifts::update_shift))
                                .route("/{id}", web::delete().to(shifts::delete_shift))
                                .route("/{id}/assign", web::post().to(shifts::assign_shift))
                                .route("/{id}/unassign", web::post().to(shifts::unassign_shift))
                                .route("/{id}/status", web::post().to(shifts::update_shift_status))
                                .route("/{id}/claim", web::post().to(shifts::claim_shift)),
                        ),
                    ),
            )
            .await;

            let req = test::TestRequest::$method().uri($uri).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
    ($test_name:ident, $method:ident, $uri:expr, $json:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();

            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(CacheLayer::new(1000, 60)))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/shifts")
                                .route("", web::post().to(shifts::create_shift))
                                .route("", web::get().to(shifts::get_shifts))
                                .route("/{id}", web::get().to(shifts::get_shift))
                                .route("/{id}", web::put().to(shifts::update_shift))
                                .route("/{id}", web::delete().to(shifts::delete_shift))
                                .route("/{id}/assign", web::post().to(shifts::assign_shift))
                                .route("/{id}/unassign", web::post().to(shifts::unassign_shift))
                                .route("/{id}/status", web::post().to(shifts::update_shift_status))
                                .route("/{id}/claim", web::post().to(shifts::claim_shift)),
                        ),
                    ),
            )
            .await;

            let req = test::TestRequest::$method()
                .uri($uri)
                .set_json(&$json)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
}

// Shift CRUD tests
test_unauthorized!(
    test_create_shift_unauthorized,
    post,
    "/api/v1/shifts",
    json!({
        "company_id": "00000000-0000-0000-0000-000000000000",
        "title": "Cashier",
        "description": null,
        "location_id": "00000000-0000-0000-0000-000000000000",
        "team_id": null,
        "start_time": "2024-01-01T10:00:00Z",
        "end_time": "2024-01-01T18:00:00Z",
        "min_duration_minutes": 60,
        "max_duration_minutes": 480,
        "max_people": 2,
        "status": "open",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    })
);

test_unauthorized!(test_get_shifts_unauthorized, get, "/api/v1/shifts");
test_unauthorized!(test_get_shift_unauthorized, get, "/api/v1/shifts/00000000-0000-0000-0000-000000000000");
test_unauthorized!(
    test_update_shift_unauthorized,
    put,
    "/api/v1/shifts/00000000-0000-0000-0000-000000000000",
    json!({
        "company_id": "00000000-0000-0000-0000-000000000000",
        "title": "Cashier",
        "description": null,
        "location_id": "00000000-0000-0000-0000-000000000000",
        "team_id": null,
        "start_time": "2024-01-01T10:00:00Z",
        "end_time": "2024-01-01T19:00:00Z",
        "min_duration_minutes": 60,
        "max_duration_minutes": 480,
        "max_people": 3,
        "status": "open",
        "created_at": "2024-01-01T00:00:00Z",
        "updated_at": "2024-01-01T00:00:00Z"
    })
);
test_unauthorized!(test_delete_shift_unauthorized, delete, "/api/v1/shifts/00000000-0000-0000-0000-000000000000");

// Shift assignment tests
test_unauthorized!(
    test_assign_shift_unauthorized,
    post,
    "/api/v1/shifts/00000000-0000-0000-0000-000000000000/assign",
    json!({
        "user_id": "00000000-0000-0000-0000-000000000000"
    })
);
test_unauthorized!(
    test_unassign_shift_unauthorized,
    post,
    "/api/v1/shifts/00000000-0000-0000-0000-000000000000/unassign",
    json!({})
);

// Shift status tests
test_unauthorized!(
    test_update_shift_status_unauthorized,
    post,
    "/api/v1/shifts/00000000-0000-0000-0000-000000000000/status",
    json!({
        "status": "open"
    })
);

// Shift claim tests
test_unauthorized!(
    test_claim_shift_unauthorized,
    post,
    "/api/v1/shifts/00000000-0000-0000-0000-000000000000/claim",
    json!({})
);
