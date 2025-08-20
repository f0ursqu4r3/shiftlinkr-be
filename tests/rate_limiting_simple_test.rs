use actix_web::{App, http::StatusCode, test, web};
use be::middleware::{RateLimitConfig, RateLimitMiddleware};
use serial_test::serial;

// Simple test endpoint that always returns 200 OK
async fn test_endpoint() -> actix_web::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Ok().json(serde_json::json!({"status": "ok"})))
}

#[actix_web::test]
#[serial]
async fn test_rate_limiting_works() {
    let app = test::init_service(
        App::new()
            .wrap(RateLimitMiddleware::new(RateLimitConfig::new(2, 60))) // 2 requests per minute
            .route("/test", web::get().to(test_endpoint)),
    )
    .await;

    // First request should pass
    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Second request should pass
    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Third request should be rate limited
    let req = test::TestRequest::get().uri("/test").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::TOO_MANY_REQUESTS);
}
