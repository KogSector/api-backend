# API Backend Troubleshooting

> Common issues and solutions for the ConFuse API Backend

## Quick Diagnostics

### Check Service Health

```bash
curl http://localhost:3003/health
```

Expected response:
```json
{
  "status": "healthy",
  "services": {
    "database": { "status": "connected" },
    "redis": { "status": "connected" },
    "auth-middleware": { "status": "healthy" }
  }
}
```

### Check Logs

```bash
# If running with cargo
cargo run 2>&1 | grep -i error

# If running in Docker
docker logs api-backend --tail 100
```

## Common Issues

### 1. Database Connection Failed

**Symptoms:**
```
Error: connect ECONNREFUSED 127.0.0.1:5432
```

**Solutions:**

1. Verify PostgreSQL is running:
```bash
# Check status
pg_isready -h localhost -p 5432

# Start if needed
# macOS
brew services start postgresql
# Linux
sudo systemctl start postgresql
```

2. Check connection string:
```bash
# Test connection manually
psql -h localhost -U confuse -d confuse
```

3. Verify database exists:
```sql
-- Connect as postgres user
psql -U postgres
-- List databases
\l
-- Create if missing
CREATE DATABASE confuse;
```

### 2. Redis Connection Failed

**Symptoms:**
```
Error: Redis connection to localhost:6379 failed
```

**Solutions:**

1. Verify Redis is running:
```bash
redis-cli ping  # Should return PONG
```

2. Start Redis:
```bash
# macOS
brew services start redis
# Linux
sudo systemctl start redis
```

3. Check if Redis is password protected:
```env
# Update .env if needed
REDIS_URL=redis://:password@localhost:6379
```

### 3. Auth Middleware Unavailable

**Symptoms:**
```
ServiceUnavailableError: auth-middleware is unavailable
```

**Solutions:**

1. Check if auth-middleware is running:
```bash
curl http://localhost:3001/health
```

2. Start auth-middleware:
```text
Start the auth-middleware service as described in its repository README (../auth-middleware/README.md).
```

3. Verify URL in environment:
```env
AUTH_MIDDLEWARE_URL=http://localhost:3001
```

### 4. 401 Unauthorized on All Requests

**Symptoms:**
- All authenticated endpoints return 401
- Token appears valid

**Solutions:**

1. Verify JWT_SECRET matches auth-middleware:
```bash
# Both services must use same secret
grep JWT_SECRET .env
grep JWT_SECRET ../auth-middleware/.env
```

2. Check token format:
```bash
# Token should be: Bearer <jwt>
curl -H "Authorization: Bearer eyJhbG..." http://localhost:3003/v1/sources
```

3. Verify token hasn't expired:
```bash
# Decode JWT (https://jwt.io)
# Check "exp" claim
```

### 5. 429 Too Many Requests

**Symptoms:**
```json
{ "error": "Rate limit exceeded", "retryAfter": 60 }
```

**Solutions:**

1. Wait for rate limit window to reset

2. Increase rate limits for development:
```env
RATE_LIMIT_MAX_REQUESTS=1000
```

3. Disable rate limiting (dev only):
```env
RATE_LIMIT_ENABLED=false
```

### 6. Migration Failures

**Symptoms:**
```
Error: Migration "20240101_create_users" failed
```

**Solutions:**

1. Check migration status:
```bash
# Using sqlx CLI
sqlx migrate info
```

2. Rollback and retry:
```bash
# Revert last migration (sqlx)
sqlx migrate revert

# Run migrations
sqlx migrate run
```

3. Check for pending transactions:
```sql
-- Connect to database
psql -U confuse confuse
-- Check for locks
SELECT * FROM pg_locks WHERE NOT granted;
```

### 7. Rust Compilation Errors

**Symptoms:**
```
error[E0425]: cannot find value `x` in this scope
```

**Solutions:**

1. Run a full build to see errors:
```bash
cargo build
```

2. Run with verbose output:
```bash
cargo build -v
```

3. Use `rust-analyzer` in your IDE for better diagnostics and quick fixes

### 8. Memory Issues / Slow Performance

**Symptoms:**
- High memory usage
- Slow response times
- OOM errors

**Solutions:**

1. Check service metrics and logs to identify hotspots.

2. Profile the service with `perf`, `pprof`, or Rust profiling tools (e.g., `cargo-flamegraph`).

3. Review database queries and add indexes where needed; enable query logging in Postgres to trace slow queries.

4. Increase available resources (CPU/memory) for the container or VM during heavy loads.

### 9. CORS Errors

**Symptoms:**
```
Access to fetch at 'http://localhost:3003' from origin 'http://localhost:3000' 
has been blocked by CORS policy
```

**Solutions:**

1. Update CORS_ORIGINS in .env:
```env
CORS_ORIGINS=http://localhost:3000,http://localhost:8080
```

2. For development, allow all:
```env
CORS_ORIGINS=*
```

### 10. Service Client Timeouts

**Symptoms:**
```
Error: timeout of 10000ms exceeded
```

**Solutions:**

1. Check if target service is healthy:
```bash
curl http://localhost:3018/health  # relation-graph
```

2. Increase timeout for slow operations:
```env
SERVICE_TIMEOUT_MS=60000
```

3. Check network connectivity:
```bash
# In Docker
docker exec api-backend ping relation-graph
```

## Debug Mode

Enable verbose logging:

```env
LOG_LEVEL=debug
DEBUG=*
```

This will output:
- All HTTP requests/responses
- Database queries
- Service client calls
- Cache operations

## Getting Help

1. Check existing issues: https://github.com/confuse/api-backend/issues
2. Review logs with DEBUG/RUST_LOG enabled
3. Create issue with:
  - Error message
  - Steps to reproduce
  - Environment (OS, Rust toolchain version)
  - Relevant log output

## Quick Fixes Checklist

- [ ] Is the database running?
- [ ] Is Redis running?
- [ ] Are environment variables set correctly?
- [ ] Are other required services running?
- [ ] Is the JWT secret consistent across services?
- [ ] Have you run migrations?
- [ ] Is the correct Rust toolchain installed (`rustup show`)?
