use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::invite_repository::InviteRepository;
use be::handlers::auth;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (
    web::Data<AppState>,
    web::Data<InviteRepository>,
    web::Data<be::Config>,
) {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let invite_repo_data = web::Data::new(InviteRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

    (app_state, invite_repo_data, config_data)
}

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, invite_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(invite_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/auth")
                                .route("/register", web::post().to(auth::register))
                                .route("/login", web::post().to(auth::login))
                                .route("/me", web::get().to(auth::me))
                                .route("/forgot-password", web::post().to(auth::forgot_password))
                                .route("/reset-password", web::post().to(auth::reset_password))
                                .route("/invite", web::post().to(auth::create_invite))
                                .route("/invite/{token}", web::get().to(auth::get_invite))
                                .route("/invite/accept", web::post().to(auth::accept_invite))
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
            let (app_state, invite_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(invite_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/auth")
                                .route("/register", web::post().to(auth::register))
                                .route("/login", web::post().to(auth::login))
                                .route("/me", web::get().to(auth::me))
                                .route("/forgot-password", web::post().to(auth::forgot_password))
                                .route("/reset-password", web::post().to(auth::reset_password))
                                .route("/invite", web::post().to(auth::create_invite))
                                .route("/invite/{token}", web::get().to(auth::get_invite))
                                .route("/invite/accept", web::post().to(auth::accept_invite))
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
