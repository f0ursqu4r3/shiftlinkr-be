use crate::handlers::admin;
use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/admin")
            .route("/locations", web::post().to(admin::create_location))
            .route("/locations", web::get().to(admin::get_locations))
            .route("/locations/{id}", web::get().to(admin::get_location))
            .route("/locations/{id}", web::put().to(admin::update_location))
            .route("/locations/{id}", web::delete().to(admin::delete_location))
            .route("/teams", web::post().to(admin::create_team))
            .route("/teams", web::get().to(admin::get_teams))
            .route("/teams/{id}", web::get().to(admin::get_team))
            .route("/teams/{id}", web::put().to(admin::update_team))
            .route("/teams/{id}", web::delete().to(admin::delete_team))
            .route(
                "/teams/{team_id}/members/{user_id}",
                web::post().to(admin::add_team_member),
            )
            .route(
                "/teams/{team_id}/members",
                web::get().to(admin::get_team_members),
            )
            .route(
                "/teams/{team_id}/members/{user_id}",
                web::delete().to(admin::remove_team_member),
            )
            .route("/users", web::get().to(admin::get_users))
            .route("/users/{id}", web::put().to(admin::update_user))
            .route("/users/{id}", web::delete().to(admin::delete_user)),
    );
}
