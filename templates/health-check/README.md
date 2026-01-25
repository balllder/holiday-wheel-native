# Health Check Patterns

Comprehensive health check implementations for monitoring, orchestration, and load balancing.

## Overview

Health checks are critical for:
- **Kubernetes liveness probes** - Detect when to restart containers
- **Kubernetes readiness probes** - Control when to send traffic
- **Load balancer health checks** - Route traffic to healthy instances
- **Monitoring systems** - Track service availability
- **CI/CD pipelines** - Verify deployments

## Files

| Stack | File | Purpose |
|-------|------|---------|
| Rust + Axum | `rust-axum/health.rs` | Axum health check handlers |
| Python + FastAPI | `python-fastapi/health.py` | FastAPI health check router |
| Node.js + Express | `nodejs-express/health.ts` | Express health check router |

---

## Endpoints

### GET /health (Liveness Probe)

**Purpose:** Check if the service is alive and running.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "service": "my-service",
  "version": "0.1.0"
}
```

**Use for:**
- Kubernetes liveness probes
- Basic uptime monitoring
- Service discovery

**When it fails:** Service should be restarted

---

### GET /health/ready (Readiness Probe)

**Purpose:** Check if the service is ready to accept traffic.

**Response (200 OK):**
```json
{
  "status": "ready",
  "service": "my-service",
  "version": "0.1.0",
  "checks": {
    "database": "ok",
    "cache": "ok"
  }
}
```

**Response (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "service": "my-service",
  "version": "0.1.0",
  "checks": {
    "database": "error",
    "cache": "ok"
  }
}
```

**Use for:**
- Kubernetes readiness probes
- Load balancer health checks
- Deployment validation

**When it fails:** Service should NOT receive traffic (but should NOT be restarted)

---

## Integration Guide

### Rust + Axum

**1. Copy the health check module:**
```bash
cp templates/health-check/rust-axum/health.rs backend/src/health.rs
```

**2. Add to src/main.rs:**
```rust
mod health;
use health::{health_check, ready_check, AppState};

#[tokio::main]
async fn main() {
    // ... setup code ...

    let state = Arc::new(AppState {
        db_pool: pool.clone(),
    });

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/health/ready", get(ready_check))
        .with_state(state);

    // ... server code ...
}
```

**3. Add dependencies to Cargo.toml:**
```toml
[dependencies]
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
tracing = "0.1"
```

---

### Python + FastAPI

**1. Copy the health check module:**
```bash
cp templates/health-check/python-fastapi/health.py backend/app/health.py
```

**2. Add to main.py:**
```python
from fastapi import FastAPI
from app.health import router as health_router

app = FastAPI()
app.include_router(health_router)
```

**3. Add dependencies to requirements.txt:**
```
fastapi>=0.104.0
pydantic>=2.0.0
sqlalchemy[asyncio]>=2.0.0
```

---

### Node.js + Express

**1. Copy the health check module:**
```bash
cp templates/health-check/nodejs-express/health.ts backend/src/health.ts
```

**2. Add to app.ts:**
```typescript
import express from 'express';
import { Pool } from 'pg';
import { healthRouter } from './health';

const app = express();

// Initialize database pool
const dbPool = new Pool({
  connectionString: process.env.DATABASE_URL,
});

// Store in app locals
app.locals.dbPool = dbPool;

// Mount health router
app.use('/health', healthRouter);
```

**3. Add dependencies to package.json:**
```json
{
  "dependencies": {
    "express": "^4.18.0",
    "pg": "^8.11.0"
  },
  "devDependencies": {
    "@types/express": "^4.17.0",
    "@types/pg": "^8.10.0"
  }
}
```

---

## Makefile Integration

Add health check command to your Makefile:

```makefile
.PHONY: health health-ready

health: ## Check service health (liveness)
	@echo "Checking service health..."
	@curl -f http://localhost:3000/health || exit 1
	@echo "✓ Service is healthy"

health-ready: ## Check service readiness
	@echo "Checking service readiness..."
	@curl -f http://localhost:3000/health/ready || exit 1
	@echo "✓ Service is ready"
```

**Usage:**
```bash
make health        # Check liveness
make health-ready  # Check readiness (includes dependencies)
```

---

## Docker Compose Health Checks

Add health checks to your `docker-compose.yml`:

```yaml
services:
  backend:
    image: my-service:latest
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    depends_on:
      postgres:
        condition: service_healthy
      redis:
        condition: service_healthy

  postgres:
    image: postgres:16
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U ${POSTGRES_USER}"]
      interval: 10s
      timeout: 5s
      retries: 5

  redis:
    image: redis:7-alpine
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 3s
      retries: 3
```

**Benefits:**
- Services start in correct order
- Dependencies must be healthy before dependents start
- Unhealthy services are automatically restarted

---

## Kubernetes Integration

### Liveness Probe

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-service
spec:
  template:
    spec:
      containers:
      - name: my-service
        image: my-service:latest
        livenessProbe:
          httpGet:
            path: /health
            port: 3000
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
```

**Behavior:**
- Kubernetes checks `/health` every 10 seconds
- After 3 consecutive failures, container is restarted
- Initial 30-second delay allows app to start

### Readiness Probe

```yaml
        readinessProbe:
          httpGet:
            path: /health/ready
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 2
```

**Behavior:**
- Kubernetes checks `/health/ready` every 5 seconds
- After 2 failures, pod is removed from service load balancer
- Pod NOT restarted (unlike liveness probe)

---

## Monitoring Integration

### Prometheus

Health check metrics can be exposed for Prometheus:

```yaml
# Example metrics endpoint
GET /metrics

# Custom health check metric
health_check_status{endpoint="database"} 1  # 1 = healthy, 0 = unhealthy
health_check_status{endpoint="cache"} 1
```

### Grafana Dashboard

Create alerts based on health check status:
- Alert when readiness check fails for > 1 minute
- Alert when liveness check fails for > 30 seconds
- Alert when database check fails

---

## Testing Health Checks

### Manual Testing

```bash
# Test liveness
curl http://localhost:3000/health

# Test readiness
curl http://localhost:3000/health/ready

# Test readiness (verbose)
curl -v http://localhost:3000/health/ready
```

### Automated Testing (E2E)

**Rust (using reqwest):**
```rust
#[tokio::test]
async fn test_health_endpoint() {
    let response = reqwest::get("http://localhost:3000/health")
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: HealthResponse = response.json().await.unwrap();
    assert_eq!(body.status, "healthy");
}
```

**Python (using httpx):**
```python
async def test_health_endpoint():
    async with httpx.AsyncClient() as client:
        response = await client.get("http://localhost:3000/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"
```

**Node.js (using supertest):**
```typescript
import request from 'supertest';
import app from './app';

describe('Health Check', () => {
  it('should return healthy status', async () => {
    const response = await request(app).get('/health');
    expect(response.status).toBe(200);
    expect(response.body.status).toBe('healthy');
  });
});
```

---

## Best Practices

### ✅ DO

1. **Keep liveness check simple**
   - Only check if process is running
   - Don't check dependencies
   - Fast response (< 100ms)

2. **Make readiness check comprehensive**
   - Check all critical dependencies
   - Database connection
   - Cache connection
   - External API availability (if critical)

3. **Use appropriate timeouts**
   - Liveness: 1-2 seconds max
   - Readiness: 3-5 seconds max
   - Allows for temporary slowness

4. **Return proper HTTP status codes**
   - 200 OK = healthy/ready
   - 503 Service Unavailable = unhealthy/not ready
   - Never return 500 (that indicates a bug)

5. **Include version information**
   - Helps verify correct deployment
   - Useful for debugging

### ❌ DON'T

1. **Don't make liveness check too complex**
   ```rust
   // ❌ BAD: Liveness checking database
   async fn health_check(State(state): State<Arc<AppState>>) {
       state.db_pool.acquire().await.unwrap();  // Can cause restart loops!
   }

   // ✅ GOOD: Liveness just checks process
   async fn health_check() -> Json<HealthResponse> {
       Json(HealthResponse { status: "healthy" })
   }
   ```

2. **Don't use health checks for authentication**
   - Health checks should be unauthenticated
   - They need to be accessible to orchestrators

3. **Don't return sensitive information**
   ```json
   // ❌ BAD
   {
     "database": "postgres://user:password@host:5432/db"
   }

   // ✅ GOOD
   {
     "database": "ok"
   }
   ```

4. **Don't use GET requests with side effects**
   - Health checks are called frequently
   - Must be idempotent and safe

---

## Troubleshooting

### Health check always fails

**Symptom:** `/health` returns 404

**Solution:**
- Verify health check route is registered
- Check server is listening on expected port
- Verify no middleware is blocking the route

---

### Readiness check always fails

**Symptom:** `/health/ready` returns 503

**Solution:**
1. Check database connection string
2. Verify database is running and accessible
3. Check firewall/network rules
4. Examine logs for specific error

---

### Container restart loop

**Symptom:** Kubernetes keeps restarting pod

**Possible causes:**
1. Liveness probe checking dependencies (wrong!)
2. Probe timeout too short
3. `initialDelaySeconds` too short for app startup

**Solution:**
- Make liveness check simpler
- Increase timeout and initialDelaySeconds
- Use readiness probe for dependency checks

---

## Examples

See implementation examples:
- [auth-service](https://github.com/brefwiz/auth-service) - Production Rust implementation
- [project-starter](https://github.com/brefwiz/project-starter) - Reference templates

---

## References

- [Kubernetes Liveness/Readiness Probes](https://kubernetes.io/docs/tasks/configure-pod-container/configure-liveness-readiness-startup-probes/)
- [Docker Health Checks](https://docs.docker.com/engine/reference/builder/#healthcheck)
- [Health Check Pattern (Martin Fowler)](https://martinfowler.com/articles/microservice-testing/#testing-component-healthCheck)

---

**Last Updated:** 2026-01-24
**Extracted from:** auth-service production implementation
