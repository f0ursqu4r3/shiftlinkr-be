use actix_web::web;

use crate::handlers::schedules;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/schedules")
            .route("", web::post().to(schedules::create_user_schedule))
            .route("/{user_id}", web::get().to(schedules::get_user_schedule))
            .route("/{user_id}", web::put().to(schedules::update_user_schedule))
            .route(
                "/{user_id}",
                web::delete().to(schedules::delete_user_schedule),
            )
            .route(
                "/{user_id}/suggestions",
                web::get().to(schedules::get_user_shift_suggestions),
            ),
    )
    .service(
        web::scope("/assignments")
            .route("", web::post().to(schedules::create_shift_assignment))
            .route("/{id}", web::get().to(schedules::get_shift_assignment))
            .route(
                "/shift/{shift_id}",
                web::get().to(schedules::get_shift_assignments_by_shift),
            )
            .route(
                "/user/{user_id}",
                web::get().to(schedules::get_shift_assignments_by_user),
            )
            .route(
                "/user/{user_id}/pending",
                web::get().to(schedules::get_pending_assignments_for_user),
            )
            .route(
                "/{id}/respond",
                web::post().to(schedules::respond_to_assignment),
            )
            .route("/{id}/cancel", web::post().to(schedules::cancel_assignment)),
    );
}
