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
}

export interface Prize {
  type: 'PRIZE';
  name: string;
  value?: number;
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
  | Prize;

export type GamePhase = 'normal' | 'tossup' | 'final';

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
