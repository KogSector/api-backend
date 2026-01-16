//! Base client utilities for service communication

use reqwest::Client;
use std::time::Duration;

use crate::error::AppError;

/// Create a configured HTTP client
pub fn create_http_client(timeout_secs: u64) -> Result<Client, AppError> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .connect_timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to create HTTP client: {}", e)))
}

/// Handle service call errors consistently
pub async fn handle_service_response<T: serde::de::DeserializeOwned>(
    response: reqwest::Response,
    service_name: &str,
) -> Result<T, AppError> {
    let status = response.status();
    
    if status.is_success() {
        response
            .json::<T>()
            .await
            .map_err(|e| AppError::Internal(format!("{} response parse error: {}", service_name, e)))
    } else if status == reqwest::StatusCode::UNAUTHORIZED {
        Err(AppError::Unauthorized("Authentication failed".to_string()))
    } else if status == reqwest::StatusCode::FORBIDDEN {
        Err(AppError::Forbidden("Access denied".to_string()))
    } else if status == reqwest::StatusCode::NOT_FOUND {
        let body = response.text().await.unwrap_or_default();
        Err(AppError::NotFound(body))
    } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
        Err(AppError::RateLimited)
    } else if status.is_server_error() {
        let body = response.text().await.unwrap_or_default();
        Err(AppError::ServiceUnavailable(format!("{} error: {}", service_name, body)))
    } else {
        let body = response.text().await.unwrap_or_default();
        Err(AppError::Internal(format!("{} error ({}): {}", service_name, status, body)))
    }
}
