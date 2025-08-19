use actix_web::{http::StatusCode, test, web, App};
use be::handlers::admin;
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
                                    "/teams/{team_id}/members",
                                    web::get().to(admin::get_team_members),
                                )
                                .route(
                                    "/teams/{team_id}/members/{user_id}",
                                    web::post().to(admin::add_team_member),
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
            common::setup_test_env();
            let _ctx = common::TestContext::new().await.unwrap();
            let cache = CacheLayer::new(1000, 60);
            let app = test::init_service(
                App::new()
                    .app_data(web::Data::new(cache))
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
                                    "/teams/{team_id}/members",
                                    web::get().to(admin::get_team_members),
                                )
                                .route(
                                    "/teams/{team_id}/members/{user_id}",
                                    web::post().to(admin::add_team_member),
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
    "phone": "555-1234",
    "email": "loc@example.com"
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
    "/api/v1/admin/locations/00000000-0000-0000-0000-000000000001"
);
test_unauthorized!(
    test_update_location_unauthorized,
    put,
    "/api/v1/admin/locations/00000000-0000-0000-0000-000000000001",
    json!({
        "name": "Updated Location",
        "address": "456 Updated St",
        "phone": "555-6789",
        "email": "updated@loc.com"
    })
);
test_unauthorized!(
    test_delete_location_unauthorized,
    delete,
    "/api/v1/admin/locations/00000000-0000-0000-0000-000000000001"
);

// Team tests
test_unauthorized!(
    test_create_team_unauthorized,
    post,
    "/api/v1/admin/teams",
    json!({
        "name": "Test Team",
    "location_id": "00000000-0000-0000-0000-000000000001",
        "description": "Test team description"
    })
);

test_unauthorized!(test_get_teams_unauthorized, get, "/api/v1/admin/teams");
test_unauthorized!(
    test_get_team_unauthorized,
    get,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001"
);
test_unauthorized!(
    test_update_team_unauthorized,
    put,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001",
    json!({
        "name": "Updated Team",
        "location_id": "00000000-0000-0000-0000-000000000001",
        "description": "Updated team description"
    })
);
test_unauthorized!(
    test_delete_team_unauthorized,
    delete,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001"
);

// Team member tests
test_unauthorized!(
    test_add_team_member_unauthorized,
    post,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001/members/00000000-0000-0000-0000-000000000002",
    json!({})
);

test_unauthorized!(
    test_get_team_members_unauthorized,
    get,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001/members"
);
test_unauthorized!(
    test_remove_team_member_unauthorized,
    delete,
    "/api/v1/admin/teams/00000000-0000-0000-0000-000000000001/members/00000000-0000-0000-0000-000000000002"
);
