# ConFuse API Backend

Central API Gateway for the ConFuse Knowledge Intelligence Platform.

## Overview

The API Backend is a **Rust/Axum** service that orchestrates all ConFuse microservices:

- **Unified REST API** for frontend applications
- **Request routing** to appropriate microservices
- **Authentication/Authorization** enforcement (JWT & API keys)
- **Rate limiting** (Redis-backed sliding window)
- **API versioning** (v1 endpoints)

## Quick Start

```bash
# Build
cargo build --release

# Run locally (requires .env file)
cargo run

# With Podman
podman build -t confuse/api-backend:latest .
podman run -p 8088:8088 --env-file .env confuse/api-backend:latest
```

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/health` | GET | Health check |
| `/status` | GET | Detailed service status |
| `/v1/sources` | GET/POST | Source management |
| `/v1/search` | POST | Hybrid search |
| `/v1/entities/:id` | GET | Entity details |
| `/v1/sync/:id` | POST | Trigger sync |
| `/v1/mcp/capabilities` | GET | MCP tools |

## Service Integration

| Service | Port | Purpose |
|---------|------|---------|
| auth-middleware | 3010 | JWT/OAuth authentication |
| data-connector | 8080 | Source management & sync |
| unified-processor | 8090 | Document & code processing |
| relation-graph | 3003 | Knowledge graph & search |
| mcp-server | 3004 | AI agent tools |
| embeddings-service | 3011 | Vector generation |
| feature-toggle | 3099 | Feature flags |

## Configuration

See `.env` for all configuration options.

## Documentation

- [API Reference](docs/api-reference.md)
- [Architecture](docs/architecture.md)
- [Development Guide](docs/development.md)