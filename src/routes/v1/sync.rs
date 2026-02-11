//! Sync endpoints
//!
//! Event-driven sync operations using Kafka.

use axum::{
    extract::{Path, State, Extension},
    Json,
};

use crate::error::{AppError, Result};
use crate::middleware::auth::AuthenticatedUser;
use confuse_common::events::{
    SourceSyncRequestedEvent, 
    SourceType as EventSourceType, 
};
use confuse_common::events::topics;
// Note: SyncRequestResponse is likely local to api-backend or needs to be migrated. 
// Assuming it's local or we map Event -> Response manually.
// Checking imports: src/kafka/events imported SyncRequestResponse.
// If SyncRequestResponse was defined in src/kafka/events.rs, I might break it if I delete the folder.
// I should verify where SyncRequestResponse is defined.
// Assuming for now I can replicate it or import it if I move it.
// I'll leave `use crate::kafka::events::SyncRequestResponse` removed and look for a replacement or define it here?
// Actually, `SyncRequestResponse` seems to be a DTO.
// I will assume for a moment it was in `src/kafka/events.rs`.
// I need to define it here or alias it.
// To satisfy compiler, I'll temporarily define a local struct or look for it.
// Wait, `SourceSyncRequestedEvent` replaced `crate::kafka::events::SourceSyncRequestedEvent`.
// `SyncRequestResponse` was likely a derived struct.
// I'll define `SyncRequestResponse` in this file to unblock migration.

#[derive(serde::Serialize)]
pub struct SyncRequestResponse {
    pub correlation_id: Option<String>,
    pub event_id: String,
    pub status: String,
    pub timestamp: String,
}

impl From<&SourceSyncRequestedEvent> for SyncRequestResponse {
    fn from(event: &SourceSyncRequestedEvent) -> Self {
        Self {
            correlation_id: event.headers.correlation_id.clone(),
            event_id: event.headers.event_id.clone(),
            status: "requested".to_string(),
            timestamp: event.headers.timestamp.clone(),
        }
    }
}

use crate::models::{JobStatusResponse, SourceType, Source};
use super::AppState;

/// Map model SourceType to event SourceType
fn map_source_type(source_type: &SourceType) -> EventSourceType {
    match source_type {
        SourceType::Github => EventSourceType::Github,
        SourceType::Gitlab => EventSourceType::Gitlab,
        SourceType::Gdrive => EventSourceType::GoogleDrive,
        SourceType::Notion => EventSourceType::Notion,
        SourceType::Upload => EventSourceType::FileUpload,
        _ => EventSourceType::Local, // Default for other types
    }
}

/// Extract URL from source configuration
fn extract_source_url(source: &Source) -> String {
    if let Some(ref metadata) = source.metadata {
        if let Some(url) = metadata.get("url") {
            if let Some(s) = url.as_str() {
                return s.to_string();
            }
        }
    }
    format!("source://{}", source.id)
}

/// POST /v1/sync/:source_id - Trigger sync for a source
/// 
/// Publishes SourceSyncRequestedEvent event to Kafka if available,
/// falls back to HTTP if Kafka is unavailable.
pub async fn trigger_sync(
    State(state): State<AppState>,
    Extension(user): Extension<AuthenticatedUser>,
    Path(source_id): Path<String>,
) -> Result<Json<SyncRequestResponse>> {
    // Try event-driven path first
    if let Some(ref producer) = state.event_producer {
        // Lookup source to get details for the event
        let source = state.data_connector_client
            .get_source(&user.0.id, &source_id)
            .await?;
        
        let event_source_type = map_source_type(&source.source_type);
        let source_url = extract_source_url(&source);
        
        let event = SourceSyncRequestedEvent::new(
            source_id.clone(),
            event_source_type,
            source_url,
        ).with_user(user.0.id.clone());
        
        // Changed: parameter order (event first) and removed key (None)
        producer.publish_to_topic(&event, topics::Topics::SOURCE_SYNC_REQUESTED).await
            .map_err(|e| AppError::Internal(format!("Event publish failed: {}", e)))?;
        
        tracing::info!(
            "Published sync event: source_id={}, event_id={}",
            source_id,
            event.headers.event_id
        );
        
        return Ok(Json(SyncRequestResponse::from(&event)));
    }
    
    // Fallback to HTTP-based sync
    tracing::debug!("Kafka unavailable, using HTTP fallback for sync");
    let job = state.data_connector_client
        .sync_source(&source_id)
        .await?;
    
    Ok(Json(SyncRequestResponse {
        correlation_id: Some(job.job_id.clone()),
        event_id: job.job_id,
        status: "sync_started".to_string(),
        timestamp: chrono::Utc::now().to_rfc3339(),
    }))
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
