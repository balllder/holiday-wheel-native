# Git Hooks Templates

Pre-configured Git hooks for maintaining code quality and preventing broken commits.

## Overview

Git hooks are scripts that run automatically at specific points in the Git workflow. The pre-commit hook runs before every commit, ensuring code meets quality standards.

## Files

| File | Purpose |
|------|---------|
| `pre-commit` | Runs validation before commits (format, lint, type-check, security) |
| `pre-push` | Runs comprehensive validation before pushes (all pre-commit checks + tests) |

---

## Quick Start

### 1. Install Hooks

```bash
# Install pre-commit hook (recommended for all developers)
cp templates/git-hooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit

# Install pre-push hook (optional but recommended)
cp templates/git-hooks/pre-push .git/hooks/pre-push
chmod +x .git/hooks/pre-push
```

### 2. Test Installation

```bash
# Make a test commit (hook should run automatically)
git commit -m "test: verify pre-commit hook"
```

### 3. Expected Output

```
🔍 Running pre-commit checks...

📝 Checking code formatting...
✓ Rust formatting OK

🔍 Running linters...
✓ Rust linting OK

🔬 Running type checks...
✓ TypeScript type checking OK

🔒 Running security checks...

✓ All pre-commit checks passed!
```

---

## Hook Comparison

| Check | Pre-commit | Pre-push |
|-------|-----------|----------|
| Code Formatting | ✅ | ✅ |
| Linting (strict mode) | ✅ | ✅ |
| Type Checking | ✅ | ✅ |
| Security Checks | ⚠️ (warnings) | ⚠️ (warnings) |
| Unit Tests | ❌ | ✅ |
| Integration Tests | ❌ | ✅ (if available) |

**Strategy:**
- **Pre-commit**: Fast feedback for code quality (1-5 seconds)
- **Pre-push**: Comprehensive validation before sharing (10-60 seconds)

---

## What the Hooks Check

### 1. Code Formatting ✅

**Rust (cargo fmt):**
- Checks code follows Rust formatting standards
- Fix: `cargo fmt`

**Python (black):**
- Checks code follows Python formatting standards
- Fix: `black .`

**JavaScript/TypeScript (prettier):**
- Checks code follows JS/TS formatting standards
- Fix: `npm run format`

### 2. Linting ✅ (STRICT MODE)

**Rust (clippy):**
- Checks for common mistakes and anti-patterns
- **Uses strict flags**: `--all-targets --all-features` to catch all warnings including tests
- **All warnings treated as errors**: `-D warnings` ensures no warnings are ignored
- Fix: `cargo clippy --all-targets --all-features --fix`

**Python (ruff):**
- Fast Python linter
- Fix: `ruff check --fix .`

**JavaScript/TypeScript (eslint):**
- Checks for code quality issues
- Fix: `npm run lint --fix`

### 3. Type Checking ✅

**TypeScript (tsc):**
- Verifies type correctness
- Fix: Address type errors shown by `npm run type-check`

**Python (mypy) (optional):**
- Static type checking for Python
- Fix: Address type errors shown by `mypy .`

### 4. Security Checks ⚠️

**Secret Detection:**
- Scans for potential API keys, tokens, passwords
- Warning only (doesn't block commit)

**Dependency Vulnerabilities:**
- `cargo audit` (Rust)
- `safety check` (Python)
- Warning only (doesn't block commit)

---

## Bypassing the Hook

**When to bypass:**
- Emergency hotfix
- Work-in-progress commit
- False positive from hook

**How to bypass:**
```bash
git commit --no-verify -m "WIP: in progress work"
```

**⚠️ Warning:** Use sparingly! Bypassing defeats the purpose of the hook.

---

## Customization

### Enable/Disable Checks

Edit `.git/hooks/pre-commit` and comment/uncomment sections:

```bash
# Disable unit tests (already commented by default)
# echo ""
# echo "🧪 Running unit tests..."
# ...

# Enable conventional commits validation
COMMIT_MSG_FILE=".git/COMMIT_EDITMSG"
if [ -f "$COMMIT_MSG_FILE" ]; then
    # ...validation code...
fi
```

### Adjust Check Strictness

**Make warnings into errors:**
```bash
# Current: Warning only
if ! cargo audit > /dev/null 2>&1; then
    print_warning "Cargo security audit found vulnerabilities"
fi

# Change to: Block commit
if ! cargo audit > /dev/null 2>&1; then
    print_error "Cargo security audit found vulnerabilities"
    CHECKS_FAILED=1
fi
```

### Add Custom Checks

```bash
#=============================================================================
# 7. Custom Project Checks
#=============================================================================

echo ""
echo "🎯 Running custom checks..."

# Example: Ensure OpenAPI spec is up to date
if ! make openapi-validate > /dev/null 2>&1; then
    print_error "OpenAPI spec is out of date"
    print_warning "Run: make openapi-generate"
    CHECKS_FAILED=1
fi
```

---

## Alternative: Husky (Node.js Projects)

For Node.js projects, consider using Husky for easier hook management:

### Setup Husky

```bash
# Install husky
npm install --save-dev husky

# Initialize
npx husky install

# Add pre-commit hook
npx husky add .husky/pre-commit "npm run pre-commit"
```

### package.json Scripts

```json
{
  "scripts": {
    "pre-commit": "npm run format:check && npm run lint && npm run type-check",
    "format:check": "prettier --check .",
    "lint": "eslint .",
    "type-check": "tsc --noEmit"
  }
}
```

**Benefits:**
- Easier to version control (hooks stored in repo)
- Cross-platform (works on Windows)
- Integrated with npm scripts

---

## Makefile Integration

Add hook installation to Makefile:

```makefile
.PHONY: setup-hooks

setup-hooks: ## Install Git hooks
	@echo "Installing Git hooks..."
	@cp templates/git-hooks/pre-commit .git/hooks/pre-commit
	@cp templates/git-hooks/pre-push .git/hooks/pre-push
	@chmod +x .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-push
	@echo "✓ Pre-commit hook installed"
	@echo "✓ Pre-push hook installed"
```

Usage:
```bash
make setup-hooks
```

---

## Troubleshooting

### Hook doesn't run

**Problem:** Commits succeed without running hook

**Solution:**
1. Check hook is executable: `ls -la .git/hooks/pre-commit`
2. If not: `chmod +x .git/hooks/pre-commit`
3. Verify file location: Must be in `.git/hooks/`, not `templates/git-hooks/`

---

### Hook always fails

**Problem:** Hook fails even when code is correct

**Solutions:**

1. **Check tool installation:**
   ```bash
   cargo --version  # Rust tools
   black --version  # Python formatter
   npm run lint     # Node.js linters
   ```

2. **Run checks manually:**
   ```bash
   cargo fmt -- --check
   cargo clippy
   npm run format:check
   npm run lint
   ```

3. **Check for hidden errors:**
   ```bash
   # Remove > /dev/null 2>&1 from hook to see errors
   cargo clippy  # Instead of cargo clippy > /dev/null 2>&1
   ```

---

### Hook is too slow

**Problem:** Hook takes too long to run

**Solutions:**

1. **Disable slow checks:**
   - Comment out unit tests (already default)
   - Comment out security scans
   - Keep only format and lint

2. **Run full checks in CI only:**
   ```bash
   # Pre-commit: Fast checks only
   - Format check
   - Linting

   # CI: Comprehensive checks
   - Format check
   - Linting
   - Type checking
   - Unit tests
   - Integration tests
   - Security scans
   ```

3. **Use --no-verify for WIP commits:**
   ```bash
   git commit --no-verify -m "WIP: in progress"
   ```

---

### False positives from secret detection

**Problem:** Hook warns about false positive secrets

**Solution:**

1. **Whitelist pattern:**
   ```bash
   # Add to pre-commit hook
   if git diff --cached | grep -iE "(api_key|secret)" | grep -v "example" > /dev/null 2>&1; then
       # Excludes lines with "example"
   fi
   ```

2. **Use placeholder values:**
   ```bash
   API_KEY=your-api-key-here  # Won't trigger
   SECRET=CHANGE_ME           # Won't trigger
   ```

---

## Best Practices

### ✅ DO

1. **Run hook on every commit**
   - Catches issues early
   - Ensures code quality

2. **Keep hook fast**
   - Only essential checks
   - Save comprehensive checks for CI

3. **Make hook easy to bypass**
   - Use `--no-verify` when needed
   - Document when to bypass

4. **Version control hook template**
   - Keep in `templates/git-hooks/`
   - Update as project evolves

5. **Document custom checks**
   - Comment why each check exists
   - Explain how to fix failures

### ❌ DON'T

1. **Don't run integration tests**
   - Too slow for pre-commit
   - Run in CI instead

2. **Don't make warnings block commits**
   - Security warnings should inform, not block
   - Let developers decide

3. **Don't require hook installation**
   - Make it opt-in
   - Some developers use other tools

4. **Don't hide error messages**
   - Show clear fix instructions
   - Colored output helps

---

## Examples

### Successful Commit

```bash
$ git commit -m "feat: add user authentication"

🔍 Running pre-commit checks...

📝 Checking code formatting...
✓ Rust formatting OK

🔍 Running linters...
✓ Rust linting OK

🔬 Running type checks...
✓ TypeScript type checking OK

🔒 Running security checks...

✓ All pre-commit checks passed!

[main abc1234] feat: add user authentication
 3 files changed, 150 insertions(+)
```

### Failed Commit (Formatting)

```bash
$ git commit -m "feat: add user authentication"

🔍 Running pre-commit checks...

📝 Checking code formatting...
✗ Rust code formatting check failed
⚠ Run: cargo fmt

Fix the issues above and try again.
Or use --no-verify to bypass (not recommended):
  git commit --no-verify
```

### Bypassing Hook

```bash
$ git commit --no-verify -m "WIP: work in progress"

[main abc1234] WIP: work in progress
 1 file changed, 10 insertions(+)
```

---

## Pre-Push Hook Details

The pre-push hook runs comprehensive validation before code reaches the remote repository.

### What Pre-Push Checks

1. **All Pre-Commit Checks** (formatting, linting, type-checking)
2. **Complete Test Suite**:
   - Rust: `cargo test --all-features`
   - Python: `pytest` (if available)
   - JavaScript/TypeScript: `npm test` (if test script exists)
3. **Security Audits**:
   - `cargo audit` for Rust dependencies
   - `safety check` for Python dependencies

### When to Use Pre-Push

**Install if:**
- You want to ensure tests pass before pushing
- Your team requires CI-like validation locally
- You want to catch issues before creating pull requests

**Skip if:**
- You rely entirely on CI for testing
- Hook execution time is too slow for your workflow
- You frequently push WIP commits

### Pre-Push vs CI

| Aspect | Pre-Push Hook | CI Pipeline |
|--------|---------------|-------------|
| Speed | Runs locally | Network latency + queue time |
| Feedback | Immediate | 2-10 minutes typical |
| Cost | Free (your machine) | CI minutes consumed |
| Bypass | `--no-verify` | Cannot bypass |
| Scope | Your changes | Full integration |

**Best Practice**: Use both! Pre-push catches issues fast, CI validates integration.

---

## Provided Hooks

This template provides two essential hooks:

| Hook | When it Runs | What it Does |
|------|--------------|--------------|
| `pre-commit` | Before commit | Fast quality checks (format, lint, type-check) |
| `pre-push` | Before push | Comprehensive validation (all pre-commit + tests) |

## Additional Hooks (Not Provided)

Git supports many other hooks you can add:

| Hook | When it Runs | Use For |
|------|--------------|---------|
| `commit-msg` | After commit message entered | Validate commit format |
| `post-merge` | After successful merge | Update dependencies |
| `pre-rebase` | Before rebase | Validate rebase safety |

**Example commit-msg hook:**
```bash
#!/bin/bash
# Enforce conventional commits

COMMIT_MSG_FILE=$1
COMMIT_MSG=$(cat "$COMMIT_MSG_FILE")

if ! echo "$COMMIT_MSG" | grep -qE "^(feat|fix|docs|style|refactor|test|chore)(\(.+\))?: .{10,}"; then
    echo "ERROR: Commit message must follow conventional commits"
    echo "Format: type(scope): description"
    echo "Example: feat(auth): add login endpoint"
    exit 1
fi
```

---

## References

- [Git Hooks Documentation](https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks)
- [Husky](https://typicode.github.io/husky/) - Git hooks made easy
- [Pre-commit Framework](https://pre-commit.com/) - Multi-language hook framework
- [Conventional Commits](https://www.conventionalcommits.org/) - Commit message format

---

**Last Updated:** 2026-01-24
**Improvements:** Added comprehensive pre-push hook with strict clippy validation
**Extracted from:** auth-service development workflow
