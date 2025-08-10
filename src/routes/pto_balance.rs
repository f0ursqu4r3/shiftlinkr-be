use actix_web::web;

use crate::handlers::pto_balance;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/pto-balance")
            .route("", web::get().to(pto_balance::get_pto_balance))
            .route("/{user_id}", web::get().to(pto_balance::get_pto_balance))
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
                "/{user_id}/accrual",
                web::post().to(pto_balance::process_pto_accrual),
            ),
    );
}
