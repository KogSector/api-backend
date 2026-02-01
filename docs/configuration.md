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

# Service Discovery (Consul)
CONSUL_ADDRESS=http://localhost:8500
SERVICE_NAME=api-backend
SERVICE_ID=api-backend-1
SERVICE_ADDRESS=localhost
SERVICE_PORT=8088
HEALTH_CHECK_URL=http://localhost:8088/health

# Service URLs
AUTH_MIDDLEWARE_URL=http://localhost:3001
DATA_CONNECTOR_URL=http://localhost:8000
RELATION_GRAPH_URL=http://localhost:3018
MCP_SERVER_URL=http://localhost:3004
EMBEDDINGS_URL=http://localhost:3005

# Observability
JAEGER_ENDPOINT=http://localhost:14268/api/traces
OTEL_SERVICE_NAME=api-backend
OTEL_SAMPLING_RATE=0.1

# Security
JWT_SECRET=your-jwt-secret-minimum-32-characters
CORS_ORIGINS=http://localhost:3000
```

### Optional Variables

```env
# Server
PORT=8088
HOST=0.0.0.0
RUST_ENV=development

# Logging
RUST_LOG=info                     # debug, info, warn, error
LOG_FORMAT=json                   # json or pretty

# Circuit Breaker
CIRCUIT_BREAKER_FAILURE_THRESHOLD=50    # Percentage (0-100)
CIRCUIT_BREAKER_TIMEOUT_SECS=30         # Seconds before retry
CIRCUIT_BREAKER_SUCCESS_THRESHOLD=3     # Successes to close
CIRCUIT_BREAKER_MIN_CALLS=10            # Min calls before tripping

# Retry Policy
RETRY_MAX_ATTEMPTS=3
RETRY_INITIAL_BACKOFF_MS=100
RETRY_MAX_BACKOFF_SECS=10
RETRY_MULTIPLIER=2.0

# Rate Limiting
RATE_LIMIT_PER_USER=100           # Requests per minute
RATE_LIMIT_PER_IP=1000            # Requests per minute
RATE_LIMIT_WINDOW_SECS=60         # Rate limit window

# Caching
CACHE_DEFAULT_TTL_SECS=300        # 5 minutes
CACHE_POOL_SIZE=10
ENABLE_CACHE_INVALIDATION=true

# Request Limits
MAX_REQUEST_SIZE=10mb
REQUEST_TIMEOUT_MS=30000

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

Redis connection for caching, rate limiting, and circuit breaker state:

```env
# Local
REDIS_URL=redis://localhost:6379

# With password
REDIS_URL=redis://:password@localhost:6379

# AWS ElastiCache
REDIS_URL=redis://confuse-cache.xxx.cache.amazonaws.com:6379
```

### CONSUL_ADDRESS

Consul server for service discovery and health checks:

```env
# Local development
CONSUL_ADDRESS=http://localhost:8500

# Production cluster
CONSUL_ADDRESS=http://consul.confuse.internal:8500
```

The service automatically registers itself with Consul on startup and deregisters on shutdown. Health checks run every 10 seconds.

### Service URLs

URLs for other ConFuse microservices:

| Variable | Default | Description |
|----------|---------|-------------|
| `AUTH_MIDDLEWARE_URL` | `http://localhost:3001` | JWT validation service |
| `DATA_CONNECTOR_URL` | `http://localhost:8000` | Data ingestion service |
| `RELATION_GRAPH_URL` | `http://localhost:3018` | Knowledge graph service |
| `MCP_SERVER_URL` | `http://localhost:3004` | MCP protocol service |
| `EMBEDDINGS_URL` | `http://localhost:3005` | Vector embedding service |

**Note**: With Consul service discovery enabled, these URLs can be replaced with service names that are automatically resolved to healthy instances.

### Observability Configuration

#### JAEGER_ENDPOINT

Jaeger collector endpoint for distributed tracing:

```env
# Local Jaeger
JAEGER_ENDPOINT=http://localhost:14268/api/traces

# Production
JAEGER_ENDPOINT=http://jaeger-collector.monitoring:14268/api/traces
```

#### OTEL_SAMPLING_RATE

Trace sampling rate (0.0 to 1.0):
- `0.1` = 10% of requests traced (recommended for production)
- `1.0` = 100% of requests traced (development only)
- `0.01` = 1% of requests traced (high-traffic production)

### Circuit Breaker Configuration

Control circuit breaker behavior per service:

```env
# Trip circuit at 50% failure rate
CIRCUIT_BREAKER_FAILURE_THRESHOLD=50

# Wait 30 seconds before testing recovery
CIRCUIT_BREAKER_TIMEOUT_SECS=30

# Require 3 successful calls to close circuit
CIRCUIT_BREAKER_SUCCESS_THRESHOLD=3

# Need at least 10 calls before calculating failure rate
CIRCUIT_BREAKER_MIN_CALLS=10
```

**Circuit Breaker States:**
- **Closed**: Normal operation
- **Open**: Rejecting requests (after failure threshold)
- **Half-Open**: Testing recovery (limited requests)

### Retry Policy Configuration

Configure exponential backoff retry behavior:

```env
# Maximum retry attempts
RETRY_MAX_ATTEMPTS=3

# Initial backoff duration
RETRY_INITIAL_BACKOFF_MS=100

# Maximum backoff duration
RETRY_MAX_BACKOFF_SECS=10

# Backoff multiplier (exponential)
RETRY_MULTIPLIER=2.0
```

**Retry Schedule Example:**
- Attempt 1: Immediate
- Attempt 2: 100ms delay
- Attempt 3: 200ms delay
- Attempt 4: 400ms delay (if max_attempts=4)

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

Control request rate limits using Redis-backed distributed counters:

```env
# 100 requests per minute per user
RATE_LIMIT_PER_USER=100
RATE_LIMIT_WINDOW_SECS=60

# 1000 requests per minute per IP
RATE_LIMIT_PER_IP=1000

# Algorithm: sliding_window or token_bucket
RATE_LIMIT_ALGORITHM=sliding_window
```

Rate limits are enforced using Redis to ensure consistency across multiple API Backend instances.

### Caching Configuration

Configure distributed caching behavior:

```env
# Default TTL for cached entries
CACHE_DEFAULT_TTL_SECS=300

# Redis connection pool size
CACHE_POOL_SIZE=10

# Enable cache invalidation pub/sub
ENABLE_CACHE_INVALIDATION=true
```

**Cache Patterns Supported:**
- **Cache-Aside**: Fetch on miss, store in cache
- **Write-Through**: Update cache on write
- **Pattern Invalidation**: Invalidate by key pattern (e.g., `user:*`)

## Configuration Files

### Connectivity Configuration (JSON)

The connectivity infrastructure can be configured via JSON file or environment variable:

```json
{
  "service_discovery": {
    "consul_address": "http://localhost:8500",
    "service_name": "api-backend",
    "service_id": "api-backend-1",
    "service_address": "localhost",
    "service_port": 8088,
    "health_check_url": "http://localhost:8088/health",
    "health_check_interval_secs": 10,
    "deregister_critical_after_secs": 30,
    "tags": ["v1", "rust"]
  },
  "circuit_breaker": {
    "failure_threshold": 50,
    "success_threshold": 3,
    "timeout_secs": 30,
    "half_open_max_calls": 5,
    "min_calls": 10
  },
  "rate_limit": {
    "redis_url": "redis://localhost:6379",
    "per_user_limit": 100,
    "per_ip_limit": 1000,
    "per_service_limit": 10000,
    "window_secs": 60,
    "algorithm": "sliding_window"
  },
  "cache": {
    "redis_url": "redis://localhost:6379",
    "default_ttl_secs": 300,
    "pool_size": 10,
    "connection_timeout_secs": 5,
    "enable_invalidation": true
  },
  "observability": {
    "jaeger_endpoint": "http://localhost:14268/api/traces",
    "service_name": "api-backend",
    "sampling_rate": 0.1,
    "enable_metrics": true,
    "metrics_port": 9090
  }
}
```

Load via environment variable:
```env
CONNECTIVITY_CONFIG='{"service_discovery": {...}}'
```

Or via file:
```rust
let config = ConnectivityConfig::from_file("config/connectivity.json")?;
```

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
RUST_ENV=development
RUST_LOG=debug
LOG_FORMAT=pretty
ENABLE_CACHE_INVALIDATION=false
OTEL_SAMPLING_RATE=1.0
CIRCUIT_BREAKER_MIN_CALLS=5
```

### Staging

```env
RUST_ENV=staging
RUST_LOG=info
ENABLE_CACHE_INVALIDATION=true
OTEL_SAMPLING_RATE=0.1
```

### Production

```env
RUST_ENV=production
RUST_LOG=warn
ENABLE_CACHE_INVALIDATION=true
ENABLE_METRICS=true
OTEL_SAMPLING_RATE=0.01
CIRCUIT_BREAKER_FAILURE_THRESHOLD=50
```

## Health Checks

The API Backend exposes health check endpoints for monitoring:

### Endpoints

- `GET /health` - Overall health status
- `GET /health/ready` - Readiness probe (Kubernetes)
- `GET /health/live` - Liveness probe (Kubernetes)

### Health Check Components

The health check verifies:
- Database connectivity (PostgreSQL)
- Cache connectivity (Redis)
- Downstream service availability (via circuit breaker state)
- Consul registration status

### Response Format

```json
{
  "status": "healthy",
  "version": "1.0.0",
  "timestamp": 1706745600,
  "checks": {
    "database": {
      "status": "healthy",
      "response_time_ms": 5
    },
    "cache": {
      "status": "healthy",
      "response_time_ms": 2
    },
    "auth_service": {
      "status": "healthy",
      "response_time_ms": 15
    },
    "data_connector": {
      "status": "degraded",
      "message": "Circuit breaker half-open",
      "response_time_ms": 100
    }
  }
}
```

**Status Values:**
- `healthy` - All systems operational (HTTP 200)
- `degraded` - Some non-critical issues (HTTP 200)
- `unhealthy` - Critical failures (HTTP 503)

## Observability

### Distributed Tracing

All requests are automatically traced with OpenTelemetry:

```rust
// Traces include:
// - Request ID (correlation ID)
// - Service name and version
// - HTTP method, path, status
// - Response time
// - Error details
// - Downstream service calls
```

View traces in Jaeger UI: `http://localhost:16686`

### Metrics

Prometheus metrics exposed on `/metrics`:

- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency histogram
- `circuit_breaker_state` - Circuit breaker state per service
- `circuit_breaker_calls_total` - Total calls through circuit breaker
- `circuit_breaker_failures_total` - Failed calls
- `cache_hits_total` - Cache hit count
- `cache_misses_total` - Cache miss count
- `rate_limit_exceeded_total` - Rate limit violations

### Logging

Structured logging with configurable format:

```env
# Pretty format for development
RUST_LOG=debug
LOG_FORMAT=pretty

# JSON format for production
RUST_LOG=info
LOG_FORMAT=json
```

## Migration from Previous Configuration

If migrating from the old `failsafe` circuit breaker:

**Before:**
```toml
[dependencies]
failsafe = "1.3"
```

**After:**
```toml
[dependencies]
confuse-connectivity = { path = "../shared-middleware-confuse/connectivity" }
```

**Configuration Changes:**
- Circuit breaker configuration moved to `CIRCUIT_BREAKER_*` environment variables
- Service discovery now uses Consul instead of hardcoded URLs
- Distributed tracing now uses OpenTelemetry instead of custom implementation
- Caching now uses shared Redis-backed implementation

See the [connectivity library documentation](../../shared-middleware-confuse/connectivity/README.md) for detailed migration guide.
