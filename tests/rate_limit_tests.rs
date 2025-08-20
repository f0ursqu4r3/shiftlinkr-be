use actix_web::{App, http::StatusCode, test, web};
use serial_test::serial;

use be::{
    handlers::auth,
    middleware::{AuthRateLimiter, CacheLayer, RateLimitConfig, RateLimitMiddleware},
};

mod common;

#[actix_web::test]
#[serial]
async fn test_basic_rate_limiting() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);

    let app =
        test::init_service(
            App::new()
                .app_data(web::Data::new(cache))
                .wrap(RateLimitMiddleware::new(RateLimitConfig::new(2, 60))) // 2 requests per minute
                .service(web::scope("/api/v1").service(
                    web::scope("/auth").route("/register", web::post().to(auth::register)),
                )),
        )
        .await;

    // First request should pass
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "email": "test1@example.com",
            "password": "password123",
            "first_name": "Test",
            "last_name": "User",
            "phone": "+1234567890",
            "company_name": "Test Company"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    // Should pass or fail due to business logic, but not rate limiting
    assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Second request should pass
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "email": "test2@example.com",
            "password": "password123",
            "first_name": "Test",
            "last_name": "User2",
            "phone": "+1234567891",
            "company_name": "Test Company 2"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);

    // Third request should be rate limited
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&serde_json::json!({
            "email": "test3@example.com",
            "password": "password123",
            "first_name": "Test",
            "last_name": "User3",
            "phone": "+1234567892",
            "company_name": "Test Company 3"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[actix_web::test]
#[serial]
async fn test_auth_rate_limiting_login() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let cache = CacheLayer::new(1000, 60);

    let app = test::init_service(
        App::new().app_data(web::Data::new(cache)).service(
            web::resource("/api/v1/auth/login")
                .wrap(AuthRateLimiter::login())
                .route(web::post().to(auth::login)),
        ),
    )
    .await;

    // Make 5 requests (the limit for login rate limiter)
    for i in 1..=5 {
        let req = test::TestRequest::post()
            .uri("/api/v1/auth/login")
            .set_json(&serde_json::json!({
                "email": format!("test{}@example.com", i),
                "password": "wrongpassword"
            }))
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Should not be rate limited yet
        assert_ne!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
    }

    // 6th request should be rate limited
    let req = test::TestRequest::post()
        .uri("/api/v1/auth/login")
        .set_json(&serde_json::json!({
            "email": "test6@example.com",
            "password": "wrongpassword"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}
