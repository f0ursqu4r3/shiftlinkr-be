use actix_web::web;

use crate::handlers::time_off;
use crate::middleware::{CacheLayer, ResponseCacheMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let cache_layer = CacheLayer::new(1000, 120);
    cfg.service(
        web::scope("/time-off")
            .app_data(web::Data::new(cache_layer.clone()))
            .wrap(ResponseCacheMiddleware::new(cache_layer.clone()))
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
    );
}
