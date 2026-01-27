//! Repository management routes

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

/// In-memory storage for repositories (for development)
static REPO_STORE: Lazy<Arc<RwLock<Vec<RepositoryRecord>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(vec![
        RepositoryRecord {
            id: "repo-001".to_string(),
            name: "frontend-app".to_string(),
            provider: "github".to_string(),
            url: "https://github.com/confuse/frontend-app".to_string(),
            branch: "main".to_string(),
            status: "active".to_string(),
            last_sync: Some("2026-01-27T10:00:00Z".to_string()),
            files_indexed: 156,
            created_at: "2026-01-15T08:00:00Z".to_string(),
        },
        RepositoryRecord {
            id: "repo-002".to_string(),
            name: "api-backend".to_string(),
            provider: "github".to_string(),
            url: "https://github.com/confuse/api-backend".to_string(),
            branch: "main".to_string(),
            status: "active".to_string(),
            last_sync: Some("2026-01-27T09:30:00Z".to_string()),
            files_indexed: 89,
            created_at: "2026-01-10T12:00:00Z".to_string(),
        },
    ]))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRecord {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub url: String,
    pub branch: String,
    pub status: String,
    pub last_sync: Option<String>,
    pub files_indexed: u32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateRepositoryRequest {
    pub name: String,
    pub provider: String,
    pub url: String,
    pub branch: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// List all repositories
pub async fn list_repositories(
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<RepositoryRecord>>> {
    let store = REPO_STORE.read().await;
    Json(ApiResponse {
        success: true,
        message: "Repositories retrieved successfully".to_string(),
        data: Some(store.clone()),
    })
}

/// Create a new repository
pub async fn create_repository(
    State(_state): State<AppState>,
    Json(payload): Json<CreateRepositoryRequest>,
) -> (StatusCode, Json<ApiResponse<RepositoryRecord>>) {
    let now = chrono::Utc::now().to_rfc3339();
    let repo = RepositoryRecord {
        id: uuid::Uuid::new_v4().to_string(),
        name: payload.name,
        provider: payload.provider,
        url: payload.url,
        branch: payload.branch.unwrap_or_else(|| "main".to_string()),
        status: "pending".to_string(),
        last_sync: None,
        files_indexed: 0,
        created_at: now,
    };

    let mut store = REPO_STORE.write().await;
    store.push(repo.clone());

    (StatusCode::CREATED, Json(ApiResponse {
        success: true,
        message: "Repository created successfully".to_string(),
        data: Some(repo),
    }))
}

/// Get a specific repository
pub async fn get_repository(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<RepositoryRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = REPO_STORE.read().await;
    
    if let Some(repo) = store.iter().find(|r| r.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "Repository retrieved successfully".to_string(),
            data: Some(repo.clone()),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
        })))
    }
}

/// Delete a repository
pub async fn delete_repository(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut store = REPO_STORE.write().await;
    
    if let Some(pos) = store.iter().position(|r| r.id == id) {
        store.remove(pos);
        Ok(Json(ApiResponse {
            success: true,
            message: "Repository deleted successfully".to_string(),
            data: None,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Repository not found".to_string(),
            data: None,
        })))
    }
}
