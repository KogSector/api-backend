//! Health and status endpoints

use axum::{extract::State, Json};
use chrono::Utc;
use std::collections::HashMap;

use crate::error::Result;
use crate::models::{HealthResponse, ServiceHealth};
use super::AppState;

/// GET /health - Basic health check
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "api-backend".to_string(),
        timestamp: Some(Utc::now().to_rfc3339()),
        services: None,
    })
}

/// GET /status - Detailed status with downstream service health
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
    
    // Overall status
    let all_healthy = auth_healthy && dc_healthy && rg_healthy && mcp_healthy;
    
    Ok(Json(HealthResponse {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        service: "api-backend".to_string(),
        timestamp: Some(Utc::now().to_rfc3339()),
        services: Some(services),
    }))
}
