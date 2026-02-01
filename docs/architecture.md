# ConFuse API Backend Architecture

## System Architecture

ConFuse is a **Knowledge Intelligence Platform** that enables AI agents to access, understand, and reason over your codebase and documents.

The API Backend serves as the central gateway (Port: 8088) that orchestrates all ConFuse microservices.

### Connectivity Infrastructure

The API Backend now integrates the **ConFuse Connectivity Infrastructure** (`confuse-connectivity`), a comprehensive shared library providing:

- **Service Discovery**: Consul-based service registration and discovery with health checks
- **Circuit Breakers**: Automatic failure detection and recovery to prevent cascading failures
- **Distributed Caching**: Redis-based caching with cache-aside, write-through patterns
- **Distributed Tracing**: OpenTelemetry integration with Jaeger for end-to-end request tracing
- **Health Monitoring**: Comprehensive health check framework for dependencies
- **Resilience Patterns**: Exponential backoff retry logic with configurable policies
- **HTTP Client**: Type-safe service client with built-in resilience and tracing

This replaces the previous `failsafe` circuit breaker library with a more comprehensive, production-ready solution designed specifically for the ConFuse platform's microservices architecture.

---

## Service Descriptions

### 1. API Backend (This Service)
**Port: 8088** | **Language: Rust (Axum)**

Central API gateway that orchestrates all ConFuse microservices. Provides:
- RESTful API endpoints for external clients
- Service-to-service communication via HTTP clients
- Circuit breaker and retry logic for resilience
- Distributed tracing and observability
- Health monitoring and service discovery integration

**Service Clients:**
- `AuthClient` - Authentication service integration
- `DataConnectorClient` - Source integration and data ingestion
- `RelationGraphClient` - Knowledge graph operations with temporal search capabilities
- `McpClient` - AI agent protocol communication
- `UnifiedProcessorClient` - Hybrid document/code processing

**Note:** The service client was renamed from `EnhancedGraphClient` to `RelationGraphClient` to align with the actual service name. The client provides access to both legacy relation graph operations and enhanced temporal search features via feature toggles.

### 9. Client Connector
**Port: 8095** | **Language: Python**

Agent gateway:
- WebSocket/SSE transport
- Multi-agent support
- Session management
- Rate limiting

### 10. Frontend
**Port: 3000** | **Language: React/Next.js**

Web interface:
- Source connection management
- Search interface
- Settings and configuration
- Analytics dashboard

---

## Data Flow

### 1. Data Ingestion Flow

```
User connects GitHub repo
         │
         ▼
┌─────────────────┐
│  Data Connector │ ◄─── Webhook events
│    (Python)     │
└────────┬────────┘
         │ file list + tokens
         ▼
┌─────────────────┐
│ Code Normalize  │
│  Fetch (Rust)   │
└────────┬────────┘
         │ normalized files + entities
         ▼
┌─────────────────┐
│    Chunker      │
│    (Rust)       │
└────────┬────────┘
         │ chunks
         ▼
┌─────────────────┐
│   Embeddings    │
│    (Rust)       │
└────────┬────────┘
         │ vectors
         ▼
┌─────────────────┐
│ Relation Graph  │
│    (Rust)       │
└─────────────────┘
         │
    Stored in:
    • Neo4j (graph)
    • Zilliz (vectors)
    • PostgreSQL (metadata)
```

### 2. Query Flow (AI Agent)

```
AI Agent asks: "How does authentication work?"
         │
         ▼
┌─────────────────┐
│Client Connector │
│   (Python)      │
└────────┬────────┘
         │ MCP request
         ▼
┌─────────────────┐
│   MCP Server    │
│    (Rust)       │
└────────┬────────┘
         │ tool call: hybrid_search
         ▼
┌─────────────────┐
│ Relation Graph  │
│    (Rust)       │
└────────┬────────┘
         │
    ┌────┴────┐
    ▼         ▼
┌───────┐ ┌───────┐
│Zilliz │ │ Neo4j │
│vectors│ │ graph │
└───┬───┘ └───┬───┘
    │         │
    └────┬────┘
         │ combined results
         ▼
    Context returned to Agent
         │
         ▼
    Agent generates response
```

---

## Port Summary

| Service | Port | Protocol | Language |
|---------|------|----------|----------|
| frontend | 3000 | HTTP | TypeScript |
| auth-middleware | 3010 | HTTP | TypeScript |
| feature-toggle | 3099 | HTTP | TypeScript |
| data-connector | 8080 | HTTP | Python |
| api-backend | 8088 | HTTP | Rust |
| mcp-server | 3004 | stdio/HTTP | Rust |
| embeddings | 3001 | HTTP | Rust |
| relation-graph | 3003 | HTTP | Python |
| code-normalize-fetch | 8090 | HTTP | Rust |
| client-connector | 3020 | WS/HTTP | Python |

---

## Database Dependencies

| Database | Purpose | Used By |
|----------|---------|---------|
| PostgreSQL | User data, metadata, job queue | api-backend, data-connector, auth |
| Neo4j | Knowledge graph relationships | relation-graph |
| Zilliz/Milvus | Vector embeddings | relation-graph |
| Redis | Caching, sessions, rate limiting, circuit breaker state | api-backend, auth, connectivity |
| Consul | Service discovery, health checks | All services via connectivity |

---

## Resilience Architecture

The API Backend implements comprehensive resilience patterns through the connectivity infrastructure:

```
Request Flow with Resilience:
         │
         ▼
┌─────────────────────┐
│  Circuit Breaker    │ ◄─── Monitors failure rates
│  (Per Service)      │      Opens on threshold
└──────────┬──────────┘
           │ If Closed/Half-Open
           ▼
┌─────────────────────┐
│  Retry Logic        │ ◄─── Exponential backoff
│  (Configurable)     │      Max 3 attempts
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Service Client     │ ◄─── HTTP with timeout
│  (Type-safe)        │      Distributed tracing
└──────────┬──────────┘
           │
           ▼
    Downstream Service
```

**Circuit Breaker States:**
- **Closed**: Normal operation, requests flow through
- **Open**: Service failing, requests rejected immediately (30s timeout)
- **Half-Open**: Testing recovery, limited requests allowed (5 max)

**Failure Handling:**
- Automatic retry with exponential backoff (100ms → 200ms → 400ms)
- Circuit breaker trips at 50% failure rate (minimum 10 calls)
- Health checks every 10 seconds via Consul
- Distributed tracing for debugging failures

---

## Security Architecture

```
Layer 1: Network Isolation (VPC)
         │
Layer 2: API Gateway (api-backend)
         │ - Request validation
         │ - Rate limiting
         │
Layer 3: Authentication (auth-middleware)
         │ - JWT validation
         │ - OAuth flows
         │
Layer 4: Authorization (per-service)
         │ - Role-based access
         │ - Resource ownership
         │
Layer 5: Data Access (row-level security)
         │ - Tenant isolation
         │ - Query filtering
         │
Layer 6: Audit Logging
           - All operations logged
           - Compliance ready
```

---

## Deployment

See [deployment.md](deployment.md) for detailed deployment instructions.

### Quick Start (Development)

```bash
# Start all services with Docker Compose
docker-compose up -d

# Or start individually
cd api-backend && cargo run
# Or with hot-reload (install cargo-watch):
# cargo watch -x 'run'
Start the `auth-middleware` service as described in its repository README (../auth-middleware/README.md).
cd data-connector && uvicorn app:app --reload
# ... etc
```

### Production

- Kubernetes manifests in each service's `k8s/` folder
- Helm charts available in `infrastructure/` repo
- CI/CD via GitHub Actions
