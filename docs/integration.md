# API Backend Integration Guide

> How the API Backend connects to other ConFuse services

## Service Communication Overview

The API Backend acts as a gateway, routing requests to appropriate microservices:

```
┌──────────────────────────────────────────────────────────────────────────┐
│                              API BACKEND                                  │
├──────────────────────────────────────────────────────────────────────────┤
│                                                                           │
│  Incoming Request                                                         │
│       │                                                                   │
│       ▼                                                                   │
│  ┌─────────────┐                                                         │
│  │  Middleware │ ──▶ Rate Limit ──▶ Auth Check ──▶ Validation            │
│  └──────┬──────┘                                                         │
│         │                                                                 │
│         ▼                                                                 │
│  ┌─────────────┐                                                         │
│  │   Router    │                                                         │
│  └──────┬──────┘                                                         │
│         │                                                                 │
│    ┌────┴────┬────────┬────────┬────────┐                               │
│    ▼         ▼        ▼        ▼        ▼                               │
│ /auth    /sources  /search  /entities  /mcp                             │
│    │         │        │        │        │                               │
└────┼─────────┼────────┼────────┼────────┼───────────────────────────────┘
     │         │        │        │        │
     ▼         ▼        ▼        ▼        ▼
┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────┐
│  Auth   │ │  Data   │ │Relation │ │Relation │ │   MCP   │
│Middleware│ │Connector│ │ Graph   │ │ Graph   │ │ Server  │
│  :3001  │ │  :8000  │ │  :3018  │ │  :3018  │ │  :3004  │
└─────────┘ └─────────┘ └─────────┘ └─────────┘ └─────────┘
```

## Service Clients

### Auth Middleware Client

Handles all authentication-related operations:

```typescript
// src/clients/auth-client.ts
import axios from 'axios';

const authClient = axios.create({
  baseURL: process.env.AUTH_MIDDLEWARE_URL,
  timeout: 5000,
});

export const auth = {
  // Verify JWT token
  async verifyToken(token: string): Promise<User> {
    const response = await authClient.get('/auth/verify', {
      headers: { Authorization: `Bearer ${token}` }
    });
    return response.data;
  },

  // Validate API key
  async validateApiKey(apiKey: string): Promise<ApiKeyInfo> {
    const response = await authClient.post('/api-keys/validate', { apiKey });
    return response.data;
  },

  // Refresh token
  async refreshToken(refreshToken: string): Promise<TokenPair> {
    const response = await authClient.post('/auth/refresh', { refreshToken });
    return response.data;
  }
};
```

**Used by:**
- Auth middleware (`src/middleware/auth.ts`)
- API key validation
- Token refresh endpoint

### Data Connector Client

Manages data source connections:

```typescript
// src/clients/data-connector-client.ts
const dataClient = axios.create({
  baseURL: process.env.DATA_CONNECTOR_URL,
  timeout: 30000, // Longer timeout for sync operations
});

export const dataConnector = {
  // List all sources for a user
  async listSources(userId: string): Promise<Source[]> {
    const response = await dataClient.get('/sources', {
      headers: { 'X-User-Id': userId }
    });
    return response.data.sources;
  },

  // Connect a new source
  async connectSource(userId: string, config: SourceConfig): Promise<Source> {
    const response = await dataClient.post('/sources', config, {
      headers: { 'X-User-Id': userId }
    });
    return response.data;
  },

  // Trigger sync
  async syncSource(sourceId: string): Promise<SyncJob> {
    const response = await dataClient.post(`/sources/${sourceId}/sync`);
    return response.data;
  },

  // Get sync status
  async getSyncStatus(jobId: string): Promise<SyncStatus> {
    const response = await dataClient.get(`/jobs/${jobId}`);
    return response.data;
  }
};
```

**Used by:**
- `/sources` endpoints
- Sync management
- Webhook forwarding

### Relation Graph Client

Queries the knowledge graph:

```typescript
// src/clients/relation-graph-client.ts
const graphClient = axios.create({
  baseURL: process.env.RELATION_GRAPH_URL,
  timeout: 10000,
});

export const relationGraph = {
  // Hybrid search (vector + graph)
  async search(query: SearchQuery): Promise<SearchResults> {
    const response = await graphClient.post('/api/search', query);
    return response.data;
  },

  // Get entity with relationships
  async getEntity(entityId: string, hops: number = 2): Promise<Entity> {
    const response = await graphClient.get(
      `/api/graph/entities/${entityId}/neighbors?hops=${hops}`
    );
    return response.data;
  },

  // Get statistics
  async getStats(): Promise<GraphStats> {
    const response = await graphClient.get('/api/graph/statistics');
    return response.data;
  }
};
```

**Used by:**
- `/search` endpoint
- `/entities` endpoints
- Dashboard statistics

### MCP Server Client

For MCP tool operations (used internally):

```typescript
// src/clients/mcp-client.ts
const mcpClient = axios.create({
  baseURL: process.env.MCP_SERVER_URL,
  timeout: 30000,
});

export const mcpServer = {
  // List available tools
  async listTools(): Promise<Tool[]> {
    const response = await mcpClient.get('/tools');
    return response.data.tools;
  },

  // Call a tool
  async callTool(name: string, args: object): Promise<ToolResult> {
    const response = await mcpClient.post('/tools/call', { name, arguments: args });
    return response.data;
  }
};
```

## Request Flow Examples

### 1. Search Request

```
Client: POST /v1/search { query: "authentication" }
        │
        ▼
API Backend:
  1. Validate JWT token (auth-middleware)
  2. Check rate limits (Redis)
  3. Forward to relation-graph
        │
        ▼
Relation Graph: POST /api/search
  - Vector search in Zilliz
  - Graph traversal in Neo4j
  - Combine and rank results
        │
        ▼
API Backend:
  1. Transform response
  2. Add pagination
  3. Return to client
```

### 2. Connect Source

```
Client: POST /v1/sources { type: "github", config: {...} }
        │
        ▼
API Backend:
  1. Validate JWT token
  2. Validate request body
  3. Forward to data-connector
        │
        ▼
Data Connector: POST /sources
  - Validate OAuth token
  - Setup webhooks
  - Queue initial sync
        │
        ▼
API Backend:
  1. Return source info
  2. Optionally: SSE for sync progress
```

## Error Handling

All service clients use consistent error handling:

```typescript
// src/clients/base-client.ts
export async function handleServiceCall<T>(
  serviceName: string,
  operation: () => Promise<T>
): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    if (axios.isAxiosError(error)) {
      if (error.code === 'ECONNREFUSED') {
        throw new ServiceUnavailableError(`${serviceName} is unavailable`);
      }
      if (error.response?.status === 401) {
        throw new UnauthorizedError('Authentication failed');
      }
      if (error.response?.status === 429) {
        throw new RateLimitError('Rate limit exceeded');
      }
      throw new ServiceError(
        serviceName,
        error.response?.data?.message || error.message
      );
    }
    throw error;
  }
}
```

## Circuit Breaker Pattern

For resilience, we use circuit breakers:

```typescript
import CircuitBreaker from 'opossum';

const graphBreaker = new CircuitBreaker(relationGraph.search, {
  timeout: 10000,
  errorThresholdPercentage: 50,
  resetTimeout: 30000,
});

graphBreaker.fallback(() => ({
  results: [],
  message: 'Search temporarily unavailable'
}));
```

## Health Checks

API Backend checks all downstream services:

```typescript
// GET /health
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": "2026-01-01T12:00:00Z",
  "services": {
    "database": { "status": "connected", "latency": 5 },
    "redis": { "status": "connected", "latency": 2 },
    "auth-middleware": { "status": "healthy", "latency": 15 },
    "data-connector": { "status": "healthy", "latency": 20 },
    "relation-graph": { "status": "healthy", "latency": 25 },
    "mcp-server": { "status": "healthy", "latency": 10 }
  }
}
```

## Timeouts and Retries

| Service | Timeout | Retries | Notes |
|---------|---------|---------|-------|
| auth-middleware | 5s | 2 | Fast, critical |
| data-connector | 30s | 1 | Slow operations |
| relation-graph | 10s | 2 | Search operations |
| mcp-server | 30s | 1 | Tool calls vary |
| embeddings | 60s | 0 | API calls to external |
