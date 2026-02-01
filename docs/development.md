 # API Backend Development Guide (Rust)

> Setting up your local development environment for the ConFuse API Backend implemented in Rust (Axum).

## Prerequisites

 - **Rust** 1.75+ (install via rustup)
 - **Cargo** (comes with Rust)
 - **PostgreSQL** 14+ (local or Docker)
 - **Redis** 7+ (local or Docker)
 - **sqlx-cli** (optional, for migrations): `cargo install sqlx-cli --no-default-features --features postgres`
 - **cargo-watch** (optional, for hot reload): `cargo install cargo-watch`
 - **Git**

## Initial Setup

### 1. Clone and build

```bash
 # Clone the repository
 git clone https://github.com/confuse/api-backend.git
 cd api-backend

 # Build the project
 cargo build

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

 Install PostgreSQL and Redis via your OS package manager and start the services.

Create the database:

```bash
 # Connect to PostgreSQL and create DB/user
 psql -U postgres
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

If using `sqlx` migrations:

```bash
 # Ensure sqlx-cli is installed and DATABASE_URL is set
 sqlx migrate run
```

If using a different migration tool, follow that tool's commands.

### 5. Start Development Server

```bash
 # Development (with hot reload)
 # Install cargo-watch once: cargo install cargo-watch
 cargo watch -x 'run'

 # Or run once
 cargo run

 # Server runs at http://localhost:3003 by default
```

## Project Structure (Rust)

```
 api-backend/
 ├── src/
 │   ├── main.rs              # Entry point
 │   ├── lib.rs               # Optional library code
 │   ├── config/              # Configuration
 │   │   └── mod.rs
 │   ├── routes/              # Route definitions
 │   │   ├── mod.rs
 │   │   └── v1/
 │   │       ├── mod.rs
 │   │       ├── sources.rs
 │   │       ├── search.rs
 │   │       └── entities.rs
 │   ├── handlers/            # Request handlers
 │   ├── services/            # Business logic
 │   ├── db/                  # Database layer (sqlx/sea-orm)
 │   ├── models/              # Domain models
 │   └── utils/               # Utilities (logging, errors)
 ├── migrations/              # sqlx or other migrations
 ├── tests/                   # Integration tests
 ├── Dockerfile
 ├── Cargo.toml
 └── README.md
```

## Development Commands

```bash
 # Build
 cargo build

 # Run (development)
 cargo run

 # Run with hot reload (requires cargo-watch)
 cargo watch -x 'run'

 # Run tests
 cargo test

 # Format code
 cargo fmt

 # Lint (clippy)
 cargo clippy --all-targets --all-features -- -D warnings

 # Database migrations (sqlx)
 sqlx migrate run
```

## Running with Other Services

For full functionality, you need other ConFuse services running. Example commands:

```bash
 # Start Data Connector (Python)
 cd ../data-connector
 uvicorn app.main:app --reload --port 8000

 # Start Client Connector (Python)
 cd ../client-connector
 python -m app.main

 # Start Relation Graph (Rust)
 cd ../relation-graph
 cargo run

 # Start Code Normalize Fetch (Rust)
 cd ../code-normalize-fetch
 cargo run
```

## Testing

Use `cargo test` for unit and integration tests. Integration tests that require external services should be run with those services available (Postgres, Redis, Neo4j, Zilliz).

## Debugging

Use VS Code with the `rust-analyzer` and `CodeLLDB` extensions for debugging. A simple launch configuration can invoke `cargo run`.

## Common Development Tasks

### Add a new API endpoint

1. Add handler in `src/routes/v1/` (e.g., `widgets.rs`) and export in `mod.rs`.
2. Implement business logic in `src/services/`.
3. Add DB migrations under `migrations/` if schema changes are required.
4. Run `cargo test` and `cargo fmt`.

### Add a new service client

1. Create a new module under `src/clients/` using `reqwest` for HTTP calls.
2. Add configuration to `src/config/mod.rs` and `.env.example`.
3. Write unit tests and update documentation.

**Available Service Clients:**
- `AuthClient` - Authentication service integration
- `DataConnectorClient` - Source integration and data ingestion
- `RelationGraphClient` - Knowledge graph operations
- `McpClient` - AI agent protocol communication
- `UnifiedProcessorClient` - Hybrid document/code processing

**Note:** `EnhancedGraphClient` is a deprecated type alias for `RelationGraphClient`. Use `RelationGraphClient` in new code for clarity.

## Code Style

- `cargo fmt` for formatting
- `cargo clippy` for linting
- Write unit tests and integration tests with `#[cfg(test)]` and `cargo test`

