//! Relation Graph client for search and entity operations

use reqwest::Client;

use crate::error::AppError;
use crate::models::{SearchRequest, SearchResponse, Entity};
use super::base::{create_http_client, handle_service_response};

/// Client for relation-graph service
#[derive(Clone)]
pub struct RelationGraphClient {
    client: Client,
    base_url: String,
}

impl RelationGraphClient {
    /// Create a new relation graph client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(10)?, // 10 second timeout
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// Hybrid search (vector + graph)
    pub async fn search(&self, request: &SearchRequest) -> Result<SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/search", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Vector-only search
    pub async fn search_vector(&self, request: &SearchRequest) -> Result<SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/search/vector", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Graph-only search
    pub async fn search_graph(&self, request: &SearchRequest) -> Result<SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/search/graph", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get entity with relationships
    pub async fn get_entity(&self, entity_id: &str, hops: u32) -> Result<Entity, AppError> {
        let response = self.client
            .get(format!(
                "{}/api/graph/entities/{}/neighbors?hops={}",
                self.base_url, entity_id, hops
            ))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get related chunks for a chunk
    pub async fn get_related_chunks(&self, chunk_id: &str) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/chunks/{}/related", self.base_url, chunk_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get context for a chunk
    pub async fn get_context(&self, chunk_id: &str) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/chunks/{}/context", self.base_url, chunk_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get graph statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/api/graph/statistics", self.base_url))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
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
