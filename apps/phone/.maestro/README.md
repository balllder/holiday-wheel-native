# Maestro E2E Tests for Holiday Wheel Phone App

## Prerequisites

1. Install Maestro CLI:
   ```bash
   # macOS/Linux
   curl -Ls "https://get.maestro.mobile.dev" | bash

   # Windows (via PowerShell)
   iex "& { $(iwr -useb 'https://get.maestro.mobile.dev') }"
   ```

2. Start the backend server:
   ```bash
   cd apps/backend-rust
   cargo run
   ```

3. Build and install the app on your device/emulator:
   ```bash
   cd apps/phone
   # For Android
   npm run android

   # For iOS
   npm run ios
   ```

## Running Tests

### Run all tests
```bash
cd apps/phone
maestro test .maestro/
```

### Run a specific test
```bash
maestro test .maestro/app-launch.yaml
maestro test .maestro/auth-flow.yaml
maestro test .maestro/lobby-navigation.yaml
maestro test .maestro/deep-linking.yaml
```

### Run tests with reporting
```bash
# Generate HTML report
maestro test --format html --output test-results.html .maestro/

# Run in continuous mode (watch for changes)
maestro test --continuous .maestro/
```

## Test Files

- **app-launch.yaml**: Basic app launch and initial screen verification
- **auth-flow.yaml**: Navigation between login and register screens
- **lobby-navigation.yaml**: Complete registration/login flow and lobby interactions
- **deep-linking.yaml**: Tests the `holidaywheel://` deep link functionality

## Configuration

### Environment Variables
Edit the `env` section in each test file to customize:
- `TEST_EMAIL`: Email for test user registration/login
- `TEST_PASSWORD`: Password for test user

### App ID
Update `appId` in test files if your bundle ID changes:
- Android: `com.holidaywheel.phone`
- iOS: `com.holidaywheel.phone`

## Troubleshooting

### Backend Connection Issues
If tests fail to connect to backend, update the server URL:
- **Android Emulator**: Use `http://10.0.2.2:5000`
- **iOS Simulator**: Use `http://localhost:5000`
- **Real Device**: Use your machine's local IP (e.g., `http://192.168.1.10:5000`)

### Deep Linking Not Working
Ensure the URL scheme is registered in:
- **Android**: `apps/phone/android/app/src/main/AndroidManifest.xml`
- **iOS**: `apps/phone/ios/phone/Info.plist`

### Test User Already Exists
The `lobby-navigation.yaml` test handles this case by falling back to login if registration fails.

## CI/CD Integration

To run Maestro tests in CI:

```yaml
# Example GitHub Actions
- name: Run Maestro Tests
  run: |
    curl -Ls "https://get.maestro.mobile.dev" | bash
    export PATH="$PATH:$HOME/.maestro/bin"
    maestro test apps/phone/.maestro/ --format junit --output maestro-results.xml
```

## Learn More

- [Maestro Documentation](https://maestro.mobile.dev/)
- [Maestro CLI Commands](https://maestro.mobile.dev/cli/commands)
- [Maestro Best Practices](https://maestro.mobile.dev/best-practices)
