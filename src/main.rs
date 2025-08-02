use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;

use be::{
    config::Config,
    database::{
        init_database,
        repositories::{
            ActivityRepository, CompanyRepository, InviteRepository, LocationRepository,
            PasswordResetTokenRepository, PtoBalanceRepository, ScheduleRepository,
            ShiftClaimRepository, ShiftRepository, ShiftSwapRepository, SkillRepository,
            StatsRepository, TeamRepository, TimeOffRepository, UserRepository,
        },
    },
    middleware::RequestId,
    routes,
    services::{ActivityLogger, AuthService, UserContextService},
};

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
    let run_migrations = config.run_migrations;
    println!("🔗 Initializing database at {}", config.database_url);
    let pool = init_database(&config.database_url, run_migrations).await?;
    println!("🔥 Database initialized");

    // Initialize repositories and services
    let company_repository = CompanyRepository::new(pool.clone());
    let invite_repository = InviteRepository::new(pool.clone());
    let location_repository = LocationRepository::new(pool.clone());
    let password_reset_repository = PasswordResetTokenRepository::new(pool.clone());
    let pto_balance_repository = PtoBalanceRepository::new(pool.clone());
    let schedule_repository = ScheduleRepository::new(pool.clone());
    let shift_claim_repository = ShiftClaimRepository::new(pool.clone());
    let shift_repository = ShiftRepository::new(pool.clone());
    let shift_swap_repository = ShiftSwapRepository::new(pool.clone());
    let skill_repository = SkillRepository::new(pool.clone());
    let stats_repository = StatsRepository::new(pool.clone());
    let team_repository = TeamRepository::new(pool.clone());
    let time_off_repository = TimeOffRepository::new(pool.clone());
    let user_repository = UserRepository::new(pool.clone());

    // Create user context service and auth service
    let user_context_service =
        UserContextService::new(user_repository.clone(), company_repository.clone());
    let auth_service = AuthService::new(
        config.clone(),
        user_repository.clone(),
        company_repository.clone(),
        password_reset_repository,
    );

    // Create activity logger
    let activity_repository = ActivityRepository::new(pool.clone());
    let activity_logger = ActivityLogger::new(activity_repository.clone());

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(activity_logger.clone()))
            .app_data(web::Data::new(auth_service.clone()))
            .app_data(web::Data::new(company_repository.clone()))
            .app_data(web::Data::new(config.clone()))
            .app_data(web::Data::new(invite_repository.clone()))
            .app_data(web::Data::new(location_repository.clone()))
            .app_data(web::Data::new(pto_balance_repository.clone()))
            .app_data(web::Data::new(schedule_repository.clone()))
            .app_data(web::Data::new(shift_claim_repository.clone()))
            .app_data(web::Data::new(shift_repository.clone()))
            .app_data(web::Data::new(shift_swap_repository.clone()))
            .app_data(web::Data::new(skill_repository.clone()))
            .app_data(web::Data::new(stats_repository.clone()))
            .app_data(web::Data::new(team_repository.clone()))
            .app_data(web::Data::new(time_off_repository.clone()))
            .app_data(web::Data::new(user_context_service.clone()))
            .app_data(web::Data::new(user_repository.clone()))
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
            .wrap(RequestId)
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
