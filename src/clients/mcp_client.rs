//! MCP Server client for tool operations

use reqwest::Client;

use crate::error::AppError;
use crate::models::{McpCapabilities, McpToolResult};
use super::base::{create_http_client, handle_service_response};

/// Client for mcp-server service
#[derive(Clone)]
pub struct McpClient {
    client: Client,
    base_url: String,
}

impl McpClient {
    /// Create a new MCP client
    pub fn new(base_url: &str) -> Result<Self, AppError> {
        Ok(Self {
            client: create_http_client(30)?, // 30 second timeout for tool calls
            base_url: base_url.trim_end_matches('/').to_string(),
        })
    }
    
    /// List available tools
    pub async fn list_tools(&self) -> Result<McpCapabilities, AppError> {
        let response = self.client
            .get(format!("{}/tools", self.base_url))
            .send()
            .await?;
        
        handle_service_response(response, "mcp-server").await
    }
    
    /// Call a tool
    pub async fn call_tool(
        &self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<McpToolResult, AppError> {
        let response = self.client
            .post(format!("{}/tools/call", self.base_url))
            .json(&serde_json::json!({
                "name": name,
                "arguments": arguments
            }))
            .send()
            .await?;
        
        handle_service_response(response, "mcp-server").await
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
