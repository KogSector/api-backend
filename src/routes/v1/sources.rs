//! Source management endpoints

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;

use crate::error::{AppError, Result};
use crate::middleware::auth::AuthenticatedUser;
use crate::models::{Source, SourceCreateRequest, SourcesListResponse};
use super::AppState;

#[derive(Debug, Deserialize)]
pub struct ListSourcesQuery {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

/// GET /v1/sources - List all sources for the authenticated user
pub async fn list_sources(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Query(query): Query<ListSourcesQuery>,
) -> Result<Json<SourcesListResponse>> {
    let sources = state.data_connector_client
        .list_sources(&user.0.id, query.limit, query.offset)
        .await?;
    
    Ok(Json(sources))
}

/// GET /v1/sources/:id - Get a specific source
pub async fn get_source(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(source_id): Path<String>,
) -> Result<Json<Source>> {
    let source = state.data_connector_client
        .get_source(&user.0.id, &source_id)
        .await?;
    
    Ok(Json(source))
}

/// POST /v1/sources - Create a new source
pub async fn create_source(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Json(request): Json<SourceCreateRequest>,
) -> Result<Json<Source>> {
    let source = state.data_connector_client
        .create_source(&user.0.id, &request)
        .await?;
    
    Ok(Json(source))
}

/// DELETE /v1/sources/:id - Delete a source
pub async fn delete_source(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(source_id): Path<String>,
) -> Result<Json<serde_json::Value>> {
    state.data_connector_client
        .delete_source(&user.0.id, &source_id)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Source deleted"
    })))
}
