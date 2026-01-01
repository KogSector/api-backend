# API Backend Development Guide

> Setting up your local development environment for ConFuse API Backend

## Prerequisites

- **Node.js** 18+ (recommend using nvm)
- **npm** 9+ or **yarn** 1.22+
- **PostgreSQL** 14+ (local or Docker)
- **Redis** 7+ (local or Docker)
- **Git**

## Initial Setup

### 1. Clone and Install

```bash
# Clone the repository
git clone https://github.com/confuse/api-backend.git
cd api-backend

# Install dependencies
npm install

# Copy environment file
cp .env.example .env
```

### 2. Database Setup

**Option A: Docker (Recommended)**

```bash
# Start PostgreSQL and Redis
docker-compose -f docker-compose.dev.yml up -d

# This starts:
# - PostgreSQL on localhost:5432
# - Redis on localhost:6379
```

**Option B: Local Installation**

```bash
# macOS
brew install postgresql redis
brew services start postgresql
brew services start redis

# Ubuntu
sudo apt install postgresql redis-server
sudo systemctl start postgresql redis
```

Create the database:

```bash
# Connect to PostgreSQL
psql -U postgres

# Create database and user
CREATE USER confuse WITH PASSWORD 'confuse_password';
CREATE DATABASE confuse OWNER confuse;
\q
```

### 3. Configure Environment

Edit `.env`:

```env
# Database
DATABASE_URL=postgresql://confuse:confuse_password@localhost:5432/confuse

# Redis
REDIS_URL=redis://localhost:6379

# Other services (use defaults for local dev)
AUTH_MIDDLEWARE_URL=http://localhost:3001
DATA_CONNECTOR_URL=http://localhost:8000
RELATION_GRAPH_URL=http://localhost:3018

# JWT (use any string for local dev)
JWT_SECRET=local-development-secret-at-least-32-chars
```

### 4. Run Migrations

```bash
# Run database migrations
npm run migrate

# Seed with test data (optional)
npm run seed
```

### 5. Start Development Server

```bash
# Start with hot reload
npm run dev

# Server runs at http://localhost:3003
```

## Project Structure

```
api-backend/
├── src/
│   ├── index.ts              # Entry point
│   ├── app.ts                # Express app setup
│   ├── config/               # Configuration
│   │   ├── index.ts
│   │   └── database.ts
│   ├── routes/               # API routes
│   │   ├── v1/
│   │   │   ├── index.ts
│   │   │   ├── sources.ts
│   │   │   ├── search.ts
│   │   │   └── entities.ts
│   │   └── health.ts
│   ├── middleware/           # Express middleware
│   │   ├── auth.ts
│   │   ├── rateLimit.ts
│   │   ├── validation.ts
│   │   └── errorHandler.ts
│   ├── clients/              # Service clients
│   │   ├── auth-client.ts
│   │   ├── data-connector-client.ts
│   │   └── relation-graph-client.ts
│   ├── services/             # Business logic
│   │   ├── search.ts
│   │   ├── sources.ts
│   │   └── entities.ts
│   ├── models/               # Database models
│   │   ├── User.ts
│   │   └── ApiKey.ts
│   ├── types/                # TypeScript types
│   │   ├── api.ts
│   │   └── services.ts
│   └── utils/                # Utilities
│       ├── logger.ts
│       └── errors.ts
├── tests/                    # Test files
│   ├── unit/
│   ├── integration/
│   └── fixtures/
├── migrations/               # Database migrations
├── docs/                     # Documentation
├── docker-compose.dev.yml    # Dev Docker setup
├── Dockerfile
├── package.json
└── tsconfig.json
```

## Development Commands

```bash
# Start development server with hot reload
npm run dev

# Build TypeScript
npm run build

# Run built server
npm start

# Run tests
npm test

# Run tests with coverage
npm run test:coverage

# Run linter
npm run lint

# Fix linting issues
npm run lint:fix

# Type check
npm run typecheck

# Run database migrations
npm run migrate

# Rollback last migration
npm run migrate:rollback

# Create new migration
npm run migrate:create -- migration_name

# Seed database
npm run seed
```

## Running with Other Services

For full functionality, you need other ConFuse services running:

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
