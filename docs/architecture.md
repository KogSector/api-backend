# ConFuse Architecture Overview

## System Architecture

ConFuse is a **Knowledge Intelligence Platform** that connects AI agents to organizational knowledge across code repositories, documents, chat, and other data sources.

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              AI AGENTS                                       │
│         Cursor, Windsurf, Claude, ChatGPT, VS Code, Custom Agents           │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │ MCP Protocol (WebSocket/SSE)
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        CLIENT-CONNECTOR (Python)                             │
│                              Port: 8095                                      │
│   • WebSocket/SSE Transport  • JWT/API Key Auth  • Session Management       │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │ subprocess (stdio) / HTTP
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MCP-SERVER (Rust)                                   │
│                              Port: 3004                                      │
│   • MCP Protocol (JSON-RPC 2.0)  • Connectors  • Tools & Resources         │
└───────────────────────────────────┬─────────────────────────────────────────┘
                                    │
┌───────────────────────────────────┼───────────────────────────────────────┐
│                                   ▼                                        │
│  ┌─────────────────────────────────────────────────────────────────────┐  │
│  │                      API-BACKEND (Rust / Axum)                       │  │
│  │                           Port: 3003                                 │  │
│  │        Central Gateway • REST API • Service Orchestration           │  │
│  └─────────────────────────────────────────────────────────────────────┘  │
│                                   │                                        │
│      ┌────────────┬───────────────┼───────────────┬────────────┐          │
│      ▼            ▼               ▼               ▼            ▼          │
│  ┌────────┐  ┌─────────┐   ┌───────────┐  ┌──────────┐  ┌──────────┐     │
│  │  Auth  │  │  Data   │   │  Code     │  │ Chunker  │  │Embeddings│     │
│  │Middlew.│  │Connector│   │Normalize  │  │          │  │          │     │
│  │ :3001  │  │ :8000   │   │Fetch:8090 │  │  :3002   │  │  :3005   │     │
│  └────────┘  └─────────┘   └───────────┘  └──────────┘  └──────────┘     │
│                                   │               │            │          │
│                                   └───────────────┼────────────┘          │
│                                                   ▼                        │
│                              ┌─────────────────────────────────┐          │
│                              │      RELATION-GRAPH (Rust)      │          │
│                              │           Port: 3018            │          │
│                              │  Neo4j + Zilliz (Hybrid Search) │          │
│                              └─────────────────────────────────┘          │
│                                                                            │
│                         CONFUSE INFRASTRUCTURE                             │
└────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                            DATA SOURCES                                      │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐   │
│  │ GitHub  │ │ GitLab  │ │Bitbucket│ │ G Drive │ │ Dropbox │ │Local FS │   │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘   │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐               │
│  │ Notion  │ │Confluenc│ │  Slack  │ │  Jira   │ │ Linear  │               │
│  └─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘               │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Service Descriptions

### 1. API Backend (This Service)
**Port: 3003** | **Language: Rust (Axum)**

# ConFuse Architecture Overview

## System Architecture

ConFuse is a Knowledge Intelligence Platform connecting AI agents to organizational knowledge across code repositories, documents, chat, and other data sources.

At a high level:

- Frontend (Next.js): UI and client interactions.
- API Backend (Rust / Axum): Central gateway and orchestration layer (Port: 3003).
- MCP Server (Rust): Connector and tool orchestration for agent integrations (Port: 3004).
- Client Connector (Python): Agent transports (WebSocket/SSE) and session management (Port: 8095).
- Data Connector (Python): Ingests external sources and triggers pipelines (Port: 8000).
- Chunker, Embeddings, Relation Graph, Code Normalize: Rust services handling processing, indexing, and graph storage.

Services communicate over HTTP/REST and gRPC where appropriate. Datastores include PostgreSQL (primary), Redis (cache/locks), Neo4j (graph), and a vector store for embeddings.
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
| auth-middleware | 3001 | HTTP | External service (see auth-middleware repo) |
| chunker | 3002 | HTTP | Rust |
| api-backend | 3003 | HTTP | Rust |
| mcp-server | 3004 | stdio/HTTP | Rust |
| embeddings | 3005 | HTTP | Rust |
| relation-graph | 3018 | HTTP | Rust |
| data-connector | 8000 | HTTP | Python |
| code-normalize-fetch | 8090 | HTTP | Rust |
| client-connector | 8095 | WS/HTTP | Python |

---

## Database Dependencies

| Database | Purpose | Used By |
|----------|---------|---------|
| PostgreSQL | User data, metadata, job queue | api-backend, data-connector, auth |
| Neo4j | Knowledge graph relationships | relation-graph |
| Zilliz/Milvus | Vector embeddings | relation-graph |
| Redis | Caching, sessions, rate limiting | api-backend, auth |

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
