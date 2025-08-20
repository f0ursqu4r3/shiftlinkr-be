use actix_web::web;

use crate::handlers::company;
use crate::middleware::GlobalRateLimiter;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/companies")
            .route("", web::get().to(company::get_user_companies))
            .route("/primary", web::get().to(company::get_user_primary_company))
            .route("/employees", web::get().to(company::get_company_employees))
            .service(
                // Apply stricter rate limiting to sensitive company operations
                web::resource("")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::post().to(company::create_company)),
            )
            .service(
                web::resource("/employees")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::post().to(company::add_employee_to_company)),
            )
            .service(
                web::resource("/employees/{user_id}")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::delete().to(company::remove_employee_from_company)),
            )
            .service(
                web::resource("/employees/{user_id}/role")
                    .wrap(GlobalRateLimiter::sensitive())
                    .route(web::put().to(company::update_employee_role)),
            ),
    );
}
