import { io, Socket } from 'socket.io-client';
import { useGameStore } from '../stores/gameStore';
import type { ServerGameState } from '../types';

type ToastCallback = (message: string) => void;

class SocketService {
  private socket: Socket | null = null;
  private onToast: ToastCallback | null = null;

  /**
   * Connect to the Socket.IO server
   */
  connect(baseUrl: string, token?: string): void {
    if (this.socket?.connected) {
      return;
    }

    this.socket = io(baseUrl, {
      transports: ['websocket', 'polling'],
      auth: token ? { token } : undefined,
      reconnection: true,
      reconnectionAttempts: 10,
      reconnectionDelay: 1000,
    });

    this.setupListeners();
  }

  /**
   * Disconnect from the server
   */
  disconnect(): void {
    this.socket?.disconnect();
    this.socket = null;
    useGameStore.getState().setConnected(false);
  }

  /**
   * Set up Socket.IO event listeners
   */
  private setupListeners(): void {
    if (!this.socket) return;

    this.socket.on('connect', () => {
      console.log('[Socket] Connected');
      useGameStore.getState().setConnected(true);
    });

    this.socket.on('disconnect', () => {
      console.log('[Socket] Disconnected');
      useGameStore.getState().setConnected(false);
    });

    this.socket.on('connect_error', (error) => {
      console.error('[Socket] Connection error:', error);
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
   * Check if connected
   */
  isConnected(): boolean {
    return this.socket?.connected ?? false;
  }
}

// Singleton instance
export const socketService = new SocketService();
