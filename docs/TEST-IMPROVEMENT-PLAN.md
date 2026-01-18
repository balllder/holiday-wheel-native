# Test Improvement Plan

## Current State

| Area | Tests | Coverage | Priority |
|------|-------|----------|----------|
| Rust Backend | 2 | Unknown | **Critical** |
| Shared Services | 1 | 12% | **Critical** |
| Shared Stores | 0 | 0% | **High** |
| Phone Screens | 2 | 10% | Medium |
| TV Screens | 3 | 3% | Medium |
| Shared Components | 1 | 95% | Low (good) |

---

## Priority 1: Critical Backend Tests (Rust)

### 1.1 Database Operations
**File:** `apps/backend-rust/src/db/mod.rs`
**Effort:** 2-3 hours

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // User operations
    #[tokio::test]
    async fn test_create_user() { }

    #[tokio::test]
    async fn test_get_user_by_email() { }

    #[tokio::test]
    async fn test_verify_password() { }

    // OAuth operations
    #[tokio::test]
    async fn test_create_oauth_account() { }

    #[tokio::test]
    async fn test_get_oauth_account() { }

    // Passkey operations
    #[tokio::test]
    async fn test_store_passkey_credential() { }

    #[tokio::test]
    async fn test_get_passkey_credentials() { }

    // Puzzle operations
    #[tokio::test]
    async fn test_get_random_puzzle() { }

    #[tokio::test]
    async fn test_puzzle_not_repeated() { }
}
```

**Test cases:**
- [ ] Create user with valid data
- [ ] Create user with duplicate email fails
- [ ] Get user by email (exists/not exists)
- [ ] Password verification (correct/incorrect)
- [ ] OAuth account linking
- [ ] Passkey credential storage/retrieval
- [ ] Puzzle selection with pack filtering
- [ ] Puzzle history (no repeats)

---

### 1.2 Authentication Logic
**File:** `apps/backend-rust/src/auth/mod.rs`
**Effort:** 2-3 hours

```rust
#[cfg(test)]
mod tests {
    // Registration
    #[tokio::test]
    async fn test_register_valid_user() { }

    #[tokio::test]
    async fn test_register_duplicate_email() { }

    #[tokio::test]
    async fn test_register_invalid_email() { }

    #[tokio::test]
    async fn test_register_weak_password() { }

    // Login
    #[tokio::test]
    async fn test_login_success() { }

    #[tokio::test]
    async fn test_login_wrong_password() { }

    #[tokio::test]
    async fn test_login_nonexistent_user() { }

    // Token verification
    #[tokio::test]
    async fn test_verify_valid_token() { }

    #[tokio::test]
    async fn test_verify_expired_token() { }
}
```

---

### 1.3 Game State Logic
**File:** `apps/backend-rust/src/game/state.rs`
**Effort:** 3-4 hours

```rust
#[cfg(test)]
mod tests {
    // Puzzle solving
    #[test]
    fn test_check_solve_exact_match() { }

    #[test]
    fn test_check_solve_case_insensitive() { }

    #[test]
    fn test_check_solve_with_punctuation() { }

    // Letter guessing
    #[test]
    fn test_guess_consonant_in_puzzle() { }

    #[test]
    fn test_guess_consonant_not_in_puzzle() { }

    #[test]
    fn test_guess_already_guessed() { }

    #[test]
    fn test_buy_vowel_success() { }

    #[test]
    fn test_buy_vowel_insufficient_funds() { }

    // Scoring
    #[test]
    fn test_score_calculation() { }

    #[test]
    fn test_bankrupt_clears_round() { }

    #[test]
    fn test_round_bank_to_total() { }

    // Turn management
    #[test]
    fn test_advance_turn() { }

    #[test]
    fn test_turn_after_wrong_guess() { }
}
```

---

### 1.4 OAuth Token Verification
**File:** `apps/backend-rust/src/auth/oauth.rs`
**Effort:** 2 hours

```rust
#[cfg(test)]
mod tests {
    // Google token parsing
    #[test]
    fn test_parse_google_claims() { }

    // Apple token parsing
    #[test]
    fn test_parse_apple_claims() { }

    // State management
    #[tokio::test]
    async fn test_oauth_state_creation() { }

    #[tokio::test]
    async fn test_oauth_state_expiration() { }
}
```

---

## Priority 2: Shared Package Tests

### 2.1 Game Store
**File:** `packages/shared/src/stores/__tests__/gameStore.test.ts`
**Effort:** 2-3 hours

```typescript
describe('gameStore', () => {
  describe('selectors', () => {
    it('selectIsMyTurn returns true when active', () => {});
    it('selectIsMyTurn returns false when not active', () => {});
    it('selectCanBuzz returns true during tossup', () => {});
    it('selectMyPlayer returns correct player', () => {});
    it('selectActivePlayer returns active player', () => {});
  });

  describe('actions', () => {
    it('setGameState updates state', () => {});
    it('setPlayerId sets player ID', () => {});
    it('clearGame resets state', () => {});
  });
});
```

**Test cases:**
- [ ] All selectors with various game states
- [ ] State updates from socket events
- [ ] Phase transitions (normal → tossup → final)
- [ ] Player score tracking

---

### 2.2 Auth Store
**File:** `packages/shared/src/stores/__tests__/authStore.test.ts`
**Effort:** 1-2 hours

```typescript
describe('authStore', () => {
  it('setUser stores user data', () => {});
  it('setToken stores auth token', () => {});
  it('logout clears all data', () => {});
  it('isAuthenticated returns correct state', () => {});
});
```

---

### 2.3 Auth Service
**File:** `packages/shared/src/services/__tests__/authService.test.ts`
**Effort:** 2-3 hours

```typescript
describe('authService', () => {
  describe('login', () => {
    it('returns user on success', async () => {});
    it('throws on invalid credentials', async () => {});
  });

  describe('register', () => {
    it('creates user and returns token', async () => {});
    it('throws on duplicate email', async () => {});
  });

  describe('verifyToken', () => {
    it('returns user for valid token', async () => {});
    it('throws for expired token', async () => {});
  });
});
```

**Mock strategy:** Use `jest.mock` for fetch/network calls

---

### 2.4 Socket Service
**File:** `packages/shared/src/services/__tests__/socketService.test.ts`
**Effort:** 2-3 hours

```typescript
describe('socketService', () => {
  describe('connection', () => {
    it('connects with valid token', () => {});
    it('emits connected event', () => {});
    it('handles disconnection', () => {});
    it('reconnects automatically', () => {});
  });

  describe('game events', () => {
    it('emits join event', () => {});
    it('emits spin event', () => {});
    it('emits guess event', () => {});
    it('receives state updates', () => {});
  });
});
```

**Mock strategy:** Use `socket.io-mock` or manual mock

---

## Priority 3: Screen Tests

### 3.1 Login Screen Tests
**Effort:** 2 hours per screen

```typescript
describe('LoginScreen', () => {
  it('renders login form', () => {});
  it('validates email format', () => {});
  it('shows error on failed login', () => {});
  it('navigates to lobby on success', () => {});
  it('shows passkey button when supported', () => {});
  it('shows social login buttons', () => {});
});
```

### 3.2 Game Screen Tests
```typescript
describe('GameScreen', () => {
  it('displays puzzle board', () => {});
  it('displays wheel', () => {});
  it('shows spin button when my turn', () => {});
  it('disables spin when not my turn', () => {});
  it('shows letter input after spin', () => {});
  it('displays player scores', () => {});
});
```

### 3.3 Controller Screen Tests
```typescript
describe('ControllerScreen', () => {
  it('shows buzz button during tossup', () => {});
  it('vibrates on button press', () => {});
  it('displays current score', () => {});
  it('shows turn indicator', () => {});
});
```

---

## Implementation Order

### Sprint 1: Backend Core (Week 1)
| Task | Effort | Impact |
|------|--------|--------|
| Database tests | 3 hrs | Critical |
| Auth logic tests | 3 hrs | Critical |
| Game state tests | 4 hrs | Critical |

**Target:** 80%+ coverage on core backend logic

### Sprint 2: Shared Package (Week 2)
| Task | Effort | Impact |
|------|--------|--------|
| gameStore tests | 3 hrs | High |
| authStore tests | 2 hrs | High |
| authService tests | 3 hrs | High |
| socketService tests | 3 hrs | High |

**Target:** 70%+ coverage on shared package

### Sprint 3: Frontend Screens (Week 3)
| Task | Effort | Impact |
|------|--------|--------|
| LoginScreen tests | 2 hrs | Medium |
| GameScreen tests | 3 hrs | Medium |
| ControllerScreen tests | 2 hrs | Medium |
| TV screen tests | 3 hrs | Medium |

**Target:** 50%+ coverage on screens

---

## Testing Infrastructure

### Backend (Rust)

**Add to Cargo.toml:**
```toml
[dev-dependencies]
tokio-test = "0.4"
tempfile = "3"  # For test databases
```

**Test database setup:**
```rust
async fn setup_test_db() -> Database {
    let tmp = tempfile::NamedTempFile::new().unwrap();
    Database::new(tmp.path().to_str().unwrap()).await.unwrap()
}
```

### Frontend (Jest)

**Add mocks directory:**
```
packages/shared/src/__mocks__/
├── socketService.ts
├── authService.ts
└── AsyncStorage.ts
```

**Mock example:**
```typescript
// __mocks__/authService.ts
export const authService = {
  login: jest.fn(),
  register: jest.fn(),
  verifyToken: jest.fn(),
  setBaseUrl: jest.fn(),
};
```

---

## Coverage Targets

| Area | Current | Target | Priority |
|------|---------|--------|----------|
| Backend DB | 0% | 80% | P1 |
| Backend Auth | 0% | 80% | P1 |
| Backend Game | ~10% | 80% | P1 |
| Shared Stores | 0% | 70% | P2 |
| Shared Services | 12% | 70% | P2 |
| Phone Screens | 10% | 50% | P3 |
| TV Screens | 3% | 50% | P3 |

---

## Test Commands

```bash
# Run all tests
npm test --workspaces

# Run with coverage
npm test --workspaces -- --coverage

# Run specific package
cd apps/phone && npm test
cd apps/tv && npm test
cd packages/shared && npm test

# Run Rust tests
cd apps/backend-rust && cargo test

# Run Rust tests with coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

---

## CI Integration

Add to `.github/workflows/ci.yml`:

```yaml
test:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4

    - name: Setup Node
      uses: actions/setup-node@v4
      with:
        node-version: '20'

    - name: Install dependencies
      run: npm ci

    - name: Run frontend tests
      run: npm test --workspaces -- --coverage --watchAll=false

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run backend tests
      working-directory: apps/backend-rust
      run: cargo test

    - name: Upload coverage
      uses: codecov/codecov-action@v3
```

---

## Quick Wins (Start Here)

These tests provide the most value with least effort:

1. **Game state solve detection** - Critical for gameplay
2. **Wheel randomization** - Already has 2 tests, add more
3. **gameStore selectors** - Pure functions, easy to test
4. **configService** - Already 100%, use as template
5. **authStore** - Simple state, quick to test

Would you like me to start implementing tests for any specific area?
