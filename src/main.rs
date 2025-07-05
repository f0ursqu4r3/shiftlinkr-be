use actix_cors::Cors;
use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use anyhow::Result;

pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;

use auth::AuthService;
use config::Config;
use database::{
    init_database, location_repository::LocationRepository, shift_repository::ShiftRepository,
    user_repository::UserRepository,
};

pub struct AppState {
    pub auth_service: AuthService,
}

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("ShiftLinkr API v1.0")
}

#[get("/health")]
async fn health() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({
        "status": "ok",
        "timestamp": chrono::Utc::now()
    }))
}

#[actix_web::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize logger
    env_logger::init();

    println!("🚀 Starting ShiftLinkr API server...");

    // Load configuration
    let config = Config::from_env()?;
    println!(
        "📋 Configuration loaded (environment: {})",
        config.environment
    );

    // Initialize database
    let pool = init_database(&config.database_url).await?;
    println!("✅ Database initialized");

    // Initialize repositories and services
    let user_repository = UserRepository::new(pool.clone());
    let location_repository = LocationRepository::new(pool.clone());
    let shift_repository = ShiftRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repository, config.clone());

    // Create app state and repository data
    let app_state = web::Data::new(AppState { auth_service });
    let location_repo_data = web::Data::new(location_repository);
    let shift_repo_data = web::Data::new(shift_repository);
    let config_data = web::Data::new(config.clone());

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(location_repo_data.clone())
            .app_data(shift_repo_data.clone())
            .app_data(config_data.clone())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        "Authorization",
                        "Content-Type",
                        "Accept",
                        "X-Requested-With",
                    ])
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .service(hello)
            .service(health)
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(handlers::auth::register))
                            .route("/login", web::post().to(handlers::auth::login))
                            .route("/me", web::get().to(handlers::auth::me)),
                    )
                    .service(
                        web::scope("/admin")
                            .route(
                                "/locations",
                                web::post().to(handlers::admin::create_location),
                            )
                            .route("/locations", web::get().to(handlers::admin::get_locations))
                            .route(
                                "/locations/{id}",
                                web::get().to(handlers::admin::get_location),
                            )
                            .route(
                                "/locations/{id}",
                                web::put().to(handlers::admin::update_location),
                            )
                            .route(
                                "/locations/{id}",
                                web::delete().to(handlers::admin::delete_location),
                            )
                            .route("/teams", web::post().to(handlers::admin::create_team))
                            .route("/teams", web::get().to(handlers::admin::get_teams))
                            .route("/teams/{id}", web::get().to(handlers::admin::get_team))
                            .route("/teams/{id}", web::put().to(handlers::admin::update_team))
                            .route(
                                "/teams/{id}",
                                web::delete().to(handlers::admin::delete_team),
                            )
                            .route(
                                "/teams/{team_id}/members/{user_id}",
                                web::post().to(handlers::admin::add_team_member),
                            )
                            .route(
                                "/teams/{team_id}/members",
                                web::get().to(handlers::admin::get_team_members),
                            )
                            .route(
                                "/teams/{team_id}/members/{user_id}",
                                web::delete().to(handlers::admin::remove_team_member),
                            ),
                    )
                    .service(
                        web::scope("/shifts")
                            .route("", web::post().to(handlers::shifts::create_shift))
                            .route("", web::get().to(handlers::shifts::get_shifts))
                            .route("/{id}", web::get().to(handlers::shifts::get_shift))
                            .route("/{id}", web::put().to(handlers::shifts::update_shift))
                            .route("/{id}", web::delete().to(handlers::shifts::delete_shift))
                            .route(
                                "/{id}/assign",
                                web::post().to(handlers::shifts::assign_shift),
                            )
                            .route(
                                "/{id}/unassign",
                                web::post().to(handlers::shifts::unassign_shift),
                            )
                            .route(
                                "/{id}/status",
                                web::post().to(handlers::shifts::update_shift_status),
                            )
                            .route("/{id}/claim", web::post().to(handlers::shifts::claim_shift)),
                    ),
            )
    })
    .bind(&server_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
