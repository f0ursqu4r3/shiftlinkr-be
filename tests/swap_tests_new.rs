use actix_web::{http::StatusCode, test, web, App};
use be::handlers::swaps;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_create_swap_unauthorized() {
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

    let swap_data = json!({
        "original_shift_id": 1,
        "requesting_user_id": "user123",
        "swap_type": "open"
    });

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/swaps")
        .set_json(&swap_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_swaps_unauthorized() {
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

    // Act
    let req = test::TestRequest::get().uri("/api/v1/swaps").to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_swap_by_id_unauthorized() {
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

    // Act
    let req = test::TestRequest::get().uri("/api/v1/swaps/1").to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_respond_to_swap_unauthorized() {
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

    let response_data = json!({
        "action": "accept",
        "notes": "I can take this shift"
    });

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/swaps/1/respond")
        .set_json(&response_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_approve_swap_unauthorized() {
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

    let approval_data = json!({
        "notes": "Swap approved"
    });

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/swaps/1/approve")
        .set_json(&approval_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_deny_swap_unauthorized() {
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

    let denial_data = json!({
        "notes": "Cannot approve swap"
    });

    // Act
    let req = test::TestRequest::post()
        .uri("/api/v1/swaps/1/deny")
        .set_json(&denial_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // Assert
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
