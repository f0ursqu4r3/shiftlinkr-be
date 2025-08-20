use actix_web::web;

use crate::handlers::auth;
use crate::middleware::{AuthRateLimiter, CacheLayer, ResponseCacheMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let cache_layer = CacheLayer::new(1000, 60);
    cfg.service(
        web::scope("/auth")
            .app_data(web::Data::new(cache_layer.clone()))
            .wrap(ResponseCacheMiddleware::new(cache_layer.clone()))
            .service(
                web::resource("/register")
                    .wrap(AuthRateLimiter::registration())
                    .route(web::post().to(auth::register)),
            )
            .service(
                web::resource("/login")
                    .wrap(AuthRateLimiter::login())
                    .route(web::post().to(auth::login)),
            )
            .service(
                web::resource("/forgot-password")
                    .wrap(AuthRateLimiter::password_reset())
                    .route(web::post().to(auth::forgot_password)),
            )
            .service(
                web::resource("/reset-password")
                    .wrap(AuthRateLimiter::password_reset())
                    .route(web::post().to(auth::reset_password)),
            )
            .route("/me", web::get().to(auth::me))
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
