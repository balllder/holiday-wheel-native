/**
 * Typed Socket.IO events for Holiday Wheel Native
 *
 * These interfaces provide type safety for all socket.emit() and socket.on() calls
 * between the client and server.
 */

import type { ServerGameState, RoomInfo } from './index';

// ============================================================================
// Server to Client Events (events the client listens for)
// ============================================================================

/**
 * Events emitted by the server that clients listen for
 */
export interface ServerToClientEvents {
  /**
   * Full game state update
   * Emitted whenever the game state changes
   */
  state: (state: ServerGameState) => void;

  /**
   * Player identification
   * Tells the client which player index they are (or null if spectator)
   */
  you: (data: YouPayload) => void;

  /**
   * Host status update
   * Tells the client whether they have host privileges
   */
  host_granted: (data: HostGrantedPayload) => void;

  /**
   * Toast notification
   * Server-initiated notification to display to the user
   */
  toast: (data: ToastPayload) => void;

  /**
   * Session invalidated
   * Emitted when user is logged in from another device
   */
  session_invalidated: (data: SessionInvalidatedPayload) => void;

  /**
   * Available rooms list
   * Emitted in lobby to show available game rooms
   */
  rooms: (rooms: RoomInfo[]) => void;

  /**
   * General notification message
   */
  notification: (message: string) => void;

  /**
   * Error message from server
   */
  error: (error: ErrorPayload) => void;
}

// ============================================================================
// Client to Server Events (events the client emits)
// ============================================================================

/**
 * Events emitted by the client that the server handles
 */
export interface ClientToServerEvents {
  // -- Authentication --

  /**
   * Authenticate the socket connection with a JWT token
   */
  auth: (data: AuthPayload) => void;

  // -- Room Management --

  /**
   * Join a room as a spectator
   */
  join: (data: RoomPayload) => void;

  /**
   * Join a room as a player
   */
  join_game: (data: RoomPayload) => void;

  /**
   * Leave the game (become spectator)
   */
  leave_game: (data: RoomPayload) => void;

  /**
   * Claim host privileges with a code
   */
  claim_host: (data: ClaimHostPayload) => void;

  /**
   * Release host privileges
   */
  release_host: (data: RoomPayload) => void;

  // -- Basic Game Actions --

  /**
   * Spin the wheel
   */
  spin: (data: RoomPayload) => void;

  /**
   * Guess a consonant letter
   */
  guess: (data: GuessPayload) => void;

  /**
   * Buy a vowel ($250)
   */
  buy_vowel: (data: VowelPayload) => void;

  /**
   * Attempt to solve the puzzle
   */
  solve: (data: SolvePayload) => void;

  /**
   * Buzz in during toss-up phase
   */
  buzz: (data: RoomPayload) => void;

  // -- Host Controls --

  /**
   * Load a new puzzle
   */
  new_puzzle: (data: RoomPayload) => void;

  /**
   * Start a new game (reset all scores)
   */
  new_game: (data: RoomPayload) => void;

  /**
   * Set the active player (host only)
   */
  set_active_player: (data: SetActivePlayerPayload) => void;

  /**
   * Reveal all letters (host only)
   */
  reveal_all: (data: RoomPayload) => void;

  // -- Toss-up Controls --

  /**
   * Start toss-up phase
   * @param player_idxs - Optional array of player indices allowed to buzz
   */
  start_tossup: (data: StartTossupPayload) => void;

  /**
   * End toss-up phase
   */
  end_tossup: (data: RoomPayload) => void;

  // -- Final Round Controls --

  /**
   * Start final/bonus round
   * @param player_idx - Optional player index for the final round
   */
  start_final: (data: StartFinalPayload) => void;

  /**
   * End final/bonus round
   */
  end_final: (data: RoomPayload) => void;

  /**
   * Pick a letter in final round (RSTLNE + player picks)
   */
  final_pick: (data: FinalPickPayload) => void;

  // -- Special Wedge Actions --

  /**
   * Make a mystery wedge choice (keep $1,000 or flip for $10,000/Bankrupt)
   */
  mystery_choice: (data: MysteryChoicePayload) => void;

  /**
   * Use a wild card to guess another consonant
   */
  use_wild_card: (data: WildCardPayload) => void;
}

// ============================================================================
// Payload Types
// ============================================================================

// -- Server to Client Payloads --

export interface YouPayload {
  player_idx: number | null;
}

export interface HostGrantedPayload {
  granted: boolean;
}

export interface ToastPayload {
  msg: string;
}

export interface SessionInvalidatedPayload {
  reason: string;
}

export interface ErrorPayload {
  message: string;
  code?: string;
}

// -- Client to Server Payloads --

export interface AuthPayload {
  token: string;
}

export interface RoomPayload {
  room: string;
}

export interface ClaimHostPayload {
  room: string;
  code: string;
}

export interface GuessPayload {
  room: string;
  letter: string;
}

export interface VowelPayload {
  room: string;
  vowel: string;
}

export interface SolvePayload {
  room: string;
  attempt: string;
}

export interface SetActivePlayerPayload {
  room: string;
  idx: number;
}

export interface StartTossupPayload {
  room: string;
  player_idxs?: number[];
}

export interface StartFinalPayload {
  room: string;
  player_idx?: number;
}

export interface FinalPickPayload {
  room: string;
  letter: string;
}

export interface MysteryChoicePayload {
  room: string;
  choice: 'keep' | 'flip';
}

export interface WildCardPayload {
  room: string;
  letter: string;
}

// ============================================================================
// Socket Type Helper
// ============================================================================

/**
 * Inter-server events (not used in client, but required for Socket.IO typing)
 */
export interface InterServerEvents {
  // Reserved for server-to-server communication
}

/**
 * Socket data attached to each socket connection
 */
export interface SocketData {
  userId?: number;
  displayName?: string;
  room?: string;
}

/**
 * Typed socket instance for use in the client
 *
 * Usage:
 * ```typescript
 * import { io } from 'socket.io-client';
 * import type { TypedClientSocket } from '@shared/types/socketEvents';
 *
 * const socket: TypedClientSocket = io(baseUrl);
 * socket.emit('spin', { room: 'main' }); // Type-safe!
 * socket.on('state', (state) => { ... }); // state is typed as ServerGameState
 * ```
 */
export type TypedClientSocket = import('socket.io-client').Socket<
  ServerToClientEvents,
  ClientToServerEvents
>;
