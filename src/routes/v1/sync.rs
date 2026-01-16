//! Sync endpoints

use axum::{
    extract::{Path, State, Extension},
    Json,
};

use crate::error::Result;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::{SyncJob, JobStatusResponse};
use super::AppState;

/// POST /v1/sync/:source_id - Trigger sync for a source
pub async fn trigger_sync(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(source_id): Path<String>,
) -> Result<Json<SyncJob>> {
    let job = state.data_connector_client
        .sync_source(&source_id)
        .await?;
    
    Ok(Json(job))
}

/// GET /v1/sync/:job_id/status - Get sync job status
pub async fn get_sync_status(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Path(job_id): Path<String>,
) -> Result<Json<JobStatusResponse>> {
    let status = state.data_connector_client
        .get_job_status(&job_id)
        .await?;
    
    Ok(Json(status))
}
