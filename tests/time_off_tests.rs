use actix_web::{http::StatusCode, test, web, App};
use be::handlers::time_off;
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
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();

            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(CacheLayer::new(1000, 60)))
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
        "userId": "00000000-0000-0000-0000-000000000000",
        "companyId": "00000000-0000-0000-0000-000000000000",
        "startDate": "2024-01-01T00:00:00Z",
        "endDate": "2024-01-05T00:00:00Z",
        "reason": "Test vacation",
        "requestType": "vacation"
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
    "/api/v1/time-off/00000000-0000-0000-0000-000000000000"
);
test_unauthorized!(
    test_update_time_off_request_unauthorized,
    put,
    "/api/v1/time-off/00000000-0000-0000-0000-000000000000",
    json!({
        "userId": "00000000-0000-0000-0000-000000000000",
        "companyId": "00000000-0000-0000-0000-000000000000",
        "startDate": "2024-01-02T00:00:00Z",
        "endDate": "2024-01-06T00:00:00Z",
        "reason": "Updated vacation",
        "requestType": "vacation"
    })
);
test_unauthorized!(
    test_delete_time_off_request_unauthorized,
    delete,
    "/api/v1/time-off/00000000-0000-0000-0000-000000000000"
);

// Time off request approval tests
test_unauthorized!(
    test_approve_time_off_request_unauthorized,
    post,
    "/api/v1/time-off/00000000-0000-0000-0000-000000000000/approve",
    json!({
        "notes": "Approved for vacation"
    })
);
test_unauthorized!(
    test_deny_time_off_request_unauthorized,
    post,
    "/api/v1/time-off/00000000-0000-0000-0000-000000000000/deny",
    json!({
        "notes": "Insufficient coverage during requested period"
    })
);
