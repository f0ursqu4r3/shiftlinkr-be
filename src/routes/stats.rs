use actix_web::web;

use crate::handlers::stats;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/stats")
            .route("/dashboard", web::get().to(stats::get_dashboard_stats))
            .route("/shifts", web::get().to(stats::get_shift_stats))
            .route("/time-off", web::get().to(stats::get_time_off_stats)),
    );
}
