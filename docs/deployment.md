# ConFuse Deployment Guide

## Deployment Options

1. **Docker Compose** - Development and small deployments
2. **Kubernetes** - Production scalable deployments
3. **Managed Cloud** - ConFuse Cloud (coming soon)

---

## Prerequisites

### Required Services

| Service | Purpose | Options |
|---------|---------|---------|
| PostgreSQL | Primary database | Local, RDS, Neon |
| Redis | Caching, sessions, circuit breaker state | Local, ElastiCache |
| Consul | Service discovery, health checks | Local, Consul Cloud |
| Neo4j | Knowledge graph | AuraDB, Self-hosted |
| Zilliz/Milvus | Vector storage | Zilliz Cloud, Self-hosted |

### Optional Services

| Service | Purpose |
|---------|---------|
| Elasticsearch | Full-text search (optional) |
| Prometheus | Metrics collection |
| Grafana | Monitoring dashboards |
| Jaeger | Distributed tracing (OpenTelemetry) |

---

## Docker Compose (Development)

### 1. Clone Repositories

```bash
git clone https://github.com/confuse/api-backend
git clone https://github.com/confuse/auth-middleware
git clone https://github.com/confuse/data-connector
git clone https://github.com/confuse/mcp-server
git clone https://github.com/confuse/client-connector
git clone https://github.com/confuse/chunker
git clone https://github.com/confuse/embeddings
git clone https://github.com/confuse/relation-graph
git clone https://github.com/confuse/code-normalize-fetch
git clone https://github.com/confuse/frontend
```

### 2. Create docker-compose.yml

```yaml
version: '3.8'

services:
  # Infrastructure
  postgres:
    image: postgres:15
    environment:
      POSTGRES_USER: confuse
      POSTGRES_PASSWORD: confuse_password
      POSTGRES_DB: confuse
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  consul:
    image: consul:1.17
    ports:
      - "8500:8500"
      - "8600:8600/udp"
    command: agent -server -ui -bootstrap-expect=1 -client=0.0.0.0
    volumes:
      - consul_data:/consul/data

  jaeger:
    image: jaegertracing/all-in-one:1.51
    ports:
      - "16686:16686"  # UI
      - "4317:4317"    # OTLP gRPC
      - "4318:4318"    # OTLP HTTP
    environment:
      COLLECTOR_OTLP_ENABLED: "true"

  neo4j:
    image: neo4j:5
    environment:
      NEO4J_AUTH: neo4j/confuse_password
    ports:
      - "7474:7474"
      - "7687:7687"
    volumes:
      - neo4j_data:/data

  # ConFuse Services
  api-backend:
    build: ./api-backend
    ports:
      - "8088:8088"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse
      REDIS_URL: redis://redis:6379
      CONSUL_URL: http://consul:8500
      AUTH_SERVICE_URL: http://auth-middleware:3010
      OTEL_EXPORTER_OTLP_ENDPOINT: http://jaeger:4317
      OTEL_SERVICE_NAME: api-backend
      # Connectivity configuration
      CONNECTIVITY_CIRCUIT_BREAKER_ENABLED: "true"
      CONNECTIVITY_RETRY_ENABLED: "true"
      CONNECTIVITY_TRACING_ENABLED: "true"
      CONNECTIVITY_CACHE_ENABLED: "true"
    depends_on:
      - postgres
      - redis
      - consul
      - jaeger

  auth-middleware:
    build: ./auth-middleware
    ports:
      - "3010:3010"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse
      JWT_SECRET: your-jwt-secret-here
      REDIS_URL: redis://redis:6379
      CONSUL_URL: http://consul:8500
      OTEL_EXPORTER_OTLP_ENDPOINT: http://jaeger:4318
      OTEL_SERVICE_NAME: auth-middleware
    depends_on:
      - postgres
      - redis
      - consul
      - jaeger

  data-connector:
    build: ./data-connector
    ports:
      - "8000:8000"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse
      CODE_NORMALIZE_FETCH_URL: http://code-normalize-fetch:8090
      CHUNKER_URL: http://chunker:3002

  code-normalize-fetch:
    build: ./code-normalize-fetch
    ports:
      - "8090:8090"

  chunker:
    build: ./chunker
    ports:
      - "3002:3002"
    environment:
      EMBEDDING_SERVICE_URL: http://embeddings:3005

  embeddings:
    build: ./embeddings
    ports:
      - "3005:3005"
    environment:
      OPENAI_API_KEY: ${OPENAI_API_KEY}

  relation-graph:
    build: ./relation-graph
    ports:
      - "3018:3018"
    environment:
      NEO4J_URI: bolt://neo4j:7687
      NEO4J_USER: neo4j
      NEO4J_PASSWORD: confuse_password
      EMBEDDING_SERVICE_URL: http://embeddings:3005

  mcp-server:
    build: ./mcp-server
    ports:
      - "3004:3004"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse

  client-connector:
    build: ./client-connector
    ports:
      - "8095:8095"
    environment:
      MCP_SERVER_MODE: http
      MCP_SERVER_URL: http://mcp-server:3004
      AUTH_MIDDLEWARE_URL: http://auth-middleware:3001

  frontend:
    build: ./frontend
    ports:
      - "3000:3000"
    environment:
      API_URL: http://api-backend:3003

volumes:
  postgres_data:
  neo4j_data:
  redis_data:
  consul_data:
```

### 3. Start Services

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

---

## Kubernetes (Production)

### 1. Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: confuse
```

### 2. Secrets

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: confuse-secrets
  namespace: confuse
type: Opaque
stringData:
  database-url: "postgresql://..."
  jwt-secret: "your-secret"
  openai-api-key: "sk-..."
```

### 3. Service Deployment (Example: api-backend)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: api-backend
  namespace: confuse
spec:
  replicas: 3
  selector:
    matchLabels:
      app: api-backend
  template:
    metadata:
      labels:
        app: api-backend
      annotations:
        prometheus.io/scrape: "true"
        prometheus.io/port: "8088"
        prometheus.io/path: "/metrics"
    spec:
      containers:
      - name: api-backend
        image: confuse/api-backend:latest
        ports:
        - containerPort: 8088
          name: http
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: confuse-secrets
              key: database-url
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: CONSUL_URL
          value: "http://consul-service:8500"
        - name: OTEL_EXPORTER_OTLP_ENDPOINT
          value: "http://jaeger-collector:4317"
        - name: OTEL_SERVICE_NAME
          value: "api-backend"
        - name: CONNECTIVITY_CIRCUIT_BREAKER_ENABLED
          value: "true"
        - name: CONNECTIVITY_RETRY_ENABLED
          value: "true"
        - name: CONNECTIVITY_TRACING_ENABLED
          value: "true"
        - name: CONNECTIVITY_CACHE_ENABLED
          value: "true"
        resources:
          requests:
            memory: "256Mi"
            cpu: "250m"
          limits:
            memory: "512Mi"
            cpu: "500m"
        livenessProbe:
          httpGet:
            path: /health
            port: 8088
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 8088
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
        startupProbe:
          httpGet:
            path: /health
            port: 8088
          initialDelaySeconds: 0
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 30
---
apiVersion: v1
kind: Service
metadata:
  name: api-backend
  namespace: confuse
spec:
  selector:
    app: api-backend
  ports:
  - port: 8088
    targetPort: 8088
    name: http
  type: ClusterIP
```

### 4. Ingress

```yaml
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: confuse-ingress
  namespace: confuse
  annotations:
    kubernetes.io/ingress.class: nginx
    cert-manager.io/cluster-issuer: letsencrypt-prod
spec:
  tls:
  - hosts:
    - api.confuse.io
    secretName: confuse-tls
  rules:
  - host: api.confuse.io
    http:
      paths:
      - path: /
        pathType: Prefix
        backend:
          service:
            name: api-backend
            port:
              number: 3003
      - path: /mcp
        pathType: Prefix
        backend:
          service:
            name: client-connector
            port:
              number: 8095
```

---

## Environment Variables

### Core Variables (All Services)

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection | `postgresql://user:pass@host:5432/db` |
| `REDIS_URL` | Redis connection | `redis://localhost:6379` |
| `CONSUL_URL` | Consul service discovery | `http://localhost:8500` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OpenTelemetry collector | `http://jaeger:4317` |
| `OTEL_SERVICE_NAME` | Service name for tracing | `api-backend` |
| `LOG_LEVEL` | Logging level | `info`, `debug`, `error` |

### Connectivity Infrastructure Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CONNECTIVITY_CIRCUIT_BREAKER_ENABLED` | Enable circuit breakers | `true` |
| `CONNECTIVITY_RETRY_ENABLED` | Enable retry logic | `true` |
| `CONNECTIVITY_TRACING_ENABLED` | Enable distributed tracing | `true` |
| `CONNECTIVITY_CACHE_ENABLED` | Enable distributed caching | `true` |
| `CONNECTIVITY_CIRCUIT_BREAKER_THRESHOLD` | Failure rate threshold (0.0-1.0) | `0.5` |
| `CONNECTIVITY_CIRCUIT_BREAKER_MIN_CALLS` | Min calls before tripping | `10` |
| `CONNECTIVITY_CIRCUIT_BREAKER_TIMEOUT_MS` | Open state timeout | `30000` |
| `CONNECTIVITY_RETRY_MAX_ATTEMPTS` | Max retry attempts | `3` |
| `CONNECTIVITY_RETRY_INITIAL_DELAY_MS` | Initial retry delay | `100` |
| `CONNECTIVITY_CACHE_TTL_SECONDS` | Default cache TTL | `300` |

### Service-Specific

See each service's `README.md` for specific environment variables.

---

## Monitoring

### Health Checks

All services expose comprehensive health endpoints:

```bash
# Basic health check
curl http://localhost:8088/health

# Detailed health with dependencies
curl http://localhost:8088/health/detailed

# Readiness check (Kubernetes)
curl http://localhost:8088/health/ready

# Liveness check (Kubernetes)
curl http://localhost:8088/health/live
```

**Health Check Response:**
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "uptime_seconds": 3600,
  "checks": {
    "database": {
      "status": "healthy",
      "latency_ms": 5,
      "message": "Connected"
    },
    "redis": {
      "status": "healthy",
      "latency_ms": 2,
      "message": "Connected"
    },
    "consul": {
      "status": "healthy",
      "latency_ms": 3,
      "message": "Registered"
    },
    "auth_service": {
      "status": "healthy",
      "latency_ms": 15,
      "circuit_breaker": "closed"
    }
  }
}
```

### Circuit Breaker Monitoring

Monitor circuit breaker states via health endpoints:

```bash
# Check circuit breaker status
curl http://localhost:8088/health/detailed | jq '.checks[].circuit_breaker'
```

**States:**
- `closed`: Normal operation
- `open`: Service failing, requests rejected
- `half_open`: Testing recovery

### Prometheus Metrics

Services expose metrics at `/metrics`:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'confuse'
    static_configs:
      - targets:
        - 'api-backend:8088'
        - 'auth-middleware:3010'
        - 'data-connector:8080'
    
  - job_name: 'consul'
    static_configs:
      - targets:
        - 'consul:8500'
```

**Key Metrics:**
- `http_requests_total` - Total HTTP requests
- `http_request_duration_seconds` - Request latency
- `circuit_breaker_state` - Circuit breaker state (0=closed, 1=open, 2=half-open)
- `circuit_breaker_failures_total` - Total failures per service
- `cache_hits_total` - Cache hit count
- `cache_misses_total` - Cache miss count
- `service_health_status` - Service health (0=unhealthy, 1=healthy)

### Distributed Tracing

Access Jaeger UI for distributed tracing:

```bash
# Open Jaeger UI
open http://localhost:16686

# Search for traces by service
# Filter by operation, tags, duration
```

**Trace Information:**
- End-to-end request flow across services
- Service dependencies and call graphs
- Latency breakdown per service
- Error propagation and root cause

### Grafana Dashboards

Import dashboards from `infrastructure/grafana/` folder:

1. **ConFuse Overview** - System-wide metrics
2. **Service Health** - Per-service health and performance
3. **Circuit Breakers** - Resilience patterns monitoring
4. **Cache Performance** - Cache hit rates and latency
5. **Distributed Tracing** - Jaeger integration

---

## Scaling Recommendations

| Service | Scaling Strategy | Notes |
|---------|------------------|-------|
| api-backend | Horizontal (3+ replicas) | Stateless, circuit breakers per instance |
| data-connector | Horizontal (based on webhook volume) | Queue-based processing |
| chunker | Horizontal (CPU-bound) | Parallel processing |
| embeddings | Horizontal (GPU if using local models) | Consider GPU nodes |
| relation-graph | Vertical (Neo4j is single-master) | Scale Neo4j separately |
| mcp-server | Per-user (1 instance per active user) | Session-based |
| consul | 3-5 nodes (HA cluster) | Odd number for quorum |
| redis | Master-replica with Sentinel | For HA caching |

---

## Backup Strategy

### PostgreSQL

```bash
# Daily backup
pg_dump -h localhost -U confuse confuse > backup.sql

# Restore
psql -h localhost -U confuse confuse < backup.sql
```

### Neo4j

```bash
# Stop Neo4j first
neo4j-admin dump --to=/backup/neo4j-backup.dump

# Restore
neo4j-admin load --from=/backup/neo4j-backup.dump
```

### Zilliz

Use Zilliz Cloud's built-in backup or:
```python
from pymilvus import utility
utility.do_bulk_insert(collection_name, files)
```

---

## Troubleshooting

### Common Issues

1. **Database connection failed**
   - Check `DATABASE_URL` format
   - Verify PostgreSQL is running
   - Check firewall rules
   - Review health endpoint: `curl http://localhost:8088/health/detailed`

2. **Neo4j connection refused**
   - Default port is 7687 (bolt)
   - Check credentials
   - Verify Neo4j is started

3. **Embeddings failing**
   - Check API key is valid
   - Verify rate limits
   - Check provider status

4. **Circuit breaker stuck open**
   - Check downstream service health
   - Review failure logs
   - Circuit breaker auto-recovers after 30s
   - Monitor via: `curl http://localhost:8088/health/detailed | jq '.checks[].circuit_breaker'`

5. **Service discovery issues**
   - Verify Consul is running: `curl http://localhost:8500/v1/status/leader`
   - Check service registration: `curl http://localhost:8500/v1/catalog/services`
   - Review Consul logs for registration errors
   - Ensure `CONSUL_URL` is correctly configured

6. **High latency / Slow responses**
   - Check distributed traces in Jaeger: `http://localhost:16686`
   - Review cache hit rates in metrics
   - Monitor circuit breaker states
   - Check database connection pool exhaustion

7. **Cache inconsistency**
   - Verify Redis is running: `redis-cli ping`
   - Check Redis memory usage: `redis-cli info memory`
   - Review cache TTL settings
   - Consider cache invalidation strategy

8. **Tracing not working**
   - Verify Jaeger is running: `curl http://localhost:16686`
   - Check `OTEL_EXPORTER_OTLP_ENDPOINT` configuration
   - Ensure `CONNECTIVITY_TRACING_ENABLED=true`
   - Review OTLP collector logs

### Logs

```bash
# Docker
docker logs <container-name>
docker logs -f api-backend  # Follow logs

# Kubernetes
kubectl logs -n confuse deployment/api-backend
kubectl logs -n confuse deployment/api-backend --previous  # Previous container
kubectl logs -n confuse -l app=api-backend --tail=100  # All pods

# Filter for errors
kubectl logs -n confuse deployment/api-backend | grep ERROR

# Check circuit breaker events
kubectl logs -n confuse deployment/api-backend | grep "circuit_breaker"
```

### Debug Mode

Enable debug logging for troubleshooting:

```bash
# Environment variable
export LOG_LEVEL=debug
export RUST_LOG=api_backend=debug,confuse_connectivity=debug

# Or in docker-compose.yml
environment:
  LOG_LEVEL: debug
  RUST_LOG: api_backend=debug,confuse_connectivity=debug
```

---

## Security Checklist

- [ ] Change default passwords
- [ ] Enable TLS/SSL for all services
- [ ] Configure firewall rules
- [ ] Set up API key rotation
- [ ] Enable audit logging
- [ ] Configure rate limiting
- [ ] Set up backup schedule
- [ ] Enable monitoring alerts
- [ ] Secure Consul with ACLs
- [ ] Enable Redis authentication
- [ ] Configure network policies (Kubernetes)
- [ ] Set up secrets management (Vault/K8s Secrets)
- [ ] Enable RBAC for service-to-service auth
- [ ] Configure CORS policies
- [ ] Set up WAF rules (if using cloud)
- [ ] Enable distributed tracing for security audits
