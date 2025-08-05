use actix_web::web;

use crate::handlers::skills;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/skills")
            .route("", web::post().to(skills::create_skill))
            .route("", web::get().to(skills::get_all_skills))
            .route("/{id}", web::get().to(skills::get_skill))
            .route("/{id}", web::put().to(skills::update_skill))
            .route("/{id}", web::delete().to(skills::delete_skill))
            .route("/{id}/users", web::get().to(skills::get_users_with_skill)),
    )
    .service(
        web::scope("/user-skills")
            .route("", web::post().to(skills::add_user_skill))
            .route("/{user_id}", web::get().to(skills::get_user_skills))
            .route(
                "/{user_id}/{skill_id}",
                web::put().to(skills::update_user_skill),
            )
            .route(
                "/{user_id}/{skill_id}",
                web::delete().to(skills::remove_user_skill),
            ),
    )
    .service(
        web::scope("/shift-skills")
            .route("", web::post().to(skills::add_shift_required_skill))
            .route(
                "/{shift_id}",
                web::get().to(skills::get_shift_required_skills),
            )
            .route(
                "/{shift_id}/{skill_id}",
                web::delete().to(skills::remove_shift_required_skill),
            ),
    );
}
