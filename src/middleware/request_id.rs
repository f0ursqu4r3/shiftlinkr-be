use std::future::{Ready, ready};

use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    http::header::HeaderValue,
};
use futures_util::future::LocalBoxFuture;
use uuid::Uuid;

// Middleware factory
pub struct RequestIdMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestIdMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestIdMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIdMiddlewareService { service }))
    }
}

pub struct RequestIdMiddlewareService<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for RequestIdMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Generate a new correlation ID
        let correlation_id = Uuid::new_v4().to_string();

        // Check if X-Correlation-ID header exists, otherwise use generated one
        let final_correlation_id = req
            .headers()
            .get("X-Correlation-ID")
            .and_then(|h| h.to_str().ok())
            .unwrap_or(&correlation_id)
            .to_string();

        // Store correlation ID in request extensions for access in handlers
        req.extensions_mut().insert(final_correlation_id.clone());

        // Add correlation ID to response headers
        let fut = self.service.call(req);

        Box::pin(async move {
            let mut res = fut.await?;

            // Add correlation ID to response headers
            res.headers_mut().insert(
                actix_web::http::header::HeaderName::from_static("x-correlation-id"),
                HeaderValue::from_str(&final_correlation_id).unwrap(),
            );

            Ok(res)
        })
    }
}

// Extension trait to easily get correlation ID from request
pub trait RequestIdExt {
    fn correlation_id(&self) -> Option<String>;
}

impl RequestIdExt for actix_web::HttpRequest {
    fn correlation_id(&self) -> Option<String> {
        self.extensions().get::<String>().cloned()
    }
}

impl RequestIdExt for ServiceRequest {
    fn correlation_id(&self) -> Option<String> {
        self.extensions().get::<String>().cloned()
    }
}
