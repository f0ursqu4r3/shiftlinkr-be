use actix_web::web;

pub mod admin;
pub mod auth;
pub mod company;
pub mod pto_balance;
pub mod schedules;
pub mod shifts;
pub mod skills;
pub mod stats;
pub mod swaps;
pub mod time_off;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .configure(auth::configure)
            .configure(admin::configure)
            .configure(shifts::configure)
            .configure(time_off::configure)
            .configure(swaps::configure)
            .configure(stats::configure)
            .configure(pto_balance::configure)
            .configure(skills::configure)
            .configure(schedules::configure)
            .configure(company::configure),
    );
}
