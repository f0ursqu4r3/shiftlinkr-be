use actix_cors::Cors;
use actix_web::{App, HttpServer, Responder, get, middleware::Logger};
use anyhow::Result;

use be::{
    config::Config,
    database::init_database,
    handlers::shared::ApiResponse,
    middleware::{
        GlobalRateLimiter, RateLimitStore, RequestIdMiddleware, RequestInfoMiddleware,
        cleanup_rate_limits,
    },
    routes,
};

#[get("/")]
async fn hello() -> impl Responder {
    ApiResponse::success("ShiftLinkr API v1.0")
}

#[get("/health")]
async fn health() -> impl Responder {
    ApiResponse::success("OK")
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
    let run_migrations = config.run_migrations;
    println!("🔗 Initializing database at {}", config.database_url);

    init_database(&config.database_url, run_migrations).await?;
    println!("🔥 Database initialized");

    // Create shared rate limit store for cleanup task
    let rate_limit_store = RateLimitStore::new();
    let cleanup_store = rate_limit_store.clone();

    // Start cleanup task for rate limits
    tokio::spawn(async move {
        cleanup_rate_limits(cleanup_store, 300).await; // Cleanup every 5 minutes
    });

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);
    println!("🛡️ Rate limiting enabled with cleanup task started");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allowed_origin(&config.client_base_url.clone())
                    .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
                    .allowed_headers(vec![
                        "Authorization",
                        "Content-Type",
                        "Accept",
                        "X-Requested-With",
                        "X-Correlation-ID",
                    ])
                    .max_age(3600),
            )
            .wrap(GlobalRateLimiter::general()) // Global rate limiting
            .wrap(RequestIdMiddleware)
            .wrap(RequestInfoMiddleware)
            .wrap(Logger::new(
                r#"%a "%r" %s %b "%{Referer}i" "%{User-Agent}i" %T correlation_id=%{x-correlation-id}o"#
            ))
            .service(hello)
            .service(health)
            .configure(routes::configure)
    })
    .bind(&server_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
