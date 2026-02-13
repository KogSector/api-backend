//! ConFuse API Backend
//!
//! Central API Gateway for the ConFuse Knowledge Intelligence Platform
//! Event-Driven Architecture with Kafka

pub mod config;
pub mod error;
pub mod health;
pub mod middleware;
pub mod routes;
pub mod clients;
pub mod models;
pub mod kafka;

pub use config::Config;
pub use error::{AppError, Result};
pub use kafka::{EventProducer, SourceSyncRequestedEvent};
pub use middleware::{CircuitBreakerRegistry, CircuitBreakerConfig, CircuitState, ResponseCache, CacheConfig, ZeroTrustLayer};
