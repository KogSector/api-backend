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
# If running with npm
npm run dev 2>&1 | grep -i error

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
```bash
cd ../auth-middleware
npm run dev
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
npm run migrate:status
```

2. Rollback and retry:
```bash
npm run migrate:rollback
npm run migrate
```

3. Check for pending transactions:
```sql
-- Connect to database
psql -U confuse confuse
-- Check for locks
SELECT * FROM pg_locks WHERE NOT granted;
```

### 7. TypeScript Compilation Errors

**Symptoms:**
```
error TS2339: Property 'x' does not exist on type 'y'
```

**Solutions:**

1. Rebuild node_modules:
```bash
rm -rf node_modules
npm install
```

2. Rebuild TypeScript:
```bash
npm run build -- --clean
```

3. Restart TypeScript server in IDE

### 8. Memory Issues / Slow Performance

**Symptoms:**
- High memory usage
- Slow response times
- OOM errors

**Solutions:**

1. Increase Node.js memory:
```bash
NODE_OPTIONS="--max-old-space-size=4096" npm run dev
```

2. Check for memory leaks:
```bash
npm run dev -- --inspect
# Open chrome://inspect
```

3. Review database queries:
```typescript
// Enable query logging
DATABASE_LOG_QUERIES=true
```

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
2. Review logs with DEBUG enabled
3. Create issue with:
   - Error message
   - Steps to reproduce
   - Environment (OS, Node version)
   - Relevant log output

## Quick Fixes Checklist

- [ ] Is the database running?
- [ ] Is Redis running?
- [ ] Are environment variables set correctly?
- [ ] Are other required services running?
- [ ] Is the JWT secret consistent across services?
- [ ] Have you run migrations?
- [ ] Is the correct Node.js version installed?
- [ ] Have you tried `rm -rf node_modules && npm install`?
