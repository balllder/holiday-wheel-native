import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated, ViewStyle } from 'react-native';
import { MODERN_THEME } from '../constants';

export interface ExpressModeIndicatorProps {
  /** Whether express mode is active */
  active: boolean;
  /** Number of correct guesses in express mode */
  correctCount: number;
  /** Value per consonant (default $1,000) */
  valuePerConsonant: number;
  /** Player name in express mode */
  playerName?: string;
  /** Custom style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * ExpressModeIndicator - Shows express mode status and progress
 *
 * Features:
 * - Pulsing glow animation when active
 * - Running total of express earnings
 * - Count of correct guesses
 *
 * @example
 * <ExpressModeIndicator
 *   active={isExpressActive}
 *   correctCount={3}
 *   valuePerConsonant={1000}
 *   playerName="John"
 * />
 */
export function ExpressModeIndicator({
  active,
  correctCount,
  valuePerConsonant,
  playerName,
  style,
  testID,
}: ExpressModeIndicatorProps): React.JSX.Element | null {
  const pulseAnim = useRef(new Animated.Value(1)).current;
  const glowAnim = useRef(new Animated.Value(0.5)).current;
  const slideAnim = useRef(new Animated.Value(-100)).current;

  // Slide in when activated
  useEffect(() => {
    if (active) {
      Animated.spring(slideAnim, {
        toValue: 0,
        friction: 8,
        tension: 80,
        useNativeDriver: true,
      }).start();
    } else {
      Animated.timing(slideAnim, {
        toValue: -100,
        duration: 200,
        useNativeDriver: true,
      }).start();
    }
  }, [active, slideAnim]);

  // Pulsing animation when active
  useEffect(() => {
    if (active) {
      const pulse = Animated.loop(
        Animated.parallel([
          Animated.sequence([
            Animated.timing(pulseAnim, {
              toValue: 1.02,
              duration: 600,
              useNativeDriver: true,
            }),
            Animated.timing(pulseAnim, {
              toValue: 1,
              duration: 600,
              useNativeDriver: true,
            }),
          ]),
          Animated.sequence([
            Animated.timing(glowAnim, {
              toValue: 1,
              duration: 600,
              useNativeDriver: true,
            }),
            Animated.timing(glowAnim, {
              toValue: 0.5,
              duration: 600,
              useNativeDriver: true,
            }),
          ]),
        ])
      );
      pulse.start();
      return () => pulse.stop();
    }
  }, [active, pulseAnim, glowAnim]);

  if (!active) {
    return null;
  }

  const totalEarnings = correctCount * valuePerConsonant;

  return (
    <Animated.View
      style={[
        styles.container,
        {
          transform: [
            { translateY: slideAnim },
            { scale: pulseAnim },
          ],
          shadowOpacity: glowAnim,
        },
        style,
      ]}
      testID={testID}
    >
      <View style={styles.header}>
        <Text style={styles.expressLabel}>EXPRESS MODE</Text>
        {playerName && (
          <Text style={styles.playerName}>{playerName}</Text>
        )}
      </View>

      <View style={styles.statsContainer}>
        <View style={styles.stat}>
          <Text style={styles.statValue}>{correctCount}</Text>
          <Text style={styles.statLabel}>CORRECT</Text>
        </View>

        <View style={styles.divider} />

        <View style={styles.stat}>
          <Text style={styles.statValue}>
            ${totalEarnings.toLocaleString()}
          </Text>
          <Text style={styles.statLabel}>EARNED</Text>
        </View>
      </View>

      <Text style={styles.warningText}>
        One wrong guess = BANKRUPT!
      </Text>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: MODERN_THEME.colors.danger,
    borderRadius: MODERN_THEME.borderRadius.lg,
    padding: MODERN_THEME.spacing[4],
    ...MODERN_THEME.shadows.large,
    shadowColor: MODERN_THEME.colors.danger,
    shadowRadius: 20,
    borderWidth: 2,
    borderColor: MODERN_THEME.colors.text,
  },
  header: {
    flexDirection: 'row',
    justifyContent: 'space-between',
    alignItems: 'center',
    marginBottom: MODERN_THEME.spacing[3],
  },
  expressLabel: {
    fontSize: MODERN_THEME.typography.fontSize.lg,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    letterSpacing: MODERN_THEME.typography.letterSpacing.wider,
  },
  playerName: {
    fontSize: MODERN_THEME.typography.fontSize.base,
    color: MODERN_THEME.colors.text,
    opacity: 0.9,
  },
  statsContainer: {
    flexDirection: 'row',
    justifyContent: 'space-around',
    alignItems: 'center',
    marginBottom: MODERN_THEME.spacing[3],
  },
  stat: {
    alignItems: 'center',
    flex: 1,
  },
  statValue: {
    fontSize: MODERN_THEME.typography.fontSize['2xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
  },
  statLabel: {
    fontSize: MODERN_THEME.typography.fontSize.xs,
    color: MODERN_THEME.colors.text,
    opacity: 0.8,
    marginTop: MODERN_THEME.spacing[1],
  },
  divider: {
    width: 1,
    height: 40,
    backgroundColor: MODERN_THEME.colors.text,
    opacity: 0.3,
  },
  warningText: {
    fontSize: MODERN_THEME.typography.fontSize.sm,
    color: MODERN_THEME.colors.text,
    textAlign: 'center',
    fontStyle: 'italic',
    opacity: 0.9,
  },
});

export default ExpressModeIndicator;
