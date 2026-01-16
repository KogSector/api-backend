//! Data Connector client for source management

use reqwest::Client;
use serde::Serialize;

use crate::error::AppError;
use crate::models::{Source, SourceCreateRequest, SyncJob, JobStatusResponse, SourcesListResponse};
use super::base::{create_http_client, handle_service_response};

/// Client for data-connector service
#[derive(Clone)]
pub struct DataConnectorClient {
    client: Client,
    base_url: String,
}

impl DataConnectorClient {
    /// Create a new data connector client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(30)?, // 30 second timeout for sync ops
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// List sources for a user
    pub async fn list_sources(
        &self, 
        user_id: &str,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<SourcesListResponse, AppError> {
        let mut url = format!("{}/sources", self.base_url);
        let mut params = vec![];
        if let Some(l) = limit {
            params.push(format!("limit={}", l));
        }
        if let Some(o) = offset {
            params.push(format!("offset={}", o));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
        
        let response = self.client
            .get(&url)
            .header("X-User-Id", user_id)
            .send()
            .await?;
        
        handle_service_response(response, "data-connector").await
    }
    
    /// Get a specific source
    pub async fn get_source(&self, user_id: &str, source_id: &str) -> Result<Source, AppError> {
        let response = self.client
            .get(format!("{}/sources/{}", self.base_url, source_id))
            .header("X-User-Id", user_id)
            .send()
            .await?;
        
        handle_service_response(response, "data-connector").await
    }
    
    /// Create a new source
    pub async fn create_source(
        &self,
        user_id: &str,
        request: &SourceCreateRequest,
    ) -> Result<Source, AppError> {
        let response = self.client
            .post(format!("{}/sources", self.base_url))
            .header("X-User-Id", user_id)
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "data-connector").await
    }
    
    /// Delete a source
    pub async fn delete_source(&self, user_id: &str, source_id: &str) -> Result<(), AppError> {
        let response = self.client
            .delete(format!("{}/sources/{}", self.base_url, source_id))
            .header("X-User-Id", user_id)
            .send()
            .await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err(AppError::Internal(format!(
                "Failed to delete source: {}",
                response.status()
            )))
        }
    }
    
    /// Start sync for a source
    pub async fn sync_source(&self, source_id: &str) -> Result<SyncJob, AppError> {
        #[derive(Serialize)]
        struct IngestRequest {
            source_id: String,
        }
        
        let response = self.client
            .post(format!("{}/ingest", self.base_url))
            .json(&IngestRequest { source_id: source_id.to_string() })
            .send()
            .await?;
        
        handle_service_response(response, "data-connector").await
    }
    
    /// Get job status
    pub async fn get_job_status(&self, job_id: &str) -> Result<JobStatusResponse, AppError> {
        let response = self.client
            .get(format!("{}/jobs/{}", self.base_url, job_id))
            .send()
            .await?;
        
        handle_service_response(response, "data-connector").await
    }
    
    /// Forward webhook payload
    pub async fn forward_webhook(
        &self,
        provider: &str,
        payload: serde_json::Value,
        headers: Vec<(String, String)>,
    ) -> Result<serde_json::Value, AppError> {
        let mut request = self.client
            .post(format!("{}/webhooks/{}", self.base_url, provider))
            .json(&payload);
        
        for (key, value) in headers {
            request = request.header(&key, &value);
        }
        
        let response = request.send().await?;
        handle_service_response(response, "data-connector").await
    }
    
    /// Health check
    pub async fn health_check(&self) -> bool {
        self.client
            .get(format!("{}/health", self.base_url))
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false)
    }
}
