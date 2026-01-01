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
│  │                      API-BACKEND (Node.js)                           │  │
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
**Port: 3003** | **Language: Node.js**

Central API gateway that:
- Routes requests to appropriate microservices
- Handles user-facing REST API
- Orchestrates multi-service workflows
- Manages API versioning

### 2. Auth Middleware
**Port: 3001** | **Language: Node.js**

Authentication and authorization:
- JWT token generation/validation
- OAuth2 flows (GitHub, Google, etc.)
- API key management
- User session handling

### 3. Data Connector
**Port: 8000** | **Language: Python**

Data ingestion service:
- Connects to data sources (GitHub, Drive, etc.)
- Webhook handling for real-time sync
- File filtering and preprocessing
- Triggers knowledge pipeline

### 4. Code Normalize Fetch
**Port: 8090** | **Language: Rust**

Code preprocessing:
- Fetches code from Git providers
- AST parsing with tree-sitter
- Entity extraction (functions, classes)
- Language detection

### 5. Chunker
**Port: 3002** | **Language: Rust**

Text segmentation:
- Semantic chunking for code/docs
- Entity-aware boundaries
- Context enrichment
- Multiple chunking strategies

### 6. Embeddings
**Port: 3005** | **Language: Rust**

Vector generation:
- Multi-provider support (OpenAI, Cohere, Voyage)
- Intelligent model routing
- Batch processing with caching
- Reranking support

### 7. Relation Graph
**Port: 3018** | **Language: Rust**

Knowledge graph:
- Neo4j for explicit relationships
- Zilliz for vector similarity
- Cross-source linking (code↔docs)
- Hybrid search capabilities

### 8. MCP Server
**Port: 3004** | **Language: Rust**

MCP Protocol server:
- JSON-RPC 2.0 over stdio
- Tools for data access
- Resource discovery
- Connector aggregation

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
| auth-middleware | 3001 | HTTP | Node.js |
| chunker | 3002 | HTTP | Rust |
| api-backend | 3003 | HTTP | Node.js |
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
cd api-backend && npm run dev
cd auth-middleware && npm run dev
cd data-connector && uvicorn app:app --reload
# ... etc
```

### Production

- Kubernetes manifests in each service's `k8s/` folder
- Helm charts available in `infrastructure/` repo
- CI/CD via GitHub Actions
