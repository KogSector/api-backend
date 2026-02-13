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
use api_backend::clients::{AuthClient, DataConnectorClient, RelationGraphClient, McpClient, UnifiedProcessorClient};
use api_backend::middleware::auth::AuthLayer;
use api_backend::middleware::circuit_breaker::{CircuitBreakerRegistry, CircuitBreakerConfig};
use api_backend::middleware::cache::{ResponseCache, CacheConfig};
use api_backend::middleware::security_headers::security_headers_middleware;
use api_backend::middleware::zero_trust::zero_trust_middleware;
use api_backend::routes::v1::{v1_router, AppState};
use confuse_common::events::{config::KafkaConfig, producer::EventProducer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    // Initialize logging
    let file_appender = tracing_appender::rolling::daily("logs", "api-backend.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,api_backend=debug".into()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .pretty()
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .json()
        )
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
    let unified_processor_client = UnifiedProcessorClient::new(&config.unified_processor_url)?;
    let enhanced_graph_client = api_backend::clients::EnhancedGraphClient::new(&config.enhanced_graph_url)?;
    
    tracing::info!("Service clients initialized (including unified-processor and enhanced-graph)");
    
    // Initialize Kafka event producer (optional - graceful fallback to HTTP)
    let kafka_enabled = std::env::var("KAFKA_ENABLED")
        .map(|v| v.to_lowercase() == "true")
        .unwrap_or(true);
    
    let event_producer = if kafka_enabled {
        match KafkaConfig::from_env() {
            Ok(kafka_config) => {
                match EventProducer::new(kafka_config) {
                    Ok(producer) => {
                        tracing::info!("✅ Kafka event producer initialized");
                        Some(Arc::new(producer))
                    },
                    Err(e) => {
                        tracing::warn!("⚠️  Kafka producer creation failed: {}", e);
                        None
                    }
                }
            },
            Err(e) => {
                tracing::warn!("⚠️  Kafka config invalid, disabled: {}", e);
                None
            }
        }
    } else {
        tracing::info!("ℹ️  Kafka disabled via KAFKA_ENABLED=false");
        None
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
    
    // Initialize circuit breaker registry
    let circuit_breaker = Arc::new(CircuitBreakerRegistry::new(CircuitBreakerConfig::default()));
    tracing::info!("Circuit breaker registry initialized");
    
    // Initialize response cache
    let response_cache = Arc::new(ResponseCache::new(CacheConfig::default()));
    tracing::info!("Response cache initialized");
    
    // Create application state
    let state = AppState {
        config: config.clone(),
        auth_client: Arc::new(auth_client),
        data_connector_client: Arc::new(data_connector_client),
        relation_graph_client: Arc::new(relation_graph_client),
        mcp_client: Arc::new(mcp_client),
        unified_processor_client: Arc::new(unified_processor_client),
        enhanced_graph_client: Arc::new(enhanced_graph_client),
        auth_layer,
        event_producer,
        circuit_breaker,
        response_cache,
    };
    
    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any) // Will be configured properly in production
        .allow_methods(Any)
        .allow_headers(Any);
    
    // Build router
    let app = v1_router(state)
        .layer(axum::middleware::from_fn(zero_trust_middleware))
        .layer(axum::middleware::from_fn(security_headers_middleware))
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
