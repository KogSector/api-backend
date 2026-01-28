//! Unified Processor client for processing, embedding, and search operations
//!
//! This client communicates with the unified-processor service which consolidates:
//! - code-normalize-fetch (Port 8090)
//! - doc-parser (Port 3019)
//! - embeddings (Port 3001)

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use super::base::{create_http_client, handle_service_response};

// ==============================================================================
// Request/Response Types
// ==============================================================================

/// Request for processing files
#[derive(Debug, Serialize)]
pub struct ProcessRequest {
    pub source_id: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository_url: Option<String>,
}

fn default_source_type() -> String {
    "local".to_string()
}

/// Request for chunking
#[derive(Debug, Serialize)]
pub struct ChunkRequest {
    pub content: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: u32,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: u32,
}

fn default_language() -> String {
    "python".to_string()
}

fn default_chunk_size() -> u32 {
    1000
}

fn default_chunk_overlap() -> u32 {
    300
}

/// Request for single text embedding
#[derive(Debug, Serialize)]
pub struct EmbedRequest {
    pub text: String,
    #[serde(default = "default_cache")]
    pub cache: bool,
}

fn default_cache() -> bool {
    true
}

/// Request for batch embedding
#[derive(Debug, Serialize)]
pub struct BatchEmbedRequest {
    pub texts: Vec<String>,
    #[serde(default = "default_cache")]
    pub cache: bool,
}

/// Request for semantic search
#[derive(Debug, Serialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
    #[serde(default)]
    pub include_embeddings: bool,
}

fn default_top_k() -> u32 {
    10
}

/// Search filters
#[derive(Debug, Serialize)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
}

/// Request for hybrid search
#[derive(Debug, Serialize)]
pub struct HybridSearchRequest {
    pub query: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
    #[serde(default = "default_vector_weight")]
    pub vector_weight: f32,
}

fn default_vector_weight() -> f32 {
    0.7
}

/// Generic service response
#[derive(Debug, Deserialize)]
pub struct ServiceResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Processed response data
#[derive(Debug, Deserialize)]
pub struct ProcessedData {
    pub source_id: String,
    #[serde(default)]
    pub files_processed: u32,
    #[serde(default)]
    pub chunks_created: u32,
    #[serde(default)]
    pub source_type: String,
}

/// Embedding response data
#[derive(Debug, Deserialize)]
pub struct EmbeddingData {
    pub embedding: Vec<f32>,
    pub dimension: u32,
    pub model: String,
    #[serde(default)]
    pub cached: bool,
}

/// Batch embedding response data
#[derive(Debug, Deserialize)]
pub struct BatchEmbeddingData {
    pub embeddings: Vec<Vec<f32>>,
    pub count: u32,
    pub dimension: u32,
    pub model: String,
    #[serde(default)]
    pub cache_hits: u32,
}

/// Search result item
#[derive(Debug, Deserialize)]
pub struct SearchResult {
    pub source_id: String,
    pub chunk_id: String,
    pub filename: String,
    pub content: String,
    pub language: String,
    pub content_type: String,
    #[serde(default)]
    pub score: f32,
    #[serde(default)]
    pub start_line: u32,
    #[serde(default)]
    pub end_line: u32,
}

/// Search response data
#[derive(Debug, Deserialize)]
pub struct SearchData {
    pub query: String,
    pub results: Vec<SearchResult>,
    pub count: u32,
    #[serde(default)]
    pub search_type: String,
}

// ==============================================================================
// Client Implementation
// ==============================================================================

/// Client for unified-processor service
#[derive(Clone)]
pub struct UnifiedProcessorClient {
    client: Client,
    base_url: String,
}

impl UnifiedProcessorClient {
    /// Create a new unified processor client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(60)?, // 60 second timeout for processing
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// Process files through the unified pipeline
    pub async fn process(&self, request: &ProcessRequest) -> Result<ServiceResponse<ProcessedData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/process", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Chunk code content
    pub async fn chunk(&self, request: &ChunkRequest) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/chunk", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Generate single text embedding
    pub async fn embed(&self, request: &EmbedRequest) -> Result<ServiceResponse<EmbeddingData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/embed", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Generate batch embeddings
    pub async fn embed_batch(&self, request: &BatchEmbedRequest) -> Result<ServiceResponse<BatchEmbeddingData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/embed/batch", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Semantic search
    pub async fn search(&self, request: &SearchRequest) -> Result<ServiceResponse<SearchData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/search", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Hybrid search (vector + keyword)
    pub async fn search_hybrid(&self, request: &HybridSearchRequest) -> Result<ServiceResponse<SearchData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/search/hybrid", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Parse document (legacy doc-parser compatibility)
    pub async fn parse_document(&self, request: &ProcessRequest) -> Result<ServiceResponse<ProcessedData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/parse", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
    }
    
    /// Get service status
    pub async fn get_status(&self) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/status", self.base_url))
            .send()
            .await?;
        
        handle_service_response(response, "unified-processor").await
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
