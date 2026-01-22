//! ConFuse API Backend - Main Entry Point
//!
//! Central API Gateway for the ConFuse Knowledge Intelligence Platform

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tower_http::cors::{CorsLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use api_backend::{Config, AppError};
use api_backend::clients::{AuthClient, DataConnectorClient, RelationGraphClient, McpClient};
use api_backend::middleware::auth::AuthLayer;
use api_backend::routes::v1::{v1_router, AppState};
use api_backend::kafka::{EventProducer, ProducerConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    
    tracing::info!("Starting ConFuse API Backend...");
    
    // Load configuration
    let config = Config::from_env().map_err(|e| {
        tracing::error!("Failed to load config: {}", e);
        e
    })?;
    
    let config = Arc::new(config);
    tracing::info!("Configuration loaded, port: {}", config.port);
    
    // Initialize service clients
    let auth_client = AuthClient::new(&config.auth_middleware_url)?;
    let data_connector_client = DataConnectorClient::new(&config.data_connector_url)?;
    let relation_graph_client = RelationGraphClient::new(&config.relation_graph_url)?;
    let mcp_client = McpClient::new(&config.mcp_server_url)?;
    
    tracing::info!("Service clients initialized");
    
    // Initialize Kafka event producer (optional - graceful fallback to HTTP)
    let event_producer = match EventProducer::new(ProducerConfig::from_env()) {
        Ok(producer) => {
            tracing::info!("✅ Kafka event producer initialized");
            Some(Arc::new(producer))
        }
        Err(e) => {
            tracing::warn!("⚠️  Kafka unavailable, falling back to HTTP: {}", e);
            None
        }
    };
    
    // Check if auth bypass is enabled via feature toggle (development only)
    // In production, this would check the feature-toggle service
    let auth_bypass_enabled = std::env::var("AUTH_BYPASS_ENABLED")
        .map(|v| v == "true")
        .unwrap_or(false);
    
    if auth_bypass_enabled {
        tracing::warn!("⚠️  AUTH BYPASS ENABLED - Development mode only!");
    }
    
    // Create auth layer
    let auth_layer = AuthLayer::new(auth_client.clone(), auth_bypass_enabled);
    
    // Create application state
    let state = AppState {
        config: config.clone(),
        auth_client: Arc::new(auth_client),
        data_connector_client: Arc::new(data_connector_client),
        relation_graph_client: Arc::new(relation_graph_client),
        mcp_client: Arc::new(mcp_client),
        auth_layer,
        event_producer,
    };
    
    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any) // Will be configured properly in production
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router
    let app = v1_router(state)
        .layer(TraceLayer::new_for_http())
        .layer(cors);
    
    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Listening on http://{}", addr);
    
    println!();
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║          🚀 ConFuse API Backend (Rust/Axum)              ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  🌐 Server: http://0.0.0.0:{}                          ║", config.port);
    println!("║  📊 Health: http://localhost:{}/health                 ║", config.port);
    println!("║  📋 Status: http://localhost:{}/status                 ║", config.port);
    println!("║  📚 API:    http://localhost:{}/v1                     ║", config.port);
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}
