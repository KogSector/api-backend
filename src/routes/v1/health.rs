//! Health and status endpoints with connectivity infrastructure

use axum::{extract::State, http::StatusCode, Json};
use chrono::Utc;
use confuse_connectivity::{Check, HealthChecker};
use confuse_connectivity::registry::health::{HealthCheckResult, ComponentHealth, HealthStatus};
use std::collections::HashMap;

use crate::error::Result;
use crate::models::{HealthResponse, ServiceHealth};
use super::AppState;

/// GET /health - Basic health check (backward compatible)
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "api-backend".to_string(),
        timestamp: Some(Utc::now().to_rfc3339()),
        services: None,
    })
}

/// GET /health/detailed - Detailed health check using connectivity infrastructure
pub async fn health_check_detailed(
    State(state): State<AppState>,
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

/// GET /health/ready - Readiness probe for Kubernetes
pub async fn readiness(
    State(state): State<AppState>,
) -> (StatusCode, Json<HealthCheckResult>) {
    health_check_detailed(State(state)).await
}

/// GET /health/live - Liveness probe for Kubernetes
pub async fn liveness() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "alive",
            "service": "api-backend",
            "version": env!("CARGO_PKG_VERSION"),
            "timestamp": chrono::Utc::now().timestamp()
        })),
    )
}

/// GET /status - Detailed status with downstream service health (backward compatible)
pub async fn status_check(State(state): State<AppState>) -> Result<Json<HealthResponse>> {
    let mut services = HashMap::new();
    
    // Check auth-middleware
    let auth_start = std::time::Instant::now();
    let auth_healthy = state.auth_client.health_check().await;
    services.insert("auth-middleware".to_string(), ServiceHealth {
        status: if auth_healthy { "healthy" } else { "unhealthy" }.to_string(),
        latency: Some(auth_start.elapsed().as_millis() as u64),
    });
    
    // Check data-connector
    let dc_start = std::time::Instant::now();
    let dc_healthy = state.data_connector_client.health_check().await;
    services.insert("data-connector".to_string(), ServiceHealth {
        status: if dc_healthy { "healthy" } else { "unhealthy" }.to_string(),
        latency: Some(dc_start.elapsed().as_millis() as u64),
    });
    
    // Check relation-graph
    let rg_start = std::time::Instant::now();
    let rg_healthy = state.relation_graph_client.health_check().await;
    services.insert("relation-graph".to_string(), ServiceHealth {
        status: if rg_healthy { "healthy" } else { "unhealthy" }.to_string(),
        latency: Some(rg_start.elapsed().as_millis() as u64),
    });
    
    // Check mcp-server
    let mcp_start = std::time::Instant::now();
    let mcp_healthy = state.mcp_client.health_check().await;
    services.insert("mcp-server".to_string(), ServiceHealth {
        status: if mcp_healthy { "healthy" } else { "unhealthy" }.to_string(),
        latency: Some(mcp_start.elapsed().as_millis() as u64),
    });

    // Check unified-processor
    let up_start = std::time::Instant::now();
    let up_healthy = state.unified_processor_client.health_check().await;
    services.insert("unified-processor".to_string(), ServiceHealth {
        status: if up_healthy { "healthy" } else { "unhealthy" }.to_string(),
        latency: Some(up_start.elapsed().as_millis() as u64),
    });
    
    // Overall status
    let all_healthy = auth_healthy && dc_healthy && rg_healthy && mcp_healthy && up_healthy;
    
    Ok(Json(HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "api-backend".to_string(),
        timestamp: Some(Utc::now().to_rfc3339()),
        services: Some(services),
    }))
}

/// GET /metrics - Prometheus metrics endpoint
pub async fn metrics() -> String {
    // TODO: Implement actual Prometheus metrics collection
    // For now, return basic uptime metric
    format!(
        "# HELP up Service up status\n# TYPE up gauge\nup 1\n# HELP api_requests_total Total requests\n# TYPE api_requests_total counter\napi_requests_total 0\n"
    )
}

async fn create_health_checker(state: &AppState) -> HealthChecker {
    let version = env!("CARGO_PKG_VERSION");
    
    HealthChecker::new(version)
        // Add downstream service checks with proper URLs
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
        .add_check(confuse_connectivity::health::checks::DependencyCheck::new(
            "mcp-server",
            format!("{}/health", state.config.mcp_server_url),
            5000,
        ))
        .add_check(confuse_connectivity::health::checks::DependencyCheck::new(
            "unified-processor",
            format!("{}/health", state.config.unified_processor_url),
            5000,
        ))
}
