use actix_web::{App, HttpResponse, HttpServer, Responder, get, middleware::Logger, web};
use anyhow::Result;

mod auth;
mod config;
mod database;
mod handlers;

use auth::AuthService;
use config::Config;
use database::{init_database, user_repository::UserRepository};

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
    let user_repository = UserRepository::new(pool);
    let auth_service = AuthService::new(user_repository, config.clone());

    // Create app state
    let app_state = web::Data::new(AppState { auth_service });

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .wrap(Logger::default())
            .service(hello)
            .service(health)
            .service(
                web::scope("/api/v1").service(
                    web::scope("/auth")
                        .route("/register", web::post().to(handlers::auth::register))
                        .route("/login", web::post().to(handlers::auth::login))
                        .route("/me", web::get().to(handlers::auth::me)),
                ),
            )
    })
    .bind(&server_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
