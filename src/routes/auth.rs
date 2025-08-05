use actix_web::web;

use crate::handlers::auth;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/register", web::post().to(auth::register))
            .route("/login", web::post().to(auth::login))
            .route("/me", web::get().to(auth::me))
            .route("/forgot-password", web::post().to(auth::forgot_password))
            .route("/reset-password", web::post().to(auth::reset_password))
            .route("/invite", web::post().to(auth::create_invite))
            .route("/invite/{token}", web::get().to(auth::get_invite))
            .route(
                "/invite/{token}/accept",
                web::post().to(auth::accept_invite),
            )
            .route(
                "/invite/{token}/reject",
                web::post().to(auth::reject_invite),
            )
            .route("/invites", web::get().to(auth::get_my_invites))
            .route("/switch-company/{id}", web::post().to(auth::switch_company)),
    );
}
