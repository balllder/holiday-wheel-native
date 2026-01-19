import { useCallback, useEffect, useRef, useState } from 'react';
import { useGameStore } from '../stores/gameStore';
import type { ConfettiVariant, ConfettiRef } from '../components/Confetti';

/**
 * Configuration options for useConfetti hook
 */
export interface UseConfettiOptions {
  /** Auto-trigger on puzzle solve (default: true) */
  autoTriggerOnSolve?: boolean;
  /** Auto-trigger on round win (default: true) */
  autoTriggerOnRoundWin?: boolean;
  /** Auto-trigger on game win (default: true) */
  autoTriggerOnGameWin?: boolean;
  /** Duration before auto-hide in ms (default: none - waits for animation) */
  autoHideDuration?: number;
  /** Callback when confetti is triggered */
  onTrigger?: (variant: ConfettiVariant) => void;
  /** Callback when confetti animation completes */
  onComplete?: () => void;
}

/**
 * Return type for useConfetti hook
 */
export interface UseConfettiReturn {
  /** Whether confetti is currently showing */
  isActive: boolean;
  /** Current variant being displayed */
  currentVariant: ConfettiVariant | null;
  /** Trigger confetti with specified variant */
  trigger: (variant?: ConfettiVariant) => void;
  /** Stop confetti animation */
  stop: () => void;
  /** Ref to attach to Confetti component for imperative control */
  confettiRef: React.RefObject<ConfettiRef | null>;
  /** Props to spread on Confetti component */
  confettiProps: {
    active: boolean;
    variant: ConfettiVariant;
    onComplete: () => void;
  };
}

/**
 * Hook to manage confetti celebrations with game state integration
 *
 * Features:
 * - Auto-triggers confetti on puzzle solve, round win, and game win
 * - Provides imperative trigger method for manual control
 * - Automatically selects appropriate variant based on event type
 * - Integrates with game store to detect celebration events
 *
 * @example
 * // Basic usage with auto-trigger
 * const { confettiProps } = useConfetti();
 * return <Confetti {...confettiProps} />;
 *
 * @example
 * // Manual trigger
 * const { trigger, confettiProps } = useConfetti({ autoTriggerOnSolve: false });
 * const handleWin = () => trigger('gameWin');
 * return <Confetti {...confettiProps} />;
 *
 * @example
 * // With ref for imperative control
 * const { confettiRef, confettiProps } = useConfetti();
 * const handleCustomEvent = () => confettiRef.current?.trigger('roundWin');
 * return <Confetti ref={confettiRef} {...confettiProps} />;
 */
export function useConfetti(options: UseConfettiOptions = {}): UseConfettiReturn {
  const {
    autoTriggerOnSolve = true,
    autoTriggerOnRoundWin = true,
    autoTriggerOnGameWin = true,
    autoHideDuration,
    onTrigger,
    onComplete,
  } = options;

  const [isActive, setIsActive] = useState(false);
  const [currentVariant, setCurrentVariant] = useState<ConfettiVariant | null>(null);
  const confettiRef = useRef<ConfettiRef | null>(null);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Track previous game state for detecting changes
  const prevRevealedCountRef = useRef<number>(0);
  const prevPhaseRef = useRef<string>('normal');
  const prevRoundRef = useRef<number>(0);

  // Game state selectors
  const revealed = useGameStore((state) => state.revealed);
  const phase = useGameStore((state) => state.phase);
  const puzzle = useGameStore((state) => state.puzzle);
  const round = useGameStore((state) => state.round);

  // Trigger confetti
  const trigger = useCallback(
    (variant: ConfettiVariant = 'solve') => {
      // Clear any pending auto-hide timeout
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
        timeoutRef.current = null;
      }

      setCurrentVariant(variant);
      setIsActive(true);
      onTrigger?.(variant);

      // Use ref for imperative trigger if available
      confettiRef.current?.trigger(variant);

      // Auto-hide after duration if specified
      if (autoHideDuration) {
        timeoutRef.current = setTimeout(() => {
          setIsActive(false);
          setCurrentVariant(null);
        }, autoHideDuration);
      }
    },
    [autoHideDuration, onTrigger]
  );

  // Stop confetti
  const stop = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
      timeoutRef.current = null;
    }
    setIsActive(false);
    setCurrentVariant(null);
    confettiRef.current?.stop();
  }, []);

  // Handle animation complete
  const handleComplete = useCallback(() => {
    setIsActive(false);
    setCurrentVariant(null);
    onComplete?.();
  }, [onComplete]);

  // Detect puzzle solve by checking if all letters are revealed
  useEffect(() => {
    if (!autoTriggerOnSolve) return;

    const answer = puzzle.answer?.toUpperCase() || '';
    const letterSet = new Set(answer.replace(/[^A-Z]/g, '').split(''));
    const allRevealed = letterSet.size > 0 && [...letterSet].every((l) => revealed.has(l));
    const prevCount = prevRevealedCountRef.current;
    const currentCount = revealed.size;

    // Trigger if all letters just became revealed (and we have letters)
    if (allRevealed && currentCount > prevCount && prevCount > 0) {
      trigger('solve');
    }

    prevRevealedCountRef.current = currentCount;
  }, [revealed, puzzle.answer, autoTriggerOnSolve, trigger]);

  // Detect round win (phase changes and round increments)
  useEffect(() => {
    if (!autoTriggerOnRoundWin) return;

    const currentRound = round.current_round;
    const prevRound = prevRoundRef.current;

    // Trigger if round just incremented (not on initial load)
    if (currentRound > prevRound && prevRound > 0) {
      trigger('roundWin');
    }

    prevRoundRef.current = currentRound;
  }, [round.current_round, autoTriggerOnRoundWin, trigger]);

  // Detect game win (final round completed)
  useEffect(() => {
    if (!autoTriggerOnGameWin) return;

    const prevPhase = prevPhaseRef.current;
    const currentPhase = phase;

    // Detect transition from final to a win state
    // This is a simplified detection - you may need to adjust based on your game logic
    if (
      prevPhase === 'final' &&
      currentPhase === 'normal' &&
      round.current_round === round.total_rounds &&
      round.total_rounds > 0
    ) {
      trigger('gameWin');
    }

    prevPhaseRef.current = currentPhase;
  }, [phase, round, autoTriggerOnGameWin, trigger]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    isActive,
    currentVariant,
    trigger,
    stop,
    confettiRef,
    confettiProps: {
      active: isActive,
      variant: currentVariant ?? 'solve',
      onComplete: handleComplete,
    },
  };
}

/**
 * Simplified hook for basic confetti state management without game integration
 *
 * @example
 * const { showConfetti, triggerConfetti, hideConfetti } = useConfettiSimple();
 *
 * return (
 *   <>
 *     <Button onPress={() => triggerConfetti('gameWin')} title="Celebrate!" />
 *     <Confetti active={showConfetti} variant="gameWin" onComplete={hideConfetti} />
 *   </>
 * );
 */
export function useConfettiSimple(autoHideDuration?: number) {
  const [showConfetti, setShowConfetti] = useState(false);
  const [variant, setVariant] = useState<ConfettiVariant>('solve');
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const triggerConfetti = useCallback(
    (v: ConfettiVariant = 'solve') => {
      setVariant(v);
      setShowConfetti(true);

      if (autoHideDuration) {
        if (timeoutRef.current) {
          clearTimeout(timeoutRef.current);
        }
        timeoutRef.current = setTimeout(() => {
          setShowConfetti(false);
        }, autoHideDuration);
      }
    },
    [autoHideDuration]
  );

  const hideConfetti = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setShowConfetti(false);
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    showConfetti,
    variant,
    triggerConfetti,
    hideConfetti,
  };
}

export default useConfetti;
