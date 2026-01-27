# ConFuse API Backend

> Central API Gateway for the ConFuse Knowledge Intelligence Platform

## What is this service?

The **api-backend** is the central nervous system of ConFuse. It's the main entry point for all client applications (web frontend, mobile apps, third-party integrations) and orchestrates requests across all microservices.

# Quick Start

```bash
# Clone the repository
git clone https://github.com/confuse/api-backend.git
cd api-backend

# Build the service
cargo build

# Configure environment
cp .env.example .env
# Edit .env with your database and service URLs

# Run database migrations (example using sqlx)
# Install sqlx-cli: `cargo install sqlx-cli --no-default-features --features postgres`
sqlx migrate run

# Start development server (hot reload)
# Install cargo-watch: `cargo install cargo-watch`
cargo watch -x 'run'

# Run tests
cargo test
```

The server starts at `http://localhost:8088`.

## Documentation

| Document | Description |
|----------|-------------|
| [Architecture](architecture.md) | System design and service interactions |
| [API Reference](api-reference.md) | Complete REST API documentation |
| [Configuration](configuration.md) | Environment variables and settings |
| [Integration](integration.md) | How this service connects to others |
| [Development](development.md) | Local development setup |
| [Deployment](deployment.md) | Production deployment guide |
| [Troubleshooting](troubleshooting.md) | Common issues and solutions |

## How It Fits in ConFuse

```
                                    ┌─────────────────┐
                                    │    Frontend     │
                                    │    (React)      │
                                    └────────┬────────┘
                                             │
                                             ▼
┌────────────────────────────────────────────────────────────────────┐
│                        API-BACKEND (This Service)                   │
│                              Port: 8088                             │
│                                                                     │
│  • REST API for all client applications                            │
│  • Request routing to microservices                                │
│  • Authentication enforcement                                       │
│  • Rate limiting and request validation                            │
│  • API versioning (/v1, /v2)                                       │
└────────────────────────────────────────────────────────────────────┘
         │              │              │              │
         ▼              ▼              ▼              ▼
   ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐
   │   Auth   │  │   Data   │  │ Relation │  │   MCP    │
   │Middleware│  │Connector │  │  Graph   │  │  Server  │
   └──────────┘  └──────────┘  └──────────┘  └──────────┘
```

## Key Responsibilities

1. **API Gateway**: Single entry point for all client requests
2. **Authentication**: Validates JWT tokens via auth-middleware
3. **Authorization**: Enforces role-based access control
4. **Rate Limiting**: Protects backend services from abuse
5. **Request Routing**: Directs requests to appropriate microservices
6. **Response Aggregation**: Combines data from multiple services
7. **Error Handling**: Standardized error responses
8. **Logging**: Request/response logging for debugging

## Technology Stack

| Technology | Purpose |
|------------|---------|
| Rust | Runtime / language |
| Axum | Web framework |
| SQLx / SeaORM | Database access (PostgreSQL) |
| PostgreSQL | Database |
| Redis | Caching, rate limiting |
| Tokio | Async runtime |
| Cargo | Build & dependency manager |
| Rust tests | Testing (`cargo test`) |

## Related Services

| Service | Port | Relationship |
|---------|------|--------------|
| auth-middleware | 3010 | JWT validation, user sessions |
| data-connector | 8081 | Source management, webhooks |
| relation-graph | 3018 | Knowledge search |
| mcp-server | 3004 | MCP tool calls |
| embeddings | 3005 | Vector operations |
| frontend | 3000 | Web UI (main consumer) |
| feature-toggle | 3099 | Feature flags, auth bypass |
| client-connector | 3020 | AI agent gateway |

## License

MIT - ConFuse Team
