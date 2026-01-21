# CLAUDE.md - Holiday Wheel Native

## Project Overview

A React Native "Wheel of Fortune" style game app built as a monorepo with phone and TV targets. Supports multiplayer gameplay with real-time synchronization.

## Architecture

```
holiday-wheel-native/
├── apps/
│   ├── phone/          # React Native app for phones/tablets
│   ├── tv/             # React Native app for Apple TV/Android TV
│   └── backend-rust/   # Rust backend server (Axum + Socket.IO)
├── packages/
│   └── shared/         # Shared types, stores, and services
├── package.json        # Root monorepo config
└── turbo.json          # Turbo build configuration
```

**Monorepo**: Uses npm workspaces + Turbo for task orchestration.
**Backend**: Rust with Axum web framework, SQLite database, Socket.IO for real-time.

## Tech Stack

- **Framework**: React Native 0.83.1, React 19.2.0
- **Language**: TypeScript 5.8.3+
- **State Management**: Zustand (stores in `packages/shared/src/stores/`)
- **Real-time**: Socket.IO for WebSocket communication
- **Navigation**: React Navigation 7 (native stack)
- **Storage**: AsyncStorage for local persistence
- **Build**: Metro bundler, Babel

## Commands

```bash
# Root level (run from project root)
npm run build          # Build all packages
npm run dev            # Start dev servers
npm run lint           # Lint all packages
npm run format         # Format with Prettier
npm run check-types    # Type check all packages

# App level (run from apps/phone or apps/tv)
npm start              # Start Metro bundler
npm run android        # Run on Android/Android TV
npm run ios            # Run on iOS/Apple TV
npm run test           # Run Jest tests

# Backend (run from apps/backend-rust)
cargo run              # Start server
cargo build            # Build server
cargo test             # Run tests
```

## Key Files

**Frontend:**
- `apps/phone/App.tsx` - Phone app entry point with deep linking
- `apps/tv/src/App.tsx` - TV app entry point
- `packages/shared/src/stores/authStore.ts` - Authentication state
- `packages/shared/src/stores/gameStore.ts` - Game state with selectors
- `packages/shared/src/services/socketService.ts` - Socket.IO singleton
- `packages/shared/src/services/authService.ts` - Auth API service
- `packages/shared/src/services/configService.ts` - Server URL configuration
- `packages/shared/src/components/AnimatedWheel.tsx` - Shared wheel animation

**Backend:**
- `apps/backend-rust/src/main.rs` - Server entry point, routes setup
- `apps/backend-rust/src/auth/mod.rs` - Authentication routes (login, register)
- `apps/backend-rust/src/auth/passkey.rs` - WebAuthn/Passkey endpoints
- `apps/backend-rust/src/auth/oauth.rs` - Google/Apple OAuth endpoints
- `apps/backend-rust/src/game/handlers.rs` - Socket.IO game event handlers
- `apps/backend-rust/src/db/mod.rs` - SQLite database operations
- `apps/backend-rust/src/routes/mod.rs` - Web client HTML pages

## State Management Pattern

Zustand stores with selectors to prevent unnecessary re-renders:

```typescript
// Selectors in gameStore.ts
selectIsMyTurn(state)   // Boolean for turn-based actions
selectCanBuzz(state)    // Boolean for toss-up phase
selectMyPlayer(state)   // Returns current player object
selectActivePlayer(state) // Returns active player
```

## Styling Conventions

- **StyleSheet.create()** for all styles (performance)
- **Theme colors**:
  - Background: `#0d0628` (dark purple)
  - Gold: `#d4af37` (primary accent)
  - Board: `#1a5cb8` (puzzle board blue)
- **TV**: Larger fonts (40-120px), more padding, focus states
- **Phone**: Compact layouts, flexbox-heavy

## Navigation Structure

**Phone** (`apps/phone/src/navigation/AppNavigator.tsx`):
- Login → Register → Lobby → Game/Controller/QRScan
- Supports deep linking via `holidaywheel://` URL scheme

**TV** (`apps/tv/src/navigation/TVNavigator.tsx`):
- TVLogin → TVLobby → TVGame
- No headers, fade animations, full-screen
- Lobby includes QR code display for phone joining

## API Configuration

Base URLs set at runtime (not hardcoded):
- **Android emulator**: `http://10.0.2.2:5000`
- **TV apps**: Configure IP in screen files

```typescript
authService.setBaseUrl(API_BASE_URL);
socketService.connect(API_BASE_URL, token);
```

## Game Flow

**Toss-up** (once at game start) → **Rounds 1-4** (winner goes first next round) → **Bonus**

## Game Phases

1. **tossup**: Game start only - determines who plays first in Round 1
2. **normal**: Standard gameplay - spin wheel, guess letters, buy vowels, solve
3. **final**: Final Spin mode - one spin sets value for all turns, free vowels
4. **bonus**: Winner's bonus round - pick letters, 10 seconds to solve for jackpot

## Rounds

Games have 4 rounds (1-4). Round winner goes first in the next round.

Host controls:
- `start_tossup` / `end_tossup` - Toss-up phase (game start)
- `advance_round` - Move to next round (1→2→3→4)
- `set_round` - Set specific round (1-4)
- `start_final_spin` / `end_final_spin` - Final Spin mode
- Round indicator shows "Round X / 4" in UI

## Socket Events

Key events handled in `socketService.ts`:
- `state` - Full game state sync (includes `round`, `phase`, `final_spin`, `bonus`)
- `rooms` - Available room list
- `notification` - Server messages
- `error` - Error handling

### Host Control Events
- `spin`, `new_round`, `new_game` - Basic game flow
- `advance_round`, `set_round` - Round management (1-4)
- `start_tossup`, `end_tossup` - Toss-up phase control
- `start_final_spin`, `end_final_spin` - Final Spin mode
- `start_bonus`, `end_bonus` - Bonus round control

## TV-Specific Patterns

- `useTVEventHandler` for remote button handling
- `TVFocusGuideView` for focus management
- `hasTVPreferredFocus` for initial focus
- Scale transforms on focus for visual feedback

## Testing

Jest with React Native preset. Run tests:
```bash
npm test
```

Tests located in `__tests__/` directories.

## Code Quality

- ESLint with `@react-native` config
- Prettier for formatting (single quotes, trailing commas)
- TypeScript strict mode

## Common Patterns

**Singleton Services**: `socketService` and `authService` are singletons exported from shared package.

**Conditional Navigation**: Auth state determines which navigator stack renders.

**Two Mobile Modes**:
- Play Mode: Full game on phone
- Controller Mode: Phone as remote for TV display

## Phone-TV Connection

Players can join games hosted on TV in multiple ways:

**QR Code Scanning**:
1. TV lobby displays QR code with room and server info
2. Phone scans QR code using `QRScanScreen`
3. Deep link opens: `holidaywheel://join?room=ROOM&server=URL`
4. Phone auto-configures and joins the room

**Deep Linking**:
- URL scheme: `holidaywheel://`
- Join format: `holidaywheel://join?room=ROOM&server=URL`
- Handled in `apps/phone/App.tsx`
- Saves server URL via `configService`

**Manual Connection**:
1. TV lobby shows server URL
2. User enters URL in phone's lobby screen
3. Phone connects to specified server

## TV Host Controls

The TV app includes host controls (`HostControlPanel.tsx`):
- **Game Flow**: New Puzzle, Spin, Reveal All, New Game
- **Round Control**: Round selector (1-4), Next Round button
- **Phase Control**: Start/End Toss-up, Start/End Final Spin, Start/End Bonus
- **Player Management**: Set active player, view scores

Toggle with Menu button during gameplay.

The web client also has host controls with:
- Round selector dropdown and "Next Round" button
- Phase buttons for Toss-up, Final Spin, and Bonus modes
- Round display in sidebar ("Round: X / 4")

## Backend Configuration

Key environment variables for `apps/backend-rust`:

```bash
# Database
DB_PATH=puzzles.db              # SQLite database path

# Admin
ADMIN_EMAIL=admin@example.com   # Auto-grant admin on login

# WebAuthn/Passkeys
WEBAUTHN_RP_ID=localhost
WEBAUTHN_RP_ORIGIN=http://localhost:5000

# OAuth - Google
GOOGLE_CLIENT_ID=xxx.apps.googleusercontent.com

# OAuth - Apple (native)
APPLE_CLIENT_ID=com.holidaywheel.phone

# OAuth - Apple (web)
APPLE_CLIENT_ID_WEB=com.holidaywheel.web
APPLE_REDIRECT_URI=https://domain.com/auth/api/oauth/apple/callback
```

See `apps/backend-rust/README.md` for full API documentation.
