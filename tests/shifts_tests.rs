use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::company_repository::CompanyRepository;
use be::database::repositories::shift_repository::ShiftRepository;
use be::handlers::shifts;
use be::{ActivityLogger, ActivityRepository, AppState};
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (
    web::Data<AppState>,
    web::Data<ShiftRepository>,
    web::Data<be::Config>,
) {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let shift_repo_data = web::Data::new(ShiftRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

    (app_state, shift_repo_data, config_data)
}

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, shift_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(shift_repo_data)
                    .app_data(config_data)
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
                                .route("/{id}/status", web::put().to(shifts::update_shift_status))
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
            let (app_state, shift_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(shift_repo_data)
                    .app_data(config_data)
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
                                .route("/{id}/status", web::put().to(shifts::update_shift_status))
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
        "location_id": 1,
        "team_id": 1,
        "start_time": "2024-01-01T10:00:00Z",
        "end_time": "2024-01-01T18:00:00Z",
        "role": "cashier",
        "capacity": 2
    })
);

test_unauthorized!(test_get_shifts_unauthorized, get, "/api/v1/shifts");
test_unauthorized!(test_get_shift_unauthorized, get, "/api/v1/shifts/1");
test_unauthorized!(
    test_update_shift_unauthorized,
    put,
    "/api/v1/shifts/1",
    json!({
        "location_id": 1,
        "team_id": 1,
        "start_time": "2024-01-01T10:00:00Z",
        "end_time": "2024-01-01T19:00:00Z",
        "role": "cashier",
        "capacity": 3
    })
);
test_unauthorized!(test_delete_shift_unauthorized, delete, "/api/v1/shifts/1");

// Shift assignment tests
test_unauthorized!(
    test_assign_shift_unauthorized,
    post,
    "/api/v1/shifts/1/assign",
    json!({
        "user_id": "user123"
    })
);
test_unauthorized!(
    test_unassign_shift_unauthorized,
    post,
    "/api/v1/shifts/1/unassign",
    json!({
        "user_id": "user123"
    })
);

// Shift status tests
test_unauthorized!(
    test_update_shift_status_unauthorized,
    put,
    "/api/v1/shifts/1/status",
    json!({
        "status": "completed"
    })
);

// Shift claim tests
test_unauthorized!(
    test_claim_shift_unauthorized,
    post,
    "/api/v1/shifts/1/claim",
    json!({})
);
