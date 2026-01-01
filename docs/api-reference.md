# ConFuse API Reference

## Base URL

```
Production: https://api.confuse.io/v1
Development: http://localhost:3003/v1
```

## Authentication

All API requests require authentication via:
- **Bearer Token**: `Authorization: Bearer <jwt_token>`
- **API Key**: `X-API-Key: <api_key>`

---

## Endpoints

### Health & Status

#### GET /health
Check API health status.

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "services": {
    "database": "connected",
    "redis": "connected",
    "embeddings": "healthy",
    "relation-graph": "healthy"
  }
}
```

---

### Sources (Data Connections)

#### GET /sources
List all connected data sources.

**Response:**
```json
{
  "sources": [
    {
      "id": "uuid",
      "type": "github",
      "name": "my-org/my-repo",
      "status": "synced",
      "lastSync": "2026-01-01T12:00:00Z",
      "stats": {
        "files": 150,
        "chunks": 1200,
        "entities": 450
      }
    }
  ]
}
```

#### POST /sources
Connect a new data source.

**Request:**
```json
{
  "type": "github",
  "config": {
    "owner": "my-org",
    "repo": "my-repo",
    "branch": "main"
  },
  "accessToken": "ghp_..."
}
```

#### DELETE /sources/:id
Disconnect a data source.

---

### Search

#### POST /search
Hybrid search across all connected sources.

**Request:**
```json
{
  "query": "how does authentication work",
  "limit": 10,
  "filters": {
    "sources": ["source-id-1", "source-id-2"],
    "types": ["code", "document"],
    "languages": ["python", "rust"]
  },
  "options": {
    "includeGraph": true,
    "graphHops": 2,
    "rerank": true
  }
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "chunk-uuid",
      "content": "def authenticate(user, password)...",
      "score": 0.92,
      "source": {
        "id": "source-uuid",
        "type": "github",
        "path": "src/auth/handler.py"
      },
      "metadata": {
        "language": "python",
        "entityType": "function",
        "entityName": "authenticate"
      }
    }
  ],
  "relatedEntities": [
    {
      "id": "entity-uuid",
      "type": "class",
      "name": "AuthHandler",
      "relationships": ["contains", "calls"]
    }
  ],
  "stats": {
    "totalResults": 45,
    "searchTimeMs": 120
  }
}
```

#### POST /search/vector
Vector-only search (faster, less context).

#### POST /search/graph
Graph-only search (relationship traversal).

---

### Entities

#### GET /entities/:id
Get entity details with relationships.

**Response:**
```json
{
  "id": "entity-uuid",
  "type": "function",
  "name": "authenticate",
  "source": {
    "path": "src/auth/handler.py",
    "startLine": 45,
    "endLine": 78
  },
  "relationships": {
    "calledBy": ["login", "register"],
    "calls": ["validatePassword", "generateToken"],
    "containedIn": "AuthHandler"
  },
  "documentation": [
    {
      "chunkId": "doc-chunk-uuid",
      "content": "Authentication is handled by...",
      "confidence": 0.87
    }
  ]
}
```

#### GET /entities/:id/neighbors
Get related entities within N hops.

---

### Sync & Ingestion

#### POST /sync/:sourceId
Trigger manual sync for a source.

**Response:**
```json
{
  "jobId": "job-uuid",
  "status": "queued",
  "estimatedTime": "2 minutes"
}
```

#### GET /sync/:jobId/status
Get sync job status.

---

### MCP Tools (for AI Agents)

These endpoints are called by `mcp-server` to access data.

#### POST /mcp/search
Internal search endpoint for MCP server.

#### POST /mcp/context
Get contextual information for a query.

#### GET /mcp/capabilities
List available MCP capabilities.

---

## Error Responses

All errors follow this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "query",
      "issue": "Required field missing"
    }
  }
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `UNAUTHORIZED` | 401 | Invalid or missing authentication |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `VALIDATION_ERROR` | 400 | Invalid request |
| `RATE_LIMITED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |

---

## Rate Limits

| Endpoint | Limit |
|----------|-------|
| `/search` | 60/min |
| `/sources` | 30/min |
| `/sync` | 10/min |
| Others | 120/min |

Headers returned:
- `X-RateLimit-Limit`: Max requests
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: Reset time (Unix timestamp)

---

## Webhooks (Incoming)

ConFuse receives webhooks from connected sources:

| Source | Endpoint | Events |
|--------|----------|--------|
| GitHub | `/webhooks/github` | push, pull_request |
| GitLab | `/webhooks/gitlab` | push, merge_request |
| Bitbucket | `/webhooks/bitbucket` | push |

---

## SDKs

- **JavaScript/TypeScript**: `npm install @confuse/sdk`
- **Python**: `pip install confuse-sdk`
- **Rust**: `cargo add confuse-sdk`

See [Integration Guide](integration.md) for usage examples.
