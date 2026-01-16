//! MCP endpoints for AI agent integration

use axum::{
    extract::{State, Extension},
    Json,
};
use serde::Deserialize;

use crate::error::Result;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::{SearchRequest, SearchResponse, McpCapabilities};
use super::AppState;

#[derive(Debug, Deserialize)]
pub struct McpContextRequest {
    pub chunk_id: String,
}

/// POST /v1/mcp/search - MCP search endpoint
pub async fn mcp_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    // Same as hybrid search, but may add MCP-specific logging or processing
    let results = state.relation_graph_client
        .search(&request)
        .await?;
    
    Ok(Json(results))
}

/// POST /v1/mcp/context - Get context for a chunk
pub async fn mcp_context(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<McpContextRequest>,
) -> Result<Json<serde_json::Value>> {
    let context = state.relation_graph_client
        .get_context(&request.chunk_id)
        .await?;
    
    Ok(Json(context))
}

/// GET /v1/mcp/capabilities - List MCP capabilities
pub async fn get_capabilities(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
) -> Result<Json<McpCapabilities>> {
    let capabilities = state.mcp_client
        .list_tools()
        .await?;
    
    Ok(Json(capabilities))
}
