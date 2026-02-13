use tonic::transport::Channel;

pub mod auth {
    tonic::include_proto!("auth");
}

pub mod graph {
    tonic::include_proto!("graph");
}

pub mod processor {
    tonic::include_proto!("processor");
}

pub mod embeddings {
    tonic::include_proto!("embeddings");
}

// Client wrappers
#[derive(Clone)]
pub struct GrpcClients {
    pub auth: auth::auth_service_client::AuthServiceClient<Channel>,
    pub graph: graph::graph_service_client::GraphServiceClient<Channel>,
    pub processor: processor::processor_service_client::ProcessorServiceClient<Channel>,
    pub embeddings: embeddings::embeddings_service_client::EmbeddingsServiceClient<Channel>,
}

impl GrpcClients {
    pub async fn connect(
        auth_url: String,
        graph_url: String,
        processor_url: String,
        embeddings_url: String,
    ) -> Result<Self, tonic::transport::Error> {
        let auth = auth::auth_service_client::AuthServiceClient::connect(auth_url).await?;
        let graph = graph::graph_service_client::GraphServiceClient::connect(graph_url).await?;
        let processor = processor::processor_service_client::ProcessorServiceClient::connect(processor_url).await?;
        let embeddings = embeddings::embeddings_service_client::EmbeddingsServiceClient::connect(embeddings_url).await?;

        Ok(Self {
            auth,
            graph,
            processor,
            embeddings,
        })
    }
}
