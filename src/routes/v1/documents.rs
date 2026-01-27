//! Document management routes

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use once_cell::sync::Lazy;

use super::AppState;

/// In-memory storage for documents (for development)
static DOC_STORE: Lazy<Arc<RwLock<Vec<DocumentRecord>>>> = Lazy::new(|| {
    Arc::new(RwLock::new(vec![
        DocumentRecord {
            id: "doc-001".to_string(),
            user_id: "user-rishabh-001".to_string(),
            name: "API Documentation".to_string(),
            doc_type: "markdown".to_string(),
            source: "upload".to_string(),
            size: "125 KB".to_string(),
            tags: vec!["api".to_string(), "docs".to_string()],
            status: "active".to_string(),
            created_at: "2026-01-20T10:00:00Z".to_string(),
            updated_at: "2026-01-25T14:30:00Z".to_string(),
        },
        DocumentRecord {
            id: "doc-002".to_string(),
            user_id: "user-rishabh-001".to_string(),
            name: "Architecture Overview".to_string(),
            doc_type: "pdf".to_string(),
            source: "google_drive".to_string(),
            size: "2.4 MB".to_string(),
            tags: vec!["architecture".to_string(), "design".to_string()],
            status: "active".to_string(),
            created_at: "2026-01-18T08:00:00Z".to_string(),
            updated_at: "2026-01-22T11:00:00Z".to_string(),
        },
    ]))
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRecord {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub doc_type: String,
    pub source: String,
    pub size: String,
    pub tags: Vec<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDocumentRequest {
    pub name: String,
    pub doc_type: String,
    pub source: String,
    pub size: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DocumentListResponse {
    pub data: Vec<DocumentRecord>,
    pub total: usize,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
}

/// List all documents
pub async fn list_documents(
    State(_state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Json<ApiResponse<DocumentListResponse>> {
    let store = DOC_STORE.read().await;
    
    let filtered: Vec<DocumentRecord> = if let Some(search) = query.search {
        store.iter()
            .filter(|d| d.name.to_lowercase().contains(&search.to_lowercase()))
            .cloned()
            .collect()
    } else {
        store.clone()
    };
    
    let total = filtered.len();
    Json(ApiResponse {
        success: true,
        message: "Documents retrieved successfully".to_string(),
        data: Some(DocumentListResponse { data: filtered, total }),
    })
}

/// Create a new document
pub async fn create_document(
    State(_state): State<AppState>,
    Json(payload): Json<CreateDocumentRequest>,
) -> (StatusCode, Json<ApiResponse<DocumentRecord>>) {
    let now = chrono::Utc::now().to_rfc3339();
    let doc = DocumentRecord {
        id: uuid::Uuid::new_v4().to_string(),
        user_id: "user-rishabh-001".to_string(),
        name: payload.name,
        doc_type: payload.doc_type,
        source: payload.source,
        size: payload.size.unwrap_or_else(|| "0 KB".to_string()),
        tags: payload.tags.unwrap_or_default(),
        status: "active".to_string(),
        created_at: now.clone(),
        updated_at: now,
    };

    let mut store = DOC_STORE.write().await;
    store.push(doc.clone());

    (StatusCode::CREATED, Json(ApiResponse {
        success: true,
        message: "Document created successfully".to_string(),
        data: Some(doc),
    }))
}

/// Delete a document
pub async fn delete_document(
    State(_state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut store = DOC_STORE.write().await;
    
    if let Some(pos) = store.iter().position(|d| d.id == id) {
        store.remove(pos);
        Ok(Json(ApiResponse {
            success: true,
            message: "Document deleted successfully".to_string(),
            data: None,
        }))
    } else {
        Err((StatusCode::NOT_FOUND, Json(ApiResponse {
            success: false,
            message: "Document not found".to_string(),
            data: None,
        })))
    }
}

/// Get document analytics
pub async fn get_analytics(
    State(_state): State<AppState>,
) -> Json<ApiResponse<serde_json::Value>> {
    Json(ApiResponse {
        success: true,
        message: "Analytics retrieved successfully".to_string(),
        data: Some(serde_json::json!({
            "total_documents": 12,
            "total_size_mb": 45.6,
            "by_type": {
                "pdf": 5,
                "markdown": 4,
                "docx": 3
            },
            "by_source": {
                "upload": 6,
                "google_drive": 4,
                "github": 2
            }
        })),
    })
}
