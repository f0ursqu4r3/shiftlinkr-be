use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::location_repository::LocationRepository;
use be::handlers::admin;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (
    web::Data<AppState>,
    web::Data<LocationRepository>,
    web::Data<be::Config>,
) {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
    });
    let location_repo_data = web::Data::new(LocationRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

    (app_state, location_repo_data, config_data)
}

// Macro to generate unauthorized access tests
macro_rules! test_unauthorized {
    ($test_name:ident, $method:ident, $uri:expr) => {
        #[actix_web::test]
        #[serial]
        async fn $test_name() {
            let (app_state, location_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(location_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/admin")
                                .route("/locations", web::post().to(admin::create_location))
                                .route("/locations", web::get().to(admin::get_locations))
                                .route("/locations/{id}", web::get().to(admin::get_location))
                                .route("/locations/{id}", web::put().to(admin::update_location))
                                .route("/locations/{id}", web::delete().to(admin::delete_location))
                                .route("/teams", web::post().to(admin::create_team))
                                .route("/teams", web::get().to(admin::get_teams))
                                .route("/teams/{id}", web::get().to(admin::get_team))
                                .route("/teams/{id}", web::put().to(admin::update_team))
                                .route("/teams/{id}", web::delete().to(admin::delete_team))
                                .route(
                                    "/teams/{id}/members",
                                    web::post().to(admin::add_team_member),
                                )
                                .route(
                                    "/teams/{id}/members",
                                    web::get().to(admin::get_team_members),
                                )
                                .route(
                                    "/teams/{team_id}/members/{user_id}",
                                    web::delete().to(admin::remove_team_member),
                                ),
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
            let (app_state, location_repo_data, config_data) = setup_test_app().await;

            let app = test::init_service(
                App::new()
                    .app_data(app_state)
                    .app_data(location_repo_data)
                    .app_data(config_data)
                    .service(
                        web::scope("/api/v1").service(
                            web::scope("/admin")
                                .route("/locations", web::post().to(admin::create_location))
                                .route("/locations", web::get().to(admin::get_locations))
                                .route("/locations/{id}", web::get().to(admin::get_location))
                                .route("/locations/{id}", web::put().to(admin::update_location))
                                .route("/locations/{id}", web::delete().to(admin::delete_location))
                                .route("/teams", web::post().to(admin::create_team))
                                .route("/teams", web::get().to(admin::get_teams))
                                .route("/teams/{id}", web::get().to(admin::get_team))
                                .route("/teams/{id}", web::put().to(admin::update_team))
                                .route("/teams/{id}", web::delete().to(admin::delete_team))
                                .route(
                                    "/teams/{id}/members",
                                    web::post().to(admin::add_team_member),
                                )
                                .route(
                                    "/teams/{id}/members",
                                    web::get().to(admin::get_team_members),
                                )
                                .route(
                                    "/teams/{team_id}/members/{user_id}",
                                    web::delete().to(admin::remove_team_member),
                                ),
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

// Location tests
test_unauthorized!(
    test_create_location_unauthorized,
    post,
    "/api/v1/admin/locations",
    json!({
        "name": "Test Location",
        "address": "123 Test St",
        "city": "Test City",
        "state": "TS",
        "zip": "12345"
    })
);

test_unauthorized!(
    test_get_locations_unauthorized,
    get,
    "/api/v1/admin/locations"
);
test_unauthorized!(
    test_get_location_unauthorized,
    get,
    "/api/v1/admin/locations/1"
);
test_unauthorized!(
    test_update_location_unauthorized,
    put,
    "/api/v1/admin/locations/1",
    json!({
        "name": "Updated Location",
        "address": "456 Updated St",
        "city": "Updated City",
        "state": "US",
        "zip": "67890"
    })
);
test_unauthorized!(
    test_delete_location_unauthorized,
    delete,
    "/api/v1/admin/locations/1"
);

// Team tests
test_unauthorized!(
    test_create_team_unauthorized,
    post,
    "/api/v1/admin/teams",
    json!({
        "name": "Test Team",
        "location_id": 1,
        "description": "Test team description"
    })
);

test_unauthorized!(test_get_teams_unauthorized, get, "/api/v1/admin/teams");
test_unauthorized!(test_get_team_unauthorized, get, "/api/v1/admin/teams/1");
test_unauthorized!(
    test_update_team_unauthorized,
    put,
    "/api/v1/admin/teams/1",
    json!({
        "name": "Updated Team",
        "location_id": 1,
        "description": "Updated team description"
    })
);
test_unauthorized!(
    test_delete_team_unauthorized,
    delete,
    "/api/v1/admin/teams/1"
);

// Team member tests
test_unauthorized!(
    test_add_team_member_unauthorized,
    post,
    "/api/v1/admin/teams/1/members",
    json!({
        "user_id": 1,
        "role": "member"
    })
);

test_unauthorized!(
    test_get_team_members_unauthorized,
    get,
    "/api/v1/admin/teams/1/members"
);
test_unauthorized!(
    test_remove_team_member_unauthorized,
    delete,
    "/api/v1/admin/teams/1/members/1"
);
