//! AI Agent management routes

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use super::AppState;

/// In-memory storage for agents (for development)
static AGENT_STORE: Lazy<Arc<RwLock<Vec<AgentRecord>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(vec![
        AgentRecord {
            id: "agent-001".to_string(),
            user_id: "user-rishabh-001".to_string(),
            name: "GitHub Copilot".to_string(),
            agent_type: "copilot".to_string(),
            endpoint: None,
            api_key: "sk-***hidden***".to_string(),
            permissions: vec!["read".to_string(), "context".to_string()],
            status: "Connected".to_string(),
            config: AgentConfig {
                model: Some("gpt-4".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(4096),
                timeout: Some(30),
                custom_instructions: None,
            },
            usage_stats: AgentUsageStats {
                total_requests: 1247,
                total_tokens: 45000,
                avg_response_time: Some(1.2),
                last_error: None,
            },
            created_at: "2026-01-10T08:00:00Z".to_string(),
            updated_at: "2026-01-27T10:00:00Z".to_string(),
            last_used: Some("2026-01-27T11:30:00Z".to_string()),
        },
        AgentRecord {
            id: "agent-002".to_string(),
            user_id: "user-rishabh-001".to_string(),
            name: "Amazon Q".to_string(),
            agent_type: "amazon_q".to_string(),
            endpoint: None,
            api_key: "amz-***hidden***".to_string(),
            permissions: vec!["read".to_string(), "context".to_string(), "write".to_string()],
            status: "Connected".to_string(),
            config: AgentConfig {
                model: None,
                temperature: None,
                max_tokens: None,
                timeout: Some(60),
                custom_instructions: None,
            },
            usage_stats: AgentUsageStats {
                total_requests: 892,
                total_tokens: 32000,
                avg_response_time: Some(0.9),
                last_error: None,
            },
            created_at: "2026-01-12T10:00:00Z".to_string(),
            updated_at: "2026-01-26T15:00:00Z".to_string(),
            last_used: Some("2026-01-27T09:45:00Z".to_string()),
        },
    ]))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub agent_type: String,
    pub endpoint: Option<String>,
    pub api_key: String,
    pub permissions: Vec<String>,
    pub status: String,
    pub config: AgentConfig,
    pub usage_stats: AgentUsageStats,
    pub created_at: String,
    pub updated_at: String,
    pub last_used: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub timeout: Option<u32>,
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUsageStats {
    pub total_requests: u32,
    pub total_tokens: u32,
    pub avg_response_time: Option<f64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub agent_type: String,
    pub endpoint: Option<String>,
    pub api_key: String,
    pub permissions: Vec<String>,
    pub config: AgentConfig,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub endpoint: Option<String>,
    pub api_key: Option<String>,
    pub permissions: Option<Vec<String>>,
    pub config: Option<AgentConfig>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AgentInvokeRequest {
    pub message: String,
    pub context_type: Option<String>,
    pub include_history: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct AgentInvokeResponse {
    pub response: String,
    pub usage: InvokeUsage,
    pub context_used: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InvokeUsage {
    pub tokens_used: u32,
    pub response_time_ms: u32,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// List all agents
pub async fn list_agents(
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<AgentRecord>>> {
    let store = AGENT_STORE.read().await;
    Json(ApiResponse {
        success: true,
        message: "Agents retrieved successfully".to_string(),
        data: Some(store.clone()),
    })
}

/// Get a specific agent
pub async fn get_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<AgentRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = AGENT_STORE.read().await;
    
    if let Some(agent) = store.iter().find(|a| a.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent retrieved successfully".to_string(),
            data: Some(agent.clone()),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}

/// Create a new agent
pub async fn create_agent(
    State(_state): State<AppState>,
    Json(payload): Json<CreateAgentRequest>,
) -> (StatusCode, Json<ApiResponse<AgentRecord>>) {
    let now = chrono::Utc::now().to_rfc3339();
    let agent = AgentRecord {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: "user-rishabh-001".to_string(),
        name: payload.name,
        agent_type: payload.agent_type,
        endpoint: payload.endpoint,
        api_key: payload.api_key,
        permissions: payload.permissions,
        status: "Pending".to_string(),
        config: payload.config,
        usage_stats: AgentUsageStats {
            total_requests: 0,
            total_tokens: 0,
            avg_response_time: None,
            last_error: None,
        },
        created_at: now.clone(),
        updated_at: now,
        last_used: None,
    };

    let mut store = AGENT_STORE.write().await;
    store.push(agent.clone());

    (StatusCode::CREATED, Json(ApiResponse {
        success: true,
        message: "Agent created successfully".to_string(),
        data: Some(agent),
    }))
}

/// Update an agent
pub async fn update_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<UpdateAgentRequest>,
) -> Result<Json<ApiResponse<AgentRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut store = AGENT_STORE.write().await;
    
    if let Some(agent) = store.iter_mut().find(|a| a.id == id) {
        if let Some(name) = payload.name {
            agent.name = name;
        }
        if let Some(endpoint) = payload.endpoint {
            agent.endpoint = Some(endpoint);
        }
        if let Some(api_key) = payload.api_key {
            agent.api_key = api_key;
        }
        if let Some(permissions) = payload.permissions {
            agent.permissions = permissions;
        }
        if let Some(config) = payload.config {
            agent.config = config;
        }
        if let Some(status) = payload.status {
            agent.status = status;
        }
        agent.updated_at = chrono::Utc::now().to_rfc3339();
        
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent updated successfully".to_string(),
            data: Some(agent.clone()),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}

/// Delete an agent
pub async fn delete_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut store = AGENT_STORE.write().await;
    
    if let Some(pos) = store.iter().position(|a| a.id == id) {
        store.remove(pos);
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent deleted successfully".to_string(),
            data: None,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}

/// Test agent connection
pub async fn test_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = AGENT_STORE.read().await;
    
    if store.iter().any(|a| a.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent connection test successful".to_string(),
            data: Some(serde_json::json!({ "connected": true })),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}

/// Invoke an agent
pub async fn invoke_agent(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(_payload): Json<AgentInvokeRequest>,
) -> Result<Json<ApiResponse<AgentInvokeResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = AGENT_STORE.read().await;
    
    if store.iter().any(|a| a.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent invoked successfully".to_string(),
            data: Some(AgentInvokeResponse {
                response: "This is a mock response from the AI agent. In production, this would connect to the actual AI service.".to_string(),
                usage: InvokeUsage {
                    tokens_used: 150,
                    response_time_ms: 850,
                },
                context_used: vec!["repo:frontend-app".to_string(), "doc:API Documentation".to_string()],
            }),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}

/// Get agent context
pub async fn get_agent_context(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = AGENT_STORE.read().await;
    
    if store.iter().any(|a| a.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "Agent context retrieved successfully".to_string(),
            data: Some(serde_json::json!({
                "repositories": ["frontend-app", "api-backend"],
                "documents": ["API Documentation", "Architecture Overview"],
                "urls": ["https://docs.confuse.dev"],
                "total_tokens": 15000
            })),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Agent not found".to_string(),
            data: None,
        })))
    }
}
