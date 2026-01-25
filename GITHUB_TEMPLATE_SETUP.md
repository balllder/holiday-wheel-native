# GitHub Template Repository Setup Guide

## Step-by-Step Instructions for Enabling Template Repository

### 1. Enable Template Repository on GitHub

**IMPORTANT: This step requires GitHub web UI access**

1. Navigate to: https://github.com/brefwiz/project-starter
2. Click **Settings** (top navigation)
3. Scroll down to **Template repository** section
4. Check the box: ✅ **"Template repository"**
5. Click **Save**

**Result:** The repository will show a "Use this template" button on the main page.

---

### 2. Configure Repository Settings

**Recommended Settings:**

1. **Visibility:** Public or Private (based on your org policy)
2. **Features to Enable:**
   - ✅ Issues
   - ✅ Discussions (optional)
   - ✅ Wiki (optional)
3. **Branch Protection:**
   - Protect `main` branch
   - Require pull request reviews
   - Require status checks to pass

---

### 3. Update Notification Workflow

Edit `.github/workflows/notify-updates.yml`:

```yaml
# Update this list with your actual repositories
const repos = [
  'my-project-1',      # Add your project names here
  'my-project-2',
  'my-project-3',
];
```

**How to find repository names:**
```bash
# List all repos in brefwiz organization
gh repo list brefwiz --limit 100
```

---

### 4. Grant Workflow Permissions

**For auto-notifications to work:**

1. Go to: Settings → Actions → General
2. Scroll to: **Workflow permissions**
3. Select: **Read and write permissions**
4. Check: ✅ **Allow GitHub Actions to create and approve pull requests**
5. Click **Save**

---

### 5. Create Organization Secret (Optional)

**For cross-repo notifications:**

If you want workflows to create issues in OTHER repos:

1. Create GitHub Personal Access Token (PAT):
   - Go to: https://github.com/settings/tokens/new
   - Name: `PROJECT_STARTER_AUTOMATION`
   - Scopes: `repo`, `workflow`
   - Generate token

2. Add as Organization Secret:
   - Go to: https://github.com/organizations/brefwiz/settings/secrets/actions
   - New organization secret
   - Name: `ORG_GITHUB_TOKEN`
   - Value: [paste PAT]
   - Repository access: All repositories

3. Update workflow to use: `${{ secrets.ORG_GITHUB_TOKEN }}`

---

### 6. Test Template Creation

**Verify template works:**

1. Go to: https://github.com/brefwiz/project-starter
2. Click: **"Use this template"** button
3. Select: **"Create a new repository"**
4. Fill in:
   - Owner: brefwiz
   - Repository name: test-template-creation
   - Description: Testing template
   - Public/Private
5. Click: **"Create repository"**
6. Verify new repo has all files
7. Delete test repo after verification

---

## Using the Template

### For New Projects

**Option 1: GitHub Web UI**
1. Go to: https://github.com/brefwiz/project-starter
2. Click: "Use this template"
3. Create new repository
4. Clone and customize

**Option 2: GitHub CLI**
```bash
gh repo create brefwiz/my-new-project \
  --template brefwiz/project-starter \
  --public

cd my-new-project
make setup
```

**Option 3: Specific Tech Stack Branch**
```bash
# Use specific tech-stack branch
gh repo create brefwiz/my-new-project \
  --template brefwiz/project-starter \
  --public

cd my-new-project

# Switch to tech-stack branch
git fetch origin rust-axum-react
git checkout -b main origin/rust-axum-react

# Customize
make setup
```

---

### For Existing Projects

**Add as subtree for syncing updates:**

```bash
cd my-existing-project

# One-time setup
git subtree add --prefix=.project-starter \
  https://github.com/brefwiz/project-starter.git main --squash

# Copy files you want
cp .project-starter/docs/testing/API_TESTING_REQUIREMENTS.md docs/testing/
cp .project-starter/templates/milestone/TEMPLATE.md templates/milestone/

# Commit
git add .
git commit -m "chore: add project-starter as subtree"
git push
```

**Pull updates later:**
```bash
git subtree pull --prefix=.project-starter \
  https://github.com/brefwiz/project-starter.git main --squash

# Review and copy updated files
git diff HEAD~1 .project-starter/
```

---

## Verification Checklist

After setup, verify:

- [ ] Repository shows "Template repository" badge on GitHub
- [ ] "Use this template" button is visible
- [ ] Can create new repo from template
- [ ] All files copy correctly to new repo
- [ ] Workflows are enabled in new repo
- [ ] Notification workflow runs on push to main
- [ ] Template validation workflow passes

---

## Troubleshooting

### Template button not showing
- Verify "Template repository" is checked in Settings
- Refresh GitHub page

### Workflows not running in new repos
- Check Actions are enabled: Settings → Actions → General
- Verify workflow permissions are set correctly

### Notifications not creating issues
- Check repository list in notify-updates.yml
- Verify workflow permissions (read/write)
- For cross-repo: verify ORG_GITHUB_TOKEN secret exists

### Subtree conflicts
- Use `--squash` flag to avoid merge history
- For conflicts: resolve manually or use fresh copy

---

## Next Steps

1. ✅ Enable template repository
2. ✅ Configure workflow permissions
3. ✅ Update notification workflow with repo list
4. ✅ Test template creation
5. ✅ Create tech-stack branches
6. ✅ Document in organization wiki
7. ✅ Train team on usage

---

## Support

For issues:
- **Template setup:** Check this guide
- **GitHub features:** https://docs.github.com/en/repositories/creating-and-managing-repositories/creating-a-template-repository
- **Workflows:** https://docs.github.com/en/actions

---

**Last Updated:** 2026-01-24
