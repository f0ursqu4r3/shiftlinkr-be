use crate::handlers::time_off;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
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
    );
}
