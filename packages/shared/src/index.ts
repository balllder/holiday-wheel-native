// Types
export * from './types';

// Constants
export * from './constants';

// UI Components (Modern Theme)
export {
  GradientBackground,
  SimpleGradient,
  GlassCard,
  GlassCardHeader,
  GlassCardContent,
  GlassCardFooter,
  GoldButton,
} from './components/ui';
export type {
  GradientBackgroundProps,
  GlassCardProps,
  GoldButtonProps,
} from './components/ui';

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
export { Confetti } from './components/Confetti';
export type { ConfettiProps, ConfettiRef, ConfettiVariant } from './components/Confetti';

// Hooks
export { useConfetti, useConfettiSimple } from './hooks/useConfetti';
export type { UseConfettiOptions, UseConfettiReturn } from './hooks/useConfetti';
export { PhaseTransition, usePhaseTransition } from './components/PhaseTransition';
export type { PhaseTransitionProps, GamePhase } from './components/PhaseTransition';
export { MysteryWedgeModal } from './components/MysteryWedgeModal';
export type { MysteryWedgeModalProps } from './components/MysteryWedgeModal';
export { ExpressModeIndicator } from './components/ExpressModeIndicator';
export type { ExpressModeIndicatorProps } from './components/ExpressModeIndicator';
export { WildCardButton } from './components/WildCardButton';
export type { WildCardButtonProps } from './components/WildCardButton';
export { RoundProgressIndicator } from './components/RoundProgressIndicator';
export type { RoundProgressIndicatorProps } from './components/RoundProgressIndicator';
export { TossupValueDisplay } from './components/TossupValueDisplay';
export type { TossupValueDisplayProps } from './components/TossupValueDisplay';

// Stores
export {
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
  selectIsMultiRoundEnabled,
  selectCurrentRound,
  selectTotalRounds,
  selectCurrentRoundConfig,
  selectRoundProgress,
  selectIsTripleTossup,
  selectTripleTossupIndex,
  selectCurrentTossupValue,
  selectTossupRevealDelay,
  selectIsAutoReveal,
  selectNextRevealIndex,
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
