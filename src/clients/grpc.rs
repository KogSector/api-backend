use tonic::transport::Channel;

pub mod auth {
    tonic::include_proto!("confuse.auth.v1");
}

pub mod graph {
    tonic::include_proto!("confuse.graph.v1");
}

pub mod processor {
    tonic::include_proto!("confuse.processor.v1");
}

pub mod embeddings {
    tonic::include_proto!("confuse.embeddings.v1");
}

pub mod mcp {
    tonic::include_proto!("confuse.mcp.v1");
}

pub mod connector {
    tonic::include_proto!("confuse.connector.v1");
}

pub mod client {
    tonic::include_proto!("confuse.client.v1");
}

// Client wrappers
#[derive(Clone)]
pub struct GrpcClients {
    pub auth: auth::auth_client::AuthClient<Channel>,
    pub graph: graph::relation_graph_client::RelationGraphClient<Channel>,
    pub processor: processor::unified_processor_client::UnifiedProcessorClient<Channel>,
    pub embeddings: embeddings::embeddings_client::EmbeddingsClient<Channel>,
    pub mcp: mcp::mcp_client::McpClient<Channel>,
    pub data_connector: connector::data_connector_client::DataConnectorClient<Channel>,
    pub client_connector: client::client_connector_client::ClientConnectorClient<Channel>,
}

impl GrpcClients {
    pub async fn connect(
        auth_url: String,
        graph_url: String,
        processor_url: String,
        embeddings_url: String,
        mcp_url: String,
        connector_url: String,
        client_url: String,
    ) -> Result<Self, tonic::transport::Error> {
        let auth = auth::auth_client::AuthClient::connect(auth_url).await?;
        let graph = graph::relation_graph_client::RelationGraphClient::connect(graph_url).await?;
        let processor = processor::unified_processor_client::UnifiedProcessorClient::connect(processor_url).await?;
        let embeddings = embeddings::embeddings_client::EmbeddingsClient::connect(embeddings_url).await?;
        let mcp = mcp::mcp_client::McpClient::connect(mcp_url).await?;
        let data_connector = connector::data_connector_client::DataConnectorClient::connect(connector_url).await?;
        let client_connector = client::client_connector_client::ClientConnectorClient::connect(client_url).await?;

        Ok(Self {
            auth,
            graph,
            processor,
            embeddings,
            mcp,
            data_connector,
            client_connector,
        })
    }
}
