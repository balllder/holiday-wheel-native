# Dev-First Approach: Infrastructure Before Features

## Philosophy

**No feature code should be written until development infrastructure is in place.**

This approach ensures:
- ✅ Quality is built-in from day one
- ✅ All code is tested before merging
- ✅ Developers are productive immediately
- ✅ Technical debt is minimized
- ✅ Onboarding is fast (< 1 day)

## The Problem with Feature-First

Traditional approach:
```
1. Write feature code
2. Realize you need tests
3. Retrofit testing infrastructure
4. Struggle with legacy code that's hard to test
5. Accumulate technical debt
```

## The Dev-First Solution

```
Milestone -1 / M-0: Developer Foundations
├── Testing Infrastructure (unit, integration, E2E)
├── CI/CD Pipeline (local-first)
├── Development Environment (Docker Compose)
├── Makefile (all commands)
├── Code Quality Tools (linting, formatting)
└── Documentation Framework

Then: Milestone 1, 2, 3... (features)
```

---

## Milestone -1 / M-0: Developer Foundations

### Priority

**P0 - CRITICAL PREREQUISITE**

This milestone MUST be complete before any feature development begins.

### Deliverables

#### 1. **Makefile Foundation**

Root Makefile with all project commands:
```makefile
.PHONY: help build test lint

help:           ## Show all commands
build:          ## Build all components
test:           ## Run all tests
lint:           ## Run linters
```

**Acceptance Criteria:**
- [ ] `make help` lists all commands
- [ ] Every action uses `make <target>`
- [ ] Commands work identically in CI and locally

---

#### 2. **Backend Project Structure**

Organized, modular codebase:
```
backend/
├── src/
│   ├── api/          # HTTP handlers
│   ├── domain/       # Business logic
│   ├── repository/   # Data access
│   └── main.rs
├── tests/
│   ├── unit/
│   └── integration/
└── Cargo.toml (or package.json, etc.)
```

**Acceptance Criteria:**
- [ ] `make backend-build` succeeds
- [ ] Clear separation of concerns
- [ ] No circular dependencies

---

#### 3. **Frontend Project Structure**

Feature-based organization:
```
frontend/
├── src/
│   ├── features/
│   │   ├── auth/
│   │   ├── dashboard/
│   │   └── settings/
│   ├── components/   # Shared
│   ├── api/          # Generated client
│   └── main.tsx
├── e2e/              # Playwright tests
└── package.json
```

**Acceptance Criteria:**
- [ ] `make frontend-build` succeeds
- [ ] Components organized by feature
- [ ] Shared components clearly separated

---

#### 4. **Backend Unit Test Framework**

```bash
make backend-test-unit
```

**Requirements:**
- ✅ Fast (< 10s for full suite)
- ✅ No external dependencies
- ✅ Coverage reporting
- ✅ Watch mode available

**Example (Rust):**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculation() {
        assert_eq!(calculate(2, 2), 4);
    }
}
```

---

#### 5. **Backend Integration Test Framework**

```bash
make backend-test-integration
```

**Requirements:**
- ✅ Uses testcontainers for database
- ✅ Isolated test data
- ✅ Runs in parallel
- ✅ Cleanup after tests

**Example (Rust + testcontainers):**
```rust
#[tokio::test]
async fn test_create_user() {
    let container = setup_test_db().await;
    let pool = create_pool(&container).await;

    let result = create_user(&pool, "test@example.com").await;

    assert!(result.is_ok());
    cleanup(&container).await;
}
```

---

#### 6. **Frontend Unit Test Framework**

```bash
make frontend-test-unit
```

**Requirements:**
- ✅ Component testing (React Testing Library, etc.)
- ✅ Fast execution
- ✅ Coverage reporting
- ✅ Watch mode

**Example (React + Vitest):**
```typescript
import { render, screen } from '@testing-library/react';
import { Button } from './Button';

test('renders button with text', () => {
  render(<Button>Click me</Button>);
  expect(screen.getByText('Click me')).toBeInTheDocument();
});
```

---

#### 7. **Frontend Integration Test Framework**

```bash
make frontend-test-integration
```

**Requirements:**
- ✅ Tests component interactions
- ✅ Mocks API calls (MSW)
- ✅ Tests routing and state management

---

#### 8. **E2E Test Framework**

```bash
make test-e2e
```

**Requirements:**
- ✅ Playwright or Cypress
- ✅ Browser automation
- ✅ Full-stack testing
- ✅ Parallel execution
- ✅ Screenshot/video on failure

**Example (Playwright):**
```typescript
test('user can login', async ({ page }) => {
  await page.goto('/login');
  await page.fill('[name="email"]', 'user@example.com');
  await page.fill('[name="password"]', 'password');
  await page.click('button[type="submit"]');

  await expect(page).toHaveURL('/dashboard');
});
```

---

#### 9. **E2E Test Suite for User Stories**

**CRITICAL: E2E tests for MVP Critical user stories BEFORE implementation.**

**Process:**
1. User story defined
2. E2E test written (fails - red)
3. Feature implemented
4. E2E test passes (green)

---

#### 10. **CI Pipeline - Tests (Local)**

```bash
make ci
```

**Runs:**
- ✅ Backend unit tests
- ✅ Backend integration tests
- ✅ Frontend unit tests
- ✅ Frontend integration tests
- ✅ E2E tests
- ✅ Coverage reporting

**Requirement:** Must complete in < 10 minutes

---

#### 11. **CI Pipeline - Quality (Local)**

```bash
make lint
make fmt-check
make security-scan
```

**Checks:**
- ✅ Code formatting
- ✅ Linting rules
- ✅ Security vulnerabilities
- ✅ Type checking

---

#### 12. **Dev Environment - Docker Compose**

```bash
make dev-up     # Start all services
make dev-down   # Stop all services
```

**Services:**
- Database (PostgreSQL, MySQL, etc.)
- Cache (Redis)
- Message Queue (RabbitMQ, Kafka)
- Object Storage (MinIO)

**Requirements:**
- ✅ One command to start everything
- ✅ Consistent across machines
- ✅ Volume mounts for data persistence

---

#### 13. **Dev Environment - On-Demand**

```bash
make dev-create NAME=feature-x
```

**Creates:**
- Isolated Docker network
- Separate database
- Independent Redis instance
- Allows parallel development

---

#### 14. **Dev Environment - Seeding**

```bash
make dev-seed
```

**Populates:**
- Test users
- Sample data
- Realistic scenarios

**Requirement:** Should complete in < 30s

---

#### 15. **Code Coverage Enforcement**

**Thresholds:**
- Backend: ≥80%
- Frontend: ≥80%
- E2E: Covers all critical user paths

**Enforcement:**
```bash
make test-coverage  # Fails if below threshold
```

---

#### 16. **Documentation - Development Guide**

**Contents:**
- Quick start (< 5 min)
- Architecture overview
- Development workflow
- Testing guide
- Troubleshooting

**Goal:** New developer productive within 1 day

---

#### 17. **CLAUDE.md - AI Guidelines**

Documentation for AI assistants:
- Development workflow
- Testing requirements
- Makefile commands
- Project methodology

---

## Why This Matters

### Without Dev-First (Traditional)

```
Week 1:  Write feature code (no tests)
Week 2:  More features (no tests)
Week 3:  Bug reports start coming in
Week 4:  Try to add tests, realize code isn't testable
Week 5:  Refactor to make testable
Week 6:  Write tests
Week 7:  More refactoring
Week 8:  Finally stable
```

**Time to stability: 8 weeks**
**Technical debt: HIGH**

### With Dev-First

```
Week 1:  Set up dev infrastructure
Week 2:  Write E2E tests + Feature 1 (TDD)
Week 3:  Write E2E tests + Feature 2 (TDD)
Week 4:  Write E2E tests + Feature 3 (TDD)
Week 5:  Features are stable, tested, documented
```

**Time to stability: 5 weeks**
**Technical debt: LOW**

---

## Acceptance Criteria for Milestone -1 / M-0

- [ ] All 17 deliverables complete
- [ ] `make help` shows all commands
- [ ] `make setup` onboards new developer in < 1 hour
- [ ] `make dev-up` starts full stack
- [ ] `make ci` passes locally
- [ ] `make test` achieves ≥80% coverage
- [ ] `make dev-seed` populates test data
- [ ] New developer productive within 1 day
- [ ] Zero feature code written yet

---

## Implementation Checklist

Use this checklist when starting a new project:

### Week 1: Foundation
- [ ] Create root Makefile
- [ ] Set up backend project structure
- [ ] Set up frontend project structure
- [ ] Docker Compose for services
- [ ] Database migrations working

### Week 2: Testing
- [ ] Backend unit test framework
- [ ] Backend integration test framework
- [ ] Frontend unit test framework
- [ ] Frontend integration test framework
- [ ] E2E test framework (Playwright)

### Week 3: Quality & CI
- [ ] Linting and formatting
- [ ] Pre-commit hooks
- [ ] Local CI pipeline
- [ ] Code coverage enforcement
- [ ] Security scanning

### Week 4: Polish
- [ ] Dev environment seeding
- [ ] On-demand environments
- [ ] Development guide
- [ ] CLAUDE.md
- [ ] Team training

---

## Common Mistakes to Avoid

### ❌ Mistake #1: "We'll add tests later"
**Result:** Tests never get added, technical debt accumulates

**Solution:** Tests are REQUIRED from day one

### ❌ Mistake #2: "Let's build one feature first to prove the concept"
**Result:** That "one feature" becomes production code without tests

**Solution:** Prove the concept with dev infrastructure, not features

### ❌ Mistake #3: "CI can wait until we have more code"
**Result:** CI setup becomes painful, tests don't run consistently

**Solution:** CI is part of foundation, not an afterthought

### ❌ Mistake #4: "We'll document once we're done"
**Result:** Documentation never happens

**Solution:** Documentation is built alongside code

---

## Success Metrics

**After completing Milestone -1 / M-0:**

- ✅ New developer onboarded in < 1 day
- ✅ `make ci` completes in < 10 minutes
- ✅ All tests pass consistently
- ✅ Code coverage ≥80%
- ✅ Zero production bugs from untested code
- ✅ Features developed 2x faster (TDD efficiency)
- ✅ Technical debt minimized

---

## Real-World Examples

This approach has been battle-tested in:

- **RentalForge**: Multi-tenant SaaS platform (Rust + React)
- **Auth Service**: Authentication microservice (Rust + React)
- **[Your Project Here]**: Ready to use this template!

---

## Next Steps

Once Milestone -1 / M-0 is complete:

1. **Celebrate!** 🎉 You have a solid foundation
2. **Plan Milestone 1** - First set of features
3. **Write E2E tests** - For user stories in M1
4. **Implement features** - TDD approach
5. **Iterate** - Repeat for M2, M3, etc.

---

**Remember: Infrastructure first, features second. Quality is not negotiable.**
