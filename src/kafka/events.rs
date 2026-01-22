//! Event Definitions
//!
//! Event schemas for API Backend to publish to Kafka.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Kafka topics
pub mod topics {
    pub const SOURCE_SYNC_REQUESTED: &str = "source.sync.requested";
    pub const SOURCE_SYNC_COMPLETED: &str = "source.sync.completed";
    pub const SOURCE_SYNC_FAILED: &str = "source.sync.failed";
}

/// Source types for ingestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Github,
    Gitlab,
    Local,
    GoogleDrive,
    Notion,
    FileUpload,
}

/// Event headers for tracing and routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHeaders {
    pub correlation_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<String>,
    pub source_service: String,
    pub event_type: String,
    pub event_version: String,
    pub timestamp: DateTime<Utc>,
}

impl EventHeaders {
    pub fn new(event_type: impl Into<String>) -> Self {
        Self {
            correlation_id: Uuid::new_v4().to_string(),
            causation_id: None,
            source_service: "api-backend".to_string(),
            event_type: event_type.into(),
            event_version: "1.0".to_string(),
            timestamp: Utc::now(),
        }
    }

    pub fn with_correlation(event_type: impl Into<String>, correlation_id: String) -> Self {
        Self {
            correlation_id,
            causation_id: None,
            source_service: "api-backend".to_string(),
            event_type: event_type.into(),
            event_version: "1.0".to_string(),
            timestamp: Utc::now(),
        }
    }
}

/// Event metadata for auditing
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tenant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Event published when a source sync is requested
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSyncRequestedEvent {
    pub event_id: String,
    pub headers: EventHeaders,
    pub metadata: EventMetadata,
    
    /// Unique source identifier
    pub source_id: String,
    /// Type of source
    pub source_type: SourceType,
    /// URL or path to the source
    pub source_url: String,
    /// Branch for git sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    /// Access token (encrypted)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    /// Whether to force full resync
    #[serde(default)]
    pub full_sync: bool,
}

impl SourceSyncRequestedEvent {
    pub fn new(
        source_id: String,
        source_type: SourceType,
        source_url: String,
    ) -> Self {
        Self {
            event_id: Uuid::new_v4().to_string(),
            headers: EventHeaders::new("source.sync.requested"),
            metadata: EventMetadata::default(),
            source_id,
            source_type,
            source_url,
            branch: None,
            access_token: None,
            full_sync: false,
        }
    }

    pub fn with_user(mut self, user_id: String) -> Self {
        self.metadata.user_id = Some(user_id);
        self
    }

    pub fn with_branch(mut self, branch: String) -> Self {
        self.branch = Some(branch);
        self
    }

    pub fn with_token(mut self, token: String) -> Self {
        self.access_token = Some(token);
        self
    }

    pub fn with_full_sync(mut self) -> Self {
        self.full_sync = true;
        self
    }
    
    pub fn correlation_id(&self) -> &str {
        &self.headers.correlation_id
    }
}

/// Response returned to client after publishing sync request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncRequestResponse {
    /// Correlation ID to track the sync request
    pub correlation_id: String,
    /// Event ID
    pub event_id: String,
    /// Status message
    pub status: String,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
}

impl From<&SourceSyncRequestedEvent> for SyncRequestResponse {
    fn from(event: &SourceSyncRequestedEvent) -> Self {
        Self {
            correlation_id: event.headers.correlation_id.clone(),
            event_id: event.event_id.clone(),
            status: "sync_requested".to_string(),
            timestamp: event.headers.timestamp,
        }
    }
}
