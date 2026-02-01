//! Service clients for communicating with other microservices

pub mod base;
pub mod auth_client;
pub mod data_connector_client;
pub mod relation_graph_client;
pub mod mcp_client;
pub mod unified_processor_client;

pub use auth_client::AuthClient;
pub use data_connector_client::DataConnectorClient;
pub use relation_graph_client::RelationGraphClient;
pub use mcp_client::McpClient;
pub use unified_processor_client::UnifiedProcessorClient;

// Alias for backward compatibility
pub type EnhancedGraphClient = RelationGraphClient;
