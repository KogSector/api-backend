//! V1 API routes

pub mod health;
pub mod sources;
pub mod search;
pub mod entities;
pub mod sync;
pub mod mcp;

use axum::{Router, routing::{get, post, delete}};
use std::sync::Arc;

use crate::middleware::auth::{AuthLayer, auth_middleware, optional_auth_middleware};
use super::webhooks;

/// Application state shared across routes
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<crate::Config>,
    pub auth_client: Arc<crate::clients::AuthClient>,
    pub data_connector_client: Arc<crate::clients::DataConnectorClient>,
    pub relation_graph_client: Arc<crate::clients::RelationGraphClient>,
    pub mcp_client: Arc<crate::clients::McpClient>,
    pub auth_layer: AuthLayer,
    /// Kafka event producer for event-driven operations (optional for graceful fallback)
    pub event_producer: Option<Arc<crate::kafka::EventProducer>>,
}

/// Create the V1 router
pub fn v1_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/status", get(health::status_check));
    
    // Protected routes (auth required)
    let protected_routes = Router::new()
        // Sources
        .route("/sources", get(sources::list_sources))
        .route("/sources", post(sources::create_source))
        .route("/sources/:id", get(sources::get_source))
        .route("/sources/:id", delete(sources::delete_source))
        // Search
        .route("/search", post(search::hybrid_search))
        .route("/search/vector", post(search::vector_search))
        .route("/search/graph", post(search::graph_search))
        // Entities
        .route("/entities/:id", get(entities::get_entity))
        .route("/entities/:id/neighbors", get(entities::get_neighbors))
        // Sync
        .route("/sync/:source_id", post(sync::trigger_sync))
        .route("/sync/:job_id/status", get(sync::get_sync_status))
        // MCP
        .route("/mcp/search", post(mcp::mcp_search))
        .route("/mcp/context", post(mcp::mcp_context))
        .route("/mcp/capabilities", get(mcp::get_capabilities))
        .layer(axum::middleware::from_fn_with_state(
            state.auth_layer.clone(),
            auth_middleware,
        ));
    
    // Webhook routes (signature verification instead of auth)
    let webhook_routes = Router::new()
        .route("/webhooks/github", post(webhooks::github_webhook))
        .route("/webhooks/gitlab", post(webhooks::gitlab_webhook));
    
    // Combine all routes under /v1
    Router::new()
        .merge(public_routes)
        .nest("/v1", protected_routes)
        .merge(webhook_routes)
        .with_state(state)
}
