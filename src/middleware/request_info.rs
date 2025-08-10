use std::{future::Future, pin::Pin, rc::Rc};

use actix_web::{
    dev::{forward_ready, Payload, Service, ServiceRequest, ServiceResponse, Transform},
    Error, FromRequest, HttpMessage, HttpRequest,
};
use futures_util::future::{ready, Ready};

#[derive(Clone, Debug)]
pub struct RequestInfo {
    pub user_agent: String,
    pub ip_address: String,
    pub method: String,
    pub path: String,
}

impl FromRequest for RequestInfo {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        // Try to get RequestInfo from extensions (set by middleware)
        if let Some(request_info) = req.extensions().get::<RequestInfo>() {
            ready(Ok(request_info.clone()))
        } else {
            // Fallback: create RequestInfo directly from HttpRequest
            let request_info = RequestInfo {
                user_agent: req
                    .headers()
                    .get("user-agent")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string(),
                ip_address: req
                    .connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string(),
                method: req.method().to_string(),
                path: req.path().to_string(),
            };
            ready(Ok(request_info))
        }
    }
}

// Middleware factory
pub struct RequestInfoMiddleware;

impl<S, B> Transform<S, ServiceRequest> for RequestInfoMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestInfoMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestInfoMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct RequestInfoMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for RequestInfoMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = Rc::clone(&self.service);

        Box::pin(async move {
            // Create request info directly from ServiceRequest
            let http_req = req.request();
            let request_info = RequestInfo {
                user_agent: http_req
                    .headers()
                    .get("user-agent")
                    .and_then(|h| h.to_str().ok())
                    .unwrap_or("unknown")
                    .to_string(),
                ip_address: http_req
                    .connection_info()
                    .realip_remote_addr()
                    .unwrap_or("unknown")
                    .to_string(),
                method: http_req.method().to_string(),
                path: http_req.path().to_string(),
            };

            req.extensions_mut().insert(request_info);

            // Call the next service
            service.call(req).await
        })
    }
}
