use actix_web::web;

use crate::handlers::company;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/companies")
            .route("", web::get().to(company::get_user_companies))
            .route("", web::post().to(company::create_company))
            .route("/primary", web::get().to(company::get_user_primary_company))
            .route("/employees", web::get().to(company::get_company_employees))
            .route(
                "/employees",
                web::post().to(company::add_employee_to_company),
            )
            .route(
                "/employees/{user_id}",
                web::delete().to(company::remove_employee_from_company),
            )
            .route(
                "/employees/{user_id}/role",
                web::put().to(company::update_employee_role),
            ),
    );
}
