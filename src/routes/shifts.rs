use actix_web::web;

use crate::handlers::shifts;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
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
    .service(
        web::scope("/shift-claims")
            .route("", web::get().to(shifts::get_pending_claims))
            .route("/my", web::get().to(shifts::get_my_claims))
            .route("/{id}/approve", web::post().to(shifts::approve_shift_claim))
            .route("/{id}/reject", web::post().to(shifts::reject_shift_claim))
            .route("/{id}/cancel", web::post().to(shifts::cancel_shift_claim)),
    );
}
