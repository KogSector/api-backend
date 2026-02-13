//! Middleware for request processing

pub mod auth;
pub mod rate_limit;
pub mod circuit_breaker;
pub mod cache;
pub mod security_headers;
pub mod zero_trust;

pub use auth::AuthLayer;
pub use circuit_breaker::{CircuitBreakerRegistry, CircuitBreakerConfig, CircuitState};
pub use cache::{ResponseCache, CacheConfig};
pub use zero_trust::ZeroTrustLayer;
