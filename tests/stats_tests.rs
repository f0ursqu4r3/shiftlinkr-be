use actix_web::{http::StatusCode, test, web, App};
use be::handlers::stats;
use be::AppState;
use pretty_assertions::assert_eq;
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
test_unauthorized!(test_get_dashboard_stats_unauthorized, get, "/api/v1/stats/dashboard");
test_unauthorized!(test_get_shift_stats_unauthorized, get, "/api/v1/stats/shifts");
test_unauthorized!(test_get_time_off_stats_unauthorized, get, "/api/v1/stats/time-off");
