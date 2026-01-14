import { create } from 'zustand';
import type {
  ServerGameState,
  Player,
  Puzzle,
  WedgeValue,
  GamePhase,
  TossupState,
  FinalState,
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
