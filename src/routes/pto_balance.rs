use actix_web::web;

use crate::handlers::pto_balance;
use crate::middleware::GlobalRateLimiter;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pto-balance")
            .route("", web::get().to(pto_balance::get_pto_balance))
            .route("/{user_id}", web::get().to(pto_balance::get_pto_balance))
            .route(
                "/{user_id}/history",
                web::get().to(pto_balance::get_pto_balance_history),
            )
            .service(
                // Apply stricter rate limiting to PTO balance modification operations
                web::resource("/{user_id}")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::put().to(pto_balance::update_pto_balance)),
            )
            .service(
                web::resource("/{user_id}/adjust")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::post().to(pto_balance::adjust_pto_balance)),
            )
            .service(
                web::resource("/{user_id}/accrual")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::post().to(pto_balance::process_pto_accrual)),
            ),
    );
}
