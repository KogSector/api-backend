//! Webhook endpoints for receiving events from external services

use axum::{
    extract::{State, Path},
    http::HeaderMap,
    Json,
};

use crate::error::Result;
use super::v1::AppState;

/// POST /webhooks/github - Handle GitHub webhook events
pub async fn github_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // Extract relevant headers for forwarding
    let mut forward_headers = vec![];
    
    if let Some(event) = headers.get("X-GitHub-Event") {
        forward_headers.push((
            "X-GitHub-Event".to_string(),
            event.to_str().unwrap_or("").to_string(),
        ));
    }
    if let Some(sig) = headers.get("X-Hub-Signature-256") {
        forward_headers.push((
            "X-Hub-Signature-256".to_string(),
            sig.to_str().unwrap_or("").to_string(),
        ));
    }
    if let Some(delivery) = headers.get("X-GitHub-Delivery") {
        forward_headers.push((
            "X-GitHub-Delivery".to_string(),
            delivery.to_str().unwrap_or("").to_string(),
        ));
    }
    
    // Forward to data-connector
    let result = state.data_connector_client
        .forward_webhook("github", payload, forward_headers)
        .await?;
    
    Ok(Json(result))
}

/// POST /webhooks/gitlab - Handle GitLab webhook events
pub async fn gitlab_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>> {
    // Extract relevant headers for forwarding
    let mut forward_headers = vec![];
    
    if let Some(event) = headers.get("X-Gitlab-Event") {
        forward_headers.push((
            "X-Gitlab-Event".to_string(),
            event.to_str().unwrap_or("").to_string(),
        ));
    }
    if let Some(token) = headers.get("X-Gitlab-Token") {
        forward_headers.push((
            "X-Gitlab-Token".to_string(),
            token.to_str().unwrap_or("").to_string(),
        ));
    }
    
    // Forward to data-connector
    let result = state.data_connector_client
        .forward_webhook("gitlab", payload, forward_headers)
        .await?;
    
    Ok(Json(result))
}
