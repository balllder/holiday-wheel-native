# E2E Testing Quick Start

## TL;DR - Run Tests Now

### Web Backend Tests (Playwright)
```bash
npm run test:e2e
```

### Mobile App Tests (Maestro)
```bash
# Install Maestro first
curl -Ls "https://get.maestro.mobile.dev" | bash

# Start backend
cd apps/backend-rust && cargo run &

# Run phone tests
npm run test:maestro:phone

# Run TV tests
npm run test:maestro:tv
```

## What Gets Tested

### ✅ Web Backend (Playwright)
- Authentication (login, register, password validation)
- Lobby page (room creation, QR codes)
- Game page (wheel, puzzle board, players)
- Socket.IO real-time connections
- Navigation flows
- Error handling (network errors, XSS, validation)

### ✅ Phone App (Maestro)
- App launch
- Login/Register navigation
- Lobby interactions
- Deep linking (`holidaywheel://` URLs)
- QR code scanning UI

### ✅ TV App (Maestro)
- App launch
- TV-specific navigation (D-pad, Menu button)
- QR code display for phone pairing
- Remote control handling
- Focus management

## Test Files Locations

```
e2e/tests/*.spec.ts          # Playwright web tests
apps/phone/.maestro/*.yaml   # Phone app tests
apps/tv/.maestro/*.yaml      # TV app tests
```

## Common Commands

```bash
# Playwright
npm run test:e2e              # Run all web tests
npm run test:e2e:headed       # With browser UI
npm run test:e2e:ui           # Interactive mode
npm run test:e2e:report       # View results

# Maestro
npm run test:maestro:phone    # Phone app tests
npm run test:maestro:tv       # TV app tests
npm run test:maestro:all      # All mobile tests

# Run specific test
cd e2e && npx playwright test tests/auth.spec.ts
cd apps/phone && maestro test .maestro/app-launch.yaml
```

## Prerequisites Checklist

### For Playwright Tests
- [x] Node.js 20+
- [x] Rust toolchain (for backend)
- [x] Backend server running on port 5000

### For Maestro Tests
- [x] Maestro CLI installed
- [x] Android/iOS emulator or device
- [x] App built and installed
- [x] Backend server running

## Quick Debug

### Test Failing?
1. Check backend server is running: `curl http://localhost:5000/health`
2. Clear test database: `rm apps/backend-rust/test-puzzles.db`
3. Rebuild app: `cd apps/phone && npm run android`
4. Check logs: Look for errors in test output

### Can't Connect to Backend?
- **Android Emulator**: Use `http://10.0.2.2:5000`
- **iOS Simulator**: Use `http://localhost:5000`
- **Real Device**: Use your machine's IP (e.g., `http://192.168.1.10:5000`)

## Need Help?

See [E2E-TESTING-GUIDE.md](./E2E-TESTING-GUIDE.md) for comprehensive documentation.
