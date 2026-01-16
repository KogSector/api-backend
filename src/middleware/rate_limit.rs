//! Rate limiting middleware
//!
//! Redis-backed sliding window rate limiting

use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
};
use redis::AsyncCommands;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Rate limit configuration
#[derive(Clone)]
pub struct RateLimitConfig {
    pub redis_client: Arc<redis::Client>,
    pub default_limit: u32,
    pub search_limit: u32,
    pub sources_limit: u32,
    pub sync_limit: u32,
    pub window_secs: u64,
    pub skip_rate_limiting: bool,
}

impl RateLimitConfig {
    /// Get limit for an endpoint path
    pub fn get_limit_for_path(&self, path: &str) -> u32 {
        if path.contains("/search") {
            self.search_limit
        } else if path.contains("/sources") {
            self.sources_limit
        } else if path.contains("/sync") {
            self.sync_limit
        } else {
            self.default_limit
        }
    }
}

/// Rate limit info for response headers
pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset: u64,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(config): State<RateLimitConfig>,
    request: Request,
    next: Next,
) -> Response {
    // Skip rate limiting if toggle is enabled
    if config.skip_rate_limiting {
        return next.run(request).await;
    }
    
    // Get client identifier (IP or user ID)
    let client_id = get_client_id(&request);
    let path = request.uri().path().to_string();
    let limit = config.get_limit_for_path(&path);
    
    // Check rate limit
    match check_rate_limit(&config, &client_id, &path, limit).await {
        Ok(info) => {
            let mut response = next.run(request).await;
            
            // Add rate limit headers
            if let Ok(val) = HeaderValue::from_str(&info.limit.to_string()) {
                response.headers_mut().insert("X-RateLimit-Limit", val);
            }
            if let Ok(val) = HeaderValue::from_str(&info.remaining.to_string()) {
                response.headers_mut().insert("X-RateLimit-Remaining", val);
            }
            if let Ok(val) = HeaderValue::from_str(&info.reset.to_string()) {
                response.headers_mut().insert("X-RateLimit-Reset", val);
            }
            
            response
        }
        Err(_) => {
            // Rate limit exceeded
            let mut response = (
                StatusCode::TOO_MANY_REQUESTS,
                serde_json::json!({
                    "error": {
                        "code": "RATE_LIMITED",
                        "message": "Too many requests"
                    }
                }).to_string(),
            ).into_response();
            
            if let Ok(val) = HeaderValue::from_str(&limit.to_string()) {
                response.headers_mut().insert("X-RateLimit-Limit", val);
            }
            if let Ok(val) = HeaderValue::from_str("0") {
                response.headers_mut().insert("X-RateLimit-Remaining", val);
            }
            
            response
        }
    }
}

/// Get client identifier for rate limiting
fn get_client_id(request: &Request) -> String {
    // Try user ID from auth middleware
    if let Some(user) = request.extensions().get::<super::auth::AuthenticatedUser>() {
        return format!("user:{}", user.0.id);
    }
    
    // Fall back to IP address
    request
        .headers()
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.trim().to_string())
        .or_else(|| {
            request
                .headers()
                .get("X-Real-IP")
                .and_then(|v| v.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_else(|| "unknown".to_string())
}

/// Check rate limit using Redis sliding window
async fn check_rate_limit(
    config: &RateLimitConfig,
    client_id: &str,
    path: &str,
    limit: u32,
) -> Result<RateLimitInfo, ()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let window_start = now - config.window_secs;
    let key = format!("ratelimit:{}:{}", client_id, path.replace('/', "_"));
    
    let mut conn = match config.redis_client.get_multiplexed_async_connection().await {
        Ok(c) => c,
        Err(_) => {
            // If Redis is unavailable, allow the request
            return Ok(RateLimitInfo {
                limit,
                remaining: limit,
                reset: now + config.window_secs,
            });
        }
    };
    
    // Remove old entries and add new one
    let _: Result<(), _> = redis::pipe()
        .atomic()
        .zrembyscore(&key, 0i64, window_start as i64)
        .zadd(&key, now.to_string(), now as f64)
        .expire(&key, config.window_secs as i64)
        .query_async(&mut conn)
        .await;
    
    // Count requests in window
    let count: u32 = conn
        .zcount(&key, window_start as f64, now as f64)
        .await
        .unwrap_or(0);
    
    if count > limit {
        Err(())
    } else {
        Ok(RateLimitInfo {
            limit,
            remaining: limit.saturating_sub(count),
            reset: now + config.window_secs,
        })
    }
}
