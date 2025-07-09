use actix_web::{http::StatusCode, test, web, App};
use be::database::time_off_repository::TimeOffRepository;
use be::handlers::time_off;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_create_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/time-off")
        .set_json(&json!({
            "user_id": "test_user",
            "start_date": "2024-01-01",
            "end_date": "2024-01-05",
            "reason": "Test vacation",
            "request_type": "Vacation"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_time_off_requests_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/time-off")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::get()
        .uri("/api/v1/time-off/123")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_update_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::put()
        .uri("/api/v1/time-off/123")
        .set_json(&json!({
            "user_id": "test_user",
            "start_date": "2024-01-01",
            "end_date": "2024-01-05",
            "reason": "Updated vacation",
            "request_type": "Vacation"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_delete_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::delete()
        .uri("/api/v1/time-off/123")
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_approve_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/time-off/123/approve")
        .set_json(&json!({
            "notes": "Approved for vacation"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_deny_time_off_request_unauthorized() {
    // Arrange
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let time_off_repo_data = web::Data::new(TimeOffRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

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

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/time-off/123/deny")
        .set_json(&json!({
            "notes": "Insufficient coverage during requested period"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
