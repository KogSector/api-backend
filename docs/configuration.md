# API Backend Configuration

> Complete guide to configuring the ConFuse API Backend service

## Environment Variables

Create a `.env` file in the project root with the following variables:

### Required Variables

```env
# Database
DATABASE_URL=postgresql://username:password@localhost:5432/confuse
DATABASE_POOL_SIZE=20

# Redis
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=

# Service URLs
AUTH_MIDDLEWARE_URL=http://localhost:3001
DATA_CONNECTOR_URL=http://localhost:8000
RELATION_GRAPH_URL=http://localhost:3018
MCP_SERVER_URL=http://localhost:3004
EMBEDDINGS_URL=http://localhost:3005

# Security
JWT_SECRET=your-jwt-secret-minimum-32-characters
CORS_ORIGINS=http://localhost:3000
```

### Optional Variables

```env
# Server
PORT=3003
HOST=0.0.0.0
NODE_ENV=development

# Logging
LOG_LEVEL=info                    # debug, info, warn, error
LOG_FORMAT=json                   # json or pretty

# Rate Limiting
RATE_LIMIT_WINDOW_MS=60000        # 1 minute
RATE_LIMIT_MAX_REQUESTS=100       # Max requests per window

# Request Limits
MAX_REQUEST_SIZE=10mb
REQUEST_TIMEOUT_MS=30000

# Caching
CACHE_TTL_SECONDS=300             # 5 minutes
ENABLE_RESPONSE_CACHE=true

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090
```

## Variable Details

### DATABASE_URL

PostgreSQL connection string format:
```
postgresql://[user]:[password]@[host]:[port]/[database]?[options]
```

Options include:
- `sslmode=require` - Required for cloud databases
- `connection_limit=20` - Connection pool size

**Examples:**
```env
# Local development
DATABASE_URL=postgresql://confuse:password@localhost:5432/confuse

# Neon (cloud)
DATABASE_URL=postgresql://neondb_owner:xxx@ep-xxx.neon.tech/neondb?sslmode=require

# AWS RDS
DATABASE_URL=postgresql://admin:password@confuse.xxx.us-east-1.rds.amazonaws.com:5432/confuse?sslmode=require
```

### REDIS_URL

Redis connection for caching and rate limiting:

```env
# Local
REDIS_URL=redis://localhost:6379

# With password
REDIS_URL=redis://:password@localhost:6379

# AWS ElastiCache
REDIS_URL=redis://confuse-cache.xxx.cache.amazonaws.com:6379
```

### Service URLs

URLs for other ConFuse microservices:

| Variable | Default | Description |
|----------|---------|-------------|
| `AUTH_MIDDLEWARE_URL` | `http://localhost:3001` | JWT validation service |
| `DATA_CONNECTOR_URL` | `http://localhost:8000` | Data ingestion service |
| `RELATION_GRAPH_URL` | `http://localhost:3018` | Knowledge graph service |
| `MCP_SERVER_URL` | `http://localhost:3004` | MCP protocol service |
| `EMBEDDINGS_URL` | `http://localhost:3005` | Vector embedding service |

### CORS_ORIGINS

Comma-separated list of allowed origins:

```env
# Development
CORS_ORIGINS=http://localhost:3000

# Production
CORS_ORIGINS=https://app.confuse.io,https://admin.confuse.io

# Allow all (not recommended for production)
CORS_ORIGINS=*
```

### Rate Limiting

Control request rate limits:

```env
# 100 requests per minute per IP
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=100

# 1000 requests per hour per user
RATE_LIMIT_USER_WINDOW_MS=3600000
RATE_LIMIT_USER_MAX_REQUESTS=1000
```

## Configuration Files

### config/default.json

```json
{
  "server": {
    "port": 3003,
    "host": "0.0.0.0"
  },
  "database": {
    "poolSize": 20,
    "idleTimeoutMs": 30000
  },
  "cache": {
    "ttlSeconds": 300,
    "maxSize": 1000
  },
  "rateLimit": {
    "windowMs": 60000,
    "maxRequests": 100
  }
}
```

### config/production.json

```json
{
  "server": {
    "trustProxy": true
  },
  "logging": {
    "level": "info",
    "format": "json"
  },
  "cache": {
    "ttlSeconds": 600
  }
}
```

## Secrets Management

### Development

Store secrets in `.env` file (never commit this file):

```bash
# .gitignore
.env
.env.local
.env.*.local
```

### Production

Use environment-specific secret management:

**Docker:**
```yaml
services:
  api-backend:
    environment:
      - DATABASE_URL
      - JWT_SECRET
    secrets:
      - db_password
```

**Kubernetes:**
```yaml
apiVersion: v1
kind: Secret
metadata:
  name: api-backend-secrets
type: Opaque
stringData:
  DATABASE_URL: postgresql://...
  JWT_SECRET: your-secret-here
```

**Cloud Providers:**
- AWS: Secrets Manager or Parameter Store
- GCP: Secret Manager
- Azure: Key Vault

## Validation

The service validates configuration on startup:

```typescript
// Validation errors will prevent startup
[ERROR] Missing required environment variable: DATABASE_URL
[ERROR] Invalid REDIS_URL format
[ERROR] JWT_SECRET must be at least 32 characters
```

## Environment-Specific Settings

### Development

```env
NODE_ENV=development
LOG_LEVEL=debug
LOG_FORMAT=pretty
ENABLE_RESPONSE_CACHE=false
```

### Staging

```env
NODE_ENV=staging
LOG_LEVEL=info
ENABLE_RESPONSE_CACHE=true
```

### Production

```env
NODE_ENV=production
LOG_LEVEL=warn
ENABLE_RESPONSE_CACHE=true
ENABLE_METRICS=true
```
