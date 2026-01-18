import { create } from 'zustand';
import type {
  ServerGameState,
  Player,
  Puzzle,
  WedgeValue,
  GamePhase,
  TossupState,
  FinalState,
  MysteryState,
  ExpressState,
} from '../types';

interface GameStore {
  // Connection state
  room: string;
  connected: boolean;

  // Game state (from server)
  phase: GamePhase;
  puzzle: Puzzle;
  revealed: Set<string>;
  usedLetters: Set<string>;
  players: Player[];
  activeIdx: number;
  wheelSlots: WedgeValue[];
  wheelIndex: number | null;
  lastSpinIndex: number | null;
  currentWedge: WedgeValue | null;
  tossup: TossupState;
  final: FinalState;

  // New mechanics state
  mystery: MysteryState;
  express: ExpressState;

  // Player state
  myPlayerIdx: number | null;
  isHost: boolean;

  // Actions
  setRoom: (room: string) => void;
  setConnected: (connected: boolean) => void;
  setMyPlayerIdx: (idx: number | null) => void;
  setIsHost: (isHost: boolean) => void;
  updateFromServer: (state: ServerGameState) => void;
  reset: () => void;
}

const initialState = {
  room: 'main',
  connected: false,
  phase: 'normal' as GamePhase,
  puzzle: { id: 0, category: '', answer: '' },
  revealed: new Set<string>(),
  usedLetters: new Set<string>(),
  players: [],
  activeIdx: 0,
  wheelSlots: [],
  wheelIndex: null,
  lastSpinIndex: null,
  currentWedge: null,
  tossup: {
    controller_player_idx: null,
    locked_player_idxs: [],
    allowed_player_idxs: [],
    remaining_seconds: null,
  },
  final: {
    stage: 'off' as const,
    picks_consonants: [],
    pick_vowel: null,
    remaining_seconds: null,
  },
  mystery: {
    stage: 'off' as const,
    player_idx: null,
    choice: null,
    flip_result: null,
  },
  express: {
    active: false,
    player_idx: null,
    correct_count: 0,
    value_per_consonant: 1000,
  },
  myPlayerIdx: null,
  isHost: false,
};

export const useGameStore = create<GameStore>((set) => ({
  ...initialState,

  setRoom: (room) => set({ room }),

  setConnected: (connected) => set({ connected }),

  setMyPlayerIdx: (idx) => set({ myPlayerIdx: idx }),

  setIsHost: (isHost) => set({ isHost }),

  updateFromServer: (state) =>
    set({
      phase: state.phase,
      puzzle: state.puzzle,
      revealed: new Set(state.revealed),
      usedLetters: new Set(state.used),
      players: state.players,
      activeIdx: state.active_idx,
      wheelSlots: state.wheel_slots,
      wheelIndex: state.wheel_index,
      lastSpinIndex: state.last_spin_index,
      currentWedge: state.current_wedge,
      tossup: state.tossup,
      final: state.final,
      mystery: state.mystery ?? {
        stage: 'off',
        player_idx: null,
        choice: null,
        flip_result: null,
      },
      express: state.express ?? {
        active: false,
        player_idx: null,
        correct_count: 0,
        value_per_consonant: 1000,
      },
    }),

  reset: () => set(initialState),
}));

// Selectors
export const selectIsMyTurn = (state: GameStore): boolean => {
  if (state.phase !== 'normal') return false;
  return state.activeIdx === state.myPlayerIdx;
};

export const selectIsTossupController = (state: GameStore): boolean => {
  if (state.phase !== 'tossup') return false;
  return state.tossup.controller_player_idx === state.myPlayerIdx;
};

export const selectIsFinalActive = (state: GameStore): boolean => {
  if (state.phase !== 'final') return false;
  return state.activeIdx === state.myPlayerIdx;
};

export const selectCanBuzz = (state: GameStore): boolean => {
  if (state.phase !== 'tossup') return false;
  if (state.myPlayerIdx === null) return false;
  if (state.tossup.locked_player_idxs.includes(state.myPlayerIdx)) return false;
  if (
    state.tossup.allowed_player_idxs.length > 0 &&
    !state.tossup.allowed_player_idxs.includes(state.myPlayerIdx)
  ) {
    return false;
  }
  return true;
};

export const selectActivePlayer = (state: GameStore): Player | null => {
  return state.players[state.activeIdx] ?? null;
};

export const selectMyPlayer = (state: GameStore): Player | null => {
  if (state.myPlayerIdx === null) return null;
  return state.players[state.myPlayerIdx] ?? null;
};

// Mystery wedge selectors
export const selectIsMysteryAwaitingChoice = (state: GameStore): boolean => {
  return (
    state.mystery.stage === 'awaiting_choice' &&
    state.mystery.player_idx === state.myPlayerIdx
  );
};

export const selectIsMysteryRevealing = (state: GameStore): boolean => {
  return state.mystery.stage === 'revealing';
};

// Express mode selectors
export const selectIsExpressActive = (state: GameStore): boolean => {
  return state.express.active && state.express.player_idx === state.myPlayerIdx;
};

export const selectExpressCorrectCount = (state: GameStore): number => {
  return state.express.correct_count;
};

// Wild card selectors
export const selectMyWildCards = (state: GameStore): number => {
  if (state.myPlayerIdx === null) return 0;
  const player = state.players[state.myPlayerIdx];
  return player?.wild_cards ?? 0;
};

export const selectCanUseWildCard = (state: GameStore): boolean => {
  if (state.phase !== 'normal') return false;
  if (state.activeIdx !== state.myPlayerIdx) return false;
  return selectMyWildCards(state) > 0;
};
