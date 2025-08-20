use actix_web::{
    Error, HttpMessage, HttpResponse, Result,
    body::EitherBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
};
use chrono::{DateTime, Duration, Utc};
use futures_util::future::LocalBoxFuture;
use std::{
    collections::HashMap,
    net::IpAddr,
    rc::Rc,
    sync::{Arc, Mutex},
};

use crate::{handlers::shared::ApiResponse, user_context::UserContext};

/// Rate limit configuration
#[derive(Clone, Debug)]
pub struct RateLimitConfig {
    /// Maximum requests per window
    pub max_requests: u32,
    /// Window duration in seconds
    pub window_seconds: i64,
    /// Message to return when rate limit is exceeded
    pub message: String,
}

impl RateLimitConfig {
    pub fn new(max_requests: u32, window_seconds: i64) -> Self {
        Self {
            max_requests,
            window_seconds,
            message: "Rate limit exceeded. Please try again later.".to_string(),
        }
    }

    pub fn with_message(mut self, message: String) -> Self {
        self.message = message;
        self
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self::new(100, 60) // 100 requests per minute by default
    }
}

/// Track request counts for rate limiting
#[derive(Debug, Clone)]
struct RequestTracker {
    count: u32,
    window_start: DateTime<Utc>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            count: 0,
            window_start: Utc::now(),
        }
    }

    fn is_expired(&self, window_seconds: i64) -> bool {
        let window_duration =
            Duration::try_seconds(window_seconds).unwrap_or(Duration::seconds(60));
        Utc::now() > self.window_start + window_duration
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 1;
        self.window_start = Utc::now();
    }
}

/// Rate limiting store
#[derive(Clone)]
pub struct RateLimitStore {
    ip_trackers: Arc<Mutex<HashMap<IpAddr, RequestTracker>>>,
    user_trackers: Arc<Mutex<HashMap<String, RequestTracker>>>,
}

impl RateLimitStore {
    pub fn new() -> Self {
        Self {
            ip_trackers: Arc::new(Mutex::new(HashMap::new())),
            user_trackers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn check_and_update_ip(&self, ip: IpAddr, config: &RateLimitConfig) -> bool {
        let mut trackers = self.ip_trackers.lock().unwrap();

        let tracker = trackers.entry(ip).or_insert_with(RequestTracker::new);

        if tracker.is_expired(config.window_seconds) {
            tracker.reset();
            true
        } else if tracker.count >= config.max_requests {
            false
        } else {
            tracker.increment();
            true
        }
    }

    fn check_and_update_user(&self, user_id: &str, config: &RateLimitConfig) -> bool {
        let mut trackers = self.user_trackers.lock().unwrap();

        let tracker = trackers
            .entry(user_id.to_string())
            .or_insert_with(RequestTracker::new);

        if tracker.is_expired(config.window_seconds) {
            tracker.reset();
            true
        } else if tracker.count >= config.max_requests {
            false
        } else {
            tracker.increment();
            true
        }
    }

    /// Clean up expired entries to prevent memory leaks
    pub fn cleanup_expired(&self, window_seconds: i64) {
        let mut ip_trackers = self.ip_trackers.lock().unwrap();
        let mut user_trackers = self.user_trackers.lock().unwrap();

        ip_trackers.retain(|_, tracker| !tracker.is_expired(window_seconds));
        user_trackers.retain(|_, tracker| !tracker.is_expired(window_seconds));
    }
}

impl Default for RateLimitStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Rate limiting middleware
pub struct RateLimitMiddleware {
    store: RateLimitStore,
    config: RateLimitConfig,
    apply_to_authenticated: bool,
}

impl RateLimitMiddleware {
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            store: RateLimitStore::new(),
            config,
            apply_to_authenticated: false,
        }
    }

    pub fn with_store(config: RateLimitConfig, store: RateLimitStore) -> Self {
        Self {
            store,
            config,
            apply_to_authenticated: false,
        }
    }

    /// Enable rate limiting for authenticated users as well
    pub fn with_authenticated_users(mut self) -> Self {
        self.apply_to_authenticated = true;
        self
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Transform = RateLimitService<S>;
    type InitError = ();
    type Future = futures_util::future::Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        futures_util::future::ready(Ok(RateLimitService {
            service: Rc::new(service),
            store: self.store.clone(),
            config: self.config.clone(),
            apply_to_authenticated: self.apply_to_authenticated,
        }))
    }
}

pub struct RateLimitService<S> {
    service: Rc<S>,
    store: RateLimitStore,
    config: RateLimitConfig,
    apply_to_authenticated: bool,
}

impl<S, B> Service<ServiceRequest> for RateLimitService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let store = self.store.clone();
        let config = self.config.clone();
        let apply_to_authenticated = self.apply_to_authenticated;

        Box::pin(async move {
            // Extract information we need from the request before using it
            let client_ip = req
                .connection_info()
                .peer_addr()
                .and_then(|addr| addr.split(':').next())
                .and_then(|ip| ip.parse::<IpAddr>().ok());

            let user_id = if apply_to_authenticated {
                req.extensions()
                    .get::<UserContext>()
                    .map(|ctx| ctx.user_id().to_string())
            } else {
                None
            };

            // Check IP-based rate limit first
            if let Some(ip) = client_ip {
                if !store.check_and_update_ip(ip, &config) {
                    log::warn!("Rate limit exceeded for IP: {}", ip);
                    let response = HttpResponse::TooManyRequests()
                        .json(ApiResponse::<()>::error(&config.message));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            }

            // Check user-specific rate limit if enabled
            if let Some(user_id) = user_id {
                if !store.check_and_update_user(&user_id, &config) {
                    log::warn!("Rate limit exceeded for user: {}", user_id);
                    let response = HttpResponse::TooManyRequests()
                        .json(ApiResponse::<()>::error(&config.message));
                    return Ok(req.into_response(response).map_into_right_body());
                }
            }

            // Continue to the service
            let res = service.call(req).await?;
            Ok(res.map_into_left_body())
        })
    }
}

/// Specialized rate limiter for authentication endpoints
pub struct AuthRateLimiter;

impl AuthRateLimiter {
    /// Create rate limiter for login attempts (stricter limits)
    pub fn login() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(5, 300) // 5 attempts per 5 minutes
                .with_message(
                    "Too many login attempts. Please try again in 5 minutes.".to_string(),
                ),
        )
    }

    /// Create rate limiter for registration (moderate limits)
    pub fn registration() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(3, 3600) // 3 registrations per hour
                .with_message(
                    "Too many registration attempts. Please try again later.".to_string(),
                ),
        )
    }

    /// Create rate limiter for password reset (moderate limits)
    pub fn password_reset() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(3, 900) // 3 attempts per 15 minutes
                .with_message(
                    "Too many password reset attempts. Please try again in 15 minutes.".to_string(),
                ),
        )
    }
}

/// Global rate limiter configurations
pub struct GlobalRateLimiter;

impl GlobalRateLimiter {
    /// General API rate limiter
    pub fn general() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(100, 60), // 100 requests per minute
        )
        .with_authenticated_users()
    }

    /// Stricter rate limiter for sensitive operations
    pub fn sensitive() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(20, 60) // 20 requests per minute
                .with_message(
                    "Rate limit exceeded for sensitive operation. Please try again later."
                        .to_string(),
                ),
        )
        .with_authenticated_users()
    }

    /// Rate limiter for administrative operations
    pub fn admin() -> RateLimitMiddleware {
        RateLimitMiddleware::new(
            RateLimitConfig::new(50, 60) // 50 requests per minute for admins
                .with_message(
                    "Administrative rate limit exceeded. Please try again later.".to_string(),
                ),
        )
        .with_authenticated_users()
    }
}

/// Background task to clean up expired rate limit entries
pub async fn cleanup_rate_limits(store: RateLimitStore, interval_seconds: u64) {
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

    loop {
        interval.tick().await;
        store.cleanup_expired(3600); // Clean up entries older than 1 hour
        log::debug!("Cleaned up expired rate limit entries");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    #[test]
    fn test_rate_limit_config() {
        let config = RateLimitConfig::new(10, 60);
        assert_eq!(config.max_requests, 10);
        assert_eq!(config.window_seconds, 60);

        let config_with_message = config.with_message("Custom message".to_string());
        assert_eq!(config_with_message.message, "Custom message");
    }

    #[test]
    fn test_request_tracker() {
        let mut tracker = RequestTracker::new();
        assert_eq!(tracker.count, 0);

        tracker.increment();
        assert_eq!(tracker.count, 1);

        tracker.reset();
        assert_eq!(tracker.count, 1);
        assert!(tracker.window_start <= Utc::now());
    }

    #[test]
    fn test_rate_limit_store() {
        let store = RateLimitStore::new();
        let config = RateLimitConfig::new(2, 60);
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // First request should pass
        assert!(store.check_and_update_ip(ip, &config));

        // Second request should pass
        assert!(store.check_and_update_ip(ip, &config));

        // Third request should be blocked
        assert!(!store.check_and_update_ip(ip, &config));
    }

    #[test]
    fn test_cleanup_expired() {
        let store = RateLimitStore::new();
        let config = RateLimitConfig::new(1, 1); // 1 second window
        let ip = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

        // Make a request
        assert!(store.check_and_update_ip(ip, &config));

        // Should have an entry
        {
            let trackers = store.ip_trackers.lock().unwrap();
            assert_eq!(trackers.len(), 1);
        }

        // Clean up expired entries (using a very long window to simulate expiry)
        store.cleanup_expired(0);

        // Should be cleaned up
        {
            let trackers = store.ip_trackers.lock().unwrap();
            assert_eq!(trackers.len(), 0);
        }
    }
}
