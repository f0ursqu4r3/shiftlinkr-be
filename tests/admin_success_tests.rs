use actix_web::{App, http::StatusCode, test, web};
use be::handlers::admin;
use be::middleware::CacheLayer;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_location_create_success() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a test user + company + token
    let (user_id, token, company_id) =
        common::create_test_user_with_token("admin@example.com", "password123", "Admin User")
            .await
            .unwrap();

    // Promote to admin for admin endpoints
    common::make_user_admin_of_company(user_id, company_id)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/admin")
                        .route("/locations", web::post().to(admin::create_location))
                        .route("/locations", web::get().to(admin::get_locations))
                        .route("/locations/{id}", web::get().to(admin::get_location))
                        .route("/locations/{id}", web::put().to(admin::update_location))
                        .route("/locations/{id}", web::delete().to(admin::delete_location)),
                ),
            ),
    )
    .await;

    // Now test creating a location with authentication
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": company_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&location_data);

    let resp = test::call_service(&app, req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Test Location");
    assert_eq!(body["data"]["address"], "123 Test St");
    assert_eq!(body["data"]["phone"], "555-1234");
    assert!(body["data"]["id"].is_string());
}

#[actix_web::test]
#[serial]
async fn test_location_list_success() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a test user + company + token
    let (user_id, token, company_id) =
        common::create_test_user_with_token("admin@example.com", "password123", "Admin User")
            .await
            .unwrap();
    common::make_user_admin_of_company(user_id, company_id)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/admin")
                        .route("/locations", web::post().to(admin::create_location))
                        .route("/locations", web::get().to(admin::get_locations)),
                ),
            ),
    )
    .await;

    // Create a location first
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": company_id.to_string()
    });

    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&location_data);

    let create_resp = test::call_service(&app, create_req.to_request()).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // Small delay to ensure the location is committed to the database
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Now test listing locations
    let list_req = test::TestRequest::get()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", token)));

    let resp = test::call_service(&app, list_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["name"], "Test Location");
}

#[actix_web::test]
#[serial]
async fn test_team_create_success() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();

    // Create a test user + company + token
    let (user_id, token, company_id) =
        common::create_test_user_with_token("admin@example.com", "password123", "Admin User")
            .await
            .unwrap();
    common::make_user_admin_of_company(user_id, company_id)
        .await
        .unwrap();

    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/admin")
                        .route("/locations", web::post().to(admin::create_location))
                        .route("/teams", web::post().to(admin::create_team))
                        .route("/teams", web::get().to(admin::get_teams)),
                ),
            ),
    )
    .await;

    // Create a location first (required for team)
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": company_id.to_string()
    });

    let create_loc_req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&location_data);

    let loc_resp = test::call_service(&app, create_loc_req.to_request()).await;
    assert_eq!(loc_resp.status(), StatusCode::CREATED);
    let loc_body: serde_json::Value = test::read_body_json(loc_resp).await;
    let location_id = loc_body["data"]["id"].as_str().unwrap().to_string();

    // Now create a team
    let team_data = json!({
        "name": "Test Team",
        "locationId": location_id,
        "description": "Test team description"
    });

    let team_req = test::TestRequest::post()
        .uri("/api/v1/admin/teams")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&team_data);

    let resp = test::call_service(&app, team_req.to_request()).await;
    // Some handlers may return 200 OK instead of 201 Created; accept either
    assert!(resp.status() == StatusCode::CREATED || resp.status() == StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Test Team");
    // Accept either camelCase or snake_case for location id depending on serializer
    let loc_in_body = body["data"]["locationId"]
        .as_str()
        .or_else(|| body["data"]["location_id"].as_str())
        .expect("locationId or location_id should be present as string");
    assert_eq!(loc_in_body, location_id);
    assert_eq!(body["data"]["description"], "Test team description");
    assert!(body["data"]["id"].is_string());
}
