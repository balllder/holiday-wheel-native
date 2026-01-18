import React, { useEffect, useRef, useState, useCallback } from 'react';
import { View, Text, StyleSheet, Animated, ViewStyle } from 'react-native';
import { MODERN_THEME } from '../constants';

export type GamePhase = 'normal' | 'tossup' | 'final' | 'solved';

export interface PhaseTransitionProps {
  /** Current game phase */
  phase: GamePhase;
  /** Previous phase (for transition direction) */
  previousPhase?: GamePhase | null;
  /** Category text for phase banners */
  category?: string;
  /** Whether transition animation is enabled */
  enabled?: boolean;
  /** Duration of transition animation in ms */
  duration?: number;
  /** Callback when transition animation starts */
  onTransitionStart?: (from: GamePhase | null, to: GamePhase) => void;
  /** Callback when transition animation completes */
  onTransitionComplete?: (phase: GamePhase) => void;
  /** Custom style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

type TransitionType = 'slide' | 'fade' | 'flash' | 'none';

const PHASE_CONFIG: Record<GamePhase, { label: string; color: string; transition: TransitionType }> = {
  normal: {
    label: 'SPIN THE WHEEL',
    color: MODERN_THEME.colors.primary,
    transition: 'fade',
  },
  tossup: {
    label: 'TOSS-UP ROUND',
    color: MODERN_THEME.colors.danger,
    transition: 'slide',
  },
  final: {
    label: 'FINAL ROUND',
    color: MODERN_THEME.colors.accent,
    transition: 'fade',
  },
  solved: {
    label: 'PUZZLE SOLVED!',
    color: MODERN_THEME.colors.success,
    transition: 'flash',
  },
};

/**
 * PhaseTransition - Animated game phase transition component
 *
 * Features:
 * - Slide-in banners for toss-up
 * - Fade transitions for final round
 * - Flash effect for puzzle solved
 * - Customizable timing and callbacks
 *
 * @example
 * <PhaseTransition
 *   phase={currentPhase}
 *   previousPhase={prevPhase}
 *   onTransitionComplete={() => setReady(true)}
 * />
 */
export function PhaseTransition({
  phase,
  previousPhase = null,
  category,
  enabled = true,
  duration = 500,
  onTransitionStart,
  onTransitionComplete,
  style,
  testID,
}: PhaseTransitionProps): React.JSX.Element | null {
  const [isAnimating, setIsAnimating] = useState(false);
  const [showBanner, setShowBanner] = useState(false);
  const [currentLabel, setCurrentLabel] = useState('');

  const slideAnim = useRef(new Animated.Value(-300)).current;
  const fadeAnim = useRef(new Animated.Value(0)).current;
  const scaleAnim = useRef(new Animated.Value(0.5)).current;
  const flashAnim = useRef(new Animated.Value(0)).current;

  const config = PHASE_CONFIG[phase];

  // Run transition animation when phase changes
  const runTransition = useCallback(() => {
    if (!enabled) {
      onTransitionComplete?.(phase);
      return;
    }

    setIsAnimating(true);
    setShowBanner(true);
    setCurrentLabel(config.label);
    onTransitionStart?.(previousPhase, phase);

    const transitionType = config.transition;

    switch (transitionType) {
      case 'slide':
        // Slide in from left
        slideAnim.setValue(-300);
        fadeAnim.setValue(1);
        scaleAnim.setValue(1);

        Animated.sequence([
          // Slide in
          Animated.spring(slideAnim, {
            toValue: 0,
            friction: 8,
            tension: 80,
            useNativeDriver: true,
          }),
          // Hold
          Animated.delay(duration),
          // Slide out
          Animated.timing(slideAnim, {
            toValue: 300,
            duration: duration / 2,
            useNativeDriver: true,
          }),
        ]).start(() => {
          setShowBanner(false);
          setIsAnimating(false);
          onTransitionComplete?.(phase);
        });
        break;

      case 'fade':
        // Fade in and out
        slideAnim.setValue(0);
        fadeAnim.setValue(0);
        scaleAnim.setValue(0.8);

        Animated.sequence([
          // Fade in with scale
          Animated.parallel([
            Animated.timing(fadeAnim, {
              toValue: 1,
              duration: duration / 3,
              useNativeDriver: true,
            }),
            Animated.spring(scaleAnim, {
              toValue: 1,
              friction: 6,
              tension: 100,
              useNativeDriver: true,
            }),
          ]),
          // Hold
          Animated.delay(duration),
          // Fade out
          Animated.timing(fadeAnim, {
            toValue: 0,
            duration: duration / 3,
            useNativeDriver: true,
          }),
        ]).start(() => {
          setShowBanner(false);
          setIsAnimating(false);
          onTransitionComplete?.(phase);
        });
        break;

      case 'flash':
        // Flash effect for solved
        slideAnim.setValue(0);
        fadeAnim.setValue(0);
        scaleAnim.setValue(0.5);
        flashAnim.setValue(0);

        Animated.sequence([
          // Flash white
          Animated.timing(flashAnim, {
            toValue: 1,
            duration: 100,
            useNativeDriver: true,
          }),
          // Flash fade
          Animated.timing(flashAnim, {
            toValue: 0,
            duration: 200,
            useNativeDriver: true,
          }),
          // Show banner with pop
          Animated.parallel([
            Animated.timing(fadeAnim, {
              toValue: 1,
              duration: 150,
              useNativeDriver: true,
            }),
            Animated.spring(scaleAnim, {
              toValue: 1.1,
              friction: 4,
              tension: 150,
              useNativeDriver: true,
            }),
          ]),
          // Settle scale
          Animated.spring(scaleAnim, {
            toValue: 1,
            friction: 6,
            tension: 100,
            useNativeDriver: true,
          }),
          // Hold
          Animated.delay(duration * 1.5),
          // Fade out
          Animated.timing(fadeAnim, {
            toValue: 0,
            duration: duration / 2,
            useNativeDriver: true,
          }),
        ]).start(() => {
          setShowBanner(false);
          setIsAnimating(false);
          onTransitionComplete?.(phase);
        });
        break;

      default:
        // No animation
        setShowBanner(false);
        setIsAnimating(false);
        onTransitionComplete?.(phase);
    }
  }, [
    enabled,
    phase,
    previousPhase,
    config,
    duration,
    slideAnim,
    fadeAnim,
    scaleAnim,
    flashAnim,
    onTransitionStart,
    onTransitionComplete,
  ]);

  // Trigger animation when phase changes
  useEffect(() => {
    if (previousPhase !== null && previousPhase !== phase) {
      runTransition();
    }
  }, [phase, previousPhase, runTransition]);

  if (!showBanner && !isAnimating) {
    return null;
  }

  const bannerStyle = [
    styles.banner,
    {
      backgroundColor: config.color,
      transform: [
        { translateX: slideAnim },
        { scale: scaleAnim },
      ],
      opacity: fadeAnim,
    },
  ];

  return (
    <View style={[styles.container, style]} pointerEvents="none" testID={testID}>
      {/* Flash overlay for solved state */}
      {config.transition === 'flash' && (
        <Animated.View
          style={[
            styles.flashOverlay,
            { opacity: flashAnim },
          ]}
        />
      )}

      {/* Phase banner */}
      <Animated.View style={bannerStyle}>
        <Text style={styles.bannerText}>{currentLabel}</Text>
        {category && phase !== 'solved' && (
          <Text style={styles.categoryText}>{category}</Text>
        )}
      </Animated.View>
    </View>
  );
}

/**
 * Hook to manage phase transitions
 *
 * @example
 * const { currentPhase, previousPhase, setPhase, isTransitioning } = usePhaseTransition('normal');
 *
 * // In effect when game state changes
 * setPhase(gameStore.phase);
 */
export function usePhaseTransition(initialPhase: GamePhase = 'normal') {
  const [currentPhase, setCurrentPhase] = useState<GamePhase>(initialPhase);
  const [previousPhase, setPreviousPhase] = useState<GamePhase | null>(null);
  const [isTransitioning, setIsTransitioning] = useState(false);

  const setPhase = useCallback((newPhase: GamePhase) => {
    if (newPhase !== currentPhase) {
      setPreviousPhase(currentPhase);
      setCurrentPhase(newPhase);
      setIsTransitioning(true);
    }
  }, [currentPhase]);

  const completeTransition = useCallback(() => {
    setIsTransitioning(false);
  }, []);

  return {
    currentPhase,
    previousPhase,
    setPhase,
    isTransitioning,
    completeTransition,
  };
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    justifyContent: 'center',
    alignItems: 'center',
    zIndex: MODERN_THEME.zIndex.overlay,
  },
  flashOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: '#ffffff',
  },
  banner: {
    paddingVertical: MODERN_THEME.spacing[6],
    paddingHorizontal: MODERN_THEME.spacing[10],
    borderRadius: MODERN_THEME.borderRadius.lg,
    alignItems: 'center',
    ...MODERN_THEME.shadows.large,
  },
  bannerText: {
    color: MODERN_THEME.colors.text,
    fontSize: MODERN_THEME.typography.fontSize['4xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    letterSpacing: MODERN_THEME.typography.letterSpacing.wider,
    textShadowColor: 'rgba(0, 0, 0, 0.5)',
    textShadowOffset: { width: 2, height: 2 },
    textShadowRadius: 4,
  },
  categoryText: {
    color: MODERN_THEME.colors.text,
    fontSize: MODERN_THEME.typography.fontSize.xl,
    fontWeight: MODERN_THEME.typography.fontWeight.medium,
    marginTop: MODERN_THEME.spacing[2],
    opacity: 0.9,
  },
});

export default PhaseTransition;
