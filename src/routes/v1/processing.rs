//! Processing endpoints for unified-processor integration
//!
//! These routes proxy requests to the unified-processor service which handles:
//! - File processing (Docling + Tree-sitter)
//! - Embeddings generation
//! - Semantic search

use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::clients::unified_processor_client as upc;
use super::AppState;

// ==============================================================================
// Request/Response Types
// ==============================================================================

#[derive(Debug, Deserialize)]
pub struct ProcessRequest {
    pub source_id: String,
    #[serde(default)]
    pub files: Vec<String>,
    pub content: Option<String>,
    pub language: Option<String>,
    #[serde(default = "default_source_type")]
    pub source_type: String,
    pub repository_url: Option<String>,
}

fn default_source_type() -> String { "local".to_string() }

#[derive(Debug, Deserialize)]
pub struct ChunkRequest {
    pub content: String,
    #[serde(default = "default_language")]
    pub language: String,
    #[serde(default = "default_chunk_size")]
    pub chunk_size: u32,
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: u32,
}

fn default_language() -> String { "python".to_string() }
fn default_chunk_size() -> u32 { 1000 }
fn default_chunk_overlap() -> u32 { 300 }

#[derive(Debug, Deserialize)]
pub struct EmbedRequest {
    pub text: String,
    #[serde(default = "default_cache")]
    pub cache: bool,
}

fn default_cache() -> bool { true }

#[derive(Debug, Deserialize)]
pub struct BatchEmbedRequest {
    pub texts: Vec<String>,
    #[serde(default = "default_cache")]
    pub cache: bool,
}

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_top_k")]
    pub top_k: u32,
    #[serde(default)]
    pub include_embeddings: bool,
}

fn default_top_k() -> u32 { 10 }

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

// ==============================================================================
// Processing Endpoints (Public for simplified access)
// ==============================================================================

/// POST /v1/process - Process files through unified pipeline
pub async fn process_files(
    State(state): State<AppState>,
    Json(request): Json<ProcessRequest>,
) -> Result<Json<serde_json::Value>> {
    let client_request = upc::ProcessRequest {
        source_id: request.source_id,
        files: request.files,
        content: request.content,
        language: request.language,
        source_type: request.source_type,
        repository_url: request.repository_url,
    };
    
    let result = state.unified_processor_client
        .process(&client_request)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "data": result.data,
        "error": result.error,
    })))
}

/// POST /v1/chunk - Chunk content with language awareness
pub async fn chunk_content(
    State(state): State<AppState>,
    Json(request): Json<ChunkRequest>,
) -> Result<Json<serde_json::Value>> {
    let client_request = upc::ChunkRequest {
        content: request.content,
        language: request.language,
        chunk_size: request.chunk_size,
        chunk_overlap: request.chunk_overlap,
    };
    
    let result = state.unified_processor_client
        .chunk(&client_request)
        .await?;
    
    Ok(Json(result))
}

/// POST /v1/embed - Generate single text embedding
pub async fn embed_text(
    State(state): State<AppState>,
    Json(request): Json<EmbedRequest>,
) -> Result<Json<serde_json::Value>> {
    let client_request = upc::EmbedRequest {
        text: request.text,
        cache: request.cache,
    };
    
    let result = state.unified_processor_client
        .embed(&client_request)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "data": result.data,
        "error": result.error,
    })))
}

/// POST /v1/embed/batch - Generate batch embeddings
pub async fn embed_batch(
    State(state): State<AppState>,
    Json(request): Json<BatchEmbedRequest>,
) -> Result<Json<serde_json::Value>> {
    let client_request = upc::BatchEmbedRequest {
        texts: request.texts,
        cache: request.cache,
    };
    
    let result = state.unified_processor_client
        .embed_batch(&client_request)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "data": result.data,
        "error": result.error,
    })))
}

/// POST /v1/search/semantic - Semantic search via unified-processor
pub async fn semantic_search(
    State(state): State<AppState>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<serde_json::Value>> {
    let client_request = upc::SearchRequest {
        query: request.query,
        top_k: request.top_k,
        filters: None,
        include_embeddings: request.include_embeddings,
    };
    
    let result = state.unified_processor_client
        .search(&client_request)
        .await?;
    
    Ok(Json(serde_json::json!({
        "success": result.success,
        "message": result.message,
        "data": result.data,
        "error": result.error,
    })))
}

/// GET /v1/processor/status - Get unified-processor status
pub async fn get_processor_status(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>> {
    let result = state.unified_processor_client
        .get_status()
        .await?;
    
    Ok(Json(result))
}
