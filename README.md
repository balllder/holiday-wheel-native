# Holiday Wheel

A multiplayer "Wheel of Fortune" style game built with React Native for mobile/TV and a Rust backend. Play with friends using your phones as controllers while displaying the game on an Apple TV or Android TV.

## Features

### Game Modes
- **Normal Round**: Players take turns spinning the wheel, guessing consonants, buying vowels, and solving puzzles
- **Toss-Up Round**: All players can buzz in to solve - fastest finger wins
- **Final Round**: Championship round with hidden letters and a time limit

### Wheel Mechanics
- 24 wedges with authentic Wheel of Fortune values
- Cash values from $300 to $5000
- Special wedges: Bankrupt, Lose a Turn, Free Play
- Configurable prize wedges
- Animated spinning wheel with smooth physics

### Multiplayer Setup
- **TV App (Host)**: Displays the game board, animated wheel, and scores
- **Phone App (Controller)**: Players use their phones to spin, guess, and solve
- **QR Code Join**: TV displays QR code for easy room joining
- Real-time synchronization via Socket.IO

### Additional Features
- User authentication with email registration
- Puzzle database with categories
- Admin panel for game configuration
- Persistent scores across rounds
- Host controls for game management

## Architecture

```
holiday-wheel-native/
├── apps/
│   ├── phone/              # React Native phone/tablet app
│   ├── tv/                 # React Native tvOS/Android TV app
│   └── backend-rust/       # Rust backend server
├── packages/
│   └── shared/             # Shared types, stores, services
├── package.json            # Root monorepo config
└── turbo.json              # Turbo build configuration
```

## Prerequisites

- **Node.js** 18+ and npm
- **Rust** (for backend)
- **Xcode 15+** (for iOS/tvOS builds)
- **Android Studio** (for Android/Android TV builds)
- **CocoaPods** (for iOS dependencies)

## Setup

### 1. Clone and Install Dependencies

```bash
git clone https://github.com/balllder/holiday-wheel-native.git
cd holiday-wheel-native
npm install
```

### 2. Build Shared Package

```bash
cd packages/shared
npm run build
cd ../..
```

## Running the Backend

The backend is a Rust server that handles game state, authentication, and real-time communication.

### Setup

```bash
cd apps/backend-rust

# Copy environment file
cp .env.example .env

# Edit .env if needed (defaults work for local development)
```

### Build and Run

```bash
# Build
cargo build

# Run (starts on http://localhost:5000)
cargo run
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite database path | `sqlite:puzzles.db` |
| `PORT` | Server port | `5000` |
| `HOST_CODE` | Code to claim host privileges | `holiday` |
| `RUST_LOG` | Log level | `info` |

### Web Admin Panel

Access the admin panel at `http://localhost:5000/admin` to:
- Manage puzzles (add, edit, delete)
- Configure game settings (vowel cost, final round time, etc.)
- Monitor active rooms

### Importing Puzzles

The backend includes sample puzzle packs that can be imported via the API:

```bash
# First, get an admin token (set ADMIN_EMAIL env var to your email)
TOKEN="your-auth-token"

# Import general puzzles
curl -X POST http://localhost:5000/auth/api/admin/puzzles/import \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @apps/backend-rust/sample-puzzles.json

# Import holiday-themed puzzles
curl -X POST http://localhost:5000/auth/api/admin/puzzles/import \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d @apps/backend-rust/holiday-puzzles.json
```

**JSON Import Format:**
```json
{
  "puzzles": [
    { "category": "Phrase", "answer": "HELLO WORLD" },
    { "category": "Thing", "answer": "SMARTPHONE" }
  ],
  "pack_name": "My Custom Pack"
}
```

Options:
- `pack_name`: Creates a new pack or uses existing one by name
- `pack_id`: Use a specific pack ID (overrides pack_name)
- If neither is provided, puzzles go to the Default pack

## Running the Phone App

The phone app can run in two modes:
- **Play Mode**: Full game experience on the phone
- **Controller Mode**: Use phone as a controller for TV display

### iOS

```bash
cd apps/phone

# Install pods
cd ios && pod install && cd ..

# Run on iOS simulator
npm run ios

# Or run on specific device
npx react-native run-ios --device "iPhone 15"
```

### Android

```bash
cd apps/phone

# Start Metro bundler
npm start

# In another terminal, run on Android
npm run android
```

### Server Configuration

The server URL can be configured in-app:
1. Open the app and log in
2. Tap the **⚙️ Server** toggle on the lobby screen
3. Enter your backend server IP (e.g., `http://192.168.1.100:5000`)
4. Tap **Save**

Default URLs:
- **iOS Simulator**: `http://localhost:5000`
- **Android Emulator**: `http://10.0.2.2:5000`
- **Physical Device**: Use your computer's local IP address

## Running the TV App

The TV app displays the game board, animated wheel, and serves as the host display.

### Apple TV (tvOS)

**Requirements:** macOS with Xcode 15+ and CocoaPods installed.

```bash
cd apps/tv/ios

# Install CocoaPods dependencies
pod install

# Open the workspace (not .xcodeproj) in Xcode
open tv.xcworkspace
```

**In Xcode:**
1. Select the `tv-tvOS` scheme from the scheme selector
2. Choose an Apple TV simulator (e.g., "Apple TV 4K (3rd generation)")
3. Build and run (Cmd+R)

**Or from command line:**
```bash
cd apps/tv

# Run on Apple TV simulator
npx react-native run-ios --scheme tv-tvOS --simulator "Apple TV"

# Or use the npm script
npm run tvos
```

### TV Server Configuration

Like the phone app, the TV app supports in-app server configuration:
1. On the lobby screen, tap **⚙️ Server** in the header
2. Enter your backend server IP address
3. Tap **Save**

The TV will use this URL for socket connections and display it in the QR code for phone connections.

### Android TV

```bash
cd apps/tv

# Start Metro bundler
npm start

# In another terminal
npm run android
```

Ensure your Android TV emulator or device is connected via ADB.

### TV App Controls

- **Menu/Play-Pause button**: Toggle host control panel
- **Back button**: Close overlays
- Remote navigation for all menu items

## How to Play

### Starting a Game

1. **Start the Backend**
   ```bash
   cd apps/backend-rust && cargo run
   ```

2. **Launch TV App (Host)**
   - Open the TV app
   - Log in or continue as guest
   - Select or create a room
   - The TV displays a QR code for players to join

3. **Players Join with Phone App**
   - Open the phone app
   - Log in or register
   - Scan the QR code on TV, or manually enter the room name
   - Select "Controller Mode" to use phone as controller

### Game Flow

#### Normal Round
1. **Spin**: Active player taps SPIN button on their phone
2. **Guess a Letter**:
   - Enter a consonant and tap GUESS
   - If correct, earn money for each occurrence
   - If wrong, turn passes to next player
3. **Buy a Vowel**:
   - Enter a vowel (A, E, I, O, U)
   - Costs $250, no money earned
4. **Solve**:
   - Enter the full puzzle answer
   - If correct, win the round and keep your money!

#### Toss-Up Round
1. Host starts toss-up from control panel
2. Letters reveal one at a time
3. Any player can tap BUZZ to attempt solving
4. First correct answer wins $1000
5. Wrong answer locks that player out

#### Final Round
1. Host starts final round
2. Standard letters (R, S, T, L, N, E) are revealed
3. Player picks 3 consonants and 1 vowel
4. Timer starts - solve before time runs out!

### Host Controls (TV App)

Press Menu button on Apple TV remote to access:
- **NEW PUZZLE**: Load a new puzzle
- **SPIN**: Spin on behalf of current player
- **REVEAL ALL**: Show all letters
- **START TOSS-UP**: Begin toss-up round
- **START FINAL**: Begin final round
- **NEW GAME**: Reset scores and start over
- **Set Active Player**: Change whose turn it is

## Development

### Commands (from root directory)

```bash
# Build all packages
npm run build

# Run linting
npm run lint

# Format code
npm run format

# Type check
npm run check-types
```

### Package-specific Commands

```bash
# Phone app
cd apps/phone
npm start          # Start Metro bundler
npm run ios        # Run on iOS
npm run android    # Run on Android
npm test           # Run tests

# TV app
cd apps/tv
npm start          # Start Metro bundler
npm run ios        # Run on iOS simulator
npm run tvos       # Run on Apple TV simulator
npm run android    # Run on Android TV

# Backend
cd apps/backend-rust
cargo build        # Build
cargo run          # Run server
cargo test         # Run tests
```

### Project Structure

| Path | Description |
|------|-------------|
| `apps/phone/src/screens/` | Phone app screens |
| `apps/tv/src/screens/` | TV app screens |
| `apps/tv/src/components/` | TV-specific components |
| `packages/shared/src/stores/` | Zustand state stores |
| `packages/shared/src/services/` | API and Socket services |
| `apps/backend-rust/src/game/` | Game logic (Rust) |
| `apps/backend-rust/src/routes/` | HTTP routes and web pages |

## Tech Stack

- **Frontend**: React Native 0.83, TypeScript, Zustand
- **TV**: react-native-tvos for Apple TV/Android TV support
- **Backend**: Rust, Axum, Socket.IO (socketioxide), SQLite
- **Monorepo**: npm workspaces + Turborepo
- **Real-time**: Socket.IO for WebSocket communication

## License

MIT
