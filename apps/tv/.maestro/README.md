# Maestro E2E Tests for Holiday Wheel TV App

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

3. Build and install the TV app on your device/emulator:
   ```bash
   cd apps/tv
   # For Android TV
   npm run android

   # For Apple TV
   npm run ios
   ```

## Running Tests

### Run all tests
```bash
cd apps/tv
maestro test .maestro/
```

### Run a specific test
```bash
maestro test .maestro/app-launch.yaml
maestro test .maestro/tv-navigation.yaml
maestro test .maestro/tv-remote-controls.yaml
maestro test .maestro/tv-qr-code-display.yaml
```

### Run tests with reporting
```bash
# Generate HTML report
maestro test --format html --output test-results.html .maestro/

# Run in continuous mode (watch for changes)
maestro test --continuous .maestro/
```

## Test Files

- **app-launch.yaml**: Basic TV app launch and login screen verification
- **tv-navigation.yaml**: Complete registration/login flow and lobby navigation
- **tv-remote-controls.yaml**: TV remote button handling (D-pad, Menu, Select)
- **tv-qr-code-display.yaml**: Verifies QR code display for phone pairing

## TV-Specific Testing

### Remote Control Keys
Maestro supports TV remote button simulation:
- `pressKey: Up` - D-pad Up
- `pressKey: Down` - D-pad Down
- `pressKey: Left` - D-pad Left
- `pressKey: Right` - D-pad Right
- `pressKey: Select` - Center/Select button
- `pressKey: Menu` - Menu button
- `pressKey: Play` - Play/Pause button
- `pressKey: Back` - Back button

### Focus Management
TV apps use focus-based navigation. Tests verify:
- Focus moves correctly with D-pad
- Focused elements have visual feedback
- `hasTVPreferredFocus` is properly set

### Large Screen UI
Tests capture screenshots to verify:
- Text is readable at TV sizes (40-120px fonts)
- Spacing is appropriate for 10-foot UI
- Colors meet TV contrast requirements

## Configuration

### Environment Variables
Edit the `env` section in each test file to customize:
- `TEST_EMAIL`: Email for test user registration/login
- `TEST_PASSWORD`: Password for test user
- `SERVER_URL`: Backend server URL

### App ID
Update `appId` in test files if your bundle ID changes:
- Android TV: `com.holidaywheel.tv`
- Apple TV: `com.holidaywheel.tv`

## Testing on Real Devices

### Apple TV
1. Connect Apple TV to your Mac via USB-C
2. Trust the device in Xcode
3. Run tests: `maestro test .maestro/`

### Android TV
1. Enable ADB debugging on Android TV:
   - Settings → Device Preferences → About → Build (tap 7 times)
   - Settings → Device Preferences → Developer Options → USB Debugging
2. Connect via ADB:
   ```bash
   adb connect <tv-ip-address>:5555
   ```
3. Run tests: `maestro test .maestro/`

## Troubleshooting

### Backend Connection Issues
Update the server URL for TV testing:
- **Android TV Emulator**: `http://10.0.2.2:5000`
- **Apple TV Simulator**: `http://localhost:5000`
- **Real TV Device**: Use your Mac/PC IP (e.g., `http://192.168.1.10:5000`)

### Focus Not Working
If focus navigation fails:
1. Verify `useTVEventHandler` is implemented in screens
2. Check `TVFocusGuideView` is properly configured
3. Ensure `hasTVPreferredFocus` is set on initial elements

### QR Code Not Visible
The QR code appears only after creating a room. Ensure:
1. Backend server is running
2. Room code is entered
3. "Create Room" button is tapped

## CI/CD Integration

To run Maestro tests in CI:

```yaml
# Example GitHub Actions
- name: Run Maestro TV Tests
  run: |
    curl -Ls "https://get.maestro.mobile.dev" | bash
    export PATH="$PATH:$HOME/.maestro/bin"
    maestro test apps/tv/.maestro/ --format junit --output maestro-results-tv.xml
```

## Learn More

- [Maestro Documentation](https://maestro.mobile.dev/)
- [Testing TV Apps](https://maestro.mobile.dev/platform-support/tv)
- [Maestro CLI Commands](https://maestro.mobile.dev/cli/commands)
