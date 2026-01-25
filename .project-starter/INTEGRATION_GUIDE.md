# Project-Starter Integration Guide

**How to integrate project-starter into your existing project and stay synchronized.**

---

## Quick Start

### 1. Add as Git Subtree (One-Time Setup)

```bash
cd your-project

# Add project-starter as subtree
git subtree add --prefix=.project-starter \
  git@github.com:brefwiz/project-starter.git main --squash

git push
```

**What this does:**
- Creates `.project-starter/` directory with full project-starter content
- Maintains connection to upstream for future updates
- Keeps clean git history with squashed commits

---

## 2. Install Automatic Sync Workflow

**Copy the sync workflow to your project:**

```bash
# Create workflows directory if it doesn't exist
mkdir -p .github/workflows

# Copy the sync template
cp .project-starter/.github/workflows/sync-template.yml.example \
   .github/workflows/sync-template.yml

# Commit
git add .github/workflows/sync-template.yml
git commit -m "ci: add project-starter auto-sync workflow"
git push
```

**What this workflow does:**
- Automatically pulls project-starter updates when notified
- Creates a PR with changes for review
- Handles merge conflicts gracefully
- Closes notification issues when complete

---

## 3. Enable Update Notifications (Optional but Recommended)

**To receive automatic notifications when project-starter is updated:**

1. **Fork project-starter** (if you haven't already)
2. **Edit `.github/workflows/notify-updates.yml`** in your fork
3. **Add your project to the repos list:**

```javascript
const repos = [
  'your-project-name',  // ← Add your repo here
  // 'my-project-2',
];
```

4. **Create a PR** to project-starter with your addition

**When enabled:**
- GitHub issues are automatically created in your repo when updates are available
- The sync workflow (step 2) automatically syncs the changes
- You get a PR to review before merging

---

## Using Project-Starter in Your Project

### Understand the Pattern, Then Adapt

**CRITICAL:** Project-starter provides **reference patterns**, not copy-paste code.

See **[CLAUDE.md - Using Project-Starter Templates](CLAUDE.md#-using-project-starter-templates-critical)** for comprehensive guidance on adapting vs copying.

### What to Copy Directly

✅ **Safe to copy as-is:**
- Makefile command conventions (`make migrate`, `make test`)
- Git hooks scripts (copy and adapt to your needs)
- Documentation structure
- CI/CD workflow patterns
- Dockerfile multi-stage build patterns

### What to Adapt to Your Project

⚠️ **MUST adapt:**
- Health check implementations (check YOUR dependencies)
- Configuration values (`.env` contents)
- Database schemas and migrations
- Service-specific business logic
- Authentication/authorization flows

**Example:**
```bash
# ❌ DON'T blindly copy
cp .project-starter/templates/health-check/rust-axum/health.rs backend/src/api/health.rs

# ✅ DO understand pattern and implement for YOUR dependencies
# 1. Read template to understand liveness vs readiness pattern
# 2. Identify YOUR dependencies (PostgreSQL, Redis, S3, etc.)
# 3. Implement health checks for YOUR actual services
```

---

## Keeping in Sync

### Manual Sync (Without Workflow)

```bash
# Pull latest changes
git subtree pull --prefix=.project-starter \
  git@github.com:brefwiz/project-starter.git main --squash

# Review changes
git diff HEAD~1 .project-starter/

# Adapt relevant updates to your project
# (Remember: adapt, don't blindly copy!)
cp .project-starter/docs/testing/NEW_DOC.md docs/testing/
# Customize for your project...

# Commit
git add .
git commit -m "docs: sync project-starter updates"
git push
```

### Automatic Sync (With Workflow)

If you installed the sync workflow (step 2):

1. **Wait for notification** - Issue created when project-starter updates
2. **Review the PR** - Workflow creates PR automatically
3. **Merge when ready** - Approve and merge the PR

---

## Contributing Back to Project-Starter

Found a pattern that would benefit others? Contribute it back!

### Option 1: Push from Subtree

```bash
# Make changes in .project-starter/
# Example: improve a template
vim .project-starter/templates/health-check/rust-axum/health.rs

# Commit locally
git add .project-starter/
git commit -m "feat: improve health check error handling"

# Push to project-starter (creates remote branch)
git subtree push --prefix=.project-starter \
  git@github.com:brefwiz/project-starter.git feature/improve-health-checks

# Create PR in project-starter repository
# Go to: https://github.com/brefwiz/project-starter/pulls
```

### Option 2: Direct PR to Project-Starter

```bash
# Clone project-starter separately
git clone git@github.com:brefwiz/project-starter.git
cd project-starter

# Create branch and make changes
git checkout -b feature/new-pattern
# ... make changes ...
git commit -m "feat: add new pattern"
git push origin feature/new-pattern

# Create PR on GitHub
```

---

## Troubleshooting

### Merge Conflicts During Sync

```bash
# If subtree pull fails with conflicts
git subtree pull --prefix=.project-starter \
  git@github.com:brefwiz/project-starter.git main --squash

# If conflicts occur:
git status  # See conflicted files
# Resolve conflicts in .project-starter/
git add .project-starter/
git commit -m "chore: resolve project-starter sync conflicts"
```

**Tip:** Conflicts usually mean you modified files in `.project-starter/` directly. Consider:
- Keep your customizations in your project root (not in subtree)
- Or contribute changes back to project-starter

### Sync Workflow Not Triggering

**Check:**
1. Workflow file exists: `.github/workflows/sync-template.yml`
2. Workflow is enabled in Settings → Actions
3. Repository has `contents: write` and `pull-requests: write` permissions
4. Notification issue title contains "Project-Starter Updates Available"

### No Notifications Received

**Possible reasons:**
1. Your repo not added to `notify-updates.yml` in project-starter
2. Notifications disabled in your GitHub settings
3. Project-starter hasn't had updates since you subscribed

---

## Integration Checklist

Use this checklist when integrating project-starter:

- [ ] **Initial Setup**
  - [ ] Add as git subtree
  - [ ] Install sync workflow
  - [ ] Request addition to notification list (optional)

- [ ] **Project Configuration**
  - [ ] Copy CLAUDE.md and customize for your project
  - [ ] Adapt templates to your tech stack
  - [ ] Set up Makefile with your commands
  - [ ] Configure git hooks

- [ ] **Understand Patterns**
  - [ ] Read "Using Project-Starter Templates" section
  - [ ] Identify which patterns to copy vs adapt
  - [ ] Document customizations in your CLAUDE.md

- [ ] **Ongoing Maintenance**
  - [ ] Monthly: Check for upstream updates
  - [ ] Quarterly: Review pattern consistency
  - [ ] As needed: Contribute improvements back

---

## Example: Auth-Service Integration

**Real-world case study:** Auth-service integrated project-starter and:

1. **Added as subtree** ✅
   ```bash
   git subtree add --prefix=.project-starter \
     git@github.com:brefwiz/project-starter.git main --squash
   ```

2. **Adapted health checks** ✅
   - Template had: Static JSON response
   - Auth-service needs: PostgreSQL + Redis + Biscuit service checks
   - Result: Implemented proper readiness probe for Kubernetes

3. **Used patterns correctly** ✅
   - Makefile commands: Copied (`make migrate`, `make test`)
   - E2E test isolation: Copied and extended
   - Health implementation: Adapted for actual dependencies

4. **Contributed back** ✅
   - 10 PRs to project-starter (6,548 lines)
   - Real-world validation of patterns
   - This integration guide based on learnings

**Lesson:** Understanding patterns > blindly copying code

---

## Additional Resources

- **[CLAUDE.md](CLAUDE.md)** - Complete development guidelines
- **[README.md](README.md)** - Project overview and quick start
- **[templates/](templates/)** - All available templates
- **[docs/](docs/)** - Comprehensive guides

---

**Last Updated:** 2026-01-24  
**Maintainer:** Project-Starter Team  
**Questions?** Open an issue at https://github.com/brefwiz/project-starter/issues
