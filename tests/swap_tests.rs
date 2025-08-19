use actix_web::{http::StatusCode, test, web, App};
use be::handlers::swaps;
use be::middleware::CacheLayer;
use serde_json::json;
use serial_test::serial;

mod common;

// Macro to generate unauthorized tests
macro_rules! test_unauthorized {
    ($name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $name() {
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(CacheLayer::new(1000, 60)))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/swaps")
                                .route("", web::post().to(swaps::create_swap_request))
                                .route("", web::get().to(swaps::get_swap_requests))
                                .route("/{id}", web::get().to(swaps::get_swap_request))
                                .route("/{id}/respond", web::post().to(swaps::respond_to_swap))
                                .route("/{id}/approve", web::post().to(swaps::approve_swap_request))
                                .route("/{id}/deny", web::post().to(swaps::deny_swap_request)),
                        ),
                    ),
            )
            .await;

            let req = test::TestRequest::$method().uri($uri).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
    ($name:ident, $method:ident, $uri:expr, $payload:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $name() {
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(CacheLayer::new(1000, 60)))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/swaps")
                                .route("", web::post().to(swaps::create_swap_request))
                                .route("", web::get().to(swaps::get_swap_requests))
                                .route("/{id}", web::get().to(swaps::get_swap_request))
                                .route("/{id}/respond", web::post().to(swaps::respond_to_swap))
                                .route("/{id}/approve", web::post().to(swaps::approve_swap_request))
                                .route("/{id}/deny", web::post().to(swaps::deny_swap_request)),
                        ),
                    ),
            )
            .await;

            let req = test::TestRequest::$method()
                .uri($uri)
                .set_json(&$payload)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
}

// Unauthorized tests for swap endpoints
test_unauthorized!(
    test_create_swap_request_unauthorized,
    post,
    "/api/v1/swaps",
    json!({
        "originalShiftId": "00000000-0000-0000-0000-000000000000",
        "requestingUserId": "00000000-0000-0000-0000-000000000001",
        "targetUserId": "00000000-0000-0000-0000-000000000002",
        "targetShiftId": null,
        "notes": "Requesting swap",
        "swapType": "targeted"
    })
);
test_unauthorized!(test_get_swap_requests_unauthorized, get, "/api/v1/swaps");
test_unauthorized!(
    test_get_swap_request_unauthorized,
    get,
    "/api/v1/swaps/00000000-0000-0000-0000-000000000000"
);
test_unauthorized!(
    test_respond_to_swap_unauthorized,
    post,
    "/api/v1/swaps/00000000-0000-0000-0000-000000000000/respond",
    json!({
        "targetShiftId": null,
        "decision": "accepted",
        "notes": "Sure"
    })
);
test_unauthorized!(
    test_approve_swap_unauthorized,
    post,
    "/api/v1/swaps/00000000-0000-0000-0000-000000000000/approve",
    json!({})
);
test_unauthorized!(
    test_deny_swap_unauthorized,
    post,
    "/api/v1/swaps/00000000-0000-0000-0000-000000000000/deny",
    json!({})
);
