# E2E Testing Guide - Holiday Wheel Native

## Overview

This project has comprehensive E2E test coverage across multiple layers:

1. **Playwright Tests** - Backend web app testing
2. **Maestro Tests** - React Native mobile app testing (Phone & TV)

## Test Architecture

```
holiday-wheel-native/
├── e2e/                          # Playwright tests (Web backend)
│   ├── tests/
│   │   ├── auth.spec.ts         # Authentication flows
│   │   ├── lobby.spec.ts        # Lobby page functionality
│   │   ├── game.spec.ts         # Game page UI
│   │   ├── admin.spec.ts        # Admin features
│   │   ├── health.spec.ts       # Health check
│   │   ├── socket-connection.spec.ts  # Socket.IO real-time
│   │   ├── navigation.spec.ts   # Navigation flows
│   │   └── error-handling.spec.ts     # Error scenarios
│   └── playwright.config.ts
│
├── apps/phone/.maestro/          # Maestro tests (Phone app)
│   ├── app-launch.yaml          # Basic app launch
│   ├── auth-flow.yaml           # Login/Register navigation
│   ├── lobby-navigation.yaml    # Lobby interactions
│   └── deep-linking.yaml        # Deep link handling
│
└── apps/tv/.maestro/             # Maestro tests (TV app)
    ├── app-launch.yaml          # TV app launch
    ├── tv-navigation.yaml       # TV navigation flow
    ├── tv-remote-controls.yaml  # D-pad/Menu button tests
    └── tv-qr-code-display.yaml  # QR code display
```

## Running Tests

### Playwright Tests (Web Backend)

```bash
# Run all Playwright tests
npm run test:e2e

# Run with browser UI (headed mode)
npm run test:e2e:headed

# Open interactive test UI
npm run test:e2e:ui

# View test results report
npm run test:e2e:report

# Run specific test file
cd e2e && npx playwright test tests/auth.spec.ts
```

### Maestro Tests (Mobile Apps)

#### Prerequisites
1. Install Maestro CLI:
   ```bash
   curl -Ls "https://get.maestro.mobile.dev" | bash
   ```

2. Start the backend server:
   ```bash
   cd apps/backend-rust && cargo run
   ```

3. Build and launch the app (Android/iOS emulator or device)

#### Phone App Tests
```bash
# Run all phone tests
npm run test:maestro:phone

# Run specific test
cd apps/phone && maestro test .maestro/app-launch.yaml
cd apps/phone && maestro test .maestro/lobby-navigation.yaml
```

#### TV App Tests
```bash
# Run all TV tests
npm run test:maestro:tv

# Run specific test
cd apps/tv && maestro test .maestro/tv-navigation.yaml
cd apps/tv && maestro test .maestro/tv-remote-controls.yaml
```

#### Run All Mobile Tests
```bash
npm run test:maestro:all
```

## Test Coverage

### Playwright Tests (8 Test Files, 40+ Tests)

#### Authentication (`auth.spec.ts`)
- ✅ Login page loads correctly
- ✅ Invalid credentials show error
- ✅ Register page accessible from login
- ✅ Registration form validation
- ✅ Password mismatch detection
- ✅ Successful registration redirects to lobby

#### Lobby (`lobby.spec.ts`)
- ✅ Unauthenticated users redirected to login
- ✅ Lobby shown after registration
- ✅ Room name input visible
- ✅ Join button visible
- ✅ QR code section exists

#### Game (`game.spec.ts`)
- ✅ Unauthenticated redirect
- ✅ Game interface loads with room parameter
- ✅ Wheel element displayed
- ✅ Puzzle board displayed
- ✅ Player area displayed

#### Socket Connection (`socket-connection.spec.ts`)
- ✅ Socket connection established in game room
- ✅ Handles disconnection gracefully
- ✅ Multiple users can join same room
- ✅ Real-time game state updates

#### Navigation (`navigation.spec.ts`)
- ✅ Complete user journey (register → lobby → game)
- ✅ Browser back button navigation
- ✅ Direct game URL access
- ✅ Session persistence after reload
- ✅ Logout functionality
- ✅ Invalid room code handling
- ✅ Case-insensitive room codes
- ✅ Concurrent navigation handling

#### Error Handling (`error-handling.spec.ts`)
- ✅ Server timeout handling
- ✅ Invalid email format validation
- ✅ Password strength validation
- ✅ Duplicate email registration
- ✅ Network errors during login
- ✅ Malformed URL parameters
- ✅ XSS prevention
- ✅ Required field validation
- ✅ Long input string handling

#### Admin (`admin.spec.ts`)
- ✅ Admin page access control

#### Health (`health.spec.ts`)
- ✅ Health check endpoint

### Maestro Tests (Phone: 4 Flows, TV: 4 Flows)

#### Phone App Tests
- ✅ **app-launch.yaml**: App launches, login screen visible
- ✅ **auth-flow.yaml**: Navigation between login/register
- ✅ **lobby-navigation.yaml**: Registration, login, lobby interactions
- ✅ **deep-linking.yaml**: `holidaywheel://` deep link handling

#### TV App Tests
- ✅ **app-launch.yaml**: TV app launches, login screen visible
- ✅ **tv-navigation.yaml**: Registration, login, lobby with QR code
- ✅ **tv-remote-controls.yaml**: D-pad, Menu, Select button navigation
- ✅ **tv-qr-code-display.yaml**: QR code display for phone pairing

## Key User Flows Tested

### 1. New User Registration
```
Open App → Register → Fill Form → Submit → Lobby
```

### 2. Existing User Login
```
Open App → Login → Enter Credentials → Submit → Lobby
```

### 3. Join Game Room
```
Lobby → Enter Room Code → Join → Game Screen
```

### 4. Phone-TV Pairing (Deep Link)
```
TV: Create Room → Display QR Code
Phone: Scan QR → Auto-navigate to Controller
```

### 5. Multi-Player Game
```
User 1: Create Room → Wait
User 2: Join Same Room → Both see game state
Real-time sync via Socket.IO
```

## Test Data Management

### Playwright Tests
- Uses unique emails per test: `test-${Date.now()}@example.com`
- Backend auto-verifies emails in test mode (EMAIL_ENABLED=false)
- Test database: `test-puzzles.db` (separate from dev)

### Maestro Tests
- Default test credentials in YAML `env` section
- Can be overridden per test run
- Backend uses same test mode as Playwright

## CI/CD Integration

### GitHub Actions Example

```yaml
name: E2E Tests

on: [push, pull_request]

jobs:
  playwright:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '20'
      - name: Install dependencies
        run: npm ci
      - name: Install Playwright
        run: cd e2e && npm ci && npx playwright install --with-deps
      - name: Run Playwright tests
        run: npm run test:e2e
      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: playwright-report
          path: e2e/playwright-report/

  maestro:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Maestro
        run: curl -Ls "https://get.maestro.mobile.dev" | bash
      - name: Run Maestro tests
        run: |
          export PATH="$PATH:$HOME/.maestro/bin"
          npm run test:maestro:all
      - name: Upload Maestro results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: maestro-results
          path: "**/.maestro/results/"
```

## Best Practices

### Playwright
1. **Isolation**: Each test is independent, clears cookies
2. **Unique Data**: Generate unique test data per run
3. **Timeouts**: Reasonable timeouts for async operations
4. **Screenshots**: Captured on failure automatically
5. **Traces**: Recorded on first retry for debugging

### Maestro
1. **Declarative**: YAML-based, readable flows
2. **Visual Verification**: Screenshots at key points
3. **Conditional Flows**: `runFlow.when` for flexible testing
4. **Environment Variables**: Configurable test data
5. **Cross-Platform**: Same tests work on iOS and Android

## Debugging Tests

### Playwright
```bash
# Run with browser visible
npm run test:e2e:headed

# Open interactive UI mode
npm run test:e2e:ui

# Debug specific test
cd e2e && npx playwright test tests/auth.spec.ts --debug

# View trace from failed test
cd e2e && npx playwright show-trace trace.zip
```

### Maestro
```bash
# Run with verbose output
maestro test --debug .maestro/app-launch.yaml

# View screenshots from last run
open .maestro/results/

# Run in continuous mode (watch for changes)
maestro test --continuous .maestro/
```

## Troubleshooting

### Playwright Tests Failing
1. Check backend server is running on port 5000
2. Verify database path in playwright.config.ts
3. Clear test database: `rm apps/backend-rust/test-puzzles.db`
4. Check test timeout settings

### Maestro Tests Failing
1. Verify app is built and installed on device/emulator
2. Check backend server URL (10.0.2.2 for Android emulator)
3. Ensure ADB/Xcode device connection
4. Verify app bundle ID matches test files

### Deep Linking Not Working
1. Check URL scheme in Android manifest / iOS Info.plist
2. Verify `holidaywheel://` is registered
3. Test with `adb shell am start -a android.intent.action.VIEW -d "holidaywheel://join?room=TEST"`

## Future Enhancements

### Planned Test Coverage
- [ ] Game puzzle interactions (spinning wheel, guessing letters)
- [ ] WebAuthn/Passkey authentication flow
- [ ] OAuth (Google/Apple) sign-in
- [ ] Admin panel functionality
- [ ] Multiplayer game scenarios
- [ ] Host controls (TV app)
- [ ] QR code scanning (phone camera)
- [ ] Network interruption recovery
- [ ] Performance testing (load time, FPS)
- [ ] Accessibility testing (screen readers, focus)

### Test Automation Improvements
- [ ] Visual regression testing (Percy/Applitools)
- [ ] API contract testing
- [ ] Load testing (Artillery/K6)
- [ ] Mobile device farm integration (BrowserStack/Sauce Labs)
- [ ] Automated test reporting dashboard

## Resources

- [Playwright Documentation](https://playwright.dev/)
- [Maestro Documentation](https://maestro.mobile.dev/)
- [Testing Best Practices](https://kentcdodds.com/blog/common-mistakes-with-react-testing-library)
- [E2E Testing Patterns](https://martinfowler.com/articles/practical-test-pyramid.html)

---

**Last Updated**: 2026-01-19
**Test Coverage**: 40+ Playwright tests, 8 Maestro flows
**Estimated Run Time**: Playwright ~5 min, Maestro ~10 min per platform
