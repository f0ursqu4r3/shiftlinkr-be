use actix_web::web;

use crate::handlers::stats;
use crate::middleware::{CacheLayer, ResponseCacheMiddleware};

pub fn configure(cfg: &mut web::ServiceConfig) {
    let cache_layer = CacheLayer::new(1000, 300); // capacity, ttl secs
    cfg.service(
        web::scope("/stats")
            .wrap(ResponseCacheMiddleware::new(cache_layer))
            .route("/dashboard", web::get().to(stats::get_dashboard_stats))
            .route("/shifts", web::get().to(stats::get_shift_stats))
            .route("/time-off", web::get().to(stats::get_time_off_stats)),
    );
}
