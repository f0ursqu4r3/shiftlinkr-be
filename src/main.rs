use actix_cors::Cors;
use actix_web::{get, middleware::Logger, web, App, HttpResponse, HttpServer, Responder};
use anyhow::Result;

use be::database::{
    init_database,
    repositories::{
        ActivityRepository, CompanyRepository, InviteRepository, LocationRepository,
        PasswordResetTokenRepository, PtoBalanceRepository, ScheduleRepository,
        ShiftClaimRepository, ShiftRepository, ShiftSwapRepository, SkillRepository,
        StatsRepository, TimeOffRepository, UserRepository,
    },
};
use be::handlers::{
    admin, auth, company, pto_balance, schedules, shifts, skills, stats, swaps, time_off,
};
use be::middleware::RequestId;
use be::services::{ActivityLogger, UserContextService};
use be::{AppState, AuthService, Config};

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
    let company_repository = CompanyRepository::new(pool.clone());
    let skill_repository = SkillRepository::new(pool.clone());
    let schedule_repository = ScheduleRepository::new(pool.clone());
    let auth_service = AuthService::new(
        user_repository.clone(),
        company_repository.clone(),
        password_reset_repository,
        config.clone(),
    );

    // Create user context service
    let user_context_service =
        UserContextService::new(user_repository.clone(), company_repository.clone());

    // Create app state and repository data
    let activity_repository = ActivityRepository::new(pool.clone());
    let activity_logger = ActivityLogger::new(activity_repository.clone());

    let app_state = web::Data::new(AppState {
        auth_service,
        company_repository: company_repository.clone(),
        activity_repository,
        activity_logger: activity_logger.clone(),
    });
    let user_repo_data = web::Data::new(user_repository);
    let location_repo_data = web::Data::new(location_repository);
    let shift_repo_data = web::Data::new(shift_repository);
    let invite_repo_data = web::Data::new(invite_repository);
    let time_off_repo_data = web::Data::new(time_off_repository);
    let shift_swap_repo_data = web::Data::new(shift_swap_repository);
    let stats_repo_data = web::Data::new(stats_repository);
    let pto_balance_repo_data = web::Data::new(pto_balance_repository);
    let shift_claim_repo_data = web::Data::new(shift_claim_repository);
    let company_repo_data = web::Data::new(company_repository);
    let skill_repo_data = web::Data::new(skill_repository);
    let schedule_repo_data = web::Data::new(schedule_repository);
    let config_data = web::Data::new(config.clone());
    let activity_logger_data = web::Data::new(activity_logger);
    let user_context_service_data = web::Data::new(user_context_service);

    let server_address = config.server_address();
    println!("🌐 Server starting on http://{}", server_address);

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(config.clone())
            .app_data(app_state.clone())
            .app_data(user_repo_data.clone())
            .app_data(location_repo_data.clone())
            .app_data(shift_repo_data.clone())
            .app_data(invite_repo_data.clone())
            .app_data(time_off_repo_data.clone())
            .app_data(shift_swap_repo_data.clone())
            .app_data(stats_repo_data.clone())
            .app_data(pto_balance_repo_data.clone())
            .app_data(shift_claim_repo_data.clone())
            .app_data(company_repo_data.clone())
            .app_data(skill_repo_data.clone())
            .app_data(schedule_repo_data.clone())
            .app_data(config_data.clone())
            .app_data(activity_logger_data.clone())
            .app_data(user_context_service_data.clone())
            .wrap(
                Cors::default()
                    .allowed_origin("http://localhost:3000")
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
            .service(
                web::scope("/api/v1")
                    .service(
                        web::scope("/auth")
                            .route("/register", web::post().to(auth::register))
                            .route("/login", web::post().to(auth::login))
                            .route("/me", web::get().to(auth::me))
                            .route("/forgot-password", web::post().to(auth::forgot_password))
                            .route("/reset-password", web::post().to(auth::reset_password))
                            .route("/invite", web::post().to(auth::create_invite))
                            .route("/invite/{token}", web::get().to(auth::get_invite))
                            .route("/invite/accept", web::post().to(auth::accept_invite))
                            .route("/invites", web::get().to(auth::get_my_invites)),
                    )
                    .service(
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
                                "/teams/{team_id}/members/{user_id}",
                                web::post().to(admin::add_team_member),
                            )
                            .route(
                                "/teams/{team_id}/members",
                                web::get().to(admin::get_team_members),
                            )
                            .route(
                                "/teams/{team_id}/members/{user_id}",
                                web::delete().to(admin::remove_team_member),
                            )
                            .route("/users", web::get().to(admin::get_users))
                            .route("/users/{id}", web::put().to(admin::update_user))
                            .route("/users/{id}", web::delete().to(admin::delete_user)),
                    )
                    .service(
                        web::scope("/shifts")
                            .route("", web::post().to(shifts::create_shift))
                            .route("", web::get().to(shifts::get_shifts))
                            .route("/{id}", web::get().to(shifts::get_shift))
                            .route("/{id}", web::put().to(shifts::update_shift))
                            .route("/{id}", web::delete().to(shifts::delete_shift))
                            .route("/{id}/assign", web::post().to(shifts::assign_shift))
                            .route("/{id}/unassign", web::post().to(shifts::unassign_shift))
                            .route("/{id}/status", web::post().to(shifts::update_shift_status))
                            .route("/{id}/claim", web::post().to(shifts::claim_shift))
                            .route("/{id}/claims", web::get().to(shifts::get_shift_claims)),
                    )
                    // Shift claims management
                    .service(
                        web::scope("/shift-claims")
                            .route("", web::get().to(shifts::get_pending_claims))
                            .route("/my", web::get().to(shifts::get_my_claims))
                            .route("/{id}/approve", web::post().to(shifts::approve_shift_claim))
                            .route("/{id}/reject", web::post().to(shifts::reject_shift_claim))
                            .route("/{id}/cancel", web::post().to(shifts::cancel_shift_claim)),
                    )
                    .service(
                        web::scope("/time-off")
                            .route("", web::post().to(time_off::create_time_off_request))
                            .route("", web::get().to(time_off::get_time_off_requests))
                            .route("/{id}", web::get().to(time_off::get_time_off_request))
                            .route("/{id}", web::put().to(time_off::update_time_off_request))
                            .route("/{id}", web::delete().to(time_off::delete_time_off_request))
                            .route(
                                "/{id}/approve",
                                web::post().to(time_off::approve_time_off_request),
                            )
                            .route(
                                "/{id}/deny",
                                web::post().to(time_off::deny_time_off_request),
                            ),
                    )
                    .service(
                        web::scope("/swaps")
                            .route("", web::post().to(swaps::create_swap_request))
                            .route("", web::get().to(swaps::get_swap_requests))
                            .route("/{id}", web::get().to(swaps::get_swap_request))
                            .route("/{id}/respond", web::post().to(swaps::respond_to_swap))
                            .route("/{id}/approve", web::post().to(swaps::approve_swap_request))
                            .route("/{id}/deny", web::post().to(swaps::deny_swap_request)),
                    )
                    .service(
                        web::scope("/stats")
                            .route("/dashboard", web::get().to(stats::get_dashboard_stats))
                            .route("/shifts", web::get().to(stats::get_shift_stats))
                            .route("/time-off", web::get().to(stats::get_time_off_stats)),
                    )
                    .service(
                        web::scope("/pto-balance")
                            .route("", web::get().to(pto_balance::get_pto_balance))
                            .route("/{user_id}", web::put().to(pto_balance::update_pto_balance))
                            .route(
                                "/{user_id}/adjust",
                                web::post().to(pto_balance::adjust_pto_balance),
                            )
                            .route(
                                "/{user_id}/history",
                                web::get().to(pto_balance::get_pto_balance_history),
                            )
                            .route(
                                "/{user_id}/accrual/{company_id}",
                                web::post().to(pto_balance::process_pto_accrual),
                            ),
                    )
                    .service(
                        web::scope("/skills")
                            .route("", web::post().to(skills::create_skill))
                            .route("", web::get().to(skills::get_all_skills))
                            .route("/{id}", web::get().to(skills::get_skill))
                            .route("/{id}", web::put().to(skills::update_skill))
                            .route("/{id}", web::delete().to(skills::delete_skill))
                            .route("/{id}/users", web::get().to(skills::get_users_with_skill))
                    )
                    .service(
                        web::scope("/user-skills")
                            .route("", web::post().to(skills::add_user_skill))
                            .route("/{user_id}", web::get().to(skills::get_user_skills))
                            .route("/{id}", web::put().to(skills::update_user_skill))
                            .route("/{user_id}/{skill_id}", web::delete().to(skills::remove_user_skill))
                    )
                    .service(
                        web::scope("/shift-skills")
                            .route("", web::post().to(skills::add_shift_required_skill))
                            .route("/{shift_id}", web::get().to(skills::get_shift_required_skills))
                            .route("/{shift_id}/{skill_id}", web::delete().to(skills::remove_shift_required_skill))
                    )
                    .service(
                        web::scope("/schedules")
                            .route("", web::post().to(schedules::create_user_schedule))
                            .route("/{user_id}", web::get().to(schedules::get_user_schedule))
                            .route("/{user_id}", web::put().to(schedules::update_user_schedule))
                            .route("/{user_id}", web::delete().to(schedules::delete_user_schedule))
                    )
                    .service(
                        web::scope("/assignments")
                            .route("", web::post().to(schedules::create_shift_assignment))
                            .route("/{id}", web::get().to(schedules::get_shift_assignment))
                            .route("/shift/{shift_id}", web::get().to(schedules::get_shift_assignments_by_shift))
                            .route("/user/{user_id}", web::get().to(schedules::get_shift_assignments_by_user))
                            .route("/user/{user_id}/pending", web::get().to(schedules::get_pending_assignments_for_user))
                            .route("/{id}/respond", web::post().to(schedules::respond_to_assignment))
                            .route("/{id}/cancel", web::post().to(schedules::cancel_assignment))
                    )
                    .service(
                        web::scope("/companies")
                            .route("", web::get().to(company::get_user_companies))
                            .route("", web::post().to(company::create_company))
                            .route("/primary", web::get().to(company::get_user_primary_company))
                            .route(
                                "/{company_id}/employees",
                                web::get().to(company::get_company_employees),
                            )
                            .route(
                                "/{company_id}/employees",
                                web::post().to(company::add_employee_to_company),
                            )
                            .route(
                                "/{company_id}/employees/{user_id}",
                                web::delete().to(company::remove_employee_from_company),
                            )
                            .route(
                                "/{company_id}/employees/{user_id}/role",
                                web::put().to(company::update_employee_role),
                            ),
                    ),
            )
    })
    .bind(&server_address)?
    .run()
    .await
    .map_err(|e| anyhow::anyhow!("Server error: {}", e))
}
