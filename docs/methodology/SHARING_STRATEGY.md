# Project-Starter Sharing Strategy for GitHub Organizations

## Overview

This document outlines strategies for efficiently sharing the project-starter template across all repositories in the **brefwiz** GitHub organization.

**Architecture:** Full-stack monorepo (backend + frontend + database + docs in ONE repository)

---

## Strategy Options

### Option 1: GitHub Template Repository (RECOMMENDED)

**Best for:** New projects starting from scratch

**Setup:**
1. Make `project-starter` a template repository in the brefwiz organization
2. Each new project uses "Use this template" button on GitHub
3. New repository is created with fresh git history

**Advantages:**
- ✅ Simple one-click project creation
- ✅ Fresh git history (no template commits)
- ✅ Each project is independent
- ✅ No ongoing sync required
- ✅ Built into GitHub UI

**Disadvantages:**
- ❌ No automatic updates when template changes
- ❌ Manual effort to apply template improvements to existing projects

**Implementation:**
```bash
# 1. On GitHub: Settings → General → Template repository ✓

# 2. Create new project from template:
# Go to: https://github.com/brefwiz/project-starter
# Click: "Use this template" → "Create a new repository"

# 3. Clone and customize:
git clone https://github.com/brefwiz/my-new-project.git
cd my-new-project

# 4. Customize for your project
make setup
# Edit CLAUDE.md, README.md, etc.

git add .
git commit -m "chore: customize project-starter for my-new-project"
git push
```

---

### Option 2: Cookiecutter Template

**Best for:** Organizations with multiple tech stacks and customization needs

**Setup:**
1. Convert project-starter to cookiecutter template
2. Add variables for project name, tech stack, etc.
3. Generate projects with cookiecutter CLI

**Advantages:**
- ✅ Dynamic project generation with variables
- ✅ Multiple templates (Rust, Python, Node.js versions)
- ✅ Interactive prompts for customization
- ✅ Fresh git history

**Disadvantages:**
- ❌ Requires cookiecutter installation
- ❌ More complex setup
- ❌ No automatic updates

**Implementation:**

Create `cookiecutter.json`:
```json
{
  "project_name": "my-project",
  "project_slug": "{{ cookiecutter.project_name.lower().replace(' ', '-') }}",
  "tech_stack": ["rust-react", "python-react", "nodejs-react"],
  "author_name": "Brefwiz Team",
  "github_org": "brefwiz",
  "use_monitoring": "yes",
  "use_canary_deployments": "yes"
}
```

**Usage:**
```bash
# Install cookiecutter
pip install cookiecutter

# Generate new project
cookiecutter gh:brefwiz/project-starter

# Follow prompts:
# project_name: MyAwesomeApp
# tech_stack: rust-react
# use_monitoring: yes
# ...

cd my-awesome-app
make setup
```

---

### Option 3: Git Subtree (Recommended for Updates)

**Best for:** Existing projects that want to sync improvements from project-starter

**Setup:**
1. Add project-starter as a subtree in existing repositories
2. Pull updates from project-starter when needed
3. Merge or cherry-pick relevant changes

**Advantages:**
- ✅ Can pull updates from template
- ✅ All files in single repository (no submodule complexity)
- ✅ Can customize and still receive updates
- ✅ Works with existing projects

**Disadvantages:**
- ❌ More complex git workflow
- ❌ Potential merge conflicts
- ❌ Requires git subtree knowledge

**Implementation:**

**Initial setup (in existing project):**
```bash
cd my-existing-project

# Add project-starter as subtree
git subtree add --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash

# Copy files you want to use
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/
cp .project-starter/templates/milestone/TEMPLATE.md templates/milestone/

git add .
git commit -m "chore: import project-starter templates"
```

**Pull updates:**
```bash
# Pull latest changes from project-starter
git subtree pull --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash

# Review changes
git diff HEAD~1 .project-starter/

# Copy updated files you want
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/

git add .
git commit -m "chore: sync project-starter updates"
```

---

### Option 4: Shared Documentation Repository

**Best for:** Sharing methodology and documentation across projects without copying code

**Setup:**
1. Keep project-starter as documentation-only repository
2. Link to it from all project READMEs
3. Teams reference canonical documentation

**Advantages:**
- ✅ Single source of truth for methodology
- ✅ No duplication
- ✅ Easy to update across all projects
- ✅ Lightweight

**Disadvantages:**
- ❌ Doesn't provide starter code/templates
- ❌ Developers must manually implement patterns
- ❌ No code scaffolding

**Implementation:**

**In project-starter:**
```bash
# Keep as documentation-only repository
# URL: https://github.com/brefwiz/project-starter
```

**In each project's README.md:**
```markdown
## Development Methodology

This project follows the **Brefwiz Project-Starter** methodology:

📚 **Documentation:** https://github.com/brefwiz/project-starter

**Key Principles:**
- [Dev-First Approach](https://github.com/brefwiz/project-starter/blob/main/docs/methodology/DEV_FIRST_APPROACH.md)
- [Testing Guide](https://github.com/brefwiz/project-starter/blob/main/docs/testing/TESTING_GUIDE.md)
- [API Testing Requirements](https://github.com/brefwiz/project-starter/blob/main/docs/testing/API_TESTING_REQUIREMENTS.md)
```

---

### Option 5: Hybrid Approach (BEST FOR BREFWIZ)

**Recommended combination for the brefwiz organization**

**Strategy:**
1. **For new projects:** Use GitHub Template Repository
2. **For documentation:** Maintain canonical docs in project-starter
3. **For updates:** Use Git Subtree or manual sync
4. **For tech-specific scaffolding:** Choose Makefile template from `templates/makefiles/`

**Implementation:**

#### 1. Setup Template Repository
```bash
# On GitHub: brefwiz/project-starter
# Settings → General → Template repository ✓
```

#### 2. New Project Creation
```bash
# Use template to create new repository
gh repo create brefwiz/my-new-project --template brefwiz/project-starter --public
cd my-new-project

# Choose and copy Makefile for your tech stack
cp templates/makefiles/rust-axum/Makefile ./Makefile
# OR: cp templates/makefiles/python-fastapi/Makefile ./Makefile
# OR: cp templates/makefiles/nodejs-express/Makefile ./Makefile

# Setup
make setup

# Customize CLAUDE.md, README.md for your project
git add .
git commit -m "chore: customize project-starter for my-new-project"
git push
```

#### 4. Sync Updates to Existing Projects
```bash
# In existing project
cd my-existing-project

# Add as subtree (one-time)
git subtree add --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash

# Pull updates later
git subtree pull --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash

# Apply relevant changes
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/
git add docs/testing/API_TESTING_REQUIREMENTS.md
git commit -m "docs: sync API testing requirements from project-starter"
```

#### 5. Documentation Links
```markdown
# In each project's CLAUDE.md

## Brefwiz Development Standards

This project follows [brefwiz/project-starter](https://github.com/brefwiz/project-starter) methodology.

**Core Documents:**
- [Dev-First Approach](https://github.com/brefwiz/project-starter/blob/main/docs/methodology/DEV_FIRST_APPROACH.md)
- [Testing Guide](https://github.com/brefwiz/project-starter/blob/main/docs/testing/TESTING_GUIDE.md)
- [API Testing](https://github.com/brefwiz/project-starter/blob/main/docs/testing/API_TESTING_REQUIREMENTS.md)
- [E2E Testing](https://github.com/brefwiz/project-starter/blob/main/docs/testing/E2E_TESTING.md)
```

---

## Automation: GitHub Actions for Sync

**Automatically notify projects when project-starter updates:**

### In brefwiz/project-starter:

`.github/workflows/notify-updates.yml`:
```yaml
name: Notify Projects of Updates

on:
  push:
    branches:
      - main
    paths:
      - 'docs/**'
      - 'templates/**'

jobs:
  notify:
    runs-on: ubuntu-latest
    steps:
      - name: Create issue in dependent repos
        uses: actions/github-script@v6
        with:
          github-token: ${{ secrets.ORG_GITHUB_TOKEN }}
          script: |
            const repos = [
              'my-project-1',
              'my-project-2',
              'my-project-3'
            ];

            const commits = context.payload.commits
              .map(c => `- ${c.message}`)
              .join('\n');

            for (const repo of repos) {
              await github.rest.issues.create({
                owner: 'brefwiz',
                repo: repo,
                title: '📋 Project-Starter Updates Available',
                body: `New updates are available in [project-starter](https://github.com/brefwiz/project-starter):\n\n${commits}\n\nSync with:\n\`\`\`bash\ngit subtree pull --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash\n\`\`\``,
                labels: ['dependencies', 'template-sync']
              });
            }
```

### In each project repository:

`.github/workflows/sync-template.yml`:
```yaml
name: Sync Template Updates

on:
  issues:
    types: [opened]

jobs:
  auto-sync:
    if: contains(github.event.issue.title, 'Project-Starter Updates Available')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Sync template
        run: |
          git subtree pull --prefix=.project-starter \
            https://github.com/brefwiz/project-starter.git main --squash || true

      - name: Create PR
        uses: peter-evans/create-pull-request@v5
        with:
          title: 'chore: sync project-starter updates'
          body: 'Automated sync of project-starter template updates. Review and merge if appropriate.'
          branch: sync/project-starter
          labels: template-sync
```

---

## Makefile for Template Management

**Add to project-starter:**

`Makefile`:
```makefile
.PHONY: help sync-to-project create-branch

help: ## Show this help
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'

create-branch: ## Create new tech-stack branch (make create-branch STACK=rust-axum)
	@git checkout -b $(STACK)-react main
	@echo "Created branch: $(STACK)-react"
	@echo "Customize this branch for $(STACK) stack and push"

sync-to-project: ## Sync to existing project (make sync-to-project PROJECT=../my-project)
	@if [ -z "$(PROJECT)" ]; then \
		echo "Error: PROJECT path required. Usage: make sync-to-project PROJECT=../my-project"; \
		exit 1; \
	fi
	@cd $(PROJECT) && git subtree pull --prefix=.project-starter https://github.com/brefwiz/project-starter.git main --squash
	@echo "Synced to $(PROJECT)"

list-branches: ## List all tech-stack branches
	@git branch -r | grep -E "(rust|python|nodejs|go)"
```

**Usage:**
```bash
# Create new tech-stack branch
make create-branch STACK=go-gin

# Sync to existing project
make sync-to-project PROJECT=../my-existing-project
```

---

## Organization-Wide Adoption Plan

### Phase 1: Setup (Week 1)
1. ✅ Enable template repository on GitHub
2. ✅ Create tech-stack specific branches
3. ✅ Document sync procedures
4. ✅ Add automation workflows

### Phase 2: New Projects (Week 2-4)
1. ✅ All new projects MUST use template
2. ✅ Add to project creation checklist
3. ✅ Review compliance in PR reviews

### Phase 3: Existing Projects (Ongoing)
1. ✅ Add subtree sync to existing projects
2. ✅ Sync critical updates (testing, monitoring)
3. ✅ Gradual adoption of full methodology

### Phase 4: Maintenance (Ongoing)
1. ✅ Regular updates to project-starter
2. ✅ Automated notifications to projects
3. ✅ Quarterly sync review

---

## Compliance Tracking

**Create dashboard repository:**

`brefwiz/project-compliance`:
```yaml
# compliance.yml
projects:
  - name: my-project-1
    template_synced: true
    last_sync: 2026-01-20
    has_api_tests: true
    has_e2e_tests: true
    has_monitoring: true

  - name: my-project-2
    template_synced: false
    last_sync: null
    has_api_tests: false
    has_e2e_tests: true
    has_monitoring: false
```

**GitHub Action to check compliance:**
```yaml
name: Check Compliance

on:
  schedule:
    - cron: '0 0 * * 1' # Weekly

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - name: Check projects
        uses: actions/github-script@v6
        with:
          script: |
            const repos = await github.rest.repos.listForOrg({
              org: 'brefwiz'
            });

            for (const repo of repos.data) {
              // Check if .project-starter exists
              // Check if docs/testing/API_TESTING_REQUIREMENTS.md exists
              // Report compliance status
            }
```

---

## Success Metrics

Project-starter sharing is successful when:

- ✅ 100% of new projects use the template
- ✅ ≥ 80% of existing projects have synced methodology docs
- ✅ Updates propagate to projects within 1 week
- ✅ All projects have API integration tests
- ✅ All projects have E2E tests
- ✅ All projects have monitoring
- ✅ Team follows consistent development practices

---

## Recommended Approach for Brefwiz

**Use the Hybrid Strategy:**

1. **Template Repository** - For new projects
2. **Tech-Stack Branches** - For different stacks (Rust, Python, Node.js)
3. **Git Subtree** - For syncing updates to existing projects
4. **Documentation Links** - For methodology reference
5. **Automation** - For update notifications

**Implementation:**
```bash
# 1. Enable template on brefwiz/project-starter
# 2. Create branches: rust-axum-react, python-fastapi-react, nodejs-express-react
# 3. Add automation workflows
# 4. Document in each project's CLAUDE.md
# 5. Add subtree sync to existing projects
```

---

## Related Documents

- [Dev-First Approach](./DEV_FIRST_APPROACH.md)
- [Testing Guide](../testing/TESTING_GUIDE.md)
- [API Testing Requirements](../testing/API_TESTING_REQUIREMENTS.md)

---

**Next Steps:**
1. Review this strategy with the team
2. Choose the best approach for brefwiz organization
3. Implement template repository setup
4. Create tech-stack branches
5. Document sync procedures
6. Train team on new workflow
