//! Health check endpoints using connectivity infrastructure

use axum::{extract::State, http::StatusCode, Json};
use confuse_connectivity::{Check, HealthChecker};
use confuse_connectivity::registry::health::{HealthCheckResult, ComponentHealth, HealthStatus};
use std::sync::Arc;
use crate::routes::v1::AppState;

/// Health check handler
pub async fn health_check(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<HealthCheckResult>) {
    let checker = create_health_checker(&state).await;
    let result = checker.check_health().await;
    
    let status_code = match result.status {
        HealthStatus::Healthy => StatusCode::OK,
        HealthStatus::Degraded => StatusCode::OK, // Still accepting traffic
        HealthStatus::Unhealthy => StatusCode::SERVICE_UNAVAILABLE,
    };
    
    (status_code, Json(result))
}

/// Readiness probe - checks if service is ready to accept traffic
pub async fn readiness(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<HealthCheckResult>) {
    health_check(State(state)).await
}

/// Liveness probe - checks if service is alive
pub async fn liveness() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "alive",
            "timestamp": chrono::Utc::now().timestamp()
        })),
    )
}

async fn create_health_checker(state: &AppState) -> HealthChecker {
    let version = env!("CARGO_PKG_VERSION");
    
    HealthChecker::new(version)
        // Add database check
        .add_check(confuse_connectivity::health::checks::DatabaseCheck::new(
            "postgres",
            || {
                // Placeholder - would check actual DB connection
                Ok(())
            },
        ))
        // Add downstream service checks
        .add_check(confuse_connectivity::health::checks::DependencyCheck::new(
            "auth-middleware",
            format!("{}/health", state.config.auth_middleware_url),
            5000,
        ))
        .add_check(confuse_connectivity::health::checks::DependencyCheck::new(
            "data-connector",
            format!("{}/health", state.config.data_connector_url),
            5000,
        ))
        .add_check(confuse_connectivity::health::checks::DependencyCheck::new(
            "relation-graph",
            format!("{}/health", state.config.relation_graph_url),
            5000,
        ))
}
