*** Begin Patch
*** End Patch

### Option 1: Docker Compose (Full Stack)

```bash
# From project root
cd ..
docker-compose up -d
```

### Option 2: Run Individual Services

Terminal 1 - Auth Middleware:
```bash
cd ../auth-middleware
npm run dev
```

Terminal 2 - Data Connector:
```bash
cd ../data-connector
uvicorn app.main:app --reload --port 8000
```

Terminal 3 - Relation Graph:
```bash
cd ../relation-graph
cargo run
```

Terminal 4 - API Backend:
```bash
cd api-backend
npm run dev
```

## Testing

### Unit Tests

```bash
# Run all unit tests
npm run test:unit

# Run specific test file
npm run test:unit -- routes/search.test.ts

# Watch mode
npm run test:unit -- --watch
```

### Integration Tests

```bash
# Requires database running
npm run test:integration
```

### Test Structure

```typescript
// tests/unit/routes/search.test.ts
import { describe, it, expect, beforeEach, vi } from 'vitest';
import request from 'supertest';
import app from '../../../src/app';

describe('Search Routes', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('POST /v1/search', () => {
    it('should return search results', async () => {
      const response = await request(app)
        .post('/v1/search')
        .set('Authorization', 'Bearer valid-token')
        .send({ query: 'authentication' });

      expect(response.status).toBe(200);
      expect(response.body.results).toBeDefined();
    });

    it('should require authentication', async () => {
      const response = await request(app)
        .post('/v1/search')
        .send({ query: 'test' });

      expect(response.status).toBe(401);
    });
  });
});
```

## Debugging

### VS Code Configuration

```json
// .vscode/launch.json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "node",
      "request": "launch",
      "name": "Debug API Backend",
      "runtimeExecutable": "npm",
      "runtimeArgs": ["run", "dev"],
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    },
    {
      "type": "node",
      "request": "launch",
      "name": "Debug Tests",
      "runtimeExecutable": "npm",
      "runtimeArgs": ["run", "test", "--", "--run"],
      "cwd": "${workspaceFolder}",
      "console": "integratedTerminal"
    }
  ]
}
```

### Logging

```typescript
import { logger } from './utils/logger';

// Different log levels
logger.debug('Detailed debug info', { data: someObject });
logger.info('General information');
logger.warn('Warning message');
logger.error('Error occurred', { error: err });
```

Set `LOG_LEVEL=debug` in `.env` for verbose logging.

## Common Development Tasks

### Adding a New API Endpoint

1. Create route handler in `src/routes/v1/`:

```typescript
// src/routes/v1/widgets.ts
import { Router } from 'express';
import { requireAuth } from '../../middleware/auth';
import { validate } from '../../middleware/validation';
import { widgetSchema } from '../../schemas/widget';

const router = Router();

router.get('/', requireAuth, async (req, res) => {
  // Implementation
});

router.post('/', requireAuth, validate(widgetSchema), async (req, res) => {
  // Implementation
});

export default router;
```

2. Register in `src/routes/v1/index.ts`:

```typescript
import widgetRoutes from './widgets';
router.use('/widgets', widgetRoutes);
```

### Adding a New Service Client

1. Create client in `src/clients/`:

```typescript
// src/clients/new-service-client.ts
import axios from 'axios';
import { handleServiceCall } from './base-client';

const client = axios.create({
  baseURL: process.env.NEW_SERVICE_URL,
  timeout: 10000,
});

export const newService = {
  async doSomething(params: Params): Promise<Result> {
    return handleServiceCall('new-service', async () => {
      const response = await client.post('/endpoint', params);
      return response.data;
    });
  },
};
```

2. Add environment variable to `.env.example`:

```env
NEW_SERVICE_URL=http://localhost:XXXX
```

## Code Style

- **ESLint** for linting
- **Prettier** for formatting
- **TypeScript** strict mode enabled

```bash
# Format code
npm run format

# Check formatting
npm run format:check
```
