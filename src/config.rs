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
    pub redis_url: String,
    
    // JWT
    pub jwt_secret: String,
    
    // Service URLs
    pub auth_middleware_url: String,
    pub data_connector_url: String,
    pub relation_graph_url: String,
    pub mcp_server_url: String,
    pub feature_toggle_url: String,
    
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
                .unwrap_or_else(|_| "8088".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("PORT".to_string()))?,
            
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnv("DATABASE_URL".to_string()))?,
            
            redis_url: env::var("REDIS_URL")
                .map_err(|_| ConfigError::MissingEnv("REDIS_URL".to_string()))?,
            
            jwt_secret: env::var("JWT_SECRET")
                .map_err(|_| ConfigError::MissingEnv("JWT_SECRET".to_string()))?,
            
            auth_middleware_url: env::var("AUTH_MIDDLEWARE_URL")
                .unwrap_or_else(|_| "http://localhost:3010".to_string()),
            
            data_connector_url: env::var("DATA_CONNECTOR_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            
            relation_graph_url: env::var("RELATION_GRAPH_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string()),
            
            mcp_server_url: env::var("MCP_SERVER_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            
            feature_toggle_url: env::var("FEATURE_TOGGLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3099".to_string()),
            
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
