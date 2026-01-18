import React, { useEffect, useRef } from 'react';
import {
  View,
  Text,
  Modal,
  StyleSheet,
  Animated,
  Pressable,
  ViewStyle,
} from 'react-native';
import { MODERN_THEME } from '../constants';

export interface MysteryWedgeModalProps {
  /** Whether the modal is visible */
  visible: boolean;
  /** Current stage of mystery interaction */
  stage: 'awaiting_choice' | 'revealing' | 'off';
  /** The choice made (for revealing stage) */
  choice?: 'keep' | 'flip' | null;
  /** Result of flip (for revealing stage) */
  flipResult?: boolean | null;
  /** Callback when player chooses to keep $1,000 */
  onKeep: () => void;
  /** Callback when player chooses to flip */
  onFlip: () => void;
  /** Callback when reveal animation completes */
  onRevealComplete?: () => void;
  /** Custom style for the modal content */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * MysteryWedgeModal - Modal for mystery wedge choice and reveal
 *
 * Features:
 * - Choice between $1,000 (keep) or flip for $10,000/Bankrupt
 * - Dramatic reveal animation for flip result
 * - Pulsing glow effects
 *
 * @example
 * <MysteryWedgeModal
 *   visible={isMysteryActive}
 *   stage="awaiting_choice"
 *   onKeep={() => socketService.mysteryChoice(room, 'keep')}
 *   onFlip={() => socketService.mysteryChoice(room, 'flip')}
 * />
 */
export function MysteryWedgeModal({
  visible,
  stage,
  choice,
  flipResult,
  onKeep,
  onFlip,
  onRevealComplete,
  style,
  testID,
}: MysteryWedgeModalProps): React.JSX.Element {
  const pulseAnim = useRef(new Animated.Value(1)).current;
  const revealAnim = useRef(new Animated.Value(0)).current;
  const flipRotation = useRef(new Animated.Value(0)).current;

  // Pulse animation for the mystery card
  useEffect(() => {
    if (visible && stage === 'awaiting_choice') {
      const pulse = Animated.loop(
        Animated.sequence([
          Animated.timing(pulseAnim, {
            toValue: 1.05,
            duration: 800,
            useNativeDriver: true,
          }),
          Animated.timing(pulseAnim, {
            toValue: 1,
            duration: 800,
            useNativeDriver: true,
          }),
        ])
      );
      pulse.start();
      return () => pulse.stop();
    }
  }, [visible, stage, pulseAnim]);

  // Reveal animation when choice is made
  useEffect(() => {
    if (stage === 'revealing' && choice === 'flip') {
      // Flip card animation
      Animated.sequence([
        Animated.timing(flipRotation, {
          toValue: 1,
          duration: 400,
          useNativeDriver: true,
        }),
        Animated.timing(revealAnim, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
        Animated.delay(1500),
      ]).start(() => {
        onRevealComplete?.();
      });
    } else if (stage === 'revealing' && choice === 'keep') {
      // Simple fade for keep
      Animated.sequence([
        Animated.timing(revealAnim, {
          toValue: 1,
          duration: 300,
          useNativeDriver: true,
        }),
        Animated.delay(1000),
      ]).start(() => {
        onRevealComplete?.();
      });
    }
  }, [stage, choice, flipRotation, revealAnim, onRevealComplete]);

  // Reset animations when modal closes
  useEffect(() => {
    if (!visible) {
      revealAnim.setValue(0);
      flipRotation.setValue(0);
    }
  }, [visible, revealAnim, flipRotation]);

  const flipInterpolate = flipRotation.interpolate({
    inputRange: [0, 0.5, 1],
    outputRange: ['0deg', '90deg', '180deg'],
  });

  const renderChoiceContent = () => (
    <Animated.View
      style={[styles.card, { transform: [{ scale: pulseAnim }] }]}
      testID={testID ? `${testID}-choice-card` : undefined}
    >
      <Text style={styles.title}>MYSTERY WEDGE</Text>
      <Text style={styles.subtitle}>Make your choice!</Text>

      <View style={styles.optionsContainer}>
        <Pressable
          style={({ pressed }) => [
            styles.optionButton,
            styles.keepButton,
            pressed && styles.buttonPressed,
          ]}
          onPress={onKeep}
          testID={testID ? `${testID}-keep-button` : undefined}
        >
          <Text style={styles.optionAmount}>$1,000</Text>
          <Text style={styles.optionLabel}>KEEP IT</Text>
        </Pressable>

        <Text style={styles.orText}>OR</Text>

        <Pressable
          style={({ pressed }) => [
            styles.optionButton,
            styles.flipButton,
            pressed && styles.buttonPressed,
          ]}
          onPress={onFlip}
          testID={testID ? `${testID}-flip-button` : undefined}
        >
          <Text style={styles.optionAmount}>FLIP</Text>
          <Text style={styles.optionLabel}>$10,000 or BANKRUPT</Text>
        </Pressable>
      </View>
    </Animated.View>
  );

  const renderRevealContent = () => {
    const isWin = flipResult === true;
    const resultText = choice === 'keep'
      ? '$1,000'
      : isWin
        ? '$10,000!'
        : 'BANKRUPT';
    const resultColor = choice === 'keep' || isWin
      ? MODERN_THEME.colors.success
      : MODERN_THEME.colors.danger;

    return (
      <Animated.View
        style={[
          styles.card,
          styles.revealCard,
          {
            transform: [{ rotateY: flipInterpolate }],
            opacity: revealAnim.interpolate({
              inputRange: [0, 1],
              outputRange: [0.5, 1],
            }),
          },
        ]}
        testID={testID ? `${testID}-reveal-card` : undefined}
      >
        <Text style={styles.revealTitle}>
          {choice === 'keep' ? 'YOU KEPT' : 'YOU GOT'}
        </Text>
        <Text style={[styles.revealResult, { color: resultColor }]}>
          {resultText}
        </Text>
      </Animated.View>
    );
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="fade"
      testID={testID}
    >
      <View style={[styles.overlay, style]}>
        {stage === 'awaiting_choice' && renderChoiceContent()}
        {stage === 'revealing' && renderRevealContent()}
      </View>
    </Modal>
  );
}

const styles = StyleSheet.create({
  overlay: {
    flex: 1,
    backgroundColor: 'rgba(0, 0, 0, 0.85)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  card: {
    backgroundColor: MODERN_THEME.colors.surface,
    borderRadius: MODERN_THEME.borderRadius.xl,
    padding: MODERN_THEME.spacing[8],
    alignItems: 'center',
    minWidth: 300,
    ...MODERN_THEME.shadows.large,
    borderWidth: 3,
    borderColor: MODERN_THEME.colors.accent,
  },
  revealCard: {
    backfaceVisibility: 'hidden',
  },
  title: {
    fontSize: MODERN_THEME.typography.fontSize['3xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.accent,
    marginBottom: MODERN_THEME.spacing[2],
    letterSpacing: MODERN_THEME.typography.letterSpacing.wider,
  },
  subtitle: {
    fontSize: MODERN_THEME.typography.fontSize.lg,
    color: MODERN_THEME.colors.textMuted,
    marginBottom: MODERN_THEME.spacing[6],
  },
  optionsContainer: {
    alignItems: 'center',
    width: '100%',
  },
  optionButton: {
    paddingVertical: MODERN_THEME.spacing[4],
    paddingHorizontal: MODERN_THEME.spacing[8],
    borderRadius: MODERN_THEME.borderRadius.lg,
    alignItems: 'center',
    minWidth: 200,
    ...MODERN_THEME.shadows.medium,
  },
  keepButton: {
    backgroundColor: MODERN_THEME.colors.success,
  },
  flipButton: {
    backgroundColor: MODERN_THEME.colors.danger,
  },
  buttonPressed: {
    transform: [{ scale: 0.95 }],
    opacity: 0.9,
  },
  optionAmount: {
    fontSize: MODERN_THEME.typography.fontSize['2xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
  },
  optionLabel: {
    fontSize: MODERN_THEME.typography.fontSize.sm,
    color: MODERN_THEME.colors.text,
    opacity: 0.9,
    marginTop: MODERN_THEME.spacing[1],
  },
  orText: {
    fontSize: MODERN_THEME.typography.fontSize.xl,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.textMuted,
    marginVertical: MODERN_THEME.spacing[4],
  },
  revealTitle: {
    fontSize: MODERN_THEME.typography.fontSize.xl,
    color: MODERN_THEME.colors.textMuted,
    marginBottom: MODERN_THEME.spacing[4],
  },
  revealResult: {
    fontSize: MODERN_THEME.typography.fontSize['4xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    letterSpacing: MODERN_THEME.typography.letterSpacing.wider,
  },
});

export default MysteryWedgeModal;
