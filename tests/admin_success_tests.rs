use actix_web::{http::StatusCode, test, web, App};
use be::database::repositories::company_repository::CompanyRepository;
use be::database::repositories::location_repository::LocationRepository;
use be::handlers::admin;
use be::AppState;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_location_create_success() {
    let (app_state, location_repo_data, config_data, _ctx) = common::create_admin_app_data().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(location_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(be::handlers::auth::register))
                            .route("/login", web::post().to(be::handlers::auth::login)),
                    )
                    .service(
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

    // First register an admin user
    let register_data = json!({
        "email": "admin@example.com",
        "password": "password123",
        "name": "Admin User",
        "role": "admin"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let reg_resp = test::call_service(&app, reg_req.to_request()).await;

    // Debug: Check what the actual response is
    if reg_resp.status() != StatusCode::OK {
        let status = reg_resp.status();
        let body: serde_json::Value = test::read_body_json(reg_resp).await;
        panic!(
            "Registration failed with status: {}, body: {:?}",
            status, body
        );
    }

    assert_eq!(reg_resp.status(), StatusCode::OK);

    let reg_body: serde_json::Value = test::read_body_json(reg_resp).await;
    let auth_token = reg_body["token"].as_str().unwrap();

    // Now test creating a location with authentication
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": 1
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&location_data);

    let resp = test::call_service(&app, req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Test Location");
    assert_eq!(body["data"]["address"], "123 Test St");
    assert_eq!(body["data"]["phone"], "555-1234");
    assert!(body["data"]["id"].is_number());
}

#[actix_web::test]
#[serial]
async fn test_location_list_success() {
    let (app_state, location_repo_data, config_data, _ctx) = common::create_admin_app_data().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(location_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(be::handlers::auth::register)),
                    )
                    .service(
                        web::scope("/admin")
                            .route("/locations", web::post().to(admin::create_location))
                            .route("/locations", web::get().to(admin::get_locations)),
                    ),
            ),
    )
    .await;

    // Register admin user and get token
    let register_data = json!({
        "email": "admin@example.com",
        "password": "password123",
        "name": "Admin User",
        "role": "admin"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let reg_resp = test::call_service(&app, reg_req.to_request()).await;
    let reg_body: serde_json::Value = test::read_body_json(reg_resp).await;
    let auth_token = reg_body["token"].as_str().unwrap();

    // Create a location first
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": 1
    });

    let create_req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&location_data);

    let create_resp = test::call_service(&app, create_req.to_request()).await;
    assert_eq!(create_resp.status(), StatusCode::CREATED);

    // Small delay to ensure the location is committed to the database
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    // Now test listing locations
    let list_req = test::TestRequest::get()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)));

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
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service,
        company_repository: CompanyRepository::new(ctx.pool.clone()),
    });
    let location_repo_data = web::Data::new(LocationRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config);

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(location_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(be::handlers::auth::register)),
                    )
                    .service(
                        web::scope("/admin")
                            .route("/locations", web::post().to(admin::create_location))
                            .route("/teams", web::post().to(admin::create_team))
                            .route("/teams", web::get().to(admin::get_teams)),
                    ),
            ),
    )
    .await;

    // Register admin user and get token
    let register_data = json!({
        "email": "admin@example.com",
        "password": "password123",
        "name": "Admin User",
        "role": "admin"
    });

    let reg_req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data);

    let reg_resp = test::call_service(&app, reg_req.to_request()).await;
    let reg_body: serde_json::Value = test::read_body_json(reg_resp).await;
    let auth_token = reg_body["token"].as_str().unwrap();

    // Create a location first (required for team)
    let location_data = json!({
        "name": "Test Location",
        "address": "123 Test St",
        "phone": "555-1234",
        "email": "test@location.com",
        "company_id": 1
    });

    let create_loc_req = test::TestRequest::post()
        .uri("/api/v1/admin/locations")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&location_data);

    let loc_resp = test::call_service(&app, create_loc_req.to_request()).await;
    assert_eq!(loc_resp.status(), StatusCode::CREATED);
    let loc_body: serde_json::Value = test::read_body_json(loc_resp).await;
    let location_id = loc_body["data"]["id"].as_i64().unwrap();

    // Now create a team
    let team_data = json!({
        "name": "Test Team",
        "location_id": location_id,
        "description": "Test team description"
    });

    let team_req = test::TestRequest::post()
        .uri("/api/v1/admin/teams")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&team_data);

    let resp = test::call_service(&app, team_req.to_request()).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Test Team");
    assert_eq!(body["data"]["location_id"], location_id);
    assert_eq!(body["data"]["description"], "Test team description");
    assert!(body["data"]["id"].is_number());
}
