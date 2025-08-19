use actix_web::{http::StatusCode, test, web, App};
use be::handlers::auth;
use be::middleware::CacheLayer;
use pretty_assertions::assert_eq;
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
            let cache = CacheLayer::new(1000, 60);
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(cache))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/auth")
                                .route("/register", web::post().to(auth::register))
                                .route("/login", web::post().to(auth::login))
                                .route("/me", web::get().to(auth::me))
                                .route(
                                    "/forgot-password",
                                    web::post().to(auth::forgot_password),
                                )
                                .route(
                                    "/reset-password",
                                    web::post().to(auth::reset_password),
                                )
                                .route("/invite", web::post().to(auth::create_invite))
                                .route("/invite/{token}", web::get().to(auth::get_invite))
                                .route(
                                    "/invite/{token}/accept",
                                    web::post().to(auth::accept_invite),
                                )
                                .route(
                                    "/invite/{token}/reject",
                                    web::post().to(auth::reject_invite),
                                )
                                .route("/invites", web::get().to(auth::get_my_invites)),
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
            let cache = CacheLayer::new(1000, 60);
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(cache))
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/auth")
                                .route("/register", web::post().to(auth::register))
                                .route("/login", web::post().to(auth::login))
                                .route("/me", web::get().to(auth::me))
                                .route(
                                    "/forgot-password",
                                    web::post().to(auth::forgot_password),
                                )
                                .route(
                                    "/reset-password",
                                    web::post().to(auth::reset_password),
                                )
                                .route("/invite", web::post().to(auth::create_invite))
                                .route("/invite/{token}", web::get().to(auth::get_invite))
                                .route(
                                    "/invite/{token}/accept",
                                    web::post().to(auth::accept_invite),
                                )
                                .route(
                                    "/invite/{token}/reject",
                                    web::post().to(auth::reject_invite),
                                )
                                .route("/invites", web::get().to(auth::get_my_invites)),
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

// Protected auth endpoint tests
test_unauthorized!(test_me_unauthorized, get, "/api/v1/auth/me");
test_unauthorized!(
    test_create_invite_unauthorized,
    post,
    "/api/v1/auth/invite",
    json!({
        "email": "newuser@example.com",
        "role": "employee"
    })
);
test_unauthorized!(
    test_get_my_invites_unauthorized,
    get,
    "/api/v1/auth/invites"
);
