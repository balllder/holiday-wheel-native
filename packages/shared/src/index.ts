// Types
export * from './types';

// Constants
export * from './constants';

// Components
export { AnimatedWheel } from './components/AnimatedWheel';
export { AnimatedScore, useScoreAnimation } from './components/AnimatedScore';
export type { AnimatedScoreProps } from './components/AnimatedScore';
export { AnimatedButton, SpinButton, BuzzButton } from './components/AnimatedButton';
export type { AnimatedButtonProps } from './components/AnimatedButton';
export { LetterCell } from './components/LetterCell';
export type { LetterCellProps, LetterState } from './components/LetterCell';
export { PuzzleBoard } from './components/PuzzleBoard';
export type { PuzzleBoardProps } from './components/PuzzleBoard';
export { Confetti, useConfetti } from './components/Confetti';
export type { ConfettiProps } from './components/Confetti';
export { PhaseTransition, usePhaseTransition } from './components/PhaseTransition';
export type { PhaseTransitionProps, GamePhase } from './components/PhaseTransition';

// Stores
export {
  useGameStore,
  selectIsMyTurn,
  selectIsTossupController,
  selectIsFinalActive,
  selectCanBuzz,
  selectActivePlayer,
  selectMyPlayer,
} from './stores/gameStore';

export {
  useAuthStore,
  selectIsAuthenticated,
} from './stores/authStore';

// Services
export { socketService } from './services/socketService';
export { authService } from './services/authService';
export { configService } from './services/configService';
export { passkeyService } from './services/passkeyService';
export { oauthService } from './services/oauthService';
export {
  soundService,
  SOUND_FILES,
  type SoundType,
  type SoundConfig,
  type AudioProvider,
} from './services/soundService';
