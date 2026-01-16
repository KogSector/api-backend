//! Entity endpoints

use axum::{
    extract::{Path, Query, State, Extension},
    Json,
};
use serde::Deserialize;

use crate::error::Result;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::Entity;
use super::AppState;

#[derive(Debug, Deserialize)]
pub struct GetNeighborsQuery {
    #[serde(default = "default_hops")]
    pub hops: u32,
}

fn default_hops() -> u32 { 2 }

/// GET /v1/entities/:id - Get entity details
pub async fn get_entity(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(entity_id): Path<String>,
) -> Result<Json<Entity>> {
    let entity = state.relation_graph_client
        .get_entity(&entity_id, 1)
        .await?;
    
    Ok(Json(entity))
}

/// GET /v1/entities/:id/neighbors - Get related entities
pub async fn get_neighbors(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(entity_id): Path<String>,
    Query(query): Query<GetNeighborsQuery>,
) -> Result<Json<Entity>> {
    let entity = state.relation_graph_client
        .get_entity(&entity_id, query.hops)
        .await?;
    
    Ok(Json(entity))
}
