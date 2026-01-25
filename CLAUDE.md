# CLAUDE.md - Project Development Guidelines

> **Purpose**: Quick reference for AI assistants. Battle-tested best practices from production projects.

## 🚨 MANDATORY RULES (Non-Negotiable)

### 1. Cost Optimization (40-60% API cost reduction)

**Model Selection Hierarchy:**
- **Skip AI** (use Edit tool): var→const, add-types, linting, simple refactoring ($0)
- **Haiku** (>60% of tasks): Bug fixes, small features, tests ($0.0002)
- **Sonnet** (<30% of tasks): Medium features, API changes ($0.003)
- **Opus** (<1% of tasks): Architecture, security audits only ($0.015)

**Token Limit Failback (When Sonnet hits limits):**
1. ✅ **PREFERRED**: Decompose into multiple Haiku tasks (4.5x cheaper)
2. ✅ Reduce context + retry with Haiku
3. ✅ Checkpoint and process in phases
4. ⚠️ **LAST RESORT**: Use Opus (only for critical architecture)

**Rule: 95% of token limit cases should use Haiku decomposition.**

### 2. utoipa Annotations (ENFORCED)

**Every API endpoint MUST have:**
```rust
#[utoipa::path(
    get, path = "/api/v1/items",
    responses(
        (status = 200, body = Vec<Item>),
        (status = 401, body = ErrorResponse),
        (status = 500, body = ErrorResponse)
    ),
    params(("limit" = Option<u32>, Query, description = "Limit")),
    tag = "items"
)]
#[derive(ToSchema)]  // Required on all DTOs
```

**Validation:** `make openapi-validate` (runs in pre-commit hook)

### 3. TypeScript Codegen (ENFORCED)

**MANDATORY: After ANY backend API changes:**

1. **Generate TypeScript client**: `make codegen-all`
2. **Commit generated files**: `frontend/src/api/schema.d.ts` and `frontend/src/api/openapi.json`
3. **Frontend MUST use generated types**: Never write manual API types

**Enforcement:**
- `make openapi-generate` validates utoipa annotations (runs in pre-commit)
- `make codegen-check` validates generated code is up-to-date (runs in pre-commit)
- TypeScript compilation fails if types are out of sync
- Pre-commit hook blocks commits with invalid/stale generated code

**Why this matters:**
- Type-safe API communication (frontend knows backend contract)
- Breaking changes caught at compile-time, not runtime
- Zero manual API client maintenance
- OpenAPI schema is single source of truth

**Verify it works:**
```bash
# 1. Generate OpenAPI schema (should succeed)
make openapi-generate

# 2. View the schema
cat frontend/src/api/openapi.json

# 3. View Swagger UI (with backend running)
# Visit: http://localhost:3000/swagger-ui

# 4. Generate TypeScript client
make codegen-all

# 5. Verify pre-commit enforcement
make pre-commit  # Should pass all checks
```

See: [API Codegen Strategy](docs/api-codegen/STRATEGY.md) | [Backend OpenAPI Guide](backend/OPENAPI.md)

### 4. Testing Requirements

- **Coverage**: ≥80% (backend + frontend)
- **E2E tests FIRST**: Write before implementing features (TDD)
- **Bug fixes**: Include regression tests
- **Enforcement**: `make pre-commit` (auto-runs before push)

### 5. Makefile-Only Commands

**NEVER run raw commands.** Always use Makefile.

```bash
# ✅ CORRECT
make backend-build
make test

# ❌ WRONG
cargo build
npm test
```

### 6. Template Adaptation (NOT Copy-Paste)

**Project-starter provides patterns, NOT production code.**

| What | Action |
|------|--------|
| Patterns, structure, conventions | ✅ Reference |
| HTTP codes, Makefile commands | ✅ Copy |
| Implementations, business logic | ❌ Must adapt to YOUR dependencies |

**Before copying ANY template:**
1. What dependencies does MY project have?
2. What failure modes matter for MY project?
3. What's the production impact if wrong?

See: [Templates README](templates/) for detailed adaptation guide.

---

## Quick Start

```bash
# First-time setup
make setup && make install-hooks && make dev-up && make dev-seed

# Daily workflow
make help           # Show all commands
make dev-up         # Start development stack
make ci-quick       # Fast checks (<3 min)
make test           # All tests
make pre-commit     # Full validation (auto-runs before push)
```

---

## Development Workflow

### API Endpoint Development

**For EVERY new/modified endpoint:**

1. **Annotate** with utoipa (MANDATORY)
2. **Validate**: `make openapi-validate`
3. **Generate**: `make codegen-all` (OpenAPI + TypeScript client)
4. **Commit generated files**: Add `frontend/src/api/schema.d.ts` and `documentation/static/api/openapi.json`
5. **Test**: Write E2E test using generated TypeScript client
6. **Implement**: Make test pass
7. **Pre-commit validation**: Hook checks everything automatically

**Frontend uses type-safe client:**
```typescript
const { data, error } = await apiClient.GET('/api/v1/items');
// data is fully typed from utoipa annotations!
```

### Testing Strategy (Test Pyramid)

| Type | Coverage | When |
|------|----------|------|
| E2E | 10-20% | Write FIRST (user stories) |
| Integration | 30-40% | Write DURING feature dev |
| Unit | 40-50% | Write DURING implementation |

**Test Isolation:** Each E2E test gets isolated database/state (parallel execution).

See: [Testing Guide](docs/testing/TESTING_GUIDE.md)

### Git Workflow

```bash
# Start feature
git fetch origin && git checkout main && git pull origin main
git checkout -b feature/module-name

# Commit (format enforced)
git commit -m "<type>(<scope>): <description>

- Detailed change 1
- Detailed change 2

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

See: [Git Workflow Guide](docs/git-workflow/GIT_WORKFLOW.md)

---

## Definition of Done

**Code Quality:**
- [ ] utoipa annotations on all endpoints (`make openapi-validate` passes)
- [ ] All DTOs have `#[derive(ToSchema)]`
- [ ] TypeScript client generated (`make codegen-all` run)
- [ ] Generated files committed (schema.d.ts, openapi.json)
- [ ] Frontend uses generated types (NO manual API types)
- [ ] `make codegen-check` passes (generated code up-to-date)
- [ ] Tests pass (≥80% coverage)
- [ ] `make pre-commit` passes (includes codegen-check)

**Monitoring (for features):**
- [ ] Health checks implemented
- [ ] Metrics instrumented
- [ ] Alerts configured
- [ ] Dashboards created
- [ ] Canary deployed and validated

See: [Continuous Monitoring](docs/monitoring/CONTINUOUS_MONITORING.md)

---

## Project Methodology

### Milestone-Driven Development

```
M-1 (Dev Foundations) → M0 (Platform) → M1 → M2 → ... → MVP
```

**Each milestone:**
- Sequential delivery
- Clear acceptance criteria
- E2E tests written BEFORE implementation
- User stories are testable and measurable

See: [Project Methodology](docs/methodology/PROJECT_METHODOLOGY.md)

### Dev-First Approach

**Milestone -1 / M-0 MUST be complete before feature work:**
- ✅ Makefile with all commands
- ✅ Testing frameworks (unit, integration, E2E)
- ✅ CI pipeline (local-first)
- ✅ Dev environment (Docker Compose)
- ✅ Database seeding
- ✅ Code coverage enforcement

See: [Dev-First Approach](docs/methodology/DEV_FIRST_APPROACH.md)

---

## Documentation Strategy

**70% Automated / 30% Manual Curation**

```bash
make docs-screenshots   # Auto-capture from E2E tests
make openapi-generate   # Auto-generate API docs
make docs-build        # Build Docusaurus site
```

**Automated:**
- API docs (OpenAPI → Docusaurus)
- Screenshots (Playwright E2E tests)
- Changelog (conventional commits)
- TypeScript types (from OpenAPI)

**Manual:** Complex workflows, conceptual explanations, troubleshooting

See: [Documentation Automation](docs/documentation/AUTOMATION.md)

---

## 🤖 Claude Flow Integration (Optional)

**If using claude-flow for multi-agent orchestration:**

### Pre-Task Routing (MANDATORY)

```bash
# BEFORE spawning agents
npx @claude-flow/cli@latest hooks pre-task --description "[task]"
```

**Routing recommendations:**
- `[AGENT_BOOSTER_AVAILABLE]` → Skip LLM, use Edit tool directly
- `[TASK_MODEL_RECOMMENDATION]` → Use specified model
- No model specified → Default to `haiku`

### Cost-Optimized Swarm Sizing

| Complexity | Agents | Topology | Models | Cost |
|-----------|--------|----------|--------|------|
| Simple bug | 1 | none | Haiku | $0.0002 |
| Small feature | 2-3 | hierarchical | 3×Haiku | $0.0006 |
| Medium feature | 4-5 | hierarchical | 1×Sonnet+4×Haiku | $0.004 |
| Large feature | 6-8 | hierarchical-mesh | 2×Sonnet+6×Haiku | $0.008 |

**Default to smallest viable swarm. Scale up only when routing suggests.**

See: [/Users/gsalingu/CLAUDE.md](file:///Users/gsalingu/CLAUDE.md) for full claude-flow integration.

---

## Key Documentation

### Methodology
- [Dev-First Approach](docs/methodology/DEV_FIRST_APPROACH.md)
- [Project Methodology](docs/methodology/PROJECT_METHODOLOGY.md)
- [Milestone Planning](docs/milestones/PLANNING.md)

### Development
- [Makefile Reference](docs/development/MAKEFILE_REFERENCE.md)
- [Development Workflow](docs/development/WORKFLOW.md)
- [Code Quality](docs/development/CODE_QUALITY.md)

### API & Code Generation
- [API Codegen Strategy](docs/api-codegen/STRATEGY.md)
- [API Codegen Guide](docs/api-codegen/GUIDE.md)

### Testing
- [Testing Guide](docs/testing/TESTING_GUIDE.md)
- [E2E Testing](docs/testing/E2E_TESTING.md)

### Monitoring
- [Continuous Monitoring](docs/monitoring/CONTINUOUS_MONITORING.md)
- [Canary Strategy](docs/monitoring/CANARY_STRATEGY.md)

### Templates
- [Milestone Template](templates/milestone/TEMPLATE.md)
- [User Story Template](templates/user-story/TEMPLATE.md)
- [Makefile Template](templates/makefile/TEMPLATE.mk)

---

## Common Patterns

**Error Handling:**
```rust
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

**Tracing:**
```rust
use tracing::instrument;

#[instrument(skip(sensitive_data))]
pub async fn process_item(item_id: Uuid, sensitive_data: &str) -> Result<Item, AppError> {
    // ...
}
```

See language-specific pattern guides in `docs/patterns/`.

---

## Resuming Work

1. Check Milestone -1/M-0 completion (Dev Foundations)
2. Find first NOT STARTED or IN PROGRESS story
3. Follow milestone dependencies (sequential)
4. Update status when starting/completing

---

## Support

- `/help` - Claude Code usage help
- **Issues**: https://github.com/anthropics/claude-code/issues

---

*Last Updated: 2026-01-24*
