use actix_web::{http::StatusCode, test, web, App};
use be::handlers::stats;
use be::AppState;
use pretty_assertions::assert_eq;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_get_dashboard_stats_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/stats/dashboard")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_shift_stats_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/stats/shifts")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_time_off_stats_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/stats/time-off")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
