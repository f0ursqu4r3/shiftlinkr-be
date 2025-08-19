use actix_web::body::to_bytes;
use actix_web::{
    Error, HttpResponse,
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::{
        Method, StatusCode,
        header::{self, HeaderName, HeaderValue},
    },
    web::Bytes,
};
use futures::future::{LocalBoxFuture, Ready, ok};
use moka::future::Cache;
use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

#[derive(Clone)]
pub struct CacheLayer {
    pub cache: Arc<Cache<String, CachedHttp>>,
    generation: Arc<AtomicU64>,
}

#[derive(Clone)]
pub struct CachedHttp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl CacheLayer {
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();
        Self {
            cache: Arc::new(cache),
            generation: Arc::new(AtomicU64::new(1)),
        }
    }

    pub fn bump(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    fn current_gen(&self) -> u64 {
        self.generation.load(Ordering::SeqCst)
    }

    fn make_key(&self, method: &Method, uri: &str, auth: Option<&str>) -> String {
        let curr_gen = self.current_gen();
        let auth_part = auth.unwrap_or("");
        format!("v{curr_gen}:{method}:{uri}:auth={auth_part}")
    }
}

pub struct ResponseCacheMiddleware {
    cache_layer: CacheLayer,
}

impl ResponseCacheMiddleware {
    pub fn new(cache_layer: CacheLayer) -> Self {
        Self { cache_layer }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ResponseCacheMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: actix_web::ResponseError,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ResponseCacheMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(ResponseCacheMiddlewareService {
            service: Rc::new(service),
            cache_layer: self.cache_layer.clone(),
        })
    }
}

pub struct ResponseCacheMiddlewareService<S> {
    service: Rc<S>,
    cache_layer: CacheLayer,
}

impl<S, B> Service<ServiceRequest> for ResponseCacheMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    <B as MessageBody>::Error: actix_web::ResponseError,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Only cache GETs
        if req.method() != Method::GET {
            let svc = self.service.clone();
            return Box::pin(async move { Ok(svc.call(req).await?.map_into_boxed_body()) });
        }

        // Build cache key
        let method = req.method().clone();
        let uri = req.uri().to_string();
        let auth_header = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());
        let key = self
            .cache_layer
            .make_key(&method, &uri, auth_header.as_deref());

        let cache = self.cache_layer.cache.clone();
        let svc = self.service.clone();

        Box::pin(async move {
            // Hit?
            if let Some(cached) = cache.get(&key).await {
                let mut builder = HttpResponse::build(
                    StatusCode::from_u16(cached.status).unwrap_or(StatusCode::OK),
                );

                for (k, v) in &cached.headers {
                    if let (Ok(name), Ok(val)) =
                        (HeaderName::try_from(k.as_str()), HeaderValue::from_str(v))
                    {
                        // It’s fine to insert; Content-Length may be recalculated.
                        builder.insert_header((name, val));
                    }
                }

                let res = builder
                    .body(Bytes::from(cached.body.clone()))
                    .map_into_boxed_body();
                return Ok(req.into_response(res));
            }

            // Miss -> call downstream
            let res = svc.call(req).await?;
            let (req, res) = res.into_parts();
            let status = res.status();
            let headers = res.headers().clone();

            // Read body into bytes
            let body_bytes = to_bytes(res.into_body()).await?;
            // Rebuild response to return to client
            let mut builder = HttpResponse::build(status);
            for (k, v) in headers.iter() {
                builder.insert_header((k.clone(), v.clone()));
            }
            let body_vec = body_bytes.to_vec();
            let out_res = builder
                .body(Bytes::from(body_vec.clone()))
                .map_into_boxed_body();

            // Cache only successful responses
            if status.is_success() {
                let hdrs_vec: Vec<(String, String)> = headers
                    .iter()
                    .map(|(k, v)| (k.as_str().to_string(), v.to_str().unwrap_or("").to_string()))
                    .collect();

                let cached = CachedHttp {
                    status: status.as_u16(),
                    headers: hdrs_vec,
                    body: body_vec,
                };
                cache.insert(key, cached).await;
            }

            Ok(ServiceResponse::new(req, out_res))
        })
    }
}
