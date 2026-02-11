//! Health check endpoints for API Backend

use axum::{extract::State, http::StatusCode, Json};
use serde::Serialize;
use crate::routes::v1::AppState;

#[derive(Debug, Serialize)]
pub struct HealthCheckResult {
    pub status: String,
    pub version: String,
    pub timestamp: String,
    pub components: Vec<ComponentHealth>,
}

#[derive(Debug, Serialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Health check handler
pub async fn health_check(
    State(state): State<AppState>,
) -> (StatusCode, Json<HealthCheckResult>) {
    let mut components = Vec::new();
    
    // Check downstream services
    let services = vec![
        ("auth-middleware", &state.config.auth_middleware_url),
        ("data-connector", &state.config.data_connector_url),
        ("relation-graph", &state.config.relation_graph_url),
    ];
    
    for (name, url) in services {
        let status = match reqwest::Client::new()
            .get(format!("{}/health", url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
        {
            Ok(resp) if resp.status().is_success() => "healthy".to_string(),
            Ok(resp) => format!("unhealthy ({})", resp.status()),
            Err(e) => format!("unreachable: {}", e),
        };
        
        components.push(ComponentHealth {
            name: name.to_string(),
            status,
            message: None,
        });
    }
    
    let all_healthy = components.iter().all(|c| c.status == "healthy");
    let status_code = if all_healthy { StatusCode::OK } else { StatusCode::OK }; // Still accept traffic
    
    let result = HealthCheckResult {
        status: if all_healthy { "healthy" } else { "degraded" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
        components,
    };
    
    (status_code, Json(result))
}

/// Readiness probe
pub async fn readiness(
    State(state): State<AppState>,
) -> (StatusCode, Json<HealthCheckResult>) {
    health_check(State(state)).await
}

/// Liveness probe
pub async fn liveness() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(serde_json::json!({
            "status": "alive",
            "timestamp": chrono::Utc::now().timestamp()
        })),
    )
}
