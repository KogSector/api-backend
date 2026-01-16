//! Auth Middleware client for authentication/authorization

use reqwest::Client;

use crate::error::AppError;
use crate::models::{User, ApiKeyInfo, TokenPair};
use super::base::{create_http_client, handle_service_response};

/// Client for auth-middleware service
#[derive(Clone)]
pub struct AuthClient {
    client: Client,
    base_url: String,
}

impl AuthClient {
    /// Create a new auth client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(5)?, // 5 second timeout
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// Verify a JWT token
    pub async fn verify_token(&self, token: &str) -> Result<User, AppError> {
        let response = self.client
            .get(format!("{}/api/auth/verify", self.base_url))
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;
        
        handle_service_response(response, "auth-middleware").await
    }
    
    /// Validate an API key
    pub async fn validate_api_key(&self, api_key: &str) -> Result<ApiKeyInfo, AppError> {
        let response = self.client
            .post(format!("{}/api/auth/api-keys/validate", self.base_url))
            .json(&serde_json::json!({ "apiKey": api_key }))
            .send()
            .await?;
        
        handle_service_response(response, "auth-middleware").await
    }
    
    /// Refresh an access token
    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AppError> {
        let response = self.client
            .post(format!("{}/api/auth/refresh", self.base_url))
            .json(&serde_json::json!({ "refreshToken": refresh_token }))
            .send()
            .await?;
        
        handle_service_response(response, "auth-middleware").await
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
}
