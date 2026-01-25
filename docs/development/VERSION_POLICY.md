# Version Policy

## Overview

This document explains project-starter's approach to dependency versioning and why specific versions are enforced.

---

## Why Specific Versions Matter

### Reproducibility

**Problem:** "Works on my machine" syndrome
**Solution:** Pin exact versions in all dependency files

```bash
# ❌ BAD: Unpredictable versions
postgres:latest           # Could be 13, 14, 15, or 16
"react": "^19.0.0"       # Could be 19.0.0 or 19.9.9

# ✅ GOOD: Explicit versions
postgres:18-alpine        # Always PostgreSQL 18
"react": "19.0.0"        # Always React 19.0.0
```

### Debugging

**Problem:** Version-specific bugs are hard to diagnose without knowing exact versions
**Solution:** Document and enforce specific versions

**Example:**
- Axum 0.6 had different error handling than 0.7
- Knowing the exact version helps find relevant issues and documentation

### Security

**Problem:** Vulnerabilities affect specific version ranges
**Solution:** Know exactly what versions you're running

**Example:**
- CVE-2023-1234 affects PostgreSQL 14.0-14.6
- With `postgres:latest`, you don't know if you're vulnerable
- With `postgres:18-alpine`, you can check security advisories

### Upgrade Path

**Problem:** Unknown current versions make upgrades risky
**Solution:** Document current versions to plan upgrades

**Example:**
- Current: React 18.2.0
- Target: React 19.0.0
- Migration guide: React 18 → 19 upgrade documentation

---

## Versioning Strategy by Component

### Rust Dependencies (Cargo.toml)

**Strategy:** Use caret requirements with known compatible versions

```toml
[dependencies]
axum = "0.7"              # Allows 0.7.x but not 0.8.0
tokio = { version = "1.35", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres"] }
```

**Update policy:**
- **Patch updates (0.7.x):** Auto-update via `cargo update`
- **Minor updates (0.x):** Test before updating
- **Major updates (x):** Review migration guides first

### Docker Images (docker-compose.yml)

**Strategy:** Use specific major versions with tag

```yaml
services:
  postgres:
    image: postgres:18-alpine  # ✅ Major version pinned
    # NOT: postgres:latest     # ❌ Unpredictable

  redis:
    image: redis:7.2-alpine    # ✅ Major + minor pinned
```

**Update policy:**
- **Patch updates:** Handled by base image maintainers
- **Minor updates:** Test before updating docker-compose.yml
- **Major updates:** Review breaking changes, test thoroughly

### Frontend Dependencies (package.json)

**Strategy:** Use exact versions (no ^ or ~)

```json
{
  "dependencies": {
    "react": "19.0.0",         // ✅ Exact version
    "typescript": "5.3.3"      // ✅ Exact version
  },
  "devDependencies": {
    "playwright": "1.40.0",    // ✅ Exact version
    "vite": "5.0.0"            // ✅ Exact version
  }
}
```

**Update policy:**
- **Patch updates:** Run `npm update` after testing
- **Minor updates:** Review changelog, test
- **Major updates:** Review migration guide, comprehensive testing

### Runtime Versions

**Strategy:** Document minimum required versions

```markdown
## Requirements

- **Node.js:** 20+ (LTS)
- **Rust:** 1.75+ (stable)
- **Docker:** 24+
- **Docker Compose:** 3.8+
```

**Update policy:**
- Update minimum version when using new features
- Test on minimum version before updating docs

---

## Version Documentation Checklist

When adding a new dependency:

- [ ] Add to README.md Tech Stack section
- [ ] Specify exact version in dependency file
- [ ] Add version to Docker Compose if applicable
- [ ] Document minimum runtime version
- [ ] Note any version-specific configuration

---

## Common Version Scenarios

### Scenario 1: New Project Setup

**Goal:** Get exact versions from template

```bash
# Clone template
git clone https://github.com/brefwiz/project-starter.git

# All versions are already specified
cat backend/Cargo.toml        # Rust dependencies
cat frontend/package.json     # Frontend dependencies
cat docker-compose.yml        # Infrastructure versions
```

**Result:** Reproducible build on any machine

### Scenario 2: Dependency Update

**Goal:** Update Axum from 0.7.0 to 0.7.5

```bash
# 1. Check current version
grep "axum" backend/Cargo.toml
# axum = "0.7"

# 2. Update Cargo.lock
cd backend && cargo update -p axum

# 3. Test
make backend-test

# 4. If tests pass, commit
git commit -am "chore: update axum to 0.7.5"
```

### Scenario 3: Major Version Upgrade

**Goal:** Upgrade PostgreSQL 17 → 18

```bash
# 1. Read migration guide
# https://www.postgresql.org/docs/18/release-18.html

# 2. Update docker-compose.yml
- image: postgres:17-alpine
+ image: postgres:18-alpine

# 3. Backup data (production)
make db-backup

# 4. Test migrations
make dev-up
make migrate

# 5. Run all tests
make test

# 6. Update documentation
- **PostgreSQL** | 15 | ...
+ **PostgreSQL** | 16 | ...
```

### Scenario 4: Security Vulnerability

**Goal:** Fix CVE in dependency

```bash
# 1. Check which version you have
cat backend/Cargo.toml

# 2. Check if vulnerability affects your version
# (Use GitHub Security Advisories, RustSec, etc.)

# 3. Update to patched version
# Update Cargo.toml with safe version

# 4. Test
make backend-test

# 5. Deploy immediately if critical
```

---

## Version Validation Commands

### Check Current Versions

```bash
# Rust
cargo --version
rustc --version

# Node.js & npm
node --version
npm --version

# Docker
docker --version
docker-compose --version

# PostgreSQL (in container)
docker-compose exec postgres psql --version
```

### Verify Dependency Versions

```bash
# Rust dependencies
cd backend && cargo tree

# Frontend dependencies
cd frontend && npm list

# Docker images
docker images | grep project-starter
```

### Test Version Compatibility

```bash
# Run full test suite
make test

# Run in fresh environment
docker-compose down -v
docker-compose up -d
make test
```

---

## When to Update Versions

### Always Update

- **Security patches** (CVEs)
- **Critical bug fixes**

### Regularly Update (Monthly)

- **Patch versions** (e.g., 0.7.3 → 0.7.4)
- **Dev dependencies** (testing tools, linters)

### Carefully Update (Quarterly)

- **Minor versions** (e.g., 0.7.x → 0.8.x)
- **Database versions** (e.g., PostgreSQL 17 → 18)

### Plan Before Updating (When Needed)

- **Major versions** (e.g., React 18 → 19)
- **Language versions** (e.g., Rust 1.75 → 1.76)
- **Framework versions** (e.g., Axum 0.7 → 1.0)

---

## Version Pinning Best Practices

### DO

✅ Use specific versions in docker-compose.yml
✅ Pin exact versions in package.json
✅ Document minimum required versions
✅ Test after updating any version
✅ Keep Cargo.lock and package-lock.json in git
✅ Review changelogs before updating

### DON'T

❌ Use `:latest` tag in Docker images
❌ Use `^` or `~` in package.json (for this template)
❌ Update all dependencies at once
❌ Skip testing after version updates
❌ Ignore security advisories
❌ Update in production without testing

---

## Example: Full Version Audit

Run this checklist quarterly:

```bash
# 1. Check for outdated Rust dependencies
cd backend && cargo outdated

# 2. Check for outdated npm packages
cd frontend && npm outdated

# 3. Check for Docker image updates
# Visit Docker Hub for each image:
# - postgres:18-alpine
# - node:20-alpine
# - rust:1.75-slim

# 4. Review security advisories
# - RustSec: https://rustsec.org
# - npm audit: npm audit
# - GitHub Security: Check Dependabot alerts

# 5. Plan updates
# - Create update plan with testing steps
# - Schedule updates during low-traffic period
# - Prepare rollback plan

# 6. Update one component at a time
# - Update, test, commit
# - If issues arise, rollback immediately

# 7. Update documentation
# - README.md Tech Stack section
# - This VERSION_POLICY.md if policy changes
```

---

## References

- [Semantic Versioning](https://semver.org/)
- [Cargo Book - Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- [npm - package.json](https://docs.npmjs.com/cli/v10/configuring-npm/package-json)
- [Docker - Image Tags Best Practices](https://docs.docker.com/develop/dev-best-practices/)
- [PostgreSQL Release Policy](https://www.postgresql.org/support/versioning/)

---

**Last Updated:** 2026-01-24
**Applies to:** project-starter v1.0+
