use actix_web::{App, http::StatusCode, test, web};
use be::handlers::stats;
use be::middleware::CacheLayer;
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
                    .app_data(web::Data::new(CacheLayer::new(500, 60)))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/stats")
                                .route("/dashboard", web::get().to(stats::get_dashboard_stats))
                                .route("/shifts", web::get().to(stats::get_shift_stats))
                                .route("/time-off", web::get().to(stats::get_time_off_stats)),
                        ),
                    ),
            )
            .await;

            let req = test::TestRequest::$method().uri($uri).to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
        }
    };
}

// Stats endpoint tests
test_unauthorized!(
    test_get_dashboard_stats_unauthorized,
    get,
    "/api/v1/stats/dashboard"
);
test_unauthorized!(
    test_get_shift_stats_unauthorized,
    get,
    "/api/v1/stats/shifts"
);
test_unauthorized!(
    test_get_time_off_stats_unauthorized,
    get,
    "/api/v1/stats/time-off"
);
