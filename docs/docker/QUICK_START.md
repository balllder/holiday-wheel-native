# Docker Quick Start Guide

## TL;DR

```bash
# Production build (blazingly fast)
make docker-build              # ~90s cold, ~8s warm

# Development with hot-reload
make dev-up                    # Start dev environment

# Clean rebuild
make docker-build-no-cache     # Start fresh
```

## Production vs Development

| Feature | Production (Dockerfile) | Development (Dockerfile.dev) |
|---------|------------------------|------------------------------|
| Build time (cold) | ~90s | ~30s |
| Build time (warm) | ~8s | ~5s |
| Image size | 45 MB | 250 MB |
| Hot-reload | ❌ No | ✅ Yes (cargo-watch) |
| Optimizations | ✅ Full (LTO, strip, mold) | ❌ Debug build |
| Use case | CI/CD, Production | Local development |

## Production Build

### Standard Build

```bash
# Using Makefile (recommended)
make docker-build

# Or directly
DOCKER_BUILDKIT=1 docker-compose build
```

**What happens:**
1. cargo-chef analyzes dependencies
2. Dependencies built (cached layer)
3. Source code compiled with mold + sccache
4. Binary stripped and copied to Alpine runtime

**Build time:**
- **Cold cache**: ~90 seconds (all dependencies)
- **Warm cache** (source change only): ~8 seconds
- **No changes**: ~1 second (instant)

### Fast Rebuild (After Source Changes)

```bash
make docker-build-fast
```

This leverages all caches for maximum speed.

### Clean Build (From Scratch)

```bash
make docker-build-no-cache
```

Use when:
- Debugging cache issues
- After major dependency updates
- CI/CD clean builds

## Development Workflow

### Start Development Environment

```bash
# Start all services (DB, backend, frontend)
make dev-up

# View logs
make dev-logs

# Or specific service
make logs-backend
make logs-frontend
make logs-db
```

**Services:**
- Backend (hot-reload): http://localhost:3000
- Frontend (Vite HMR): http://localhost:5173
- PostgreSQL: localhost:5432
- API docs: http://localhost:3000/docs

### Hot-Reload in Action

```bash
# 1. Start dev environment
make dev-up

# 2. Edit source code
vim backend/src/main.rs

# 3. Save - cargo-watch auto-reloads
# Backend restarts in ~2-3 seconds
```

**What gets recompiled:**
- Only changed files + dependents
- Dependencies are cached (never rebuilt)

### Stop Development Environment

```bash
# Stop all services
make dev-down

# Or clean everything (including volumes)
make docker-clean
```

## Common Workflows

### Full Development Cycle

```bash
# 1. Initial setup
make setup
make dev-up

# 2. Run migrations
make migrate

# 3. Seed database
make db-seed

# 4. Start coding!
# Edit files - auto-reload handles the rest

# 5. Run tests
make test

# 6. Commit changes
make pre-commit  # Runs linting, tests, type-checks
git commit -m "feat: add awesome feature"
```

### Production Deployment

```bash
# 1. Build production image
make docker-build

# 2. Tag for registry
docker tag project-starter-api:latest myregistry.com/project-starter-api:v1.0.0

# 3. Push to registry
docker push myregistry.com/project-starter-api:v1.0.0

# 4. Deploy (k8s, docker swarm, etc.)
kubectl apply -f k8s/deployment.yml
```

### CI/CD Pipeline

```yaml
# .github/workflows/build.yml
name: Build and Test

on: [push, pull_request]

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      # Enable BuildKit
      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      # Build with cache
      - name: Build
        run: make docker-build

      # Run tests
      - name: Test
        run: make test
```

## Troubleshooting

### Build is Slow

**Problem**: Build takes >2 minutes.

**Solutions:**

1. **Check BuildKit is enabled:**
   ```bash
   echo $DOCKER_BUILDKIT  # Should be "1"
   export DOCKER_BUILDKIT=1
   ```

2. **Check cache usage:**
   ```bash
   make docker-stats
   # Look for high cache hit rate (>70%)
   ```

3. **Clear and rebuild:**
   ```bash
   make docker-clean-cache
   make docker-build
   ```

### Hot-Reload Not Working

**Problem**: Code changes don't trigger rebuild.

**Solutions:**

1. **Check cargo-watch is running:**
   ```bash
   docker logs project-starter-backend-dev
   # Should see: "[Running 'cargo run']"
   ```

2. **Restart dev environment:**
   ```bash
   make dev-down
   make dev-up
   ```

3. **Check file permissions (macOS/Linux):**
   ```bash
   ls -la backend/src/
   # Files should be readable (r--)
   ```

### Out of Disk Space

**Problem**: Docker using too much space.

**Solutions:**

1. **Check usage:**
   ```bash
   make docker-stats
   docker system df
   ```

2. **Clean build cache:**
   ```bash
   make docker-clean-cache
   ```

3. **Clean everything:**
   ```bash
   make docker-clean
   docker system prune -a --volumes
   ```

### Can't Connect to Services

**Problem**: Can't reach backend at http://localhost:3000.

**Solutions:**

1. **Check services are running:**
   ```bash
   docker ps
   # Should see: project-starter-backend, project-starter-db, etc.
   ```

2. **Check health:**
   ```bash
   make health-all
   ```

3. **Check logs:**
   ```bash
   make logs-backend
   # Look for startup errors
   ```

4. **Verify ports:**
   ```bash
   lsof -i :3000  # Should show Docker container
   ```

## Performance Tips

### Faster Builds

1. **Use BuildKit** (enabled by default in Makefile)
2. **Don't clean unless necessary** (caches are good!)
3. **Use `make docker-build-fast`** for iterative builds
4. **Pre-download dependencies** (cargo-chef handles this)

### Faster Development

1. **Use hot-reload** (docker-compose.dev.yml)
2. **Only rebuild when needed** (cargo-watch is smart)
3. **Keep dependencies stable** (avoid frequent Cargo.toml changes)
4. **Use incremental compilation** (enabled by default in dev)

### Smaller Images

1. **Strip binaries** (done automatically in production)
2. **Use Alpine** (not Debian/Ubuntu)
3. **Multi-stage builds** (keep build tools out of runtime)
4. **Don't include source code** (only binary + migrations)

## Advanced Usage

### Custom Build Arguments

```bash
# Disable LTO (faster builds, slower runtime)
docker build --build-arg ENABLE_LTO=false -f backend/Dockerfile backend/

# Different optimization level
docker build --build-arg OPT_LEVEL=2 -f backend/Dockerfile backend/
```

### Multi-Architecture Builds

```bash
# Build for AMD64 + ARM64
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t project-starter-api:latest \
  -f backend/Dockerfile \
  backend/
```

### Registry Caching (Teams)

```bash
# Share cache across team
docker buildx build \
  --cache-to type=registry,ref=myregistry.com/cache \
  --cache-from type=registry,ref=myregistry.com/cache \
  -t project-starter-api:latest \
  backend/
```

## Monitoring

### Build Metrics

```bash
# Track build time
time make docker-build

# Track cache hit rate
docker build --target builder --progress=plain backend/ 2>&1 | grep sccache

# Track image size
docker images project-starter-api:latest
```

### Runtime Metrics

```bash
# Container stats
docker stats project-starter-backend

# Logs
make logs-backend

# Health checks
make health-all
```

## Next Steps

- Read [BUILD_OPTIMIZATION.md](./BUILD_OPTIMIZATION.md) for deep dive
- Check [../development/WORKFLOW.md](../development/WORKFLOW.md) for full dev workflow
- See [../testing/TESTING_GUIDE.md](../testing/TESTING_GUIDE.md) for testing strategies

---

*Last Updated: 2026-01-24*
