//! V1 API routes

pub mod health;
pub mod sources;
pub mod search;
pub mod entities;
pub mod sync;
pub mod mcp;
pub mod urls;
pub mod dashboard;
pub mod repositories;
pub mod documents;
pub mod agents;
pub mod processing;
pub mod compliance;

use axum::{Router, routing::{get, post, delete, put}};
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
    pub unified_processor_client: Arc<crate::clients::UnifiedProcessorClient>,
    pub enhanced_graph_client: Arc<crate::clients::EnhancedGraphClient>,
    pub auth_layer: AuthLayer,
    /// Kafka event producer for event-driven operations (optional for graceful fallback)
    pub event_producer: Option<Arc<confuse_common::events::producer::EventProducer>>,
    /// Circuit breaker registry for downstream service calls
    pub circuit_breaker: Arc<crate::middleware::CircuitBreakerRegistry>,
    /// Response cache for auth/data responses
    pub response_cache: Arc<crate::middleware::ResponseCache>,
}

/// Create the V1 router
pub fn v1_router(state: AppState) -> Router {
    // Public routes (no auth required)
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/health/detailed", get(health::health_check_detailed))
        .route("/health/ready", get(health::readiness))
        .route("/health/live", get(health::liveness))
        .route("/status", get(health::status_check))
        .route("/metrics", get(health::metrics));
    
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
        // Processing (unified-processor integration)
        .route("/process", post(processing::process_files))
        .route("/chunk", post(processing::chunk_content))
        .route("/embed", post(processing::embed_text))
        .route("/embed/batch", post(processing::embed_batch))
        .route("/search/semantic", post(processing::semantic_search))
        .route("/processor/status", get(processing::get_processor_status));
    
    // URL routes (public for now to simplify development)
    let url_routes = Router::new()
        .route("/api/urls", get(urls::list_urls))
        .route("/api/urls", post(urls::create_url))
        .route("/api/urls/:id", get(urls::get_url))
        .route("/api/urls/:id", delete(urls::delete_url));
    
    // Dashboard routes
    let dashboard_routes = Router::new()
        .route("/api/dashboard/stats", get(dashboard::get_stats));
    
    // Repository routes
    let repository_routes = Router::new()
        .route("/api/repositories", get(repositories::list_repositories))
        .route("/api/repositories", post(repositories::create_repository))
        .route("/api/repositories/:id", get(repositories::get_repository))
        .route("/api/repositories/:id", delete(repositories::delete_repository));
    
    // Document routes
    let document_routes = Router::new()
        .route("/api/documents", get(documents::list_documents))
        .route("/api/documents", post(documents::create_document))
        .route("/api/documents/:id", delete(documents::delete_document))
        .route("/api/documents/analytics", get(documents::get_analytics));
    
    // Agent routes
    let agent_routes = Router::new()
        .route("/api/agents", get(agents::list_agents))
        .route("/api/agents", post(agents::create_agent))
        .route("/api/agents/:id", get(agents::get_agent))
        .route("/api/agents/:id", put(agents::update_agent))
        .route("/api/agents/:id", delete(agents::delete_agent))
        .route("/api/agents/:id/test", post(agents::test_agent))
        .route("/api/agents/:id/invoke", post(agents::invoke_agent))
        .route("/api/agents/:id/context", get(agents::get_agent_context));
    
    // Compliance / Governance routes
    let compliance_routes = Router::new()
        .route("/api/compliance/dashboard", get(compliance::compliance_dashboard))
        .route("/api/compliance/gdpr/export", post(compliance::gdpr_data_export))
        .route("/api/compliance/gdpr/delete", post(compliance::gdpr_data_deletion));
    
    // Apply auth middleware to protected routes
    let protected_routes = protected_routes
        .layer(axum::middleware::from_fn_with_state(
            state.auth_layer.clone(),
            auth_middleware,
        ));
    
    // Webhook routes (signature verification instead of auth)
    let webhook_routes = Router::new()
        .route("/webhooks/github", post(webhooks::github_webhook))
        .route("/webhooks/gitlab", post(webhooks::gitlab_webhook));
    
    // Combine all routes
    Router::new()
        .merge(public_routes)
        .nest("/v1", protected_routes)
        .merge(url_routes)
        .merge(dashboard_routes)
        .merge(repository_routes)
        .merge(document_routes)
        .merge(agent_routes)
        .merge(compliance_routes)
        .merge(webhook_routes)
        .with_state(state)
}
