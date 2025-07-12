use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::company_repository::CompanyRepository;
use be::database::repositories::time_off_repository::TimeOffRepository;
use be::handlers::time_off;
use be::{ActivityLogger, ActivityRepository, AppState};
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (
    web::Data<AppState>,
    web::Data<TimeOffRepository>,
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
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

    (app_state, time_off_repo_data, config_data)
}

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, time_off_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(time_off_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/time-off")
                                .route("", web::post().to(time_off::create_time_off_request))
                                .route("", web::get().to(time_off::get_time_off_requests))
                                .route("/{id}", web::get().to(time_off::get_time_off_request))
                                .route("/{id}", web::put().to(time_off::update_time_off_request))
                                .route("/{id}", web::delete().to(time_off::delete_time_off_request))
                                .route(
                                    "/{id}/approve",
                                    web::post().to(time_off::approve_time_off_request),
                                )
                                .route(
                                    "/{id}/deny",
                                    web::post().to(time_off::deny_time_off_request),
                                ),
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
            let (app_state, time_off_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(time_off_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/time-off")
                                .route("", web::post().to(time_off::create_time_off_request))
                                .route("", web::get().to(time_off::get_time_off_requests))
                                .route("/{id}", web::get().to(time_off::get_time_off_request))
                                .route("/{id}", web::put().to(time_off::update_time_off_request))
                                .route("/{id}", web::delete().to(time_off::delete_time_off_request))
                                .route(
                                    "/{id}/approve",
                                    web::post().to(time_off::approve_time_off_request),
                                )
                                .route(
                                    "/{id}/deny",
                                    web::post().to(time_off::deny_time_off_request),
                                ),
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

// Time off request CRUD tests
test_unauthorized!(
    test_create_time_off_request_unauthorized,
    post,
    "/api/v1/time-off",
    json!({
        "user_id": "test_user",
        "start_date": "2024-01-01",
        "end_date": "2024-01-05",
        "reason": "Test vacation",
        "request_type": "Vacation"
    })
);

test_unauthorized!(
    test_get_time_off_requests_unauthorized,
    get,
    "/api/v1/time-off"
);
test_unauthorized!(
    test_get_time_off_request_unauthorized,
    get,
    "/api/v1/time-off/123"
);
test_unauthorized!(
    test_update_time_off_request_unauthorized,
    put,
    "/api/v1/time-off/123",
    json!({
        "user_id": "test_user",
        "start_date": "2024-01-01",
        "end_date": "2024-01-05",
        "reason": "Updated vacation",
        "request_type": "Vacation"
    })
);
test_unauthorized!(
    test_delete_time_off_request_unauthorized,
    delete,
    "/api/v1/time-off/123"
);

// Time off request approval tests
test_unauthorized!(
    test_approve_time_off_request_unauthorized,
    post,
    "/api/v1/time-off/123/approve",
    json!({
        "notes": "Approved for vacation"
    })
);
test_unauthorized!(
    test_deny_time_off_request_unauthorized,
    post,
    "/api/v1/time-off/123/deny",
    json!({
        "notes": "Insufficient coverage during requested period"
    })
);
