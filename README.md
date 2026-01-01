# ConFuse API Backend

Central API Gateway for the ConFuse Knowledge Intelligence Platform.

## Overview

This is the **main API gateway** that orchestrates all ConFuse microservices. It provides:
- Unified REST API for frontend applications
- Request routing to appropriate microservices
- Authentication/Authorization enforcement
- Rate limiting and request validation
- API versioning

## Architecture

See [docs/architecture.md](docs/architecture.md) for complete system architecture.

## Quick Start

```bash
# Build the service
cargo build

# Configure
cp .env.example .env

# Run database migrations (example using sqlx)
# Install sqlx-cli if needed: cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run

# Start development server (hot reload optional)
cargo run

# Run tests
cargo test
```

## Documentation

All documentation is in the `docs/` folder:

- [Architecture Overview](docs/architecture.md)
- [API Reference](docs/api-reference.md)
- [Integration Guide](docs/integration.md)
- [Deployment Guide](docs/deployment.md)

## Related Services

| Service | Purpose | Port |
|---------|---------|------|
| auth-middleware | JWT/OAuth authentication | 3001 |
| data-connector | Data ingestion from sources | 8000 |
| mcp-server | MCP protocol for AI agents | 3004 |
| client-connector | Agent connection gateway | 8095 |
| embeddings | Vector generation | 3005 |
| relation-graph | Knowledge graph | 3018 |
| chunker | Text segmentation | 3002 |
| code-normalize-fetch | Code preprocessing | 8090 |
| frontend | Web UI | 3000 |

## License

MIT