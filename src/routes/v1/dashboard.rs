//! Dashboard routes - stats and overview endpoints

use axum::{
    extract::State,
    Json,
};
use serde::{Deserialize, Serialize};

use super::AppState;

#[derive(Debug, Serialize)]
pub struct DashboardStats {
    pub repositories: u32,
    pub documents: u32,
    pub urls: u32,
    pub agents: u32,
    pub connections: u32,
    pub context_requests: u32,
    pub security_score: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// Get dashboard statistics
pub async fn get_stats(
    State(_state): State<AppState>,
) -> Json<DashboardStats> {
    // Return mock stats for development
    Json(DashboardStats {
        repositories: 3,
        documents: 12,
        urls: 5,
        agents: 2,
        connections: 4,
        context_requests: 1247,
        security_score: 98,
    })
}
