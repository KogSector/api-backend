//! URL management routes

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

/// In-memory storage for URLs (for development)
static URL_STORE: Lazy<Arc<RwLock<Vec<UrlRecord>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(Vec::new()))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlRecord {
    pub id: String,
    pub url: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateUrlRequest {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// List all URLs
pub async fn list_urls(
    State(_state): State<AppState>,
) -> Json<ApiResponse<Vec<UrlRecord>>> {
    let store = URL_STORE.read().await;
    Json(ApiResponse {
        success: true,
        message: "URLs retrieved successfully".to_string(),
        data: Some(store.clone()),
    })
}

/// Create a new URL
pub async fn create_url(
    State(_state): State<AppState>,
    Json(payload): Json<CreateUrlRequest>,
) -> (StatusCode, Json<ApiResponse<UrlRecord>>) {
    let now = chrono::Utc::now().to_rfc3339();
    let url_record = UrlRecord {
        id: uuid::Uuid::new_v4().to_string(),
        url: payload.url.clone(),
        title: payload.title.unwrap_or_else(|| payload.url.clone()),
        description: payload.description,
        tags: payload.tags.unwrap_or_default(),
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut store = URL_STORE.write().await;
    store.push(url_record.clone());

    (StatusCode::CREATED, Json(ApiResponse {
        success: true,
        message: "URL created successfully".to_string(),
        data: Some(url_record),
    }))
}

/// Get a specific URL by ID
pub async fn get_url(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<UrlRecord>>, (StatusCode, Json<ApiResponse<()>>)> {
    let store = URL_STORE.read().await;
    
    if let Some(url) = store.iter().find(|u| u.id == id) {
        Ok(Json(ApiResponse {
            success: true,
            message: "URL retrieved successfully".to_string(),
            data: Some(url.clone()),
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "URL not found".to_string(),
            data: None,
        })))
    }
}

/// Delete a URL by ID
pub async fn delete_url(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut store = URL_STORE.write().await;
    
    if let Some(pos) = store.iter().position(|u| u.id == id) {
        store.remove(pos);
        Ok(Json(ApiResponse {
            success: true,
            message: "URL deleted successfully".to_string(),
            data: None,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "URL not found".to_string(),
            data: None,
        })))
    }
}
