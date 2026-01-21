# Holiday Wheel Backend (Rust)

A real-time game server for the Holiday Wheel of Fortune game, built with Axum and Socket.IO.

## Features

- **Multiple Authentication Methods:**
  - Email/Password with optional email verification
  - Passkeys (WebAuthn) for passwordless authentication
  - Google Sign-In (OAuth)
  - Apple Sign-In (OAuth)
- **Real-time Gameplay** via Socket.IO
- **Web Client** with login, registration, lobby, and game pages
- **Admin Panel** for user and puzzle management
- **Optional HTTPS/TLS** support

## Quick Start

```bash
# Copy environment file
cp .env.example .env

# Run with cargo
cargo run

# Or with Docker/Podman
docker build -t holiday-wheel-backend .
docker run -p 5000:5000 holiday-wheel-backend
```

## Environment Variables

See `.env.example` for all available options.

### Core Settings

| Variable | Description | Default |
|----------|-------------|---------|
| `DB_PATH` | SQLite database path | `puzzles.db` |
| `DATABASE_URL` | Alternative: SQLite URL (e.g., `sqlite:puzzles.db`) | - |
| `PORT` | Server port | `5000` |
| `HOST_CODE` | Code to claim host mode | `holiday` |
| `ADMIN_EMAIL` | Email to auto-grant admin on login | - |
| `RUST_LOG` | Log level | `info` |

### SSL/TLS (HTTPS)

| Variable | Description | Default |
|----------|-------------|---------|
| `SSL_ENABLED` | Enable HTTPS | `false` |
| `SSL_CERT` | Path to certificate PEM file | - |
| `SSL_KEY` | Path to private key PEM file | - |

```bash
# Generate self-signed certificate for testing
openssl req -x509 -newkey rsa:4096 -keyout key.pem -out cert.pem -days 365 -nodes -subj "/CN=localhost"

# Run with HTTPS
docker run -p 5000:5000 \
  -e SSL_ENABLED=true \
  -e SSL_CERT=/app/certs/cert.pem \
  -e SSL_KEY=/app/certs/key.pem \
  -v ./certs:/app/certs:ro \
  holiday-wheel-backend
```

> **Note:** For local development, use HTTP (`SSL_ENABLED=false`). WebAuthn has a special exception allowing localhost over HTTP. For production, use a valid SSL certificate.

### Email Verification

| Variable | Description | Default |
|----------|-------------|---------|
| `EMAIL_ENABLED` | Enable email verification | `false` |
| `SMTP_HOST` | SMTP server hostname | - |
| `SMTP_PORT` | SMTP server port | `587` |
| `SMTP_USER` | SMTP username | - |
| `SMTP_PASS` | SMTP password | - |
| `FROM_EMAIL` | Sender email address | - |
| `BASE_URL` | Server URL for email links | `http://localhost:5000` |

### Passkey (WebAuthn) Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `WEBAUTHN_RP_ID` | Relying party ID (domain) | `localhost` |
| `WEBAUTHN_RP_NAME` | Relying party display name | `Holiday Wheel` |
| `WEBAUTHN_RP_ORIGIN` | Expected origin URL | `http://localhost:5000` |

> **Important:** For HTTPS, set `WEBAUTHN_RP_ORIGIN=https://yourdomain.com`

### OAuth Providers

| Variable | Description | Default |
|----------|-------------|---------|
| `GOOGLE_CLIENT_ID` | Google OAuth web client ID | - |
| `GOOGLE_CLIENT_ID_IOS` | Google OAuth iOS client ID | - |
| `GOOGLE_CLIENT_ID_ANDROID` | Google OAuth Android client ID | - |
| `APPLE_CLIENT_ID` | Apple Sign-In bundle ID (native apps) | - |
| `APPLE_CLIENT_ID_WEB` | Apple Services ID (web flow) | - |
| `APPLE_REDIRECT_URI` | Apple callback URL (web flow) | - |

---

## API Endpoints

### Health Check

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check endpoint |

---

### Authentication

#### Email/Password

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| POST | `/auth/api/register` | Register new user | No |
| POST | `/auth/api/login` | Login with email/password | No |
| GET | `/auth/api/verify` | Verify JWT token | Bearer |
| GET | `/auth/verify/{token}` | Verify email address | No |
| POST | `/auth/logout` | Logout | Bearer |
| GET | `/auth/me` | Get current user info | Bearer |

**Register Request:**
```json
{
  "email": "user@example.com",
  "password": "password123",
  "display_name": "Player Name"
}
```

**Login Request:**
```json
{
  "email": "user@example.com",
  "password": "password123"
}
```

**Auth Response:**
```json
{
  "ok": true,
  "token": "jwt-token",
  "user": {
    "id": 1,
    "email": "user@example.com",
    "display_name": "Player Name",
    "avatar_id": 1
  }
}
```

---

### Passkey (WebAuthn)

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| POST | `/auth/api/passkey/register/start` | Start passkey registration | No |
| POST | `/auth/api/passkey/register/finish` | Complete passkey registration | No |
| POST | `/auth/api/passkey/login/start` | Start passkey authentication | No |
| POST | `/auth/api/passkey/login/finish` | Complete passkey authentication | No |
| POST | `/auth/api/passkey/list` | List user's passkeys | Bearer |
| POST | `/auth/api/passkey/add/start` | Start adding passkey to account | Bearer |
| POST | `/auth/api/passkey/add/finish` | Complete adding passkey | Bearer |
| POST | `/auth/api/passkey/delete` | Delete a passkey | Bearer |

**Register Start Request:**
```json
{
  "email": "user@example.com",
  "display_name": "Player Name"
}
```

**Register Start Response:**
```json
{
  "ok": true,
  "options": {
    "challenge": "base64url-challenge",
    "rp": { "name": "Holiday Wheel", "id": "example.com" },
    "user": { "id": "base64url-id", "name": "user@example.com", "displayName": "Player Name" },
    "pubKeyCredParams": [...],
    "timeout": 300000,
    "authenticatorSelection": { "userVerification": "preferred" }
  }
}
```

**Login Start Request:**
```json
{
  "email": "user@example.com"
}
```

---

### OAuth

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| POST | `/auth/api/oauth/google` | Authenticate with Google | No |
| POST | `/auth/api/oauth/apple` | Authenticate with Apple (native) | No |
| GET | `/auth/api/oauth/apple/authorize` | Start Apple web sign-in (redirect) | No |
| POST | `/auth/api/oauth/apple/callback` | Apple web callback (form_post) | No |

**Google Auth Request (supports both token types):**
```json
{
  "id_token": "google-id-token-from-mobile-sdk"
}
// OR for web client:
{
  "access_token": "google-access-token-from-gis"
}
```

**Apple Auth Request (native apps):**
```json
{
  "identity_token": "apple-identity-token",
  "user_identifier": "apple-user-id",
  "email": "user@example.com",
  "full_name": {
    "given_name": "John",
    "family_name": "Doe"
  }
}
```

**Apple Web Flow:**
1. Client redirects to `GET /auth/api/oauth/apple/authorize`
2. User authenticates with Apple
3. Apple POSTs to `/auth/api/oauth/apple/callback`
4. Server redirects to `/lobby#auth_token=xxx&user=yyy`

**OAuth Response:**
```json
{
  "ok": true,
  "token": "jwt-token",
  "user": {
    "id": 1,
    "email": "user@example.com",
    "display_name": "John Doe",
    "avatar_id": 1
  }
}
```

---

### Rooms

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | `/auth/api/rooms` | List available game rooms | Bearer |

**Rooms Response:**
```json
{
  "ok": true,
  "rooms": [
    {
      "name": "main",
      "player_count": 3,
      "has_host": true
    }
  ]
}
```

---

### Profile

| Method | Path | Description | Auth |
|--------|------|-------------|------|
| GET | `/auth/api/profile` | Get current user's profile | Bearer |
| PUT | `/auth/api/profile` | Update profile (display name, avatar) | Bearer |

**Get Profile Response:**
```json
{
  "ok": true,
  "profile": {
    "id": 1,
    "email": "user@example.com",
    "display_name": "Player Name",
    "avatar_id": 1,
    "is_admin": false
  }
}
```

**Update Profile Request:**
```json
{
  "display_name": "New Name",  // Optional, 2-24 chars
  "avatar_id": 5               // Optional, 1-12
}
```

> **Note:** Avatar IDs range from 1-12. Values outside this range are clamped.

---

### Admin Endpoints

All admin endpoints require Bearer token with admin privileges.

#### User Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/api/admin/users` | List all users |
| POST | `/auth/api/admin/users/{id}/admin` | Set user admin status |
| POST | `/auth/api/admin/users/{id}/verify` | Manually verify user email |
| DELETE | `/auth/api/admin/users/{id}` | Delete user |

#### Puzzle Packs

| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/api/admin/packs` | List puzzle packs |
| POST | `/auth/api/admin/packs` | Create puzzle pack |
| DELETE | `/auth/api/admin/packs/{id}` | Delete puzzle pack |

#### Puzzles

| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/api/admin/puzzles` | List puzzles (with optional `?pack_id=`) |
| POST | `/auth/api/admin/puzzles` | Add single puzzle |
| POST | `/auth/api/admin/puzzles/import` | Bulk import puzzles |
| DELETE | `/auth/api/admin/puzzles/{id}` | Delete puzzle |

**Add Puzzle Request:**
```json
{
  "category": "PHRASE",
  "answer": "HELLO WORLD",
  "pack_id": 1
}
```

**Import Puzzles Request:**
```json
{
  "pack_id": 1,
  "puzzles": [
    { "category": "PHRASE", "answer": "HELLO WORLD" },
    { "category": "THING", "answer": "BICYCLE" }
  ]
}
```

#### Rooms Management

| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/api/admin/rooms` | List active rooms |
| DELETE | `/auth/api/admin/rooms/{name}` | Delete/close room |

#### Game Settings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/auth/api/admin/settings/{room}` | Get room settings |
| POST | `/auth/api/admin/settings/{room}` | Save room settings |

**Room Settings:**
```json
{
  "vowel_cost": 250,                    // Cost to buy a vowel
  "bonus_seconds": 30,                  // Seconds for bonus round timer
  "bonus_jackpot": 10000,               // Jackpot amount for bonus round
  "prize_replace_cash_values": [500, 1000, 1500, 2000],  // Wheel values
  "puzzle_display_seconds": 30,         // Puzzle reveal animation time
  "prize_wedge_names": ["GIFT CARD"],   // Prize wedge labels
  "pack_id": 1,                         // Active puzzle pack
  "disconnect_timeout_secs": 300,       // Player disconnect timeout
  "turn_timer_seconds": 10,             // Per-turn timer
  "buzz_timer_seconds": 5               // Toss-up buzz-in timer
}
```

---

## Socket.IO Events

Connect to the server with Socket.IO at the root path `/`.

### Client → Server

| Event | Payload | Description |
|-------|---------|-------------|
| `join` | `{ room: string, token: string }` | Join a game room |
| `claim_host` | `{ code: string }` | Claim host mode |
| `claim_player` | `{ player_idx: number }` | Claim a player slot |
| `spin` | `{}` | Spin the wheel (host/active player) |
| `guess_letter` | `{ letter: string }` | Guess a consonant |
| `buy_vowel` | `{ letter: string }` | Buy a vowel ($250) |
| `solve` | `{ answer: string }` | Attempt to solve puzzle |
| `buzz` | `{}` | Buzz in during toss-up |
| `new_round` | `{}` | Start new puzzle (host) |
| `new_game` | `{}` | Reset entire game (host) |
| `advance_round` | `{}` | Advance to next round 1→2→3→4 (host) |
| `set_round` | `{ round: number }` | Set specific round 1-4 (host) |
| `start_tossup` | `{}` | Start toss-up phase (host) |
| `end_tossup` | `{}` | End toss-up phase (host) |
| `start_final_spin` | `{}` | Start Final Spin mode (host) |
| `end_final_spin` | `{}` | End Final Spin mode (host) |
| `start_bonus` | `{}` | Start Bonus round (host) |
| `end_bonus` | `{}` | End Bonus round (host) |
| `bonus_pick` | `{ consonants: string[], vowel: string }` | Bonus round letter picks |

### Server → Client

| Event | Payload | Description |
|-------|---------|-------------|
| `state` | `GameState` | Full game state update |
| `rooms` | `Room[]` | Available rooms list |
| `notification` | `{ message: string }` | Server notification |
| `error` | `{ message: string }` | Error message |

---

## Game State Structure

```typescript
interface GameState {
  puzzle: {
    category: string;
    revealed: string;  // "H E L L O   _ O R L D"
    answer?: string;   // Only visible to host
  };
  players: Player[];
  active_idx: number;
  phase: "normal" | "tossup" | "final" | "bonus";
  wheel_value: number | "BANKRUPT" | "LOSE A TURN" | "FREE PLAY" | Prize;
  round: number;       // Current round (1-4)
  tossup?: TossupState;
  final_spin?: FinalSpinState;
  bonus?: BonusState;
}

interface Player {
  id: number;
  name: string;
  total: number;
  round_bank: number;
  prizes: Prize[];
  round_prizes: Prize[];
  claimed_user_id: number | null;
  avatar_id: number;   // Avatar ID (1-12)
}

interface FinalSpinState {
  spin_value: number;      // Fixed spin value for all turns
  turns_remaining: number; // Turns left in Final Spin
  free_vowels: boolean;    // Vowels are free during Final Spin
}

interface BonusState {
  consonants_picked: string[];  // 3 consonants chosen
  vowel_picked: string | null;  // 1 vowel chosen
  time_remaining: number;       // Seconds left to solve
  running: boolean;             // Timer is active
}
```

---

## Web Client

The server includes a built-in web client accessible at the root URL.

### Pages

| Path | Description |
|------|-------------|
| `/` | Login page |
| `/register` | Registration page |
| `/lobby` | Game lobby (requires auth) |
| `/game?room={name}` | Game room (requires auth) |
| `/join?room={name}` | Universal link - tries app deep link, falls back to web |
| `/admin` | Admin panel (requires admin) |

### Authentication Options

#### Login Page (`/`)

- **Sign in with Passkey** - Passwordless authentication using device biometrics or security key
- **Sign in with Google** - OAuth authentication (requires `GOOGLE_CLIENT_ID`)
- **Sign in with Apple** - OAuth authentication (web requires `APPLE_CLIENT_ID_WEB` + `APPLE_REDIRECT_URI`)
- **Email/Password** - Traditional login

#### Register Page (`/register`)

- **Create Account with Passkey** - Register with passkey (no password needed)
- **Email/Password Registration** - Traditional registration with optional email verification

### Lobby Features

- View and join active game rooms
- QR code for phone app connection
- **Passkey Management** - Add, view, and delete passkeys for your account
- Admin access (if authorized)

### Game Features

- **Real-time puzzle board** with animated letter reveals
- **Animated wheel** with smooth easing and 24 unique wedge values
- **Interactive controls**: Spin wheel, guess letters, buy vowels, solve puzzle
- **Visual feedback**:
  - Spin button flashes when waiting for player to spin
  - Letter input box flashes when waiting for letter guess
  - Large letter display notifications for guess results (green for correct, red for miss)
  - Inline notifications between puzzle board and controls
  - Letter stays visible in input box briefly after guessing
- **Host controls** (New Game, Reveal All, Round management) when host mode is claimed
- **Sound effects** for wheel spin, correct/incorrect guesses, bankrupt, and solve
- **Player sidebar** with scores, room name, current round, and phase

### Game Flow

A typical game follows this structure:

```
Game Start: Toss-up (determines who goes first)
Round 1:    Normal gameplay → Solve (winner goes first in Round 2)
Round 2:    Normal gameplay → Solve (winner goes first in Round 3)
Round 3:    Normal gameplay → Solve (winner goes first in Round 4)
Round 4:    Normal gameplay → Final Spin → Solve
Bonus:      Winner's bonus round
```

### Game Phases

1. **Toss-up** (`phase: "tossup"`): Determines starting player
   - Used once at game start to determine who plays first in Round 1
   - Letters reveal automatically one at a time
   - Any player can buzz in to solve
   - First correct answer wins and becomes active player
   - Wrong answer locks player out for remainder

2. **Normal** (`phase: "normal"`): Standard gameplay
   - Players take turns spinning the wheel
   - Spin → Guess consonant → Earn money per letter
   - Buy vowels for $250
   - Solve puzzle to win round bank
   - Round winner goes first in the next round

3. **Final Spin** (`phase: "final"`): End-game accelerated play
   - Typically used in Round 4 when time is limited
   - Host spins once to set a fixed value for all turns
   - Each consonant is worth: spin value + $1,000
   - Vowels are FREE (no $250 cost)
   - Limited turns rotate through all players

4. **Bonus** (`phase: "bonus"`): Winner's bonus round
   - R, S, T, L, N, E are automatically revealed
   - Winner picks 3 additional consonants + 1 vowel
   - 10-second timer to solve the puzzle
   - Correct solve wins the bonus jackpot (default: $10,000)

### Rounds

Games are divided into 4 rounds:

- **Toss-up**: Played once at game start to determine first player
- **Round 1-4**: Normal gameplay; round winner goes first in next round
- **Round 4**: Often uses Final Spin mode to finish quickly

Host controls:
- `start_tossup` / `end_tossup` - Begin/end toss-up phase (game start)
- `advance_round` - Move to next round (1→2→3→4)
- `set_round` - Set specific round (1-4)
- `start_final_spin` / `end_final_spin` - Enable Final Spin mode
- Round resets to 1 on new game

---

## Authentication Flows

### Passkey Registration (New User)

1. User enters email and display name
2. Client calls `POST /auth/api/passkey/register/start`
3. Server returns WebAuthn creation options
4. Browser prompts user to create passkey (biometric/PIN)
5. Client calls `POST /auth/api/passkey/register/finish` with credential
6. Server creates user (auto-verified) and returns JWT token

### Passkey Login (Existing User)

1. User enters email (optional for discoverable credentials)
2. Client calls `POST /auth/api/passkey/login/start`
3. Server returns WebAuthn request options
4. Browser prompts user to authenticate with passkey
5. Client calls `POST /auth/api/passkey/login/finish` with assertion
6. Server verifies and returns JWT token

### Adding Passkey to Existing Account

1. User logs in with any method
2. In lobby, click "+ Add Passkey"
3. Client calls `POST /auth/api/passkey/add/start` (with Bearer token)
4. Browser prompts to create new passkey
5. Client calls `POST /auth/api/passkey/add/finish`
6. Passkey is linked to user's account

### OAuth (Google/Apple)

1. User clicks social login button
2. Native SDK handles authentication
3. Client receives ID token from provider
4. Client calls `POST /auth/api/oauth/google` or `/auth/api/oauth/apple`
5. Server verifies token with provider's public keys
6. Server creates/finds user (auto-verified) and returns JWT token

---

## Development

```bash
# Run with hot reload
cargo watch -x run

# Run tests
cargo test

# Run specific test
cargo test migration  # Run migration tests

# Format code
cargo fmt

# Lint
cargo clippy
```

### Database Migrations

The backend uses SQLx migrations stored in `migrations/`. Migrations run automatically on startup.

```bash
# Check migration status
sqlx migrate info --database-url sqlite:puzzles.db

# Create new migration
sqlx migrate add <migration_name>
```

**Current migrations:**
- `001_initial_schema.sql` - Core tables (users, puzzles, rooms, etc.)
- `002_additional_indexes.sql` - Performance indexes
- `003_add_avatar.sql` - User avatar support
- `004_rename_final_to_bonus.sql` - Rename final_* columns to bonus_*

**Migration best practices:**
- Always test migrations against existing databases with data
- Use `ALTER TABLE RENAME COLUMN` for column renames (SQLite 3.25+)
- Add new columns with `DEFAULT` values for backwards compatibility
- Migration tests are in `src/db/mod.rs` (search for "MIGRATION TESTS")

### Docker / Podman

The Dockerfile is compatible with both Docker and Podman:

```bash
# Docker
docker build -t holiday-wheel-backend .
docker run -p 5000:5000 -v ./data:/app/data holiday-wheel-backend

# Podman (same commands work)
podman build -t holiday-wheel-backend .
podman run -p 5000:5000 -v ./data:/app/data holiday-wheel-backend
```

> **Note:** The Dockerfile uses fully-qualified image names (`docker.io/library/...`) for Podman compatibility.

---

## Security Notes

- Passkey users have no password (password_hash is NULL)
- OAuth users are auto-verified (no email confirmation needed)
- WebAuthn challenges expire after 5 minutes
- JWT tokens are sent via `Authorization: Bearer <token>` header
- Web client stores tokens in `localStorage` (convenient but vulnerable to XSS)
  - For higher security, consider HttpOnly cookies or in-memory storage
- For production, always use HTTPS with a valid certificate
- Rate limiting on auth endpoints is recommended for production
