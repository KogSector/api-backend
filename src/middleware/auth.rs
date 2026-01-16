//! Authentication middleware
//!
//! Handles JWT Bearer token and API key authentication

use axum::{
    extract::{Request, State},
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;

use crate::clients::AuthClient;
use crate::error::AppError;
use crate::models::User;

/// Extension type for authenticated user
#[derive(Clone)]
pub struct AuthenticatedUser(pub User);

/// Authentication layer configuration
#[derive(Clone)]
pub struct AuthLayer {
    pub auth_client: Arc<AuthClient>,
    pub auth_bypass_enabled: bool,
}

impl AuthLayer {
    pub fn new(auth_client: AuthClient, auth_bypass_enabled: bool) -> Self {
        Self {
            auth_client: Arc::new(auth_client),
            auth_bypass_enabled,
        }
    }
}

/// Demo user for auth bypass in development
fn demo_user() -> User {
    User {
        id: "demo-user-001".to_string(),
        email: "demo@confuse.dev".to_string(),
        name: Some("Demo User".to_string()),
        picture: None,
        roles: vec!["user".to_string()],
    }
}

/// Authentication middleware function
pub async fn auth_middleware(
    State(auth_layer): State<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    // Check for auth bypass (development only)
    if auth_layer.auth_bypass_enabled {
        tracing::debug!("Auth bypass enabled, using demo user");
        request.extensions_mut().insert(AuthenticatedUser(demo_user()));
        return Ok(next.run(request).await);
    }
    
    // Try to extract authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    let api_key = request
        .headers()
        .get("X-API-Key")
        .and_then(|v| v.to_str().ok());
    
    let user = if let Some(auth_value) = auth_header {
        // Bearer token authentication
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            auth_layer.auth_client.verify_token(token).await?
        } else {
            return Err(AppError::Unauthorized("Invalid authorization header format".to_string()));
        }
    } else if let Some(key) = api_key {
        // API key authentication
        let api_key_info = auth_layer.auth_client.validate_api_key(key).await?;
        // Convert API key info to user
        User {
            id: api_key_info.user_id,
            email: format!("api-key-{}@confuse.dev", api_key_info.id),
            name: Some(api_key_info.name),
            picture: None,
            roles: api_key_info.scopes,
        }
    } else {
        return Err(AppError::Unauthorized("No authentication provided".to_string()));
    };
    
    // Attach user to request extensions
    request.extensions_mut().insert(AuthenticatedUser(user));
    
    Ok(next.run(request).await)
}

/// Optional authentication - doesn't fail if no auth provided
pub async fn optional_auth_middleware(
    State(auth_layer): State<AuthLayer>,
    mut request: Request,
    next: Next,
) -> Response {
    // Check for auth bypass
    if auth_layer.auth_bypass_enabled {
        request.extensions_mut().insert(AuthenticatedUser(demo_user()));
        return next.run(request).await;
    }
    
    // Try to extract and validate authorization
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok());
    
    if let Some(auth_value) = auth_header {
        if let Some(token) = auth_value.strip_prefix("Bearer ") {
            if let Ok(user) = auth_layer.auth_client.verify_token(token).await {
                request.extensions_mut().insert(AuthenticatedUser(user));
            }
        }
    }
    
    next.run(request).await
}

/// Extract authenticated user from request
pub fn get_user(request: &Request) -> Option<User> {
    request.extensions().get::<AuthenticatedUser>().map(|u| u.0.clone())
}
