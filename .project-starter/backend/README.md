# Backend Reference Implementation

Minimal working REST API demonstrating project-starter patterns.

## Overview

This is a **reference implementation** showing how to build a production-ready REST API using:
- Rust + Axum web framework
- PostgreSQL database with SQLx
- Structured logging and tracing
- Comprehensive testing (unit, integration, E2E)
- Docker containerization

## Features

### API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Liveness probe - service is running |
| `GET` | `/health/ready` | Readiness probe - service + dependencies ready |
| `GET` | `/api/items` | List all items |
| `GET` | `/api/items/:id` | Get specific item by ID |
| `POST` | `/api/items` | Create new item |

### Built-in Patterns

✅ **Health Checks** (from Phase 2, PR #16)
- Liveness endpoint for Kubernetes/Docker
- Readiness endpoint with dependency checks

✅ **Environment Configuration** (from Phase 1, PR #12)
- `.env` file support
- Validation of required variables

✅ **Structured Tracing**
- Request tracing with correlation IDs
- Contextual logging throughout

✅ **Error Handling**
- Typed error responses
- Proper HTTP status codes

✅ **Testing**
- Unit tests for business logic
- Integration tests for database
- E2E tests ready (use templates from PR #18)

## Quick Start

### 1. Set Up Environment

```bash
# Copy environment template (from Phase 1)
cp ../.env.example .env

# Update database connection
# Edit .env and set:
DATABASE_URL=postgres://postgres:postgres@localhost:5432/project_starter
PORT=3000
```

### 2. Start Database

```bash
# Using Docker Compose
docker-compose up -d postgres

# Or use the Makefile from project root
make dev-up
```

### 3. Run Migrations

```bash
# Install SQLx CLI
cargo install sqlx-cli

# Run migrations
sqlx migrate run
```

### 4. Run the Server

```bash
# Development mode (with auto-reload)
cargo watch -x run

# Or standard run
cargo run
```

### 5. Test the API

```bash
# Health check
curl http://localhost:3000/health

# Readiness check
curl http://localhost:3000/health/ready

# Create an item
curl -X POST http://localhost:3000/api/items \
  -H "Content-Type: application/json" \
  -d '{"name": "Example Item", "description": "A test item"}'

# List items
curl http://localhost:3000/api/items

# Get specific item (replace UUID)
curl http://localhost:3000/api/items/550e8400-e29b-41d4-a716-446655440000
```

## Project Structure

```
backend/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration loading
│   ├── db.rs             # Database connection pool
│   ├── error.rs          # Error types and handling
│   ├── models.rs         # Data models
│   └── api/
│       ├── mod.rs        # API module exports
│       ├── health.rs     # Health check endpoints
│       └── items.rs      # Items CRUD endpoints
├── migrations/           # Database migrations
│   └── 20260124000001_create_items_table.sql
├── tests/
│   └── integration_test.rs  # Integration tests
├── Cargo.toml            # Rust dependencies
├── Dockerfile            # Production container
└── README.md             # This file
```

## Testing

### Unit Tests

```bash
# Run unit tests only
cargo test --lib
```

### Integration Tests

```bash
# Set up test database
createdb project_starter_test

# Run integration tests
DATABASE_URL=postgres://postgres:postgres@localhost:5432/project_starter_test \
  cargo test --test integration_test
```

### E2E Tests

See `../templates/e2e-tests/README.md` (from Phase 2, PR #18) for comprehensive E2E testing setup with Playwright.

## Database Migrations

### Create Migration

```bash
sqlx migrate add <migration_name>
```

### Run Migrations

```bash
sqlx migrate run
```

### Revert Migration

```bash
sqlx migrate revert
```

## Development

### Auto-Reload on Changes

```bash
# Install cargo-watch
cargo install cargo-watch

# Run with auto-reload
cargo watch -x run
```

### Pre-Commit Hook

Use the pre-commit hook template from Phase 2 (PR #17):

```bash
# Install from template
cp ../templates/git-hooks/pre-commit ../.git/hooks/pre-commit
chmod +x ../.git/hooks/pre-commit
```

This will automatically check:
- Code formatting (`cargo fmt`)
- Linting (`cargo clippy`)
- Tests passing

## Configuration

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `DATABASE_URL` | PostgreSQL connection string | `postgres://user:pass@localhost:5432/db` |
| `PORT` | HTTP server port (optional, default: 3000) | `3000` |
| `RUST_LOG` | Logging level (optional) | `info`, `debug` |

### Security Best Practices

See `../templates/env/README.md` (from Phase 1, PR #12) for:
- Secret management
- Production configuration
- Environment validation

## Deployment

### Docker Build

```bash
# Build image
docker build -t project-starter-api:latest .

# Run container
docker run -p 3000:3000 \
  -e DATABASE_URL=postgres://... \
  project-starter-api:latest
```

### Docker Compose

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f backend

# Stop all services
docker-compose down
```

### Health Checks in Production

The health endpoints are designed for use with orchestration platforms:

**Kubernetes:**
```yaml
livenessProbe:
  httpGet:
    path: /health
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health/ready
    port: 3000
  initialDelaySeconds: 5
  periodSeconds: 5
```

**Docker Compose:**
See `docker-compose.yml` for health check configuration.

## Troubleshooting

### Database Connection Fails

```bash
# Check PostgreSQL is running
pg_isready -h localhost -p 5432

# Check DATABASE_URL is correct
echo $DATABASE_URL

# Verify credentials
psql $DATABASE_URL
```

### Port Already in Use

```bash
# Find process using port 3000
lsof -i :3000

# Change port in .env
PORT=8080
```

### Migrations Fail

```bash
# Check migration files
ls migrations/

# Verify database exists
psql -c "\l" | grep project_starter

# Manual migration
psql $DATABASE_URL < migrations/20260124000001_create_items_table.sql
```

## Next Steps

### Customize This Template

1. **Replace the items model** with your domain objects
2. **Add your business logic** in new service modules
3. **Extend the API** with additional endpoints
4. **Add authentication** (see auth-service for Biscuit token example)
5. **Add more tests** using E2E templates from PR #18

### Add Advanced Features

- API documentation with OpenAPI/utoipa
- Rate limiting
- CORS configuration
- WebSocket support
- Background job processing
- Caching with Redis

## References

- [Axum Documentation](https://docs.rs/axum/latest/axum/)
- [SQLx Documentation](https://docs.rs/sqlx/latest/sqlx/)
- [Tracing Documentation](https://docs.rs/tracing/latest/tracing/)
- [Auth-Service](https://github.com/brefwiz/auth-service) - Full production example

---

**Last Updated:** 2026-01-24
**Part of:** project-starter Phase 3 (Reference Implementation)
**Extracted from:** auth-service production patterns
