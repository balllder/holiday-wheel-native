import { socketService } from '../socketService';
import { useGameStore } from '../../stores/gameStore';

// Mock socket.io-client
const mockSocket = {
  connected: false,
  on: jest.fn(),
  emit: jest.fn(),
  disconnect: jest.fn(),
};

jest.mock('socket.io-client', () => ({
  io: jest.fn(() => mockSocket),
}));

// Import the mocked io
import { io } from 'socket.io-client';

describe('socketService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    mockSocket.connected = false;
    mockSocket.on.mockReset();
    mockSocket.emit.mockReset();
    mockSocket.disconnect.mockReset();
    useGameStore.getState().reset();
    // Reset the socket service internal state
    socketService.disconnect();
  });

  describe('connect', () => {
    it('creates a socket connection with correct options', () => {
      socketService.connect('http://localhost:5000', 'my-token');

      expect(io).toHaveBeenCalledWith('http://localhost:5000', {
        transports: ['websocket', 'polling'],
        auth: { token: 'my-token' },
        reconnection: true,
        reconnectionAttempts: 10,
        reconnectionDelay: 1000,
      });
    });

    it('connects without token when not provided', () => {
      socketService.connect('http://localhost:5000');

      expect(io).toHaveBeenCalledWith('http://localhost:5000', {
        transports: ['websocket', 'polling'],
        auth: undefined,
        reconnection: true,
        reconnectionAttempts: 10,
        reconnectionDelay: 1000,
      });
    });

    it('sets up event listeners after connecting', () => {
      socketService.connect('http://localhost:5000');

      expect(mockSocket.on).toHaveBeenCalledWith('connect', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('disconnect', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('connect_error', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('state', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('you', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('host_granted', expect.any(Function));
      expect(mockSocket.on).toHaveBeenCalledWith('toast', expect.any(Function));
    });

    it('does not reconnect if already connected', () => {
      mockSocket.connected = true;
      socketService.connect('http://localhost:5000');
      socketService.connect('http://localhost:5000'); // Second call

      // io should only be called once
      expect(io).toHaveBeenCalledTimes(1);
    });
  });

  describe('disconnect', () => {
    it('disconnects the socket', () => {
      socketService.connect('http://localhost:5000');
      socketService.disconnect();

      expect(mockSocket.disconnect).toHaveBeenCalled();
    });

    it('sets connected state to false', () => {
      socketService.connect('http://localhost:5000');
      useGameStore.getState().setConnected(true);
      socketService.disconnect();

      expect(useGameStore.getState().connected).toBe(false);
    });
  });

  describe('event handlers', () => {
    let eventHandlers: Record<string, Function>;

    beforeEach(() => {
      eventHandlers = {};
      mockSocket.on.mockImplementation((event, handler) => {
        eventHandlers[event] = handler;
      });
      socketService.connect('http://localhost:5000');
    });

    it('handles connect event', () => {
      eventHandlers['connect']();

      expect(useGameStore.getState().connected).toBe(true);
    });

    it('handles disconnect event', () => {
      useGameStore.getState().setConnected(true);
      eventHandlers['disconnect']();

      expect(useGameStore.getState().connected).toBe(false);
    });

    it('handles state event', () => {
      const serverState = {
        phase: 'normal',
        puzzle: { id: 1, category: 'TEST', answer: 'HELLO' },
        revealed: ['H', 'E'],
        used: ['H', 'E', 'X'],
        players: [{ id: 1, name: 'Player', total: 100, prizes: [], round_bank: 0, round_prizes: [], claimed_sid: null, claimed_user_id: null }],
        active_idx: 0,
        wheel_slots: [500, 600],
        wheel_index: 1,
        last_spin_index: 0,
        current_wedge: 500,
        packs: [],
        active_pack_id: null,
        active_pack_name: null,
        db: { used: 0, total: 10, unused: 10 },
        host: { claimed: false, player_idx: null },
        tossup: { controller_player_idx: null, locked_player_idxs: [], allowed_player_idxs: [], remaining_seconds: null },
        final: { stage: 'off', picks_consonants: [], pick_vowel: null, remaining_seconds: null },
      };

      eventHandlers['state'](serverState);

      const state = useGameStore.getState();
      expect(state.phase).toBe('normal');
      expect(state.puzzle.answer).toBe('HELLO');
      expect(state.revealed).toEqual(new Set(['H', 'E']));
    });

    it('handles you event', () => {
      eventHandlers['you']({ player_idx: 2 });

      expect(useGameStore.getState().myPlayerIdx).toBe(2);
    });

    it('handles you event with null', () => {
      useGameStore.getState().setMyPlayerIdx(1);
      eventHandlers['you']({ player_idx: null });

      expect(useGameStore.getState().myPlayerIdx).toBeNull();
    });

    it('handles host_granted event', () => {
      eventHandlers['host_granted']({ granted: true });

      expect(useGameStore.getState().isHost).toBe(true);
    });

    it('handles toast event with callback', () => {
      const toastCallback = jest.fn();
      socketService.setToastCallback(toastCallback);
      socketService.connect('http://localhost:5000');

      // Re-get event handlers since we reconnected
      eventHandlers = {};
      mockSocket.on.mock.calls.forEach(([event, handler]) => {
        eventHandlers[event] = handler;
      });

      eventHandlers['toast']({ msg: 'Hello World' });

      expect(toastCallback).toHaveBeenCalledWith('Hello World');
    });
  });

  describe('emit methods', () => {
    beforeEach(() => {
      socketService.connect('http://localhost:5000');
    });

    describe('joinRoom', () => {
      it('emits join event and updates store', () => {
        socketService.joinRoom('test-room');

        expect(useGameStore.getState().room).toBe('test-room');
        expect(mockSocket.emit).toHaveBeenCalledWith('join', { room: 'test-room' });
      });
    });

    describe('joinGame', () => {
      it('emits join_game event', () => {
        socketService.joinGame('game-room');

        expect(mockSocket.emit).toHaveBeenCalledWith('join_game', { room: 'game-room' });
      });
    });

    describe('leaveGame', () => {
      it('emits leave_game event', () => {
        socketService.leaveGame('game-room');

        expect(mockSocket.emit).toHaveBeenCalledWith('leave_game', { room: 'game-room' });
      });
    });

    describe('claimHost', () => {
      it('emits claim_host event with code', () => {
        socketService.claimHost('room', 'secret123');

        expect(mockSocket.emit).toHaveBeenCalledWith('claim_host', { room: 'room', code: 'secret123' });
      });
    });

    describe('releaseHost', () => {
      it('emits release_host event', () => {
        socketService.releaseHost('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('release_host', { room: 'room' });
      });
    });

    describe('spin', () => {
      it('emits spin event', () => {
        socketService.spin('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('spin', { room: 'room' });
      });
    });

    describe('guess', () => {
      it('emits guess event with uppercase letter', () => {
        socketService.guess('room', 't');

        expect(mockSocket.emit).toHaveBeenCalledWith('guess', { room: 'room', letter: 'T' });
      });

      it('handles already uppercase letter', () => {
        socketService.guess('room', 'R');

        expect(mockSocket.emit).toHaveBeenCalledWith('guess', { room: 'room', letter: 'R' });
      });
    });

    describe('buyVowel', () => {
      it('emits buy_vowel event with uppercase vowel', () => {
        socketService.buyVowel('room', 'a');

        expect(mockSocket.emit).toHaveBeenCalledWith('buy_vowel', { room: 'room', vowel: 'A' });
      });
    });

    describe('solve', () => {
      it('emits solve event with attempt', () => {
        socketService.solve('room', 'Hello World');

        expect(mockSocket.emit).toHaveBeenCalledWith('solve', { room: 'room', attempt: 'Hello World' });
      });
    });

    describe('buzz', () => {
      it('emits buzz event', () => {
        socketService.buzz('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('buzz', { room: 'room' });
      });
    });

    describe('newPuzzle', () => {
      it('emits new_puzzle event', () => {
        socketService.newPuzzle('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('new_puzzle', { room: 'room' });
      });
    });

    describe('newGame', () => {
      it('emits new_game event', () => {
        socketService.newGame('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('new_game', { room: 'room' });
      });
    });

    describe('startTossup', () => {
      it('emits start_tossup event without player indexes', () => {
        socketService.startTossup('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('start_tossup', { room: 'room', player_idxs: undefined });
      });

      it('emits start_tossup event with player indexes', () => {
        socketService.startTossup('room', [0, 2]);

        expect(mockSocket.emit).toHaveBeenCalledWith('start_tossup', { room: 'room', player_idxs: [0, 2] });
      });
    });

    describe('endTossup', () => {
      it('emits end_tossup event', () => {
        socketService.endTossup('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('end_tossup', { room: 'room' });
      });
    });

    describe('startFinal', () => {
      it('emits start_final event without player index', () => {
        socketService.startFinal('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('start_final', { room: 'room', player_idx: undefined });
      });

      it('emits start_final event with player index', () => {
        socketService.startFinal('room', 1);

        expect(mockSocket.emit).toHaveBeenCalledWith('start_final', { room: 'room', player_idx: 1 });
      });
    });

    describe('endFinal', () => {
      it('emits end_final event', () => {
        socketService.endFinal('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('end_final', { room: 'room' });
      });
    });

    describe('finalPick', () => {
      it('emits final_pick event with uppercase letter', () => {
        socketService.finalPick('room', 's');

        expect(mockSocket.emit).toHaveBeenCalledWith('final_pick', { room: 'room', letter: 'S' });
      });
    });

    describe('revealAll', () => {
      it('emits reveal_all event', () => {
        socketService.revealAll('room');

        expect(mockSocket.emit).toHaveBeenCalledWith('reveal_all', { room: 'room' });
      });
    });

    describe('setActivePlayer', () => {
      it('emits set_active_player event', () => {
        socketService.setActivePlayer('room', 2);

        expect(mockSocket.emit).toHaveBeenCalledWith('set_active_player', { room: 'room', idx: 2 });
      });
    });
  });

  describe('isConnected', () => {
    it('returns false when not connected', () => {
      expect(socketService.isConnected()).toBe(false);
    });

    it('returns true when connected', () => {
      socketService.connect('http://localhost:5000');
      mockSocket.connected = true;

      expect(socketService.isConnected()).toBe(true);
    });
  });

  describe('setToastCallback', () => {
    it('sets the toast callback', () => {
      const callback = jest.fn();
      socketService.setToastCallback(callback);

      // Verify callback is called when toast event fires
      let toastHandler: Function;
      mockSocket.on.mockImplementation((event, handler) => {
        if (event === 'toast') {
          toastHandler = handler;
        }
      });

      socketService.connect('http://localhost:5000');
      toastHandler!({ msg: 'Test toast' });

      expect(callback).toHaveBeenCalledWith('Test toast');
    });
  });
});
