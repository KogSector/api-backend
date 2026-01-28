//! Relation Graph client for temporal knowledge graph operations
//!
//! This client communicates with the relation-graph service (Graphiti-powered),
//! providing temporal knowledge graph capabilities.

use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::error::AppError;
use super::base::{create_http_client, handle_service_response};

// ==============================================================================
// Request/Response Types
// ==============================================================================

/// Request for building relationships
#[derive(Debug, Serialize)]
pub struct BuildRelationshipsRequest {
    pub source_id: String,
    #[serde(default)]
    pub force_rebuild: bool,
}

/// Request for temporal search
#[derive(Debug, Serialize)]
pub struct TemporalSearchRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(default)]
    pub include_nodes: bool,
    #[serde(default = "default_true")]
    pub include_edges: bool,
}

fn default_limit() -> u32 {
    10
}

fn default_true() -> bool {
    true
}

/// Request for entity evolution
#[derive(Debug, Serialize)]
pub struct EntityEvolutionRequest {
    pub entity_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// Request for adding an episode
#[derive(Debug, Serialize)]
pub struct AddEpisodeRequest {
    pub name: String,
    pub content: serde_json::Value,
    #[serde(default = "default_episode_type")]
    pub episode_type: String,
    #[serde(default)]
    pub source_description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reference_time: Option<String>,
}

fn default_episode_type() -> String {
    "text".to_string()
}

/// Generic service response
#[derive(Debug, Deserialize)]
pub struct GraphServiceResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(default)]
    pub data: Option<T>,
    #[serde(default)]
    pub error: Option<String>,
}

/// Build relationships response data
#[derive(Debug, Deserialize, Default)]
pub struct BuildResponseData {
    pub source_id: String,
    #[serde(default)]
    pub chunks_found: u32,
    #[serde(default)]
    pub episodes_added: u32,
    #[serde(default)]
    pub errors: Vec<String>,
}

/// Edge/fact from graph
#[derive(Debug, Deserialize, Default)]
pub struct Edge {
    pub uuid: String,
    pub fact: String,
    #[serde(default)]
    pub valid_at: Option<String>,
    #[serde(default)]
    pub invalid_at: Option<String>,
    #[serde(default)]
    pub source_node_uuid: Option<String>,
    #[serde(default)]
    pub target_node_uuid: Option<String>,
}

/// Node from graph
#[derive(Debug, Deserialize, Default)]
pub struct Node {
    pub uuid: String,
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
}

/// Temporal search response data
#[derive(Debug, Deserialize, Default)]
pub struct TemporalSearchData {
    pub query: String,
    #[serde(default)]
    pub timestamp: String,
    #[serde(default)]
    pub edges: Vec<Edge>,
    #[serde(default)]
    pub edge_count: u32,
    #[serde(default)]
    pub nodes: Vec<Node>,
    #[serde(default)]
    pub node_count: u32,
}

/// Evolution record
#[derive(Debug, Deserialize, Default)]
pub struct EvolutionRecord {
    pub uuid: String,
    pub fact: String,
    #[serde(default)]
    pub valid_from: Option<String>,
    #[serde(default)]
    pub valid_until: Option<String>,
    #[serde(default)]
    pub is_current: bool,
}

/// Entity evolution response data
#[derive(Debug, Deserialize, Default)]
pub struct EntityEvolutionData {
    pub entity: String,
    pub current_state: Vec<EvolutionRecord>,
    pub historical: Vec<EvolutionRecord>,
    #[serde(default)]
    pub total_records: u32,
}

/// Episode added response data
#[derive(Debug, Deserialize, Default)]
pub struct EpisodeAddedData {
    pub name: String,
    pub episode_type: String,
    pub reference_time: String,
}

// ==============================================================================
// Client Implementation
// ==============================================================================

/// Client for relation-graph service (Graphiti-powered)
#[derive(Clone)]
pub struct RelationGraphClient {
    client: Client,
    base_url: String,
}

impl RelationGraphClient {
    /// Create a new relation graph client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(30)?, // 30 second timeout
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// Build relationships for a source (legacy compatibility)
    pub async fn build_relationships(&self, request: &BuildRelationshipsRequest) -> Result<GraphServiceResponse<BuildResponseData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/build", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Temporal search over knowledge graph
    pub async fn temporal_search(&self, request: &TemporalSearchRequest) -> Result<GraphServiceResponse<TemporalSearchData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/temporal-search", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Simple search (GET)
    pub async fn search_simple(&self, query: &str, limit: u32) -> Result<GraphServiceResponse<TemporalSearchData>, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/search?query={}&limit={}", self.base_url, query, limit))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get entity evolution
    pub async fn get_entity_evolution(&self, entity_name: &str) -> Result<GraphServiceResponse<EntityEvolutionData>, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/entity-evolution/{}", self.base_url, entity_name))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get entity evolution with time range
    pub async fn get_entity_evolution_detailed(&self, request: &EntityEvolutionRequest) -> Result<GraphServiceResponse<EntityEvolutionData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/entity-evolution", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Add an episode to the knowledge graph
    pub async fn add_episode(&self, request: &AddEpisodeRequest) -> Result<GraphServiceResponse<EpisodeAddedData>, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/episodes", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get relationships for a source (legacy compatibility)
    pub async fn get_relationships(&self, source_id: &str) -> Result<GraphServiceResponse<TemporalSearchData>, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/relationships/{}", self.base_url, source_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get context for a chunk (legacy compatibility)
    pub async fn get_context_legacy(&self, chunk_id: &str) -> Result<GraphServiceResponse<TemporalSearchData>, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/context/{}", self.base_url, chunk_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get related chunks (legacy compatibility)
    pub async fn get_related(&self, chunk_id: &str) -> Result<GraphServiceResponse<TemporalSearchData>, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/related/{}", self.base_url, chunk_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get graph statistics
    pub async fn get_stats(&self) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/stats", self.base_url))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get service status
    pub async fn get_status(&self) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/status", self.base_url))
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
    
    /// Unified search (hybrid vector + graph)
    pub async fn search(&self, request: &crate::models::SearchRequest) -> Result<crate::models::SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/search", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Vector-only search
    pub async fn search_vector(&self, request: &crate::models::SearchRequest) -> Result<crate::models::SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/search/vector", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Graph-only search
    pub async fn search_graph(&self, request: &crate::models::SearchRequest) -> Result<crate::models::SearchResponse, AppError> {
        let response = self.client
            .post(format!("{}/api/v1/search/graph", self.base_url))
            .json(request)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get entity by ID
    pub async fn get_entity(&self, entity_id: &str, hops: u32) -> Result<crate::models::Entity, AppError> {
        let url = format!("{}/api/v1/entities/{}?hops={}", self.base_url, entity_id, hops);
        
        let response = self.client
            .get(url)
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
    
    /// Get context for a chunk (for MCP)
    pub async fn get_context(&self, chunk_id: &str) -> Result<serde_json::Value, AppError> {
        let response = self.client
            .get(format!("{}/api/v1/context/{}", self.base_url, chunk_id))
            .send()
            .await?;
        
        handle_service_response(response, "relation-graph").await
    }
}

