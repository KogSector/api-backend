//! Source-related models

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source types supported by the platform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceType {
    Github,
    Gitlab,
    Bitbucket,
    Gdrive,
    Dropbox,
    LocalFs,
    Notion,
    Confluence,
    Slack,
    Jira,
    Linear,
    Upload,
}

/// Source status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SourceStatus {
    Pending,
    Syncing,
    Synced,
    Failed,
    Disconnected,
}

/// Data source representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub name: String,
    pub status: SourceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_sync: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<SourceStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Source statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceStats {
    pub files: u64,
    pub chunks: u64,
    pub entities: u64,
}

/// Request to create a new source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceCreateRequest {
    #[serde(rename = "type")]
    pub source_type: SourceType,
    pub config: SourceConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

/// Source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Sync job
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncJob {
    pub job_id: String,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_time: Option<String>,
}

/// Job status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Job status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: JobStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}
