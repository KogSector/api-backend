fn main() -> Result<(), Box<dyn std::error::Error>> {
    let protos = &[
        "../auth-middleware/proto/auth.proto",
        "../relation-graph/proto/graph.proto",
        "../unified-processor/proto/processor.proto",
        "../embeddings-service/proto/embeddings.proto",
    ];

    // Only compile if enabled
    if std::env::var("GRPC_ENABLED").unwrap_or_else(|_| "false".to_string()) == "true" {
        tonic_build::configure()
            .build_client(true)
            .build_server(false)
            .compile(protos, &["../"])?;
    } else {
        println!("cargo:warning=gRPC compilation skipped (GRPC_ENABLED!=true)");
    }
    
    // Re-run if proto files change
    for proto in protos {
        println!("cargo:rerun-if-changed={}", proto);
    }

    Ok(())
}
