fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "../auth-middleware/proto/auth.proto",
        "../relation-graph/proto/graph.proto",
        "../unified-processor/proto/processor.proto",
        "../embeddings-service/proto/embeddings.proto",
        "../mcp-server/proto/mcp.proto",
        "../data-connector/proto/connector.proto",
        "../client-connector/proto/client.proto",
    ];

    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .compile(protos, &["../"])?;

    // Re-run if proto files change
    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    Ok(())
}
