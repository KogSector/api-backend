//! Search-related models

use serde::{Deserialize, Serialize};

/// Search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filters: Option<SearchFilters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub options: Option<SearchOptions>,
}

fn default_limit() -> u32 { 10 }

/// Search filters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFilters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sources: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub types: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages: Option<Vec<String>>,
}

/// Search options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    #[serde(default)]
    pub include_graph: bool,
    #[serde(default = "default_graph_hops")]
    pub graph_hops: u32,
    #[serde(default)]
    pub rerank: bool,
}

fn default_graph_hops() -> u32 { 2 }

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub score: f64,
    pub source: SearchResultSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<SearchResultMetadata>,
}

/// Search result source info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultSource {
    pub id: String,
    #[serde(rename = "type")]
    pub source_type: String,
    pub path: String,
}

/// Search result metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entity_name: Option<String>,
}

/// Search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related_entities: Option<Vec<RelatedEntity>>,
    pub stats: SearchStats,
}

/// Related entity in search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    pub relationships: Vec<String>,
}

/// Search statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchStats {
    pub total_results: u64,
    pub search_time_ms: u64,
}

/// Entity with relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: String,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<EntitySource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationships: Option<EntityRelationships>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub documentation: Option<Vec<EntityDoc>>,
}

/// Entity source location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntitySource {
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_line: Option<u32>,
}

/// Entity relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRelationships {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub called_by: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calls: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contained_in: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contains: Option<Vec<String>>,
}

/// Entity documentation reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDoc {
    pub chunk_id: String,
    pub content: String,
    pub confidence: f64,
}
