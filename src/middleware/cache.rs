//! Smart Cache Middleware with Tags-Based Invalidation
//!
//! This module provides an intelligent caching layer that:
//! - Automatically tags cache entries based on URL patterns and JWT context
//! - Enables precise cache invalidation using resource-specific tags
//! - Extracts user/company context from JWT tokens for better tagging
//! - Supports cross-resource relationship invalidation

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
use regex::Regex;
use serde::Deserialize;
use std::{
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    task::{Context, Poll},
    time::Duration,
};

//============================================================================
// Data Structures
//============================================================================

/// JWT Claims structure for extracting user context from auth tokens
#[derive(Debug, Deserialize)]
struct JwtClaims {
    pub sub: uuid::Uuid,        // user_id
    pub company_id: uuid::Uuid, // company_id
    pub role: String,           // user role
    #[allow(dead_code)]
    pub exp: usize, // expiration (unused but part of JWT standard)
}

/// Extracted and validated user context from JWT token
#[derive(Debug, Clone)]
struct UserContext {
    pub user_id: uuid::Uuid,
    pub company_id: uuid::Uuid,
    #[allow(dead_code)]
    pub role: String, // Available for future use in role-based tagging
}

/// Resource pattern for intelligent URL parsing and tag inference
#[derive(Clone)]
struct ResourcePattern {
    pub name: &'static str,
    pub path_regex: Regex,
    pub id_capture_group: Option<usize>,
    pub query_params: Vec<&'static str>,
}

/// Cache entry with automatic tag tracking
#[derive(Clone)]
pub struct CachedHttp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
    pub tags: Vec<String>,
}

/// Context for precise cache invalidation
#[derive(Default)]
pub struct InvalidationContext {
    pub company_id: Option<uuid::Uuid>,
    pub user_id: Option<uuid::Uuid>,
    pub resource_id: Option<uuid::Uuid>,
}

//============================================================================
// Tag Inference Engine
//============================================================================

/// Tag inference engine for intelligent cache tagging
struct TagInferenceEngine {
    resource_patterns: Vec<ResourcePattern>,
}

//============================================================================
// Cache Layer
//============================================================================

/// Main cache layer with intelligent tagging support
#[derive(Clone)]
pub struct CacheLayer {
    pub cache: Arc<Cache<String, CachedHttp>>,
    generation: Arc<AtomicU64>,
    tag_engine: Arc<TagInferenceEngine>,
}

impl TagInferenceEngine {
    /// Create a new tag inference engine with predefined resource patterns
    fn new() -> Self {
        let resource_patterns = vec![
            ResourcePattern {
                name: "shifts",
                path_regex: Regex::new(r"/api/v1/shifts(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id", "team_id", "location_id"],
            },
            ResourcePattern {
                name: "users",
                path_regex: Regex::new(r"/api/v1/users(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "role", "team_id"],
            },
            ResourcePattern {
                name: "time-off",
                path_regex: Regex::new(r"/api/v1/time-off(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id", "status"],
            },
            ResourcePattern {
                name: "swaps",
                path_regex: Regex::new(r"/api/v1/swaps(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["shift_id", "user_id", "status"],
            },
            ResourcePattern {
                name: "stats",
                path_regex: Regex::new(r"/api/v1/stats").unwrap(),
                id_capture_group: None,
                query_params: vec!["company_id", "period", "type"],
            },
            ResourcePattern {
                name: "companies",
                path_regex: Regex::new(r"/api/v1/companies(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec![],
            },
            // Auth routes
            ResourcePattern {
                name: "auth",
                path_regex: Regex::new(r"/api/v1/auth/(register|login|forgot-password|reset-password|me|invite|invites|switch-company)").unwrap(),
                id_capture_group: None,
                query_params: vec![],
            },
            ResourcePattern {
                name: "auth_invite",
                path_regex: Regex::new(r"/api/v1/auth/invite/([^/]+)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec![],
            },
            ResourcePattern {
                name: "auth_invite_action",
                path_regex: Regex::new(r"/api/v1/auth/invite/([^/]+)/(accept|reject)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec![],
            },
            ResourcePattern {
                name: "auth_switch_company",
                path_regex: Regex::new(r"/api/v1/auth/switch-company/([a-f0-9-]+)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec![],
            },
            // Admin routes
            ResourcePattern {
                name: "admin_locations",
                path_regex: Regex::new(r"/api/v1/admin/locations(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "admin_teams",
                path_regex: Regex::new(r"/api/v1/admin/teams(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "admin_team_members",
                path_regex: Regex::new(r"/api/v1/admin/teams/(\d+)/members(?:/(\d+))?").unwrap(),
                id_capture_group: Some(2),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "admin_users",
                path_regex: Regex::new(r"/api/v1/admin/users(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            // Schedules routes
            ResourcePattern {
                name: "schedules",
                path_regex: Regex::new(r"/api/v1/schedules(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id"],
            },
            ResourcePattern {
                name: "schedule_suggestions",
                path_regex: Regex::new(r"/api/v1/schedules/(\d+)/suggestions").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            // Shift assignments routes
            ResourcePattern {
                name: "shift_assignments",
                path_regex: Regex::new(r"/api/v1/assignments(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id", "shift_id"],
            },
            ResourcePattern {
                name: "shift_assignments_by_shift",
                path_regex: Regex::new(r"/api/v1/assignments/shift/(\d+)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "shift_assignments_by_user",
                path_regex: Regex::new(r"/api/v1/assignments/user/(\d+)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "pending_assignments",
                path_regex: Regex::new(r"/api/v1/assignments/user/(\d+)/pending").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "assignment_action",
                path_regex: Regex::new(r"/api/v1/assignments/(\d+)/(respond|cancel)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            // PTO Balance routes
            ResourcePattern {
                name: "pto_balance",
                path_regex: Regex::new(r"/api/v1/pto-balance(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id"],
            },
            ResourcePattern {
                name: "pto_balance_history",
                path_regex: Regex::new(r"/api/v1/pto-balance/(\d+)/history").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "pto_balance_action",
                path_regex: Regex::new(r"/api/v1/pto-balance/(\d+)/(adjust|accrual)").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            // Skills routes
            ResourcePattern {
                name: "skills",
                path_regex: Regex::new(r"/api/v1/skills(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            ResourcePattern {
                name: "skill_users",
                path_regex: Regex::new(r"/api/v1/skills/(\d+)/users").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
            // User skills routes
            ResourcePattern {
                name: "user_skills",
                path_regex: Regex::new(r"/api/v1/user-skills(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "user_id"],
            },
            ResourcePattern {
                name: "user_skill",
                path_regex: Regex::new(r"/api/v1/user-skills/(\d+)/(\d+)").unwrap(),
                id_capture_group: Some(2),
                query_params: vec!["company_id"],
            },
            // Shift skills routes
            ResourcePattern {
                name: "shift_skills",
                path_regex: Regex::new(r"/api/v1/shift-skills(?:/(\d+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id", "shift_id"],
            },
            ResourcePattern {
                name: "shift_skill",
                path_regex: Regex::new(r"/api/v1/shift-skills/(\d+)/(\d+)").unwrap(),
                id_capture_group: Some(2),
                query_params: vec!["company_id"],
            },
            // Subscription routes
            ResourcePattern {
                name: "subscription",
                path_regex: Regex::new(r"/api/v1/subscription(?:/([a-f0-9-]+))?").unwrap(),
                id_capture_group: Some(1),
                query_params: vec!["company_id"],
            },
        ];

        Self { resource_patterns }
    }

    //------------------------------------------------------------------------
    // Main Tag Inference
    //------------------------------------------------------------------------

    /// Intelligently infer tags from URI and auth context
    fn infer_tags(&self, uri: &str, auth_context: Option<&str>) -> Vec<String> {
        let mut tags = Vec::new();

        // 1. Extract user context from JWT
        let user_ctx = self.extract_user_context(auth_context);

        // 2. Always add user/company context if available
        if let Some(ctx) = &user_ctx {
            tags.push(format!("user:{}", ctx.user_id));
            tags.push(format!("company:{}", ctx.company_id));
        }

        // 3. Match against resource patterns
        for pattern in &self.resource_patterns {
            if let Some(captures) = pattern.path_regex.captures(uri) {
                // Base resource tag
                tags.push(pattern.name.to_string());

                // Company-scoped resource
                if let Some(ctx) = &user_ctx {
                    tags.push(format!("{}:company:{}", pattern.name, ctx.company_id));
                }

                // Specific resource ID
                if let Some(id_group) = pattern.id_capture_group {
                    if let Some(resource_id) = captures.get(id_group) {
                        tags.push(format!("{}:{}", pattern.name, resource_id.as_str()));

                        // Also add user-specific resource tag if available
                        if let Some(ctx) = &user_ctx {
                            tags.push(format!(
                                "{}:{}:user:{}",
                                pattern.name,
                                resource_id.as_str(),
                                ctx.user_id
                            ));
                        }
                    }
                }

                // Query parameter context
                self.add_query_param_tags(&mut tags, uri, pattern, &user_ctx);
                break; // Only match the first pattern to avoid duplicates
            }
        }

        // 4. Add cross-resource relationship tags
        self.add_relationship_tags(&mut tags, uri, &user_ctx);

        tags
    }

    //------------------------------------------------------------------------
    // JWT Processing
    //------------------------------------------------------------------------

    /// Extract user context from JWT token
    fn extract_user_context(&self, auth_header: Option<&str>) -> Option<UserContext> {
        let token = auth_header
            .and_then(|h| h.strip_prefix("Bearer "))
            .or(auth_header)?;

        // For now, we'll do a simple JWT parsing without verification
        // In production, you should use proper JWT validation
        if let Ok(decoded) = self.decode_jwt_claims(token) {
            Some(UserContext {
                user_id: decoded.sub,
                company_id: decoded.company_id,
                role: decoded.role,
            })
        } else {
            None
        }
    }

    /// Simple JWT claim extraction (without verification for now)
    fn decode_jwt_claims(&self, token: &str) -> Result<JwtClaims, Box<dyn std::error::Error>> {
        // Split JWT into parts
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid JWT format".into());
        }

        // Decode the payload (second part)
        let payload = parts[1];
        // Add padding if needed for base64 decoding
        let padded = match payload.len() % 4 {
            2 => format!("{}==", payload),
            3 => format!("{}=", payload),
            _ => payload.to_string(),
        };

        use base64::{Engine, engine::general_purpose};
        let decoded = general_purpose::URL_SAFE_NO_PAD.decode(&padded)?;
        let claims: JwtClaims = serde_json::from_slice(&decoded)?;
        Ok(claims)
    }

    //------------------------------------------------------------------------
    // Helper Methods
    //------------------------------------------------------------------------

    /// Add tags based on query parameters
    fn add_query_param_tags(
        &self,
        tags: &mut Vec<String>,
        uri: &str,
        pattern: &ResourcePattern,
        _user_ctx: &Option<UserContext>, // Reserved for future role-based tagging
    ) {
        for param in &pattern.query_params {
            if let Some(value) = self.extract_query_param(uri, param) {
                match *param {
                    "user_id" => tags.push(format!("user:{}", value)),
                    "company_id" => tags.push(format!("company:{}", value)),
                    "team_id" => tags.push(format!("team:{}", value)),
                    "location_id" => tags.push(format!("location:{}", value)),
                    "shift_id" => tags.push(format!("shifts:{}", value)),
                    "role" => tags.push(format!("role:{}", value)),
                    "status" => tags.push(format!("status:{}", value)),
                    "period" => tags.push(format!("period:{}", value)),
                    "type" => tags.push(format!("type:{}", value)),
                    _ => tags.push(format!("{}:{}", param, value)),
                }
            }
        }
    }

    /// Add cross-resource relationship tags
    fn add_relationship_tags(
        &self,
        tags: &mut Vec<String>,
        uri: &str,
        user_ctx: &Option<UserContext>,
    ) {
        // Stats depend on multiple resource types
        if uri.contains("/stats") {
            tags.extend([
                "shifts".to_string(),
                "users".to_string(),
                "time-off".to_string(),
                "swaps".to_string(),
                "shift_assignments".to_string(),
                "pto_balance".to_string(),
            ]);
        }

        // User changes affect their shifts, assignments, PTO, skills, and potentially company stats
        if uri.contains("/users/") || uri.contains("/admin/users/") {
            tags.extend([
                "shifts".to_string(),
                "shift_assignments".to_string(),
                "pto_balance".to_string(),
                "user_skills".to_string(),
                "schedules".to_string(),
            ]);
            if user_ctx.is_some() {
                tags.push("stats".to_string());
            }
        }

        // Time-off affects shift availability, assignments, and stats
        if uri.contains("/time-off") || uri.contains("/pto-balance") {
            tags.extend([
                "shifts".to_string(),
                "shift_assignments".to_string(),
                "schedules".to_string(),
                "stats".to_string(),
            ]);
        }

        // Swaps affect specific shifts, assignments, and stats
        if uri.contains("/swaps") {
            if let Some(shift_id) = self.extract_query_param(uri, "shift_id") {
                tags.push(format!("shifts:{}", shift_id));
                tags.push(format!("shift_assignments_by_shift:{}", shift_id));
            }
            tags.extend(["shift_assignments".to_string(), "stats".to_string()]);
        }

        // Shift changes affect assignments, schedules, skills, and stats
        if uri.contains("/shifts") {
            tags.extend([
                "shift_assignments".to_string(),
                "schedules".to_string(),
                "shift_skills".to_string(),
                "stats".to_string(),
            ]);
        }

        // Company changes affect all company-scoped resources
        if uri.contains("/companies") || uri.contains("/admin/") {
            tags.extend([
                "users".to_string(),
                "shifts".to_string(),
                "time-off".to_string(),
                "swaps".to_string(),
                "shift_assignments".to_string(),
                "pto_balance".to_string(),
                "skills".to_string(),
                "schedules".to_string(),
                "stats".to_string(),
            ]);
        }

        // Auth changes affect user context and related resources
        if uri.contains("/auth/") {
            tags.extend(["users".to_string(), "companies".to_string()]);
        }

        // Skills changes affect shifts and users
        if uri.contains("/skills") {
            tags.extend([
                "shifts".to_string(),
                "users".to_string(),
                "shift_assignments".to_string(),
            ]);
        }

        // Schedule changes affect shifts and assignments
        if uri.contains("/schedules") || uri.contains("/assignments") {
            tags.extend([
                "shifts".to_string(),
                "users".to_string(),
                "stats".to_string(),
            ]);
        }

        // Subscription changes affect company resources
        if uri.contains("/subscription") {
            tags.extend([
                "companies".to_string(),
                "users".to_string(),
                "shifts".to_string(),
                "stats".to_string(),
            ]);
        }
    }

    /// Extract query parameter value
    fn extract_query_param(&self, uri: &str, param: &str) -> Option<String> {
        let pattern = format!("{}=", param);
        if let Some(start) = uri.find(&pattern) {
            let after_param = &uri[start + pattern.len()..];
            if let Some(end) = after_param.find('&') {
                Some(after_param[..end].to_string())
            } else {
                Some(after_param.to_string())
            }
        } else {
            None
        }
    }
}

//============================================================================
// CacheLayer Implementation
//============================================================================

impl CacheLayer {
    /// Create a new cache layer with intelligent tagging
    pub fn new(max_capacity: u64, ttl_seconds: u64) -> Self {
        let cache = Cache::builder()
            .max_capacity(max_capacity)
            .time_to_live(Duration::from_secs(ttl_seconds))
            .build();
        Self {
            cache: Arc::new(cache),
            generation: Arc::new(AtomicU64::new(1)),
            tag_engine: Arc::new(TagInferenceEngine::new()),
        }
    }

    //------------------------------------------------------------------------
    // Cache Invalidation
    //------------------------------------------------------------------------

    /// Invalidate all cache entries (existing behavior - use sparingly)
    pub fn bump(&self) {
        self.generation.fetch_add(1, Ordering::SeqCst);
    }

    /// Main invalidation method - invalidate by resource type and context
    pub async fn invalidate(&self, resource: &str, context: &InvalidationContext) {
        let mut tags_to_invalidate = vec![resource.to_string()];

        // Add company-specific tag if provided
        if let Some(company_id) = context.company_id {
            tags_to_invalidate.push(format!("company:{}", company_id));
            // Also invalidate company-scoped resource listings
            tags_to_invalidate.push(format!("{}:company:{}", resource, company_id));
        }

        // Add user-specific tag if provided
        if let Some(user_id) = context.user_id {
            tags_to_invalidate.push(format!("user:{}", user_id));
            tags_to_invalidate.push(format!("{}:user:{}", resource, user_id));
        }

        // Add resource-specific tag if provided
        if let Some(resource_id) = context.resource_id {
            tags_to_invalidate.push(format!("{}:{}", resource, resource_id));
        }

        self.invalidate_by_tags(&tags_to_invalidate).await;
    }

    /// Invalidate cache entries by tags
    pub async fn invalidate_by_tags(&self, tags: &[String]) {
        // Get all cache entries and check their tags
        // Unfortunately, moka doesn't expose iterating over entries,
        // so we'll use a generation-based approach but more selectively

        // For now, we'll track entries by tags using the cache itself
        for tag in tags {
            let tag_key = format!("__tag__:{}", tag);
            if let Some(cached_keys) = self.cache.get(&tag_key).await {
                // Deserialize the list of keys for this tag
                if let Ok(keys_str) = String::from_utf8(cached_keys.body.clone()) {
                    let keys: Vec<String> = keys_str
                        .split('\n')
                        .filter(|k| !k.is_empty())
                        .map(|k| k.to_string())
                        .collect();

                    // Invalidate all keys with this tag
                    for key in keys {
                        self.cache.invalidate(&key).await;
                    }
                }

                // Remove the tag tracking entry
                self.cache.invalidate(&tag_key).await;
            }
        }
    }

    //------------------------------------------------------------------------
    // Internal Cache Management
    //------------------------------------------------------------------------

    /// Store cache entry with automatic tag inference
    async fn store_with_tags(&self, key: String, cached: CachedHttp) {
        // Store the main cache entry
        self.cache.insert(key.clone(), cached.clone()).await;

        // Track this key under each tag
        for tag in &cached.tags {
            let tag_key = format!("__tag__:{}", tag);

            // Get existing keys for this tag
            let mut keys = if let Some(existing) = self.cache.get(&tag_key).await {
                String::from_utf8(existing.body)
                    .unwrap_or_default()
                    .split('\n')
                    .filter(|k| !k.is_empty())
                    .map(|k| k.to_string())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

            // Add this key if not already present
            if !keys.contains(&key) {
                keys.push(key.clone());
            }

            // Store updated key list for this tag
            let keys_str = keys.join("\n");
            let tag_entry = CachedHttp {
                status: 200,
                headers: vec![],
                body: keys_str.into_bytes(),
                tags: vec![], // Tag tracking entries don't have tags themselves
            };
            self.cache.insert(tag_key, tag_entry).await;
        }
    }

    //------------------------------------------------------------------------
    // Utility Methods
    //------------------------------------------------------------------------

    /// Infer tags from URL and auth context (using the smart tag engine)
    fn infer_tags(&self, uri: &str, auth_context: Option<&str>) -> Vec<String> {
        self.tag_engine.infer_tags(uri, auth_context)
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

//============================================================================
// Actix-Web Middleware Implementation
//============================================================================

/// Response cache middleware factory
pub struct ResponseCacheMiddleware {
    cache_layer: CacheLayer,
}

impl ResponseCacheMiddleware {
    /// Create a new cache middleware with the given cache layer
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

/// Actix-web middleware service for handling cache operations
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
        let cache_layer = self.cache_layer.clone();
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

                // Infer tags for this cache entry
                let tags = cache_layer.infer_tags(&uri, auth_header.as_deref());

                let cached = CachedHttp {
                    status: status.as_u16(),
                    headers: hdrs_vec,
                    body: body_vec,
                    tags,
                };

                // Store with tags
                cache_layer.store_with_tags(key, cached).await;
            }

            Ok(ServiceResponse::new(req, out_res))
        })
    }
}
