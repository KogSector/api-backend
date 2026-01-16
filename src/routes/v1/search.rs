//! Search endpoints

use axum::{
    extract::{State, Extension},
    Json,
};

use crate::error::Result;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::{SearchRequest, SearchResponse};
use super::AppState;

/// POST /v1/search - Hybrid search (vector + graph)
pub async fn hybrid_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    let results = state.relation_graph_client
        .search(&request)
        .await?;
    
    Ok(Json(results))
}

/// POST /v1/search/vector - Vector-only search
pub async fn vector_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    let results = state.relation_graph_client
        .search_vector(&request)
        .await?;
    
    Ok(Json(results))
}

/// POST /v1/search/graph - Graph-only search
pub async fn graph_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    let results = state.relation_graph_client
        .search_graph(&request)
        .await?;
    
    Ok(Json(results))
}
