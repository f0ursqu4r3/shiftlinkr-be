use actix_web::{http::StatusCode, test, web, App};
use be::database::models::SkillInput;
use be::database::repositories::{company::CompanyRepository, skill::SkillRepository};
use be::handlers::{auth, skills};
use be::{ActivityLogger, ActivityRepository, AppState, Config};
use pretty_assertions::assert_eq;
use serde_json::json;
use serial_test::serial;

mod common;

// Helper function to create test app state and dependencies
async fn setup_test_app() -> (
    web::Data<AppState>,
    web::Data<SkillRepository>,
    web::Data<CompanyRepository>,
    web::Data<Config>,
    common::TestContext,
) {
    common::setup_test_env();
    let ctx = common::TestContext::new().await.unwrap();

    let app_state = web::Data::new(AppState {
        auth_service: ctx.auth_service.clone(),
        company_repository: CompanyRepository::new(ctx.pool.clone()),
        activity_repository: ActivityRepository::new(ctx.pool.clone()),
        activity_logger: ActivityLogger::new(ActivityRepository::new(ctx.pool.clone())),
    });
    let skill_repo_data = web::Data::new(SkillRepository::new(ctx.pool.clone()));
    let company_repo_data = web::Data::new(CompanyRepository::new(ctx.pool.clone()));
    let config_data = web::Data::new(ctx.config.clone());

    (
        app_state,
        skill_repo_data,
        company_repo_data,
        config_data,
        ctx,
    )
}

#[actix_web::test]
#[serial]
async fn test_skills_unauthorized_access() {
    let (app_state, skill_repo_data, company_repo_data, config_data, _ctx) = setup_test_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(skill_repo_data)
            .app_data(company_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/skills")
                        .route("", web::post().to(skills::create_skill))
                        .route("", web::get().to(skills::get_all_skills)),
                ),
            ),
    )
    .await;

    // Test creating skill without auth - should fail
    let skill_data = json!({
        "name": "Test Skill",
        "description": "A test skill description"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/skills")
        .set_json(&skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // Test getting skills without auth - should fail
    let req = test::TestRequest::get().uri("/api/v1/skills").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn test_get_all_skills_success() {
    let (app_state, skill_repo_data, company_repo_data, config_data, _ctx) = setup_test_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(skill_repo_data)
            .app_data(company_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(
                        web::scope("/skills").route("", web::get().to(skills::get_all_skills)),
                    ),
            ),
    )
    .await;

    // Register a test user
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
    let (app_state, skill_repo_data, company_repo_data, config_data, _ctx) = setup_test_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(skill_repo_data)
            .app_data(company_repo_data)
            .app_data(config_data)
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
    let (app_state, skill_repo_data, company_repo_data, config_data, ctx) = setup_test_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(skill_repo_data)
            .app_data(company_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill))
                            .route("/{user_id}", web::get().to(skills::get_user_skills)),
                    ),
            ),
    )
    .await;

    // Register a test user
    let register_data = json!({
        "email": "user@example.com",
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
    let user_id = body["user"]["id"].as_str().unwrap();

    // Create a skill directly in the repository for testing
    let skill_repo = SkillRepository::new(ctx.pool.clone());
    let skill_input = SkillInput {
        name: "Test Skill".to_string(),
        description: Some("A test skill".to_string()),
    };
    let skill = skill_repo.create_skill(skill_input).await.unwrap();

    // Add skill to user
    let user_skill_data = json!({
        "user_id": user_id,
        "skill_id": skill.id,
        "proficiency_level": "Intermediate"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/user-skills")
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
        .set_json(&user_skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    // Get user skills
    let req = test::TestRequest::get()
        .uri(&format!("/api/v1/user-skills/{}", user_id))
        .insert_header(("Authorization", format!("Bearer {}", auth_token)))
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
    let (app_state, skill_repo_data, company_repo_data, config_data, ctx) = setup_test_app().await;

    let app = test::init_service(
        App::new()
            .app_data(app_state)
            .app_data(skill_repo_data)
            .app_data(company_repo_data)
            .app_data(config_data)
            .service(
                web::scope("/api/v1")
                    .service(web::scope("/auth").route("/register", web::post().to(auth::register)))
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill)),
                    ),
            ),
    )
    .await;

    // Register first user
    let register_data1 = json!({
        "email": "user1@example.com",
        "password": "password123",
        "name": "User One"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data1)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let auth_token1 = body["token"].as_str().unwrap().to_string();

    // Register second user
    let register_data2 = json!({
        "email": "user2@example.com",
        "password": "password123",
        "name": "User Two"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/auth/register")
        .set_json(&register_data2)
        .to_request();
    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let user2_id = body["user"]["id"].as_str().unwrap().to_string();

    // Create a skill
    let skill_repo = SkillRepository::new(ctx.pool.clone());
    let skill_input = SkillInput {
        name: "Test Skill".to_string(),
        description: Some("A test skill".to_string()),
    };
    let skill = skill_repo.create_skill(skill_input).await.unwrap();

    // Try to add skill to user2 using user1's token - should fail
    let user_skill_data = json!({
        "user_id": user2_id,
        "skill_id": skill.id,
        "proficiency_level": "Intermediate"
    });

    let req = test::TestRequest::post()
        .uri("/api/v1/user-skills")
        .insert_header(("Authorization", format!("Bearer {}", auth_token1)))
        .set_json(&user_skill_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}
