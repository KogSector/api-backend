//! User-related models

use serde::{Deserialize, Serialize};

/// Authenticated user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    pub roles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace_id: Option<String>,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyInfo {
    pub id: String,
    pub user_id: String,
    pub name: String,
    pub scopes: Vec<String>,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
}

/// Token pair for auth refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub token_type: String,
}

/// User context for cross-service propagation
/// Contains user identity and active workspace for multi-tenant data isolation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub workspace_id: String,
    pub roles: Vec<String>,
}

impl UserContext {
    /// Create UserContext from User with workspace
    pub fn from_user(user: &User, workspace_id: String) -> Self {
        Self {
            user_id: user.id.clone(),
            email: user.email.clone(),
            workspace_id,
            roles: user.roles.clone(),
        }
    }
    
    /// Get header name for user ID
    pub const HEADER_USER_ID: &'static str = "X-User-Id";
    /// Get header name for workspace ID
    pub const HEADER_WORKSPACE_ID: &'static str = "X-Workspace-Id";
    /// Get header name for user email
    pub const HEADER_USER_EMAIL: &'static str = "X-User-Email";
}

