use actix_web::{http::StatusCode, test, web, App};
use be::handlers::swaps;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (web::Data<AppState>, web::Data<be::Config>) {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let config_data = web::Data::new(ctx.config);

    (app_state, config_data)
}

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, config_data) = setup_test_app().await;
            
            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(config_data)
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
    ($test_name:ident, $method:ident, $uri:expr, $json:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, config_data) = setup_test_app().await;
            
            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(config_data)
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
                .set_json(&$json)
                .to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
}

// Swap request tests
test_unauthorized!(test_create_swap_unauthorized, post, "/api/v1/swaps", json!({
    "original_shift_id": 1,
    "requesting_user_id": "user123",
    "swap_type": "open"
}));

test_unauthorized!(test_get_swaps_unauthorized, get, "/api/v1/swaps");
test_unauthorized!(test_get_swap_by_id_unauthorized, get, "/api/v1/swaps/123");
test_unauthorized!(test_respond_to_swap_unauthorized, post, "/api/v1/swaps/123/respond", json!({
    "response": "accept",
    "responding_user_id": "user456"
}));
test_unauthorized!(test_approve_swap_unauthorized, post, "/api/v1/swaps/123/approve", json!({
    "approving_user_id": "manager123"
}));
test_unauthorized!(test_deny_swap_unauthorized, post, "/api/v1/swaps/123/deny", json!({
    "denying_user_id": "manager123",
    "reason": "Insufficient coverage"
}));
