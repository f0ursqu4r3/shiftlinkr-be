use crate::handlers::swaps;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/swaps")
            .route("", web::post().to(swaps::create_swap_request))
            .route("", web::get().to(swaps::get_swap_requests))
            .route("/{id}", web::get().to(swaps::get_swap_request))
            .route("/{id}/respond", web::post().to(swaps::respond_to_swap))
            .route("/{id}/approve", web::post().to(swaps::approve_swap_request))
            .route("/{id}/deny", web::post().to(swaps::deny_swap_request)),
    );
}
