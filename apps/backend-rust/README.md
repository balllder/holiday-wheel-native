# Holiday Wheel Backend (Rust)

A real-time game server for the Holiday Wheel of Fortune game, built with Axum and Socket.IO.

## Quick Start

```bash
# Copy environment file
cp .env.example .env

# Run with cargo
cargo run

# Or with Docker
docker build -t holiday-wheel-backend .
docker run -p 5000:5000 holiday-wheel-backend
```

## Environment Variables

See `.env.example` for all available options.

| Variable | Description | Default |
|----------|-------------|---------|
| `DATABASE_URL` | SQLite database path | `sqlite:puzzles.db` |
| `PORT` | Server port | `5000` |
| `HOST_CODE` | Code to claim host mode | `holiday` |
| `EMAIL_ENABLED` | Enable email verification | `false` |
| `WEBAUTHN_RP_ID` | Passkey relying party ID | `localhost` |
| `GOOGLE_CLIENT_ID` | Google OAuth web client ID | - |
| `APPLE_CLIENT_ID` | Apple Sign-In bundle ID | - |

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
| POST | `/auth/api/oauth/apple` | Authenticate with Apple | No |

**Google Auth Request:**
```json
{
  "id_token": "google-id-token-from-sdk"
}
```

**Apple Auth Request:**
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
