//! Configuration module for API Backend
//!
//! Loads configuration from environment variables

use std::env;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    // Server
    pub port: u16,
    
    // Database
    pub database_url: String,
    
    // JWT
    pub jwt_secret: String,
    
    // Service URLs
    pub auth_middleware_url: String,
    pub data_connector_url: String,
    pub relation_graph_url: String,
    pub mcp_server_url: String,
    pub feature_toggle_url: String,
    pub unified_processor_url: String,
    pub enhanced_graph_url: String,  // Added for new graph service
    
    // CORS
    pub cors_origins: Vec<String>,
    
    // Rate limiting
    pub rate_limit_default: u32,
    pub rate_limit_search: u32,
    pub rate_limit_sources: u32,
    pub rate_limit_sync: u32,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        
        Ok(Self {
            port: env::var("PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .unwrap_or(8000),
            
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".to_string()))?,
            
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnv("JWT_SECRET".to_string()))?,
            
            auth_middleware_url: env::var("AUTH_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("AUTH_SERVICE_URL".to_string()))?,
            
            data_connector_url: env::var("DATA_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("DATA_SERVICE_URL".to_string()))?,
            
            relation_graph_url: env::var("RELATION_GRAPH_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("RELATION_GRAPH_SERVICE_URL".to_string()))?,
            
            mcp_server_url: env::var("MCP_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("MCP_SERVICE_URL".to_string()))?,
            
            feature_toggle_url: env::var("FEATURE_TOGGLE_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("FEATURE_TOGGLE_SERVICE_URL".to_string()))?,
            
            unified_processor_url: env::var("UNIFIED_PROCESSOR_SERVICE_URL")
                .map_err(|_| ConfigError::MissingEnv("UNIFIED_PROCESSOR_SERVICE_URL".to_string()))?,
                
            enhanced_graph_url: env::var("ENHANCED_GRAPH_URL")
                .map_err(|_| ConfigError::MissingEnv("ENHANCED_GRAPH_URL".to_string()))?,
            
            cors_origins: env::var("CORS_ORIGINS")
                .unwrap_or_else(|_| "http://localhost:3000".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            
            rate_limit_default: env::var("RATE_LIMIT_DEFAULT")
                .unwrap_or_else(|_| "120".to_string())
                .parse()
                .unwrap_or(120),
            
            rate_limit_search: env::var("RATE_LIMIT_SEARCH")
                .unwrap_or_else(|_| "60".to_string())
                .parse()
                .unwrap_or(60),
            
            rate_limit_sources: env::var("RATE_LIMIT_SOURCES")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            
            rate_limit_sync: env::var("RATE_LIMIT_SYNC")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        })
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing required environment variable: {0}")]
    MissingEnv(String),
    
    #[error("Invalid value for environment variable: {0}")]
    InvalidValue(String),
}
