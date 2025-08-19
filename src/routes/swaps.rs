use actix_web::web;

use crate::handlers::swaps;
use crate::middleware::{CacheLayer, ResponseCacheMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let cache_layer = CacheLayer::new(1000, 120);
    cfg.service(
        web::scope("/swaps")
            .app_data(web::Data::new(cache_layer.clone()))
            .wrap(ResponseCacheMiddleware::new(cache_layer.clone()))
            .route("", web::post().to(swaps::create_swap_request))
            .route("", web::get().to(swaps::get_swap_requests))
            .route("/{id}", web::get().to(swaps::get_swap_request))
            .route("/{id}/respond", web::post().to(swaps::respond_to_swap))
            .route("/{id}/approve", web::post().to(swaps::approve_swap_request))
            .route("/{id}/deny", web::post().to(swaps::deny_swap_request)),
    );
}
