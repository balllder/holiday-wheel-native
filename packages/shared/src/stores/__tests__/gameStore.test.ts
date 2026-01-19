import {
  useGameStore,
  selectIsMyTurn,
  selectIsTossupController,
  selectIsFinalActive,
  selectCanBuzz,
  selectActivePlayer,
  selectMyPlayer,
  selectIsMysteryAwaitingChoice,
  selectIsMysteryRevealing,
  selectIsExpressActive,
  selectExpressCorrectCount,
  selectMyWildCards,
  selectCanUseWildCard,
} from '../gameStore';
import type { ServerGameState, Player } from '../../types';

// Helper to create a player
const createPlayer = (overrides: Partial<Player> = {}): Player => ({
  id: 1,
  name: 'Player 1',
  total: 0,
  prizes: [],
  round_bank: 0,
  round_prizes: [],
  claimed_sid: null,
  claimed_user_id: null,
  ...overrides,
});

// Helper to create server game state
const createServerState = (
  overrides: Partial<ServerGameState> = {}
): ServerGameState => ({
  phase: 'normal',
  puzzle: { id: 1, category: 'Test', answer: 'HELLO WORLD' },
  revealed: [],
  used: [],
  players: [],
  active_idx: 0,
  wheel_slots: [500, 600, 700, 'BANKRUPT', 'LOSE A TURN'],
  wheel_index: null,
  last_spin_index: null,
  current_wedge: null,
  packs: [],
  active_pack_id: null,
  active_pack_name: null,
  db: { used: 0, total: 100, unused: 100 },
  host: { claimed: false, player_idx: null },
  tossup: {
    controller_player_idx: null,
    locked_player_idxs: [],
    allowed_player_idxs: [],
    remaining_seconds: null,
  },
  final: {
    stage: 'off',
    picks_consonants: [],
    pick_vowel: null,
    remaining_seconds: null,
  },
  ...overrides,
});

describe('gameStore', () => {
  beforeEach(() => {
    // Reset the store before each test
    useGameStore.getState().reset();
  });

  describe('actions', () => {
    describe('setRoom', () => {
      it('updates the room name', () => {
        useGameStore.getState().setRoom('test-room');
        expect(useGameStore.getState().room).toBe('test-room');
      });
    });

    describe('setConnected', () => {
      it('sets connected to true', () => {
        useGameStore.getState().setConnected(true);
        expect(useGameStore.getState().connected).toBe(true);
      });

      it('sets connected to false', () => {
        useGameStore.getState().setConnected(true);
        useGameStore.getState().setConnected(false);
        expect(useGameStore.getState().connected).toBe(false);
      });
    });

    describe('setMyPlayerIdx', () => {
      it('sets player index', () => {
        useGameStore.getState().setMyPlayerIdx(2);
        expect(useGameStore.getState().myPlayerIdx).toBe(2);
      });

      it('sets player index to null', () => {
        useGameStore.getState().setMyPlayerIdx(2);
        useGameStore.getState().setMyPlayerIdx(null);
        expect(useGameStore.getState().myPlayerIdx).toBeNull();
      });
    });

    describe('setIsHost', () => {
      it('sets isHost to true', () => {
        useGameStore.getState().setIsHost(true);
        expect(useGameStore.getState().isHost).toBe(true);
      });

      it('sets isHost to false', () => {
        useGameStore.getState().setIsHost(true);
        useGameStore.getState().setIsHost(false);
        expect(useGameStore.getState().isHost).toBe(false);
      });
    });

    describe('updateFromServer', () => {
      it('updates all game state from server', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', total: 1000 }),
          createPlayer({ id: 2, name: 'Bob', total: 500 }),
        ];

        const serverState = createServerState({
          phase: 'normal',
          puzzle: { id: 5, category: 'PHRASE', answer: 'TESTING' },
          revealed: ['T', 'E'],
          used: ['T', 'E', 'R'],
          players,
          active_idx: 1,
          wheel_index: 3,
          last_spin_index: 2,
          current_wedge: 500,
        });

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.phase).toBe('normal');
        expect(state.puzzle.answer).toBe('TESTING');
        expect(state.revealed).toEqual(new Set(['T', 'E']));
        expect(state.usedLetters).toEqual(new Set(['T', 'E', 'R']));
        expect(state.players).toHaveLength(2);
        expect(state.activeIdx).toBe(1);
        expect(state.wheelIndex).toBe(3);
        expect(state.lastSpinIndex).toBe(2);
        expect(state.currentWedge).toBe(500);
      });

      it('updates tossup state', () => {
        const serverState = createServerState({
          phase: 'tossup',
          tossup: {
            controller_player_idx: 1,
            locked_player_idxs: [0],
            allowed_player_idxs: [1, 2],
            remaining_seconds: 15,
          },
        });

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.phase).toBe('tossup');
        expect(state.tossup.controller_player_idx).toBe(1);
        expect(state.tossup.locked_player_idxs).toEqual([0]);
        expect(state.tossup.allowed_player_idxs).toEqual([1, 2]);
        expect(state.tossup.remaining_seconds).toBe(15);
      });

      it('updates final round state', () => {
        const serverState = createServerState({
          phase: 'final',
          final: {
            stage: 'pick',
            picks_consonants: ['R', 'S', 'T'],
            pick_vowel: 'E',
            remaining_seconds: 30,
          },
        });

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.phase).toBe('final');
        expect(state.final.stage).toBe('pick');
        expect(state.final.picks_consonants).toEqual(['R', 'S', 'T']);
        expect(state.final.pick_vowel).toBe('E');
        expect(state.final.remaining_seconds).toBe(30);
      });
    });

    describe('reset', () => {
      it('resets all state to initial values', () => {
        // Set some state
        useGameStore.getState().setRoom('test-room');
        useGameStore.getState().setConnected(true);
        useGameStore.getState().setMyPlayerIdx(1);
        useGameStore.getState().setIsHost(true);
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            players: [createPlayer()],
          })
        );

        // Reset
        useGameStore.getState().reset();
        const state = useGameStore.getState();

        expect(state.room).toBe('main');
        expect(state.connected).toBe(false);
        expect(state.myPlayerIdx).toBeNull();
        expect(state.isHost).toBe(false);
        expect(state.phase).toBe('normal');
        expect(state.players).toHaveLength(0);
      });
    });
  });

  describe('selectors', () => {
    describe('selectIsMyTurn', () => {
      it('returns true when phase is normal and activeIdx matches myPlayerIdx', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMyTurn(useGameStore.getState())).toBe(true);
      });

      it('returns false when phase is normal but activeIdx does not match', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            active_idx: 0,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMyTurn(useGameStore.getState())).toBe(false);
      });

      it('returns false when phase is tossup', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMyTurn(useGameStore.getState())).toBe(false);
      });

      it('returns false when phase is final', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'final',
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMyTurn(useGameStore.getState())).toBe(false);
      });

      it('returns false when myPlayerIdx is null', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            active_idx: 0,
          })
        );

        expect(selectIsMyTurn(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectIsTossupController', () => {
      it('returns true when in tossup phase and player is controller', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: 2,
              locked_player_idxs: [],
              allowed_player_idxs: [],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(2);

        expect(selectIsTossupController(useGameStore.getState())).toBe(true);
      });

      it('returns false when in tossup phase but not controller', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: 2,
              locked_player_idxs: [],
              allowed_player_idxs: [],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsTossupController(useGameStore.getState())).toBe(false);
      });

      it('returns false when not in tossup phase', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
          })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectIsTossupController(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectIsFinalActive', () => {
      it('returns true when in final phase and player is active', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'final',
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsFinalActive(useGameStore.getState())).toBe(true);
      });

      it('returns false when in final phase but not active', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'final',
            active_idx: 0,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsFinalActive(useGameStore.getState())).toBe(false);
      });

      it('returns false when not in final phase', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsFinalActive(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectCanBuzz', () => {
      it('returns true when in tossup and player is allowed', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: null,
              locked_player_idxs: [],
              allowed_player_idxs: [],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectCanBuzz(useGameStore.getState())).toBe(true);
      });

      it('returns false when player is locked out', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: null,
              locked_player_idxs: [1],
              allowed_player_idxs: [],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectCanBuzz(useGameStore.getState())).toBe(false);
      });

      it('returns false when allowed list exists and player not in it', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: null,
              locked_player_idxs: [],
              allowed_player_idxs: [0, 2],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectCanBuzz(useGameStore.getState())).toBe(false);
      });

      it('returns true when allowed list exists and player is in it', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            tossup: {
              controller_player_idx: null,
              locked_player_idxs: [],
              allowed_player_idxs: [0, 1, 2],
              remaining_seconds: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectCanBuzz(useGameStore.getState())).toBe(true);
      });

      it('returns false when not in tossup phase', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectCanBuzz(useGameStore.getState())).toBe(false);
      });

      it('returns false when myPlayerIdx is null', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
          })
        );

        expect(selectCanBuzz(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectActivePlayer', () => {
      it('returns the active player', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice' }),
          createPlayer({ id: 2, name: 'Bob' }),
          createPlayer({ id: 3, name: 'Charlie' }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({
            players,
            active_idx: 1,
          })
        );

        const activePlayer = selectActivePlayer(useGameStore.getState());
        expect(activePlayer).not.toBeNull();
        expect(activePlayer?.name).toBe('Bob');
      });

      it('returns null when no players', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            players: [],
            active_idx: 0,
          })
        );

        expect(selectActivePlayer(useGameStore.getState())).toBeNull();
      });

      it('returns null when activeIdx is out of bounds', () => {
        const players = [createPlayer({ id: 1, name: 'Alice' })];

        useGameStore.getState().updateFromServer(
          createServerState({
            players,
            active_idx: 5,
          })
        );

        expect(selectActivePlayer(useGameStore.getState())).toBeNull();
      });
    });

    describe('selectMyPlayer', () => {
      it('returns the current player', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', total: 1000 }),
          createPlayer({ id: 2, name: 'Bob', total: 500 }),
          createPlayer({ id: 3, name: 'Charlie', total: 750 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );
        useGameStore.getState().setMyPlayerIdx(2);

        const myPlayer = selectMyPlayer(useGameStore.getState());
        expect(myPlayer).not.toBeNull();
        expect(myPlayer?.name).toBe('Charlie');
        expect(myPlayer?.total).toBe(750);
      });

      it('returns null when myPlayerIdx is null', () => {
        const players = [createPlayer({ id: 1, name: 'Alice' })];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );

        expect(selectMyPlayer(useGameStore.getState())).toBeNull();
      });

      it('returns null when myPlayerIdx is out of bounds', () => {
        const players = [createPlayer({ id: 1, name: 'Alice' })];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );
        useGameStore.getState().setMyPlayerIdx(5);

        expect(selectMyPlayer(useGameStore.getState())).toBeNull();
      });
    });
  });

  describe('initial state', () => {
    it('has correct default values', () => {
      const state = useGameStore.getState();

      expect(state.room).toBe('main');
      expect(state.connected).toBe(false);
      expect(state.phase).toBe('normal');
      expect(state.puzzle).toEqual({ id: 0, category: '', answer: '' });
      expect(state.revealed).toEqual(new Set());
      expect(state.usedLetters).toEqual(new Set());
      expect(state.players).toEqual([]);
      expect(state.activeIdx).toBe(0);
      expect(state.wheelSlots).toEqual([]);
      expect(state.wheelIndex).toBeNull();
      expect(state.lastSpinIndex).toBeNull();
      expect(state.currentWedge).toBeNull();
      expect(state.myPlayerIdx).toBeNull();
      expect(state.isHost).toBe(false);
    });

    it('has correct default tossup state', () => {
      const state = useGameStore.getState();

      expect(state.tossup).toEqual({
        controller_player_idx: null,
        locked_player_idxs: [],
        allowed_player_idxs: [],
        remaining_seconds: null,
      });
    });

    it('has correct default final state', () => {
      const state = useGameStore.getState();

      expect(state.final).toEqual({
        stage: 'off',
        picks_consonants: [],
        pick_vowel: null,
        remaining_seconds: null,
      });
    });

    it('has correct default mystery state', () => {
      const state = useGameStore.getState();

      expect(state.mystery).toEqual({
        stage: 'off',
        player_idx: null,
        choice: null,
        flip_result: null,
      });
    });

    it('has correct default express state', () => {
      const state = useGameStore.getState();

      expect(state.express).toEqual({
        active: false,
        player_idx: null,
        correct_count: 0,
        value_per_consonant: 1000,
      });
    });
  });

  describe('mystery and express state', () => {
    describe('updateFromServer with mystery', () => {
      it('updates mystery state from server', () => {
        const serverState = createServerState({
          mystery: {
            stage: 'awaiting_choice',
            player_idx: 1,
            choice: null,
            flip_result: null,
          },
        });

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.mystery.stage).toBe('awaiting_choice');
        expect(state.mystery.player_idx).toBe(1);
      });

      it('uses default mystery state when server state is missing mystery', () => {
        const serverState = createServerState({});
        // Explicitly remove mystery to simulate old server
        delete (serverState as Partial<ServerGameState>).mystery;

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.mystery).toEqual({
          stage: 'off',
          player_idx: null,
          choice: null,
          flip_result: null,
        });
      });
    });

    describe('updateFromServer with express', () => {
      it('updates express state from server', () => {
        const serverState = createServerState({
          express: {
            active: true,
            player_idx: 2,
            correct_count: 3,
            value_per_consonant: 1000,
          },
        });

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.express.active).toBe(true);
        expect(state.express.player_idx).toBe(2);
        expect(state.express.correct_count).toBe(3);
      });

      it('uses default express state when server state is missing express', () => {
        const serverState = createServerState({});
        delete (serverState as Partial<ServerGameState>).express;

        useGameStore.getState().updateFromServer(serverState);
        const state = useGameStore.getState();

        expect(state.express).toEqual({
          active: false,
          player_idx: null,
          correct_count: 0,
          value_per_consonant: 1000,
        });
      });
    });
  });

  describe('mystery selectors', () => {
    describe('selectIsMysteryAwaitingChoice', () => {
      it('returns true when mystery is awaiting choice for current player', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            mystery: {
              stage: 'awaiting_choice',
              player_idx: 1,
              choice: null,
              flip_result: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMysteryAwaitingChoice(useGameStore.getState())).toBe(true);
      });

      it('returns false when mystery is awaiting choice for different player', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            mystery: {
              stage: 'awaiting_choice',
              player_idx: 2,
              choice: null,
              flip_result: null,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMysteryAwaitingChoice(useGameStore.getState())).toBe(false);
      });

      it('returns false when mystery stage is not awaiting_choice', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            mystery: {
              stage: 'revealing',
              player_idx: 1,
              choice: 'flip',
              flip_result: true,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsMysteryAwaitingChoice(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectIsMysteryRevealing', () => {
      it('returns true when mystery stage is revealing', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            mystery: {
              stage: 'revealing',
              player_idx: 1,
              choice: 'flip',
              flip_result: true,
            },
          })
        );

        expect(selectIsMysteryRevealing(useGameStore.getState())).toBe(true);
      });

      it('returns false when mystery stage is not revealing', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            mystery: {
              stage: 'off',
              player_idx: null,
              choice: null,
              flip_result: null,
            },
          })
        );

        expect(selectIsMysteryRevealing(useGameStore.getState())).toBe(false);
      });
    });
  });

  describe('express selectors', () => {
    describe('selectIsExpressActive', () => {
      it('returns true when express is active for current player', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            express: {
              active: true,
              player_idx: 1,
              correct_count: 0,
              value_per_consonant: 1000,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsExpressActive(useGameStore.getState())).toBe(true);
      });

      it('returns false when express is active for different player', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            express: {
              active: true,
              player_idx: 2,
              correct_count: 0,
              value_per_consonant: 1000,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsExpressActive(useGameStore.getState())).toBe(false);
      });

      it('returns false when express is not active', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            express: {
              active: false,
              player_idx: null,
              correct_count: 0,
              value_per_consonant: 1000,
            },
          })
        );
        useGameStore.getState().setMyPlayerIdx(1);

        expect(selectIsExpressActive(useGameStore.getState())).toBe(false);
      });
    });

    describe('selectExpressCorrectCount', () => {
      it('returns the correct count', () => {
        useGameStore.getState().updateFromServer(
          createServerState({
            express: {
              active: true,
              player_idx: 1,
              correct_count: 5,
              value_per_consonant: 1000,
            },
          })
        );

        expect(selectExpressCorrectCount(useGameStore.getState())).toBe(5);
      });
    });
  });

  describe('wild card selectors', () => {
    describe('selectMyWildCards', () => {
      it('returns wild card count for current player', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 2 }),
          createPlayer({ id: 2, name: 'Bob', wild_cards: 1 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectMyWildCards(useGameStore.getState())).toBe(2);
      });

      it('returns 0 when myPlayerIdx is null', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 2 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );

        expect(selectMyWildCards(useGameStore.getState())).toBe(0);
      });

      it('returns 0 when player has no wild_cards property', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice' }), // no wild_cards
        ];

        useGameStore.getState().updateFromServer(
          createServerState({ players })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectMyWildCards(useGameStore.getState())).toBe(0);
      });
    });

    describe('selectCanUseWildCard', () => {
      it('returns true when in normal phase, is active player, and has wild cards', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 1 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            players,
            active_idx: 0,
          })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectCanUseWildCard(useGameStore.getState())).toBe(true);
      });

      it('returns false when not in normal phase', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 1 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'tossup',
            players,
            active_idx: 0,
          })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectCanUseWildCard(useGameStore.getState())).toBe(false);
      });

      it('returns false when not active player', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 1 }),
          createPlayer({ id: 2, name: 'Bob', wild_cards: 0 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            players,
            active_idx: 1,
          })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectCanUseWildCard(useGameStore.getState())).toBe(false);
      });

      it('returns false when no wild cards', () => {
        const players = [
          createPlayer({ id: 1, name: 'Alice', wild_cards: 0 }),
        ];

        useGameStore.getState().updateFromServer(
          createServerState({
            phase: 'normal',
            players,
            active_idx: 0,
          })
        );
        useGameStore.getState().setMyPlayerIdx(0);

        expect(selectCanUseWildCard(useGameStore.getState())).toBe(false);
      });
    });
  });
});
