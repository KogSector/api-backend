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
| Redis | Caching, sessions | Local, ElastiCache |
| Neo4j | Knowledge graph | AuraDB, Self-hosted |
| Zilliz/Milvus | Vector storage | Zilliz Cloud, Self-hosted |

### Optional Services

| Service | Purpose |
|---------|---------|
| Elasticsearch | Full-text search (optional) |
| Prometheus | Metrics collection |
| Grafana | Monitoring dashboards |

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
      - "3003:3003"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse
      REDIS_URL: redis://redis:6379
      AUTH_SERVICE_URL: http://auth-middleware:3001
    depends_on:
      - postgres
      - redis

  auth-middleware:
    build: ./auth-middleware
    ports:
      - "3001:3001"
    environment:
      DATABASE_URL: postgresql://confuse:confuse_password@postgres:5432/confuse
      JWT_SECRET: your-jwt-secret-here
      REDIS_URL: redis://redis:6379

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
    spec:
      containers:
      - name: api-backend
        image: confuse/api-backend:latest
        ports:
        - containerPort: 3003
        env:
        - name: DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: confuse-secrets
              key: database-url
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
            port: 3003
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 3003
          initialDelaySeconds: 5
          periodSeconds: 5
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
  - port: 3003
    targetPort: 3003
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
| `LOG_LEVEL` | Logging level | `info`, `debug`, `error` |

### Service-Specific

See each service's `README.md` for specific environment variables.

---

## Monitoring

### Health Checks

All services expose `/health` endpoint:

```bash
curl http://localhost:3003/health
```

### Prometheus Metrics

Services expose metrics at `/metrics`:

```yaml
# prometheus.yml
scrape_configs:
  - job_name: 'confuse'
    static_configs:
      - targets:
        - 'api-backend:3003'
        - 'chunker:3002'
        - 'embeddings:3005'
```

### Grafana Dashboards

Import dashboards from `infrastructure/grafana/` folder.

---

## Scaling Recommendations

| Service | Scaling Strategy |
|---------|------------------|
| api-backend | Horizontal (3+ replicas) |
| data-connector | Horizontal (based on webhook volume) |
| chunker | Horizontal (CPU-bound) |
| embeddings | Horizontal (GPU if using local models) |
| relation-graph | Vertical (Neo4j is single-master) |
| mcp-server | Per-user (1 instance per active user) |

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

2. **Neo4j connection refused**
   - Default port is 7687 (bolt)
   - Check credentials
   - Verify Neo4j is started

3. **Embeddings failing**
   - Check API key is valid
   - Verify rate limits
   - Check provider status

### Logs

```bash
# Docker
docker logs <container-name>

# Kubernetes
kubectl logs -n confuse deployment/api-backend
```

---

## Security Checklist

- [ ] Change default passwords
- [ ] Enable TLS/SSL
- [ ] Configure firewall rules
- [ ] Set up API key rotation
- [ ] Enable audit logging
- [ ] Configure rate limiting
- [ ] Set up backup schedule
- [ ] Enable monitoring alerts
