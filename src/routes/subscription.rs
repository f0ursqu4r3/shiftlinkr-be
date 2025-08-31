use actix_web::web;

use crate::handlers::subscription;
use crate::middleware::CacheLayer;

pub fn configure(cfg: &mut web::ServiceConfig) {
    let cache_layer = CacheLayer::new(1000, 120);
    cfg.service(
        web::scope("/subscription")
            .app_data(web::Data::new(cache_layer.clone()))
            .route(
                "/plans",
                web::get().to(subscription::get_subscription_plans),
            )
            .route(
                "/{company_id}",
                web::get().to(subscription::get_company_subscription),
            )
            .route(
                "/{company_id}",
                web::post().to(subscription::create_subscription),
            )
            .route(
                "/{company_id}/cancel",
                web::post().to(subscription::cancel_subscription),
            )
            .route(
                "/{company_id}/payment-methods",
                web::get().to(subscription::get_payment_methods),
            )
            .route(
                "/{company_id}/invoices",
                web::get().to(subscription::get_invoices),
            )
            .route(
                "/owner-status",
                web::get().to(subscription::check_owner_status),
            ),
    );
}
