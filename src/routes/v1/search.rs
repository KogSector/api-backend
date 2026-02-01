//! Search endpoints

use axum::{
    extract::{State, Extension},
    Json,
};
use reqwest::Client;
use std::time::SystemTime;
use serde_json::Value;

use crate::error::Result;
use crate::middleware::auth::AuthenticatedUser;
use crate::models::{SearchRequest, SearchResponse, SearchResult, SearchResultSource, SearchStats, RelatedEntity};
use crate::clients::relation_graph_client::{TemporalSearchData, Edge, Node};
use super::AppState;

/// Helper to check feature toggle
async fn is_toggle_enabled(base_url: &str, toggle_name: &str) -> bool {
    // In a real implementation, this should use a cached client
    // For now, we do a direct call with short timeout
    let client = Client::builder()
        .timeout(std::time::Duration::from_millis(200)) // Fast timeout
        .build()
        .unwrap_or_default();
        
    match client.get(format!("{}/api/toggles/{}", base_url, toggle_name)).send().await {
        Ok(res) => {
            if let Ok(json) = res.json::<Value>().await {
                json.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false)
            } else {
                false
            }
        },
        Err(_) => false,
    }
}

/// Helper to map Enhanced Graph response to legacy SearchResponse
fn map_enhanced_response(data: TemporalSearchData) -> SearchResponse {
    let start_time = SystemTime::now();
    
    // Map nodes to SearchResults
    let results: Vec<SearchResult> = data.nodes.into_iter().map(|node| {
        SearchResult {
            id: node.uuid,
            content: node.summary, // Use summary as content mock
            score: 1.0, // Placeholder
            source: SearchResultSource {
                id: "enhanced-graph".to_string(),
                source_type: "knowledge_graph".to_string(),
                path: node.name,
            },
            metadata: None,
        }
    }).collect();
    
    // Map edges to RelatedEntities
    let related: Vec<RelatedEntity> = data.edges.into_iter().map(|edge| {
        RelatedEntity {
            id: edge.uuid,
            entity_type: "relationship".to_string(),
            name: edge.fact,
            relationships: vec![],
        }
    }).collect();
    
    SearchResponse {
        results,
        related_entities: Some(related),
        stats: SearchStats {
            total_results: (data.node_count + data.edge_count) as u64,
            search_time_ms: start_time.elapsed().unwrap_or_default().as_millis() as u64,
        },
    }
}

/// POST /v1/search - Hybrid search (vector + graph)
pub async fn hybrid_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    // Check feature toggle
    let use_enhanced = is_toggle_enabled(&state.config.feature_toggle_url, "useEnhancedGraph").await;
    
    if use_enhanced {
        // Use new Enhanced Graph service with temporal search
        let response = state.enhanced_graph_client.search_simple(&request.query, request.limit).await?;
        
        if let Some(data) = response.data {
             return Ok(Json(map_enhanced_response(data)));
        }
        
        // Fallback or empty if no data
        Ok(Json(SearchResponse {
            results: vec![],
            related_entities: None,
            stats: SearchStats { total_results: 0, search_time_ms: 0 },
        }))
    } else {
        // Use legacy Relation Graph service
        let results = state.relation_graph_client
            .search(&request)
            .await?;
        
        Ok(Json(results))
    }
}

/// POST /v1/search/vector - Vector-only search
pub async fn vector_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    // Vector search is also handled by enhanced graph's temporal search (it does hybrid)
    // But specific vector-only might not be exposed directly in enhanced-graph yet used as such
    // For now, we route same way if enhanced
    
    let use_enhanced = is_toggle_enabled(&state.config.feature_toggle_url, "useEnhancedGraph").await;
    
    if use_enhanced {
        // Enhanced graph usually does hybrid, but we can use it
        let response = state.enhanced_graph_client.search_simple(&request.query, request.limit).await?;
        
        if let Some(data) = response.data {
             return Ok(Json(map_enhanced_response(data)));
        }
         Ok(Json(SearchResponse {
            results: vec![],
            related_entities: None,
            stats: SearchStats { total_results: 0, search_time_ms: 0 },
        }))
    } else {
        let results = state.relation_graph_client
            .search_vector(&request)
            .await?;
        
        Ok(Json(results))
    }
}

/// POST /v1/search/graph - Graph-only search
pub async fn graph_search(
    State(state): State<AppState>,
    Extension(_user): Extension<AuthenticatedUser>,
    Json(request): Json<SearchRequest>,
) -> Result<Json<SearchResponse>> {
    let use_enhanced = is_toggle_enabled(&state.config.feature_toggle_url, "useEnhancedGraph").await;
    
    if use_enhanced {
        // Enhanced graph is graph-first
        let response = state.enhanced_graph_client.search_simple(&request.query, request.limit).await?;
        
        if let Some(data) = response.data {
             return Ok(Json(map_enhanced_response(data)));
        }
         Ok(Json(SearchResponse {
            results: vec![],
            related_entities: None,
            stats: SearchStats { total_results: 0, search_time_ms: 0 },
        }))
    } else {
        let results = state.relation_graph_client
            .search_graph(&request)
            .await?;
        
        Ok(Json(results))
    }
}
