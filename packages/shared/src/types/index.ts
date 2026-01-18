// Types matching the Flask backend game state

export interface Player {
  id: number;
  name: string;
  total: number;
  prizes: Prize[];
  round_bank: number;
  round_prizes: Prize[];
  claimed_sid: string | null;
  claimed_user_id: number | null;
  /** Number of wild cards the player has collected */
  wild_cards?: number;
}

export interface Prize {
  type: 'PRIZE';
  name: string;
  value?: number;
}

/** Mystery wedge - player can keep $1,000 or flip for $10,000/Bankrupt */
export interface MysteryWedge {
  type: 'MYSTERY';
}

/** Express wedge - rapid-fire consonant guessing at $1,000 each */
export interface ExpressWedge {
  type: 'EXPRESS';
}

/** Wild Card wedge - collectible token for extra consonant guess */
export interface WildCardWedge {
  type: 'WILD_CARD';
}

export interface Puzzle {
  id: number;
  category: string;
  answer: string;
}

export type WedgeValue =
  | number
  | 'BANKRUPT'
  | 'LOSE A TURN'
  | 'FREE PLAY'
  | Prize
  | MysteryWedge
  | ExpressWedge
  | WildCardWedge;

export type GamePhase = 'normal' | 'tossup' | 'final';

/** Type of round in multi-round game structure */
export type RoundType = 'normal' | 'tossup' | 'speed' | 'bonus';

/** Round configuration for multi-round games */
export interface RoundConfig {
  /** Round number (1-indexed) */
  number: number;
  /** Type of round */
  type: RoundType;
  /** Base wheel values multiplier (e.g., 1 for round 1, 2 for round 3) */
  value_multiplier: number;
  /** Whether this round has mystery wedge */
  has_mystery: boolean;
  /** Whether this round has express wedge */
  has_express: boolean;
}

/** State for multi-round game structure */
export interface RoundState {
  /** Current round number (1-indexed, 0 = not started) */
  current_round: number;
  /** Total number of rounds in this game */
  total_rounds: number;
  /** Configuration for each round */
  rounds: RoundConfig[];
  /** Whether multi-round mode is enabled */
  enabled: boolean;
}

/** Enhanced toss-up configuration */
export interface TossupConfig {
  /** Whether this is a triple toss-up (3 consecutive puzzles) */
  is_triple: boolean;
  /** Current puzzle index in triple toss-up (0-2) */
  triple_index: number;
  /** Values for each toss-up in triple ($1000, $2000, $3000) */
  values: number[];
  /** Delay between letter reveals in ms */
  reveal_delay: number;
  /** Whether letters reveal automatically */
  auto_reveal: boolean;
  /** Index of next letter to reveal */
  next_reveal_index: number;
}

export type FinalStage = 'off' | 'pick' | 'reveal' | 'solve';

export interface TossupState {
  controller_player_idx: number | null;
  locked_player_idxs: number[];
  allowed_player_idxs: number[];
  remaining_seconds: number | null;
}

export interface FinalState {
  stage: FinalStage;
  picks_consonants: string[];
  pick_vowel: string | null;
  remaining_seconds: number | null;
}

/** Mystery wedge state - tracks when player needs to make a choice */
export type MysteryStage = 'off' | 'pending_guess' | 'awaiting_choice' | 'revealing';

export interface MysteryState {
  /** Current stage of mystery wedge interaction */
  stage: MysteryStage;
  /** Player index who landed on mystery */
  player_idx: number | null;
  /** The choice made by player: 'keep' ($1,000) or 'flip' ($10,000/Bankrupt) */
  choice: 'keep' | 'flip' | null;
  /** Result of flip: true = $10,000, false = Bankrupt */
  flip_result: boolean | null;
}

/** Express mode state - tracks rapid-fire guessing */
export interface ExpressState {
  /** Whether express mode is currently active */
  active: boolean;
  /** Player index in express mode */
  player_idx: number | null;
  /** Number of correct guesses made in express mode */
  correct_count: number;
  /** Per-consonant value in express mode (default $1,000) */
  value_per_consonant: number;
}

export interface HostState {
  claimed: boolean;
  player_idx: number | null;
}

export interface PackInfo {
  id: number;
  name: string;
}

export interface DbCounts {
  used: number;
  total: number;
  unused: number;
}

// Main game state received from server via Socket.IO
export interface ServerGameState {
  phase: GamePhase;
  puzzle: Puzzle;
  revealed: string[];
  used: string[];
  players: Player[];
  active_idx: number;
  wheel_slots: WedgeValue[];
  wheel_index: number | null;
  last_spin_index: number | null;
  current_wedge: WedgeValue | null;
  packs: PackInfo[];
  active_pack_id: number | null;
  active_pack_name: string | null;
  db: DbCounts;
  host: HostState;
  tossup: TossupState;
  final: FinalState;
  /** Mystery wedge state (optional for backwards compatibility) */
  mystery?: MysteryState;
  /** Express mode state (optional for backwards compatibility) */
  express?: ExpressState;
  /** Multi-round game state (optional for backwards compatibility) */
  round?: RoundState;
  /** Enhanced toss-up configuration (optional for backwards compatibility) */
  tossup_config?: TossupConfig;
}

// User from auth system
export interface User {
  id: number;
  email: string;
  display_name: string;
}

// Auth response
export interface AuthResponse {
  ok: boolean;
  token?: string;
  user?: User;
  error?: string;
}

// Room info for lobby
export interface RoomInfo {
  name: string;
  player_count: number;
  last_activity: string;
}

// Passkey types
export interface PasskeyInfo {
  id: string;
  device_name: string | null;
  created_at: number;
  last_used_at: number | null;
}

export interface PasskeyStartResponse {
  ok: boolean;
  options?: PublicKeyCredentialCreationOptionsJSON | PublicKeyCredentialRequestOptionsJSON;
  error?: string;
}

export interface PasskeyFinishResponse {
  ok: boolean;
  token?: string;
  user?: User;
  error?: string;
}

export interface PasskeyListResponse {
  ok: boolean;
  passkeys?: PasskeyInfo[];
  error?: string;
}

// WebAuthn JSON types (matching the browser API)
export interface PublicKeyCredentialCreationOptionsJSON {
  rp: {
    name: string;
    id?: string;
  };
  user: {
    id: string;
    name: string;
    displayName: string;
  };
  challenge: string;
  pubKeyCredParams: { type: string; alg: number }[];
  timeout?: number;
  excludeCredentials?: { type: string; id: string; transports?: string[] }[];
  authenticatorSelection?: {
    authenticatorAttachment?: string;
    residentKey?: string;
    requireResidentKey?: boolean;
    userVerification?: string;
  };
  attestation?: string;
}

export interface PublicKeyCredentialRequestOptionsJSON {
  challenge: string;
  timeout?: number;
  rpId?: string;
  allowCredentials?: { type: string; id: string; transports?: string[] }[];
  userVerification?: string;
}

// OAuth types
export interface OAuthResponse {
  ok: boolean;
  token?: string;
  user?: User;
  is_new_user?: boolean;
  error?: string;
}

export interface AppleFullName {
  givenName?: string;
  familyName?: string;
}
