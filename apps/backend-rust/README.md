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
    "display_name": "Player Name"
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
    "display_name": "John Doe"
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

---

## Socket.IO Events

Connect to the server with Socket.IO at the root path `/`.

### Client → Server

| Event | Payload | Description |
|-------|---------|-------------|
| `join` | `{ room: string, token: string }` | Join a game room |
| `claim_host` | `{ code: string }` | Claim host mode |
| `claim_player` | `{ player_idx: number }` | Claim a player slot |
| `spin` | `{}` | Spin the wheel (host) |
| `guess_letter` | `{ letter: string }` | Guess a consonant |
| `buy_vowel` | `{ letter: string }` | Buy a vowel ($250) |
| `solve` | `{ answer: string }` | Attempt to solve puzzle |
| `buzz` | `{}` | Buzz in during toss-up |
| `new_round` | `{}` | Start new round (host) |
| `new_game` | `{}` | Start new game (host) |
| `final_pick` | `{ consonants: string[], vowel: string }` | Final round picks |

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
  phase: "normal" | "tossup" | "final";
  wheel_value: number | "BANKRUPT" | "LOSE A TURN" | "FREE PLAY" | Prize;
  round: number;
  tossup?: TossupState;
  final?: FinalState;
}

interface Player {
  id: number;
  name: string;
  total: number;
  round_bank: number;
  prizes: Prize[];
  round_prizes: Prize[];
  claimed_user_id: number | null;
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

- Real-time puzzle board
- Spin wheel, guess letters, buy vowels, solve puzzle
- Host controls (New Game, Reveal All) when host mode is claimed

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

# Format code
cargo fmt

# Lint
cargo clippy
```

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
