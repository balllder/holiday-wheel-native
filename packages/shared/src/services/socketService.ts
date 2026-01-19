import { io, Socket } from 'socket.io-client';
import { useGameStore, ConnectionStatus } from '../stores/gameStore';
import type { ServerGameState } from '../types';

type ToastCallback = (message: string) => void;
type SessionInvalidatedCallback = (reason: string) => void;
type ConnectionStatusCallback = (
  status: ConnectionStatus,
  attempt?: number
) => void;

/**
 * Reconnection configuration with exponential backoff
 */
interface ReconnectionConfig {
  /** Initial delay before first reconnection attempt (ms) */
  initialDelay: number;
  /** Maximum delay between reconnection attempts (ms) */
  maxDelay: number;
  /** Multiplier for exponential backoff */
  factor: number;
  /** Maximum number of reconnection attempts (0 = unlimited) */
  maxAttempts: number;
  /** Add randomness to prevent thundering herd (0-1) */
  jitter: number;
}

const DEFAULT_RECONNECTION_CONFIG: ReconnectionConfig = {
  initialDelay: 1000, // Start with 1 second
  maxDelay: 30000, // Cap at 30 seconds
  factor: 2, // Double each time
  maxAttempts: 15, // Try 15 times before giving up
  jitter: 0.3, // Add up to 30% randomness
};

class SocketService {
  private socket: Socket | null = null;
  private onToast: ToastCallback | null = null;
  private onSessionInvalidated: SessionInvalidatedCallback | null = null;
  private onConnectionStatusChange: ConnectionStatusCallback | null = null;
  private currentToken: string | undefined = undefined;
  private currentBaseUrl: string = '';
  private reconnectionConfig: ReconnectionConfig = DEFAULT_RECONNECTION_CONFIG;
  private manualDisconnect: boolean = false;

  /**
   * Calculate delay with exponential backoff and jitter
   */
  private calculateBackoffDelay(attempt: number): number {
    const { initialDelay, maxDelay, factor, jitter } = this.reconnectionConfig;
    const exponentialDelay = initialDelay * Math.pow(factor, attempt - 1);
    const cappedDelay = Math.min(exponentialDelay, maxDelay);
    // Add jitter to prevent all clients reconnecting simultaneously
    const jitterRange = cappedDelay * jitter;
    const randomJitter = Math.random() * jitterRange - jitterRange / 2;
    return Math.round(cappedDelay + randomJitter);
  }

  /**
   * Configure reconnection parameters
   */
  setReconnectionConfig(config: Partial<ReconnectionConfig>): void {
    this.reconnectionConfig = { ...this.reconnectionConfig, ...config };
  }

  /**
   * Connect to the Socket.IO server
   */
  connect(baseUrl: string, token?: string): void {
    if (this.socket?.connected) {
      return;
    }

    this.manualDisconnect = false;
    this.currentToken = token;
    this.currentBaseUrl = baseUrl;

    const store = useGameStore.getState();
    store.setConnectionStatus('connecting');
    store.setReconnectAttempt(0);

    this.notifyConnectionStatus('connecting');

    this.socket = io(baseUrl, {
      transports: ['websocket', 'polling'],
      auth: token ? { token } : undefined,
      reconnection: true,
      reconnectionAttempts: this.reconnectionConfig.maxAttempts,
      reconnectionDelay: this.reconnectionConfig.initialDelay,
      reconnectionDelayMax: this.reconnectionConfig.maxDelay,
      randomizationFactor: this.reconnectionConfig.jitter,
    });

    this.setupListeners();
  }

  /**
   * Disconnect from the server
   */
  disconnect(): void {
    this.manualDisconnect = true;
    this.socket?.disconnect();
    this.socket = null;
    const store = useGameStore.getState();
    store.setConnectionStatus('disconnected');
    store.setReconnectAttempt(0);
    this.notifyConnectionStatus('disconnected');
  }

  /**
   * Manually trigger a reconnection attempt
   */
  reconnect(): void {
    if (this.socket?.connected) {
      return;
    }

    if (this.currentBaseUrl) {
      this.connect(this.currentBaseUrl, this.currentToken);
    }
  }

  /**
   * Notify connection status change
   */
  private notifyConnectionStatus(
    status: ConnectionStatus,
    attempt?: number
  ): void {
    if (this.onConnectionStatusChange) {
      this.onConnectionStatusChange(status, attempt);
    }
  }

  /**
   * Set up Socket.IO event listeners
   */
  private setupListeners(): void {
    if (!this.socket) return;

    this.socket.on('connect', () => {
      console.log('[Socket] Connected');
      const store = useGameStore.getState();
      store.setConnectionStatus('connected');
      store.setReconnectAttempt(0);
      this.notifyConnectionStatus('connected');

      // Authenticate the socket for session management
      if (this.currentToken) {
        this.socket?.emit('auth', { token: this.currentToken });
      }
    });

    this.socket.on('disconnect', (reason) => {
      console.log('[Socket] Disconnected:', reason);
      const store = useGameStore.getState();

      // Only set to disconnected if it was intentional
      if (this.manualDisconnect || reason === 'io client disconnect') {
        store.setConnectionStatus('disconnected');
        store.setReconnectAttempt(0);
        this.notifyConnectionStatus('disconnected');
      }
      // Otherwise socket.io will handle reconnection automatically
    });

    this.socket.on('connect_error', (error) => {
      console.error('[Socket] Connection error:', error.message);
      // Socket.io handles reconnection automatically
    });

    // Track reconnection attempts
    this.socket.io.on('reconnect_attempt', (attempt) => {
      const delay = this.calculateBackoffDelay(attempt);
      console.log(
        `[Socket] Reconnection attempt ${attempt}/${this.reconnectionConfig.maxAttempts}, delay: ${delay}ms`
      );
      const store = useGameStore.getState();
      store.setConnectionStatus('reconnecting');
      store.setReconnectAttempt(attempt);
      this.notifyConnectionStatus('reconnecting', attempt);
    });

    this.socket.io.on('reconnect', (attempt) => {
      console.log(`[Socket] Reconnected after ${attempt} attempt(s)`);
      const store = useGameStore.getState();
      store.setConnectionStatus('connected');
      store.setReconnectAttempt(0);
      this.notifyConnectionStatus('connected');
    });

    this.socket.io.on('reconnect_error', (error) => {
      console.error('[Socket] Reconnection error:', error.message);
    });

    this.socket.io.on('reconnect_failed', () => {
      console.error(
        '[Socket] Reconnection failed after max attempts:',
        this.reconnectionConfig.maxAttempts
      );
      const store = useGameStore.getState();
      store.setConnectionStatus('disconnected');
      this.notifyConnectionStatus('disconnected');

      // Notify via toast if callback is set
      if (this.onToast) {
        this.onToast(
          'Connection lost. Please check your network and try again.'
        );
      }
    });

    // Session invalidation (logged in from another device)
    this.socket.on('session_invalidated', (data: { reason: string }) => {
      console.log('[Socket] Session invalidated:', data.reason);
      if (this.onSessionInvalidated) {
        this.onSessionInvalidated(data.reason);
      }
    });

    // Main game state update
    this.socket.on('state', (data: ServerGameState) => {
      useGameStore.getState().updateFromServer(data);
    });

    // Player identification
    this.socket.on('you', (data: { player_idx: number | null }) => {
      useGameStore.getState().setMyPlayerIdx(data.player_idx);
    });

    // Host status
    this.socket.on('host_granted', (data: { granted: boolean }) => {
      useGameStore.getState().setIsHost(data.granted);
    });

    // Toast notifications
    this.socket.on('toast', (data: { msg: string }) => {
      if (this.onToast) {
        this.onToast(data.msg);
      }
    });
  }

  /**
   * Set toast notification callback
   */
  setToastCallback(callback: ToastCallback): void {
    this.onToast = callback;
  }

  /**
   * Set session invalidated callback (for handling logout when logged in elsewhere)
   */
  setSessionInvalidatedCallback(callback: SessionInvalidatedCallback): void {
    this.onSessionInvalidated = callback;
  }

  /**
   * Set connection status change callback
   */
  setConnectionStatusCallback(callback: ConnectionStatusCallback): void {
    this.onConnectionStatusChange = callback;
  }

  /**
   * Get current reconnection attempt count
   */
  getReconnectAttempt(): number {
    return useGameStore.getState().reconnectAttempt;
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): ConnectionStatus {
    return useGameStore.getState().connectionStatus;
  }

  /**
   * Join a room
   */
  joinRoom(room: string): void {
    useGameStore.getState().setRoom(room);
    this.socket?.emit('join', { room });
  }

  /**
   * Join the game as a player
   */
  joinGame(room: string): void {
    this.socket?.emit('join_game', { room });
  }

  /**
   * Leave the game
   */
  leaveGame(room: string): void {
    this.socket?.emit('leave_game', { room });
  }

  /**
   * Claim host privileges
   */
  claimHost(room: string, code: string): void {
    this.socket?.emit('claim_host', { room, code });
  }

  /**
   * Release host privileges
   */
  releaseHost(room: string): void {
    this.socket?.emit('release_host', { room });
  }

  /**
   * Spin the wheel
   */
  spin(room: string): void {
    this.socket?.emit('spin', { room });
  }

  /**
   * Guess a letter
   */
  guess(room: string, letter: string): void {
    this.socket?.emit('guess', { room, letter: letter.toUpperCase() });
  }

  /**
   * Buy a vowel
   */
  buyVowel(room: string, vowel: string): void {
    this.socket?.emit('buy_vowel', { room, vowel: vowel.toUpperCase() });
  }

  /**
   * Attempt to solve the puzzle
   */
  solve(room: string, attempt: string): void {
    this.socket?.emit('solve', { room, attempt });
  }

  /**
   * Buzz in during toss-up
   */
  buzz(room: string): void {
    this.socket?.emit('buzz', { room });
  }

  /**
   * Start a new puzzle
   */
  newPuzzle(room: string): void {
    this.socket?.emit('new_puzzle', { room });
  }

  /**
   * Start a new game (reset scores)
   */
  newGame(room: string): void {
    this.socket?.emit('new_game', { room });
  }

  /**
   * Start toss-up round
   */
  startTossup(room: string, playerIdxs?: number[]): void {
    this.socket?.emit('start_tossup', { room, player_idxs: playerIdxs });
  }

  /**
   * End toss-up round
   */
  endTossup(room: string): void {
    this.socket?.emit('end_tossup', { room });
  }

  /**
   * Start final round
   */
  startFinal(room: string, playerIdx?: number): void {
    this.socket?.emit('start_final', { room, player_idx: playerIdx });
  }

  /**
   * End final round
   */
  endFinal(room: string): void {
    this.socket?.emit('end_final', { room });
  }

  /**
   * Pick a letter in final round
   */
  finalPick(room: string, letter: string): void {
    this.socket?.emit('final_pick', { room, letter: letter.toUpperCase() });
  }

  /**
   * Reveal all letters (host only)
   */
  revealAll(room: string): void {
    this.socket?.emit('reveal_all', { room });
  }

  /**
   * Set active player (host only)
   */
  setActivePlayer(room: string, playerIdx: number): void {
    this.socket?.emit('set_active_player', { room, idx: playerIdx });
  }

  /**
   * Make mystery wedge choice (keep $1,000 or flip for $10,000/Bankrupt)
   */
  mysteryChoice(room: string, choice: 'keep' | 'flip'): void {
    this.socket?.emit('mystery_choice', { room, choice });
  }

  /**
   * Use a wild card to guess another consonant
   */
  useWildCard(room: string, letter: string): void {
    this.socket?.emit('use_wild_card', { room, letter: letter.toUpperCase() });
  }

  /**
   * Check if connected
   */
  isConnected(): boolean {
    return this.socket?.connected ?? false;
  }
}

// Singleton instance
export const socketService = new SocketService();
