# Rust Clippy Configuration Best Practices

> **Purpose**: Production-tested Clippy configuration for consistent code quality across Rust projects. Based on RentalForge implementation with zero production panics.

---

## Quick Start

**Copy recommended configuration to your `Cargo.toml`:**

```bash
# From templates/rust/Cargo.toml.clippy
cat templates/rust/Cargo.toml.clippy >> Cargo.toml
```

**Run Clippy:**

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

---

## Configuration Philosophy

### Three-Tier Approach

| Level | Severity | Action | Example |
|-------|----------|--------|---------|
| **Deny** | Critical | MUST fix before merge | `unwrap_used`, `panic`, `dbg_macro` |
| **Warn** | Important | Should fix eventually | `pedantic`, `nursery` |
| **Allow** | Justified | Explicitly allowed with reason | `clone_on_copy`, `large_enum_variant` |

### Core Principles

1. **No Panics in Production** - All `unwrap()`, `expect()`, `panic!()` are denied
2. **Explicit Error Handling** - Every failure case is handled
3. **Complexity Limits** - Functions stay under cognitive complexity 15
4. **Justified Exceptions** - Every `#[allow]` has a comment explaining why

---

## Recommended Configuration

**Add to your `Cargo.toml`:**

```toml
[workspace.lints.clippy]
# === DENIAL LIST - MUST BE FIXED ===
# These are critical issues that can cause production failures

# Debugging artifacts - Must be removed before merge
dbg_macro = "deny"
todo = "deny"
unimplemented = "deny"

# Panic prevention - No unwrap/expect/panic in production
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"

# === WARNING LIST - SHOULD BE FIXED ===
# These improve code quality but aren't critical

# Pedantic lint group - Catches common mistakes
pedantic = "warn"

# Nursery lint group - Experimental but useful
nursery = "warn"

# === ALLOWED EXCEPTIONS (WITH JUSTIFICATION) ===
# These have specific use cases where they're acceptable

# Allow .clone() for simplicity in non-hot paths
# Justification: Readability > micro-optimization outside hot loops
clone_on_copy = "allow"

# Allow large enum variants (our errors are intentionally detailed)
# Justification: Error types benefit from rich context
large_enum_variant = "allow"

# Allow module inception (auth/auth.rs is clear)
# Justification: Standard Rust module pattern
module_inception = "allow"
```

---

## Panic Prevention

### The Problem

```rust
// ❌ WRONG - These can panic in production
let value = option.unwrap();
let result = fallible_operation().expect("should work");
let user = users.get(0).unwrap();

// Result:
// thread 'main' panicked at 'called `Option::unwrap()` on a `None` value'
// Production downtime! 🔥
```

### The Solution

**Use explicit error handling:**

```rust
// ✅ CORRECT - Explicit error propagation
let value = option.ok_or(AppError::ValueMissing)?;

let result = fallible_operation()
    .map_err(|e| AppError::OperationFailed(e.to_string()))?;

let user = users
    .first()
    .ok_or(AppError::UserNotFound)?;
```

**Or use default values:**

```rust
// ✅ CORRECT - Safe defaults
let value = option.unwrap_or_default();
let count = option.unwrap_or(0);
let name = option.unwrap_or_else(|| "Guest".to_string());
```

---

## Error Handling Patterns

### Pattern 1: Result Propagation

```rust
pub async fn create_user(
    db: &PgPool,
    email: String,
) -> Result<User, AppError> {
    // Validate input
    validate_email(&email)
        .map_err(AppError::Validation)?;

    // Insert into database
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email) VALUES ($1) RETURNING *",
        email
    )
    .fetch_one(db)
    .await
    .map_err(AppError::Database)?;

    Ok(user)
}
```

### Pattern 2: Option to Result Conversion

```rust
pub fn find_user_by_id(id: i32) -> Result<User, AppError> {
    let user = USERS
        .iter()
        .find(|u| u.id == id)
        .ok_or(AppError::UserNotFound(id))?;

    Ok(user.clone())
}
```

### Pattern 3: Early Returns

```rust
pub fn process_request(req: Request) -> Result<Response, AppError> {
    // Guard clauses - fail fast
    if !req.is_authenticated {
        return Err(AppError::Unauthorized);
    }

    if req.body.is_empty() {
        return Err(AppError::EmptyBody);
    }

    // Happy path
    let data = parse_body(&req.body)?;
    let result = process_data(data)?;
    Ok(Response::success(result))
}
```

---

## Allowed Exceptions

### When to Use `#[allow]`

**ONLY use `#[allow]` when:**
1. You have a specific, documented reason
2. The alternative is worse for readability/maintainability
3. You've considered refactoring first

**ALWAYS include a comment explaining why:**

```rust
// ✅ CORRECT - Justified exception
// Allow: Error types are intentionally detailed for better debugging
#[allow(clippy::large_enum_variant)]
pub enum AppError {
    Database(DatabaseError),      // Large: 256 bytes
    Authentication(AuthError),     // Large: 128 bytes
    Validation(ValidationError),   // Small: 32 bytes
}
```

### Common Justified Exceptions

#### 1. `clone_on_copy` - Readability in Non-Hot Paths

```rust
// Allow: Improves readability, not in hot path
#[allow(clippy::clone_on_copy)]
let user_id = current_user.id.clone();
log_action(user_id);
```

**When NOT to allow:**
- Inside loops
- In hot paths (called frequently)
- When performance matters

#### 2. `large_enum_variant` - Detailed Error Types

```rust
// Allow: Rich error context is more valuable than memory optimization
#[allow(clippy::large_enum_variant)]
pub enum PaymentError {
    // Large variant with all payment context
    ProcessingFailed {
        amount: Decimal,
        currency: String,
        provider: PaymentProvider,
        metadata: HashMap<String, String>,
        trace_id: Uuid,
    },
    // Small variant
    InvalidAmount,
}
```

**Alternatives to consider:**
- Box large variants: `ProcessingFailed(Box<PaymentDetails>)`
- Split into separate error types
- Use `anyhow::Error` for internal errors

#### 3. `module_inception` - Standard Rust Pattern

```rust
// Allow: auth/auth.rs is a clear, standard Rust module pattern
#[allow(clippy::module_inception)]
mod auth;
```

**When this is acceptable:**
- Following standard Rust module conventions
- Matches cargo conventions (lib.rs, main.rs)

---

## Debugging vs Production

### Development Workflow

```rust
// ✅ CORRECT - Use for debugging only
#[cfg(debug_assertions)]
{
    dbg!(&user);
    println!("DEBUG: Processing user {}", user.id);
}

// Production code uses proper logging
tracing::debug!("processing user {}", user.id);
```

### Testing Exceptions

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_user_creation() {
        let user = create_user("test@example.com").unwrap();
        // ✅ CORRECT - unwrap() is fine in tests
        assert_eq!(user.email, "test@example.com");
    }
}
```

**Why unwrap is OK in tests:**
- Tests should fail fast on unexpected errors
- Stack traces help debug test failures
- Tests aren't running in production

---

## CI/CD Integration

### GitHub Actions

```yaml
- name: Run Clippy
  run: cargo clippy --all-targets --all-features -- -D warnings
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running Clippy..."
cargo clippy --all-targets --all-features -- -D warnings

if [ $? -ne 0 ]; then
    echo "❌ Clippy failed - fix issues before committing"
    exit 1
fi

echo "✅ Clippy passed"
```

### Makefile Integration

```makefile
.PHONY: lint
lint:
	@echo "Running Clippy..."
	@cargo clippy --all-targets --all-features -- -D warnings

.PHONY: lint-fix
lint-fix:
	@echo "Auto-fixing Clippy issues..."
	@cargo clippy --fix --all-targets --all-features --allow-dirty
```

---

## Function Complexity

**See [FUNCTION_COMPLEXITY.md](FUNCTION_COMPLEXITY.md) for detailed guidance.**

**Quick rule:** Functions should have cognitive complexity ≤ 15.

```rust
// ❌ TOO COMPLEX - Cognitive complexity: 25
pub fn validate_and_process(data: &Data) -> Result<Output> {
    if data.is_valid {
        if let Some(config) = data.config {
            if config.enabled {
                for item in data.items {
                    if item.active {
                        // Deeply nested logic...
                    }
                }
            }
        }
    }
}

// ✅ REFACTORED - Cognitive complexity: 8
pub fn validate_and_process(data: &Data) -> Result<Output> {
    validate_data(data)?;
    let config = extract_config(data)?;
    process_items(data, &config)
}
```

---

## Continuous Improvement

### Monthly Review

1. **Check new nursery lints**
   ```bash
   rustup update
   cargo clippy -- -W clippy::nursery
   ```

2. **Review allowed exceptions**
   - Are they still justified?
   - Can any be removed?

3. **Update documentation**
   - New patterns discovered?
   - Better alternatives found?

### Team Alignment

- Discuss Clippy findings in code review
- Document new allowed exceptions
- Share patterns that improve code quality

---

## Real-World Example: RentalForge

**Results after implementing these practices:**

| Metric | Before | After | Impact |
|--------|--------|-------|--------|
| Production panics | 3/month | 0 | ✅ Zero downtime |
| Avg function complexity | 22 | 11 | ✅ 50% reduction |
| Code review time | 45 min | 30 min | ✅ 33% faster |
| Unwrap calls | 127 | 0 | ✅ All explicit |

**Key learnings:**
- Initial cleanup took 2 weeks
- Team resisted at first ("too strict")
- After 1 month: "We'll never go back"
- Prevented 2 critical bugs caught by Clippy

---

## Troubleshooting

### "Too many warnings to fix at once"

**Approach incrementally:**

1. **Start with deny list only**
   ```toml
   [workspace.lints.clippy]
   unwrap_used = "deny"
   panic = "deny"
   ```

2. **Add warnings over time**
   ```toml
   pedantic = "warn"  # Add after deny list is clean
   ```

3. **Use `#[allow]` temporarily**
   ```rust
   // TODO: Refactor this function
   #[allow(clippy::cognitive_complexity)]
   fn legacy_function() { /* ... */ }
   ```

### "Pedantic is too noisy"

**Disable specific pedantic lints:**

```toml
[workspace.lints.clippy]
pedantic = "warn"

# Disable noisy pedantic lints
must_use_candidate = "allow"
missing_errors_doc = "allow"
```

### "Conflicts with team style"

**Document your exceptions:**

```toml
# We allow `clone_on_copy` for readability
# Team voted 5-2 in favor (2024-01-15)
clone_on_copy = "allow"
```

---

## References

- [Clippy Lint List](https://rust-lang.github.io/rust-clippy/master/index.html)
- [Cognitive Complexity Paper](https://www.sonarsource.com/resources/cognitive-complexity/)
- [Rust API Guidelines - Error Handling](https://rust-lang.github.io/api-guidelines/errors.html)

---

## Related Documentation

- [Function Complexity Guide](FUNCTION_COMPLEXITY.md)
- [Code Quality Standards](CODE_QUALITY.md) *(if exists)*
- [Testing Best Practices](../testing/TESTING_GUIDE.md) *(if exists)*

---

*Last Updated: 2026-01-24*
*Based on: RentalForge production implementation*
