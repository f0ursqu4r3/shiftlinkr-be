use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;

pub mod auth;
pub mod config;
pub mod database;
pub mod handlers;

use auth::AuthService;
use config::Config;
use database::{
    init_database, invite_repository::InviteRepository, location_repository::LocationRepository,
    password_reset_repository::PasswordResetTokenRepository,
    pto_balance_repository::PtoBalanceRepository, shift_claim_repository::ShiftClaimRepository,
    shift_repository::ShiftRepository, shift_swap_repository::ShiftSwapRepository,
    stats_repository::StatsRepository, time_off_repository::TimeOffRepository,
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
    let password_reset_repository = PasswordResetTokenRepository::new(pool.clone());
    let invite_repository = InviteRepository::new(pool.clone());
    let time_off_repository = TimeOffRepository::new(pool.clone());
    let shift_swap_repository = ShiftSwapRepository::new(pool.clone());
    let stats_repository = StatsRepository::new(pool.clone());
    let pto_balance_repository = PtoBalanceRepository::new(pool.clone());
    let shift_claim_repository = ShiftClaimRepository::new(pool.clone());
    let auth_service = AuthService::new(user_repository, password_reset_repository, config.clone());

    // Create app state and repository data
    let app_state = web::Data::new(AppState { auth_service });
    let location_repo_data = web::Data::new(location_repository);
    let shift_repo_data = web::Data::new(shift_repository);
    let invite_repo_data = web::Data::new(invite_repository);
    let time_off_repo_data = web::Data::new(time_off_repository);
    let shift_swap_repo_data = web::Data::new(shift_swap_repository);
    let stats_repo_data = web::Data::new(stats_repository);
    let pto_balance_repo_data = web::Data::new(pto_balance_repository);
    let shift_claim_repo_data = web::Data::new(shift_claim_repository);
    let config_data = web::Data::new(config.clone());

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .app_data(location_repo_data.clone())
            .app_data(shift_repo_data.clone())
            .app_data(invite_repo_data.clone())
            .app_data(time_off_repo_data.clone())
            .app_data(shift_swap_repo_data.clone())
            .app_data(stats_repo_data.clone())
            .app_data(pto_balance_repo_data.clone())
            .app_data(shift_claim_repo_data.clone())
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
                            .route("/me", web::get().to(handlers::auth::me))
                            .route(
                                "/forgot-password",
                                web::post().to(handlers::auth::forgot_password),
                            )
                            .route(
                                "/reset-password",
                                web::post().to(handlers::auth::reset_password),
                            )
                            .route("/invite", web::post().to(handlers::auth::create_invite))
                            .route("/invite/{token}", web::get().to(handlers::auth::get_invite))
                            .route(
                                "/invite/accept",
                                web::post().to(handlers::auth::accept_invite),
                            )
                            .route("/invites", web::get().to(handlers::auth::get_my_invites)),
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
                            .route("/{id}/claim", web::post().to(handlers::shifts::claim_shift))
                            .route(
                                "/{id}/claims",
                                web::get().to(handlers::shifts::get_shift_claims),
                            ),
                    )
                    // Shift claims management
                    .service(
                        web::scope("/shift-claims")
                            .route("", web::get().to(handlers::shifts::get_pending_claims))
                            .route("/my", web::get().to(handlers::shifts::get_my_claims))
                            .route(
                                "/{id}/approve",
                                web::post().to(handlers::shifts::approve_shift_claim),
                            )
                            .route(
                                "/{id}/reject",
                                web::post().to(handlers::shifts::reject_shift_claim),
                            )
                            .route(
                                "/{id}/cancel",
                                web::post().to(handlers::shifts::cancel_shift_claim),
                            ),
                    )
                    .service(
                        web::scope("/time-off")
                            .route(
                                "",
                                web::post().to(handlers::time_off::create_time_off_request),
                            )
                            .route("", web::get().to(handlers::time_off::get_time_off_requests))
                            .route(
                                "/{id}",
                                web::get().to(handlers::time_off::get_time_off_request),
                            )
                            .route(
                                "/{id}",
                                web::put().to(handlers::time_off::update_time_off_request),
                            )
                            .route(
                                "/{id}",
                                web::delete().to(handlers::time_off::delete_time_off_request),
                            )
                            .route(
                                "/{id}/approve",
                                web::post()
                                    .to(handlers::time_off::approve_time_off_request_endpoint),
                            )
                            .route(
                                "/{id}/deny",
                                web::post().to(handlers::time_off::deny_time_off_request),
                            ),
                    )
                    .service(
                        web::scope("/swaps")
                            .route("", web::post().to(handlers::swaps::create_swap_request))
                            .route("", web::get().to(handlers::swaps::get_swap_requests))
                            .route("/{id}", web::get().to(handlers::swaps::get_swap_request))
                            .route(
                                "/{id}/respond",
                                web::post().to(handlers::swaps::respond_to_swap),
                            )
                            .route(
                                "/{id}/approve",
                                web::post().to(handlers::swaps::approve_swap_request),
                            )
                            .route(
                                "/{id}/deny",
                                web::post().to(handlers::swaps::deny_swap_request),
                            ),
                    )
                    .service(
                        web::scope("/stats")
                            .route(
                                "/dashboard",
                                web::get().to(handlers::stats::get_dashboard_stats),
                            )
                            .route("/shifts", web::get().to(handlers::stats::get_shift_stats))
                            .route(
                                "/time-off",
                                web::get().to(handlers::stats::get_time_off_stats),
                            ),
                    )
                    .service(
                        web::scope("/pto-balance")
                            .route("", web::get().to(handlers::pto_balance::get_pto_balance))
                            .route(
                                "/{user_id}",
                                web::put().to(handlers::pto_balance::update_pto_balance),
                            )
                            .route(
                                "/{user_id}/adjust",
                                web::post().to(handlers::pto_balance::adjust_pto_balance),
                            )
                            .route(
                                "/{user_id}/history",
                                web::get().to(handlers::pto_balance::get_pto_balance_history),
                            )
                            .route(
                                "/{user_id}/accrual",
                                web::post().to(handlers::pto_balance::process_pto_accrual),
                            ),
                    ),
            )
    })
    .bind(&server_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
