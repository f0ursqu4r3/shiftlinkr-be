use actix_web::{App, http::StatusCode, test, web};
use be::handlers::{auth, skills};
use be::middleware::CacheLayer;
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

#[actix_web::test]
#[serial]
async fn test_skills_unauthorized_access() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(auth::register))
                            .route("/login", web::post().to(auth::login))
                            .route("/me", web::get().to(auth::me)),
                    )
                    .service(
                        web::scope("/skills")
                            .route("", web::post().to(skills::create_skill))
                            .route("", web::get().to(skills::get_all_skills))
                            .route("/{id}", web::get().to(skills::get_skill))
                            .route("/{id}", web::put().to(skills::update_skill))
                            .route("/{id}", web::delete().to(skills::delete_skill)),
                    )
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill))
                            .route("/{user_id}", web::get().to(skills::get_user_skills))
                            .route(
                                "/{user_id}/{skill_id}",
                                web::put().to(skills::update_user_skill),
                            ),
                    ),
            ),
    )
    .await;

    // Create skill without auth -> unauthorized
    let skill_data = json!({
        "name": "Test Skill",
        "description": "A test skill description",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .set_json(&skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Get skills without auth -> unauthorized
    let req = test::TestRequest::get().uri("/api/v1/skills").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_all_skills_success() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(
                        web::scope("/skills").route("", web::get().to(skills::get_all_skills)),
                    ),
            ),
    )
    .await;

    // Register user
    let register_data = json!({
        "email": "test@example.com",
        "password": "password123",
        "name": "Test User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let auth_token = body["token"].as_str().unwrap();

    // Test getting skills with auth - should succeed
    let req = test::TestRequest::get()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].is_array());
}

#[actix_web::test]
#[serial]
async fn test_create_skill_admin_required() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(web::scope("/skills").route("", web::post().to(skills::create_skill))),
            ),
    )
    .await;

    // Register a regular user (not admin)
    let register_data = json!({
        "email": "user@example.com",
        "password": "password123",
        "name": "Regular User"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    let auth_token = body["token"].as_str().unwrap();

    // Try to create skill as regular user - should fail
    let skill_data = json!({
        "name": "Test Skill",
        "description": "A test skill description"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
#[serial]
async fn test_user_skills_workflow() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(web::scope("/skills").route("", web::post().to(skills::create_skill)))
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill))
                            .route("/{user_id}", web::get().to(skills::get_user_skills)),
                    ),
            ),
    )
    .await;

    // Create an admin in a company
    let (admin_id, company_id, admin_token) =
        common::create_user_with_company("admin@example.com", "password123", "Admin", "SkillCo")
            .await
            .unwrap();
    common::make_user_admin_of_company(admin_id, company_id)
        .await
        .unwrap();

    // Create a skill via API using admin token
    let skill_input = json!({
        "name": "Test Skill",
        "description": "A test skill",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&skill_input)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let created: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = created["data"]["id"].as_str().unwrap().to_string();

    // Add skill to admin user (manager required)
    let user_skill_data = json!({
        "user_id": admin_id.to_string(),
        "skill_id": skill_id,
        "proficiency_level": "intermediate",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/user-skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&user_skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Get user skills (same user)
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/user-skills/{}", admin_id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].is_array());
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[actix_web::test]
#[serial]
async fn test_user_skill_permission_check() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(web::scope("/skills").route("", web::post().to(skills::create_skill)))
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill)),
                    ),
            ),
    )
    .await;

    // Create two users in separate companies
    let (user1_id, user1_token, user1_company_id) =
        common::create_test_user_with_token("user1@example.com", "password123", "User One")
            .await
            .unwrap();
    // Promote user1 to admin to create a skill in their company
    common::make_user_admin_of_company(user1_id, user1_company_id)
        .await
        .unwrap();

    let (user2_id, _user2_token, _company2_id) =
        common::create_test_user_with_token("user2@example.com", "password123", "User Two")
            .await
            .unwrap();

    // Create a skill via admin route (user1's company)
    let skill_input = json!({
        "name": "Test Skill",
        "description": "A test skill",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", user1_token)))
        .set_json(&skill_input)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let skill_body: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = skill_body["data"]["id"].as_str().unwrap();

    // Try to add skill to user2 (different company) using user1's token -> forbidden
    let user_skill_data = json!({
        "user_id": user2_id.to_string(),
        "skill_id": skill_id,
        "proficiency_level": "intermediate",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/user-skills")
        .insert_header(("Authorization", format!("Bearer {}", user1_token)))
        .set_json(&user_skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[actix_web::test]
#[serial]
async fn test_get_skill_by_id() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(
                        web::scope("/skills")
                            .route("/{id}", web::get().to(skills::get_skill))
                            .route("", web::post().to(skills::create_skill)),
                    ),
            ),
    )
    .await;

    // Create admin and skill
    let (admin_id, company_id, admin_token) = common::create_user_with_company(
        "skilladmin@example.com",
        "password123",
        "Admin User",
        "Company",
    )
    .await
    .unwrap();
    common::make_user_admin_of_company(admin_id, company_id)
        .await
        .unwrap();

    let skill_input = json!({
        "name": "Test Get Skill",
        "description": "A skill for testing get endpoint",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&skill_input)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let created: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = created["data"]["id"].as_str().unwrap();

    // Get the skill with same company token
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Test Get Skill");
    assert_eq!(
        body["data"]["description"],
        "A skill for testing get endpoint"
    );
}

#[actix_web::test]
#[serial]
async fn test_update_skill() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/skills")
                        .route("", web::post().to(skills::create_skill))
                        .route("/{id}", web::put().to(skills::update_skill)),
                ),
            ),
    )
    .await;

    // Create an admin user with a company
    let (admin_id, company_id, admin_token) =
        common::create_user_with_company("admin@test.com", "password123", "Admin User", "Co")
            .await
            .unwrap();
    common::make_user_admin_of_company(admin_id, company_id)
        .await
        .unwrap();

    // Create a skill
    let create_data = json!({
        "name": "Original Skill",
        "description": "Original description",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&create_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let created: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = created["data"]["id"].as_str().unwrap();

    // Update the skill (manager required)
    let update_data = json!({
        "name": "Updated Skill",
        "description": "Updated description",
    });
    let req = test::TestRequest::put()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["name"], "Updated Skill");
    assert_eq!(body["data"]["description"], "Updated description");
}

#[actix_web::test]
#[serial]
async fn test_delete_skill() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1").service(
                    web::scope("/skills")
                        .route("", web::post().to(skills::create_skill))
                        .route("/{id}", web::get().to(skills::get_skill))
                        .route("/{id}", web::delete().to(skills::delete_skill)),
                ),
            ),
    )
    .await;

    // Create an admin user within a company
    let (admin_id, company_id, admin_token) =
        common::create_user_with_company("admin@test.com", "password123", "Admin User", "Co")
            .await
            .unwrap();
    common::make_user_admin_of_company(admin_id, company_id)
        .await
        .unwrap();

    // Create a skill first
    let skill_data = json!({
        "name": "Skill to Delete",
        "description": "This skill will be deleted",
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let skill: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = skill["data"]["id"].as_str().unwrap().to_string();

    // Delete the skill
    let req = test::TestRequest::delete()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Try to get the deleted skill - should fail
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/skills/{}", skill_id))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[actix_web::test]
#[serial]
async fn test_update_user_skill() {
    common::setup_test_env();
    let _ctx = common::TestContext::new().await.unwrap();
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(CacheLayer::new(1000, 60)))
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/skills").route("", web::post().to(skills::create_skill)))
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill))
                            .route(
                                "/{user_id}/{skill_id}",
                                web::put().to(skills::update_user_skill),
                            ),
                    ),
            ),
    )
    .await;

    // Create an admin user in a company
    let (admin_id, company_id, admin_token) =
        common::create_user_with_company("admin@test.com", "password123", "Admin User", "Co")
            .await
            .unwrap();
    common::make_user_admin_of_company(admin_id, company_id)
        .await
        .unwrap();

    // Create a skill first (as admin)
    let create_skill = json!({
        "name": "General Labor",
        "description": "",
    });
    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&create_skill)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let skill_val: serde_json::Value = test::read_body_json(resp).await;
    let skill_id = skill_val["data"]["id"].as_str().unwrap().to_string();

    // Add a skill to user first (manager required)
    let user_skill_data = json!({
        "user_id": admin_id.to_string(),
        "skill_id": skill_id,
        "proficiency_level": "beginner",
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/user-skills")
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&user_skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    // Update proficiency level
    let update_data = json!({
        "proficiency_level": "advanced",
    });

    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/v1/user-skills/{}/{}",
            admin_id,
            user_skill_data["skill_id"].as_str().unwrap()
        ))
        .insert_header(("Authorization", format!("Bearer {}", admin_token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["data"]["proficiency_level"], "advanced");
}
