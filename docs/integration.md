# ConFuse Integration Guide

## Overview

This guide covers how to integrate ConFuse into your AI agents, applications, and workflows.

---

## Integration Methods

### 1. MCP Protocol (Recommended for AI Agents)

The **Model Context Protocol (MCP)** is the standard way for AI agents to connect to ConFuse.

#### WebSocket Connection

```javascript
// Connect via WebSocket
const ws = new WebSocket('wss://api.confuse.io/mcp/ws?key=YOUR_API_KEY');

// Initialize connection
ws.send(JSON.stringify({
  jsonrpc: "2.0",
  id: 1,
  method: "initialize",
  params: {
    clientInfo: {
      name: "MyAgent",
      version: "1.0.0"
    }
  }
}));

// List available tools
ws.send(JSON.stringify({
  jsonrpc: "2.0",
  id: 2,
  method: "tools/list"
}));

// Search for context
ws.send(JSON.stringify({
  jsonrpc: "2.0",
  id: 3,
  method: "tools/call",
  params: {
    name: "confuse.search",
    arguments: {
      query: "how does authentication work",
      limit: 10
    }
  }
}));
```

#### For Cursor IDE

Add to your MCP configuration (`~/.cursor/mcp.json`):

```json
{
  "mcpServers": {
    "confuse": {
      "transport": "websocket",
      "url": "wss://api.confuse.io/mcp/ws",
      "headers": {
        "Authorization": "Bearer YOUR_API_KEY"
      }
    }
  }
}
```

#### For Windsurf

Similar configuration in Windsurf's MCP settings.

---

### 2. REST API (For Applications)

Use the REST API for web applications and backend services.

#### JavaScript/TypeScript

```typescript
import { ConFuseClient } from '@confuse/sdk';

const client = new ConFuseClient({
  apiKey: 'YOUR_API_KEY',
  baseUrl: 'https://api.confuse.io/v1'
});

// Search
const results = await client.search({
  query: 'authentication flow',
  limit: 10,
  filters: {
    types: ['code', 'document']
  }
});

// Get entity details
const entity = await client.getEntity('entity-uuid');

// List sources
const sources = await client.listSources();
```

#### Python

```python
from confuse import ConFuseClient

client = ConFuseClient(api_key="YOUR_API_KEY")

# Search
results = client.search(
    query="authentication flow",
    limit=10,
    filters={"types": ["code", "document"]}
)

# Get entity with relationships
entity = client.get_entity("entity-uuid", include_neighbors=True)
```

#### cURL

```bash
# Search
curl -X POST https://api.confuse.io/v1/search \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"query": "authentication flow", "limit": 10}'

# List sources
curl https://api.confuse.io/v1/sources \
  -H "Authorization: Bearer YOUR_API_KEY"
```

---

### 3. Webhooks (For Real-time Updates)

Configure webhooks to receive notifications when data changes.

#### Setup

```bash
curl -X POST https://api.confuse.io/v1/webhooks \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{
    "url": "https://your-app.com/webhooks/confuse",
    "events": ["source.synced", "entity.created", "entity.updated"],
    "secret": "your-webhook-secret"
  }'
```

#### Webhook Payload

```json
{
  "event": "source.synced",
  "timestamp": "2026-01-01T12:00:00Z",
  "data": {
    "sourceId": "source-uuid",
    "stats": {
      "filesProcessed": 150,
      "chunksCreated": 1200,
      "entitiesExtracted": 450
    }
  },
  "signature": "sha256=..."
}
```

---

## Available MCP Tools

When connected via MCP, these tools are available:

### confuse.search
Hybrid search across all knowledge.

```json
{
  "name": "confuse.search",
  "arguments": {
    "query": "string",
    "limit": 10,
    "filters": {
      "sources": ["source-id"],
      "types": ["code", "document"],
      "languages": ["python"]
    }
  }
}
```

### confuse.get_entity
Get entity with full context.

```json
{
  "name": "confuse.get_entity",
  "arguments": {
    "entityId": "uuid",
    "includeNeighbors": true,
    "neighborHops": 2
  }
}
```

### confuse.get_file
Read file content from a connected source.

```json
{
  "name": "confuse.get_file",
  "arguments": {
    "sourceId": "uuid",
    "path": "src/auth/handler.py"
  }
}
```

### confuse.list_sources
List connected data sources.

```json
{
  "name": "confuse.list_sources",
  "arguments": {}
}
```

---

## Authentication Flows

### API Key (Simplest)

1. Generate API key in ConFuse dashboard
2. Use in requests: `X-API-Key: key_...`

### OAuth2 (User Context)

1. Redirect user to: `https://confuse.io/oauth/authorize`
2. Receive callback with authorization code
3. Exchange for tokens: `POST /oauth/token`
4. Use access token: `Authorization: Bearer <token>`

### Service Account (Server-to-Server)

1. Create service account in dashboard
2. Download JSON credentials
3. Exchange for short-lived token:
   ```bash
   curl -X POST https://api.confuse.io/v1/auth/token \
     -d @service-account.json
   ```

---

## Best Practices

### 1. Caching
Cache search results for frequently asked questions:
```typescript
const cache = new LRUCache({ maxAge: 5 * 60 * 1000 }); // 5 min

async function search(query) {
  const cached = cache.get(query);
  if (cached) return cached;
  
  const results = await client.search({ query });
  cache.set(query, results);
  return results;
}
```

### 2. Rate Limit Handling
```typescript
async function searchWithRetry(query, retries = 3) {
  try {
    return await client.search({ query });
  } catch (err) {
    if (err.status === 429 && retries > 0) {
      const delay = err.headers['retry-after'] * 1000;
      await sleep(delay);
      return searchWithRetry(query, retries - 1);
    }
    throw err;
  }
}
```

### 3. Filtering for Relevance
Always filter by source type for better results:
```typescript
// For code questions
const codeResults = await client.search({
  query: "implement retry logic",
  filters: { types: ["code"] }
});

// For documentation
const docResults = await client.search({
  query: "API rate limits",
  filters: { types: ["document"] }
});
```

---

## SDKs

### Official SDKs

| Language | Package | Install |
|----------|---------|---------|
| JavaScript | `@confuse/sdk` | `npm install @confuse/sdk` |
| Python | `confuse-sdk` | `pip install confuse-sdk` |
| Rust | `confuse-sdk` | `cargo add confuse-sdk` |

### Community SDKs

- Go: `github.com/confuse/go-sdk`
- Ruby: `gem install confuse`

---

## Example: Building a Code Assistant

```python
from confuse import ConFuseClient
import openai

client = ConFuseClient(api_key="YOUR_CONFUSE_KEY")

def answer_code_question(question: str) -> str:
    # Get relevant context from ConFuse
    context = client.search(
        query=question,
        limit=5,
        filters={"types": ["code", "document"]}
    )
    
    # Build context string
    context_text = "\n\n".join([
        f"[{r.source.path}]\n{r.content}"
        for r in context.results
    ])
    
    # Call LLM with context
    response = openai.chat.completions.create(
        model="gpt-4",
        messages=[
            {"role": "system", "content": f"Use this context:\n{context_text}"},
            {"role": "user", "content": question}
        ]
    )
    
    return response.choices[0].message.content
```

---

## Support

- Documentation: https://docs.confuse.io
- GitHub Issues: https://github.com/confuse/confuse/issues
- Discord: https://discord.gg/confuse
- Email: support@confuse.io
