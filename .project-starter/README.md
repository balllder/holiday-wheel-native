# Project Starter Template

A battle-tested template for bootstrapping software projects with industry best practices baked in from day one.

**Default Stack:** Rust 1.75+ (Axum 0.7) + React 19 + PostgreSQL 18+ (TypeScript 5.3, Playwright 1.40, Docker)

## What This Is

This repository contains **proven patterns and methodologies** extracted from production projects:
- ✅ **Dev-First Approach**: Infrastructure before features
- ✅ **Test-Driven Development**: E2E tests before implementation
- ✅ **API Code Generation**: Type-safe frontend/backend communication
- ✅ **Milestone-Driven Planning**: Clear roadmap with user stories
- ✅ **Documentation Automation**: Screenshots and API docs auto-generated
- ✅ **Local-First CI**: All validation runs on developer machines

## Why Use This Template

### Without This Template (Traditional Approach)
```
Week 1:  Write features (no tests, no CI)
Week 2:  More features (technical debt accumulates)
Week 3:  Bugs appear, realize testing is needed
Week 4:  Try to retrofit tests (code isn't testable)
Week 5:  Refactor to make testable
Week 6:  Finally add tests
Result: 6 weeks, HIGH technical debt
```

### With This Template (Dev-First Approach)
```
Week 1:  Set up dev infrastructure (this template)
Week 2:  Write E2E tests + Feature 1 (TDD)
Week 3:  Write E2E tests + Feature 2 (TDD)
Week 4:  Features are stable, tested, documented
Result: 4 weeks, LOW technical debt
```

---

## Tech Stack

Project-starter uses specific, tested versions for reproducibility and stability.

### Backend

| Technology | Version | Purpose |
|------------|---------|---------|
| **Rust** | 1.75+ | Compiled systems language |
| **Axum** | 0.7 | Web framework |
| **SQLx** | 0.7 | Database toolkit |
| **Tokio** | 1.35+ | Async runtime |
| **Serde** | 1.0+ | Serialization |

### Database

| Technology | Version | Purpose |
|------------|---------|---------|
| **PostgreSQL** | 18+ | Primary database |
| **SQLx Migrations** | - | Schema versioning |

### Frontend

| Technology | Version | Purpose |
|------------|---------|---------|
| **React** | 19 | UI framework |
| **TypeScript** | 5.3+ | Type-safe JavaScript |
| **Vite** | 5.0+ | Build tool |

### Testing

| Technology | Version | Purpose |
|------------|---------|---------|
| **Playwright** | 1.40+ | E2E testing |
| **cargo test** | - | Rust unit/integration tests |

### Infrastructure

| Technology | Version | Purpose |
|------------|---------|---------|
| **Docker** | 24+ | Containerization |
| **Docker Compose** | 3.8+ | Multi-container orchestration |

### Version Policy

**Why specific versions?**
- ✅ **Reproducibility** - Same build on any machine
- ✅ **Dependency tracking** - Know what you're running
- ✅ **Easier debugging** - Version-specific issues are easier to diagnose
- ✅ **Clear upgrade path** - Know when to update

**Update strategy:**
- **Major versions:** Update template when stable (e.g., Rust 1.75 → 1.76)
- **Minor versions:** Pin in Cargo.toml/package.json
- **Docker images:** Use specific tags (e.g., `postgres:16-alpine`, not `postgres:latest`)

**See also:** [Version Policy](docs/development/VERSION_POLICY.md)

---

## Quick Start

### 1. Use This Template

```bash
# Option A: GitHub Web UI
# Go to: https://github.com/brefwiz/project-starter
# Click: "Use this template" → "Create a new repository"

# Option B: GitHub CLI
gh repo create brefwiz/my-new-project --template brefwiz/project-starter --public
cd my-new-project

# Makefile is already included (Rust + Axum by default)
# To use a different stack, copy the appropriate Makefile:
# cp templates/makefiles/python-fastapi/Makefile ./Makefile
# cp templates/makefiles/nodejs-express/Makefile ./Makefile
```

### 2. Customize for Your Project

Edit these files:
- `CLAUDE.md` - Add project-specific details
- `docs/methodology/PROJECT_METHODOLOGY.md` - Your project methodology
- `docs/milestones/` - Plan your milestones
- `templates/` - Adapt templates to your tech stack

### 3. Setup Environment Configuration

```bash
# Copy environment template
cp templates/env/.env.example .env

# Customize for your project
# Update database credentials, ports, secrets, etc.
nano .env
```

### 4. Initialize Your Project

```bash
# Setup development environment
make setup

# Start services
make dev-up

# Verify everything works
make ci-quick
```

### 5. Start Building

Follow the **Dev-First Approach**:
1. **Milestone -1 / M-0**: Set up development infrastructure
2. **Milestone 1+**: Build features with TDD

---

## Services

When running `make dev-up`, the following services start:

- **Backend API**: http://localhost:3000
- **Frontend**: http://localhost:4200
- **PostgreSQL**: localhost:5432
- **Redis** (optional): localhost:6379

### Quick Reference

| Service | URL | Purpose |
|---------|-----|---------|
| API | http://localhost:3000 | Backend REST API |
| API Docs | http://localhost:3000/api/docs | OpenAPI/Swagger documentation |
| Frontend | http://localhost:4200 | Development UI |
| Database | localhost:5432 | PostgreSQL database |
| Cache | localhost:6379 | Redis cache (optional) |

**Note:** Port numbers are configurable via `.env` file. See [Environment Configuration](templates/env/README.md) for details.

---

## Keeping Your Project Updated

> **📖 For complete integration instructions, see [INTEGRATION_GUIDE.md](INTEGRATION_GUIDE.md)**
>
> This section provides quick reference commands. For comprehensive setup including:
> - Automatic sync workflows
> - Update notifications
> - Contributing patterns back
> - Troubleshooting
>
> See the **[Integration Guide](INTEGRATION_GUIDE.md)**.

### For Existing Projects: Add Template as Subtree

If you have an existing project and want to sync updates from project-starter:

```bash
cd your-existing-project

# Add project-starter as subtree (one-time setup)
git subtree add --prefix=.project-starter \
  https://github.com/brefwiz/project-starter.git main --squash

# Copy files you want to use
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/
cp .project-starter/templates/milestone/TEMPLATE.md templates/milestone/
cp .project-starter/CLAUDE.md ./CLAUDE.md

# Commit
git add .
git commit -m "chore: add project-starter template"
git push
```

### Pulling Updates

When project-starter is updated, sync the latest changes:

```bash
# Pull latest changes from project-starter
git subtree pull --prefix=.project-starter \
  https://github.com/brefwiz/project-starter.git main --squash

# Review what changed
git diff HEAD~1 .project-starter/

# Copy updated files you want to sync
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/
cp .project-starter/docs/testing/TESTING_GUIDE.md docs/testing/
cp .project-starter/docs/methodology/SHARING_STRATEGY.md docs/methodology/

# Commit the updates
git add .
git commit -m "docs: sync updates from project-starter"
git push
```

### What to Sync

**Always sync (critical updates):**
- `docs/testing/API_TESTING_REQUIREMENTS.md` - Testing standards
- `docs/testing/TESTING_GUIDE.md` - Testing methodology
- `docs/testing/E2E_TESTING.md` - E2E best practices
- `docs/monitoring/CONTINUOUS_MONITORING.md` - Monitoring strategy

**Consider syncing (methodology updates):**
- `docs/methodology/DEV_FIRST_APPROACH.md` - Development philosophy
- `docs/api-codegen/STRATEGY.md` - Code generation patterns
- `templates/milestone/TEMPLATE.md` - Milestone template
- `templates/user-story/TEMPLATE.md` - User story template

**Don't sync (project-specific):**
- `README.md` - Your project's README
- `Makefile` - Your customized Makefile
- `CLAUDE.md` - Your project's AI guidelines
- `docs/milestones/` - Your project's milestones

### Automated Updates (Optional)

Copy the sync workflow to auto-create PRs when template updates:

```bash
# Copy the example workflow
cp .project-starter/.github/workflows/sync-template.yml.example \
   .github/workflows/sync-template.yml

# Customize repository name in the workflow
# Then commit and push
git add .github/workflows/sync-template.yml
git commit -m "ci: add automatic template sync workflow"
git push
```

Now when project-starter updates, a PR will be auto-created in your repo!

---

## What's Included

### 📚 Documentation

#### Methodology
- **[Dev-First Approach](docs/methodology/DEV_FIRST_APPROACH.md)** - Why infrastructure comes first
- **[Project Methodology](docs/methodology/PROJECT_METHODOLOGY.md)** - Milestone-driven development
- **[Milestone Planning](docs/milestones/PLANNING.md)** - How to structure milestones
- **[Sharing Strategy](docs/methodology/SHARING_STRATEGY.md)** - How to share this template across your organization

#### API & Code Generation
- **[API Codegen Strategy](docs/api-codegen/STRATEGY.md)** - OpenAPI + TypeScript generation (CRITICAL)
- **[API Codegen Guide](docs/api-codegen/GUIDE.md)** - Step-by-step implementation

#### Development
- **[Makefile Reference](docs/development/MAKEFILE_REFERENCE.md)** - All available commands
- **[Development Workflow](docs/development/WORKFLOW.md)** - Daily development process
- **[Code Quality](docs/development/CODE_QUALITY.md)** - Linting, formatting, standards

#### Testing
- **[Testing Guide](docs/testing/TESTING_GUIDE.md)** - Complete testing strategy (unit, integration, E2E)
- **[API Testing Requirements](docs/testing/API_TESTING_REQUIREMENTS.md)** - Mandatory backend API testing standards (CRITICAL)
- **[E2E Testing Best Practices](docs/testing/E2E_TESTING.md)** - Comprehensive E2E testing guide (CRITICAL)

#### Documentation
- **[Documentation Strategy](docs/documentation/STRATEGY.md)** - Automated + manual docs
- **[Documentation Automation](docs/documentation/AUTOMATION.md)** - Screenshot generation, CI/CD

#### Git Workflow
- **[Git Workflow](docs/git-workflow/GIT_WORKFLOW.md)** - Branching, commits, PRs

#### Monitoring & Observability
- **[Continuous Monitoring](docs/monitoring/CONTINUOUS_MONITORING.md)** - Production monitoring strategy (CRITICAL)
- **[Canary Deployments](docs/monitoring/CANARY_STRATEGY.md)** - Gradual rollout with automatic rollback

---

### 📝 Templates

#### Milestone Template
`templates/milestone/TEMPLATE.md` - Structure for planning milestones

**Contains:**
- Status tracking
- Dependencies
- User stories
- Acceptance criteria
- Technical stack
- Testing requirements
- Security considerations
- Success metrics

#### User Story Template
`templates/user-story/TEMPLATE.md` - Structure for user stories

**Contains:**
- User story format
- Acceptance criteria
- E2E test scenario (TDD)
- Technical approach
- Testing strategy
- Security/performance considerations
- Definition of done

#### Makefile Template
`templates/makefile/TEMPLATE.mk` - Starter Makefile with common patterns

**Includes:**
- Build commands
- Test commands
- Code generation commands
- Documentation commands
- CI/CD commands

#### CI/CD Template
`templates/ci-cd/github-actions.yml.disabled` - GitHub Actions workflow (disabled by default)

**Includes:**
- Code generation checks
- Test execution
- Documentation builds
- Deployment pipelines

#### Environment Configuration Templates
`templates/env/` - Environment configuration for different deployment stages

**Files:**
- `.env.example` - Development configuration template
- `.env.production.example` - Production configuration template
- `README.md` - Configuration guide and best practices

**Includes:**
- Database configuration (PostgreSQL, connection pooling)
- Cache configuration (Redis, optional)
- Application settings (ports, CORS, logging)
- Security configuration (JWT, API keys, encryption)
- External services (email, payment, storage)
- Feature flags
- Monitoring & observability
- Production-specific settings (SSL, performance tuning)

**Security features:**
- Strong secret generation commands
- Production secret management patterns
- Environment-specific security policies
- CORS configuration guidance

See: [Environment Configuration Guide](templates/env/README.md)

#### Health Check Patterns
`templates/health-check/` - Production-ready health check implementations

**Files:**
- `rust-axum/health.rs` - Rust + Axum health check handlers
- `python-fastapi/health.py` - Python + FastAPI health check router
- `nodejs-express/health.ts` - Node.js + Express health check router
- `docker-compose.example.yml` - Docker Compose health check configuration
- `README.md` - Comprehensive health check guide

**Endpoints:**
- `GET /health` - Liveness probe (is service running?)
- `GET /health/ready` - Readiness probe (is service ready for traffic?)

**Includes:**
- Database connection health checks
- Cache (Redis) health checks (optional)
- Proper HTTP status codes (200 OK, 503 Service Unavailable)
- Version information in responses
- Makefile health check commands
- Docker Compose health check configuration
- Kubernetes probe examples
- Comprehensive testing examples

**Use for:**
- Kubernetes liveness/readiness probes
- Load balancer health checks
- Docker Compose service dependencies
- Monitoring and alerting
- Deployment validation

See: [Health Check Guide](templates/health-check/README.md)

#### E2E Testing Templates
`templates/e2e-tests/` - Comprehensive end-to-end testing with Playwright

**Files:**
- `example.spec.ts` - Complete E2E test examples (API + UI)
- `playwright.config.ts` - Playwright configuration
- `utils/testData.ts` - Test data generation utilities
- `package.json.example` - npm dependencies and scripts
- `README.md` - Comprehensive E2E testing guide

**Test Categories:**
- **Health checks** - API endpoint availability
- **API tests** - Direct backend testing (CRUD operations)
- **UI tests** - Browser interaction (forms, navigation, workflows)
- **Authentication** - Login/logout flows
- **Performance** - Load time assertions

**Features:**
- Test isolation for parallel execution (4-6 workers)
- Unique test data generation (no conflicts)
- Multi-browser support (Chrome, Firefox, Safari)
- Visual debugging (UI mode, headed mode)
- Screenshots and videos on failure
- Comprehensive test patterns and examples

**Use for:**
- E2E testing before feature implementation (TDD)
- Regression testing
- Cross-browser compatibility
- User workflow validation
- Performance testing

See: [E2E Testing Guide](templates/e2e-tests/README.md)

---

### 🚀 Reference Implementation

`backend/` - Minimal working REST API demonstrating all project-starter patterns

**IMPORTANT**: This is a **reference implementation** showing how all the patterns work together. Developers should customize it for their specific needs, not use it as-is.

**Features:**
- ✅ Health check endpoints (`/health`, `/health/ready`)
- ✅ Simple CRUD API (`/api/items`)
- ✅ PostgreSQL database with migrations
- ✅ Structured logging and tracing
- ✅ Error handling with typed errors
- ✅ Environment configuration (.env support)
- ✅ Docker containerization
- ✅ Comprehensive testing (unit, integration, API)

**Stack:**
- **Backend:** Rust 1.75+ with Axum 0.7 web framework
- **Database:** PostgreSQL 18+ with SQLx 0.7
- **Container:** Docker + Docker Compose 3.8+

**API Endpoints:**

| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/health` | Liveness probe |
| `GET` | `/health/ready` | Readiness probe with dependency checks |
| `GET` | `/api/items` | List all items |
| `GET` | `/api/items/:id` | Get item by ID |
| `POST` | `/api/items` | Create new item |

**Quick Start:**
```bash
# Copy environment configuration
cp backend/.env.example backend/.env

# Start services with Docker Compose
make dev-up

# Run tests
make backend-test

# Check health
curl http://localhost:3000/health
```

**Project Structure:**
```
backend/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration loading (with unit tests)
│   ├── db.rs             # Database connection pool
│   ├── error.rs          # Error types and handling
│   ├── models.rs         # Data models (with validation tests)
│   └── api/
│       ├── health.rs     # Health check endpoints
│       └── items.rs      # Items CRUD endpoints
├── migrations/           # Database migrations
├── tests/
│   ├── integration_test.rs  # Database integration tests
│   └── api_test.rs          # API tests
├── Cargo.toml            # Dependencies
├── Dockerfile            # Production container
└── README.md             # Detailed documentation
```

**What You Get:**
- Working example of all Phase 1 & 2 patterns
- Production-ready code structure
- Comprehensive inline documentation
- Complete test coverage (unit + integration + API)
- Docker deployment ready

**Customization Guide:**
1. Replace the `items` model with your domain objects
2. Add your business logic in new service modules
3. Extend the API with additional endpoints
4. Add authentication (see auth-service for Biscuit token example)
5. Add more tests using E2E templates

See: [Backend Reference Implementation](backend/README.md)

---

## Core Principles

### 1. Dev-First: Infrastructure Before Features

**No feature code until infrastructure is ready.**

**Milestone -1 / M-0 Deliverables:**
- ✅ Makefile with all commands
- ✅ Testing frameworks (unit, integration, E2E)
- ✅ CI pipeline (local)
- ✅ Dev environment (Docker Compose)
- ✅ Code coverage enforcement (≥80%)
- ✅ Documentation framework

See: [Dev-First Approach](docs/methodology/DEV_FIRST_APPROACH.md)

---

### 2. Test-Driven Development (TDD)

**Write E2E tests BEFORE implementing features.**

**Process:**
1. User story defined
2. E2E test written (fails - red)
3. Feature implemented (test passes - green)
4. Refactor (test still passes)

**E2E Testing Principles:**
- ✅ **Complete isolation**: Each test gets own database state (enables parallel execution)
- ✅ **Real-looking data**: Use realistic test data, not minimal fixtures
- ✅ **Real internal services**: Use actual services we develop (different repos)
- ✅ **Mock third-party**: Mock external services (Stripe, Twilio, etc.)
- ✅ **Mobile coverage**: 80/20 rule (critical flows on mobile, all flows on desktop)
- ✅ **Performance testing**: Assert on load times, run load tests with k6
- ✅ **Visual testing**: UI/visual checks are part of E2E tests, not separate

**Benefits:**
- ✅ Tests drive design
- ✅ Features are testable by default
- ✅ No untested code
- ✅ Documentation through tests

See: [E2E Testing Best Practices](docs/testing/E2E_TESTING.md) (CRITICAL READ)

---

### 3. API Code Generation (CRITICAL)

**Automatically generate TypeScript clients from backend schemas.**

**Workflow:**
```
Backend Code (annotated)
        ↓
    OpenAPI Schema (generated)
        ↓
TypeScript Client (generated)
        ↓
    Frontend Code (type-safe)
```

**Commands:**
```bash
make openapi-generate     # Generate schema
make codegen-frontend     # Generate client
make codegen-all          # Generate both
```

**Benefits:**
- ✅ Type safety between frontend/backend
- ✅ Zero manual API client code
- ✅ Always up-to-date documentation
- ✅ Refactoring is safe (TypeScript catches breaks)

See: [API Codegen Strategy](docs/api-codegen/STRATEGY.md)

---

### 4. Makefile-Driven Operations

**Every action goes through the Makefile.**

```bash
# ✅ CORRECT
make backend-build
make frontend-test
make lint

# ❌ WRONG
cargo build
npm test
```

**Why:**
- Consistent across all environments
- Same commands in CI and locally
- Self-documenting (`make help`)

---

### 5. Monitoring is Part of the Feature (CRITICAL)

**As you develop features, you MUST develop monitoring that continuously tests the application behaves as intended in production.**

**Every feature includes:**
- Health check endpoints
- Application metrics (requests, latency, errors)
- Business metrics (conversions, revenue, user actions)
- Synthetic monitoring tests (production validation)
- Alerts (error rates, latency, success rates)
- Grafana dashboards
- Canary deployment configuration

**Canary Deployments:**
- Deploy to 5% of traffic first
- Monitor for 15 minutes (error rates, latency, success rates)
- Automatic rollback if metrics degrade
- Gradual rollout: 5% → 25% → 50% → 100%

```bash
make canary-deploy VERSION=v1.3.0   # Deploy canary (5%)
# System monitors metrics, auto-rolls back if issues detected
make canary-promote                  # Promote to 100% if successful
```

**Monitoring is NOT optional:**
- ❌ Feature without monitoring = Not done
- ✅ Feature with monitoring = Production ready

See: [Continuous Monitoring](docs/monitoring/CONTINUOUS_MONITORING.md)
See: [Canary Strategy](docs/monitoring/CANARY_STRATEGY.md)

---

### 6. Local-First CI

**All validation runs locally before CI.**

```bash
make pre-commit     # Runs automatically before push
make ci-quick       # Fast checks (format, lint, unit tests)
make ci             # Full CI (all tests)
```

**Benefits:**
- Fix issues locally before CI
- No waiting for remote builds
- Faster feedback loop

---

## Milestones

Track project progress through planned milestones:

- [x] M-0: Development Infrastructure ✓
- [ ] M-1: Core Features
- [ ] M-2: Advanced Features
- [ ] M-3: Testing & Quality
- [ ] M-4: Documentation
- [ ] M-5: Production Readiness

See [docs/milestones/](docs/milestones/) for detailed milestone plans and user stories.

**Milestone Status Legend:**
- ✓ Completed
- 🔄 In Progress
- ⏳ Blocked/Waiting
- 📋 Planned

---

## Project Structure

```
project-starter/
├── CLAUDE.md                  # AI assistant guidelines (concise, references detailed docs)
├── README.md                  # This file
├── docs/
│   ├── methodology/           # Development philosophy
│   │   ├── DEV_FIRST_APPROACH.md
│   │   └── PROJECT_METHODOLOGY.md
│   ├── api-codegen/           # Type-safe API communication
│   │   ├── STRATEGY.md
│   │   └── GUIDE.md
│   ├── development/           # Development workflow
│   │   ├── MAKEFILE_REFERENCE.md
│   │   └── WORKFLOW.md
│   ├── testing/               # Testing strategy
│   │   ├── TESTING_GUIDE.md
│   │   └── E2E_TESTING.md
│   ├── documentation/         # Documentation automation
│   │   ├── STRATEGY.md
│   │   └── AUTOMATION.md
│   ├── git-workflow/          # Git best practices
│   │   └── GIT_WORKFLOW.md
│   ├── milestones/            # Milestone planning
│   │   └── PLANNING.md
│   └── user-stories/          # User story guidelines
│       └── GUIDELINES.md
└── templates/
    ├── milestone/             # Milestone template
    │   └── TEMPLATE.md
    ├── user-story/            # User story template
    │   └── TEMPLATE.md
    ├── makefile/              # Makefile template
    │   └── TEMPLATE.mk
    └── ci-cd/                 # CI/CD templates
        └── github-actions.yml.disabled
```

---

## Technology Stack

**Default:** Rust + Axum + React (included in root `Makefile`)

This template also supports other tech stacks. Alternative Makefiles available:

### Rust + React (DEFAULT - Included)
- **Backend:** Rust (Axum), PostgreSQL
- **Frontend:** React 19 + TypeScript + Vite
- **OpenAPI:** utoipa
- **Testing:** Playwright (E2E), cargo test (unit)
- **Location:** Root `Makefile`

### Python + React (Alternative)
- **Backend:** FastAPI, PostgreSQL
- **Frontend:** React + TypeScript
- **OpenAPI:** Built-in (Pydantic)
- **Testing:** Playwright (E2E), pytest (unit)
- **Location:** `templates/makefiles/python-fastapi/Makefile`

### Node.js + React (Alternative)
- **Backend:** TypeScript + Express
- **Frontend:** React + TypeScript
- **OpenAPI:** tsoa
- **Testing:** Playwright (E2E), Jest (unit)
- **Location:** `templates/makefiles/nodejs-express/Makefile`

---

## Customization Guide

### 1. Adapt CLAUDE.md

Replace generic placeholders with project-specific:
- Project name
- Tech stack
- Team conventions
- Custom commands

### 2. Plan Your Milestones

Use the milestone template:
```bash
cp templates/milestone/TEMPLATE.md docs/milestones/m-0-foundation.md
# Fill in the details
```

### 3. Create User Stories

Use the user story template:
```bash
mkdir docs/milestones/m-1/
cp templates/user-story/TEMPLATE.md docs/milestones/m-1/us-1.md
# Fill in the details
```

### 4. Adapt Makefile

Copy and customize for your stack:
```bash
cp templates/makefile/TEMPLATE.mk Makefile
# Customize commands for your backend/frontend
```

---

## Success Stories

This template has been validated in production projects:

- **RentalForge** - Multi-tenant SaaS platform (Rust + React)
- **Auth Service** - Authentication microservice (Rust + React)
- **[Your Project Here]** - Use this template!

---

## Common Mistakes to Avoid

### ❌ Skipping Milestone -1 / M-0
**Don't start features before infrastructure is ready.**

**Solution:** Complete dev infrastructure first.

### ❌ Writing Features Before Tests
**Don't write code before E2E tests exist.**

**Solution:** TDD - tests first, then implementation.

### ❌ Manual API Clients
**Don't manually write API client code.**

**Solution:** Generate from OpenAPI schema.

### ❌ Ignoring the Makefile
**Don't run raw commands directly.**

**Solution:** Always use `make <target>`.

---

## Getting Help

- **Review Documentation:** Start with `docs/methodology/DEV_FIRST_APPROACH.md`
- **Check Templates:** See `templates/` for examples
- **Read CLAUDE.md:** Quick reference for AI assistants

---

## Contributing

If you improve this template:
1. Create an issue describing the improvement
2. Submit a PR with updated documentation
3. Share your experience

---

## License

[Your License Here]

---

## Quick Reference

### Essential Commands
```bash
make help                  # Show all commands
make setup                 # Initial setup
make dev-up                # Start development stack
make ci-quick              # Fast validation
make test                  # Run all tests
make codegen-all           # Generate API client
make docs-screenshots      # Generate documentation screenshots
make monitoring-synthetic  # Run production monitoring tests
make canary-deploy         # Deploy canary (5% traffic)
make canary-promote        # Promote canary to 100%
```

### Key Documents to Read First
1. [Dev-First Approach](docs/methodology/DEV_FIRST_APPROACH.md) - Start here
2. [API Codegen Strategy](docs/api-codegen/STRATEGY.md) - Critical for type safety
3. [Testing Guide](docs/testing/TESTING_GUIDE.md) - TDD approach
4. [Continuous Monitoring](docs/monitoring/CONTINUOUS_MONITORING.md) - Production confidence
5. [Canary Deployments](docs/monitoring/CANARY_STRATEGY.md) - Safe rollouts

### Templates to Use
1. [Milestone Template](templates/milestone/TEMPLATE.md) - For planning
2. [User Story Template](templates/user-story/TEMPLATE.md) - For features
3. [Makefile Template](templates/makefile/TEMPLATE.mk) - For commands

---

**Start your next project with confidence. Quality is not negotiable.**
