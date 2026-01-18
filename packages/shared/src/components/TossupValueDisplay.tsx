import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated, ViewStyle } from 'react-native';
import { MODERN_THEME } from '../constants';

export interface TossupValueDisplayProps {
  /** Current toss-up value in dollars */
  value: number;
  /** Whether this is a triple toss-up */
  isTriple: boolean;
  /** Current toss-up index in triple (0-2) */
  tripleIndex: number;
  /** Whether to show the display (toss-up phase active) */
  visible: boolean;
  /** Custom style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * TossupValueDisplay - Shows toss-up round value and triple status
 *
 * Features:
 * - Animated value display
 * - Triple toss-up progress indicator
 * - Pulse animation on value change
 *
 * @example
 * <TossupValueDisplay
 *   value={2000}
 *   isTriple={true}
 *   tripleIndex={1}
 *   visible={true}
 * />
 */
export function TossupValueDisplay({
  value,
  isTriple,
  tripleIndex,
  visible,
  style,
  testID,
}: TossupValueDisplayProps): React.JSX.Element | null {
  const scaleAnim = useRef(new Animated.Value(1)).current;
  const opacityAnim = useRef(new Animated.Value(0)).current;

  // Fade in/out based on visibility
  useEffect(() => {
    Animated.timing(opacityAnim, {
      toValue: visible ? 1 : 0,
      duration: 300,
      useNativeDriver: true,
    }).start();
  }, [visible, opacityAnim]);

  // Pulse animation when value changes
  useEffect(() => {
    if (visible) {
      Animated.sequence([
        Animated.timing(scaleAnim, {
          toValue: 1.15,
          duration: 150,
          useNativeDriver: true,
        }),
        Animated.spring(scaleAnim, {
          toValue: 1,
          friction: 4,
          tension: 100,
          useNativeDriver: true,
        }),
      ]).start();
    }
  }, [value, visible, scaleAnim]);

  if (!visible) {
    return null;
  }

  const formattedValue = `$${value.toLocaleString()}`;

  return (
    <Animated.View
      style={[
        styles.container,
        {
          opacity: opacityAnim,
          transform: [{ scale: scaleAnim }],
        },
        style,
      ]}
      testID={testID}
    >
      {isTriple && (
        <View style={styles.tripleHeader}>
          <Text style={styles.tripleLabel}>TRIPLE TOSS-UP</Text>
          <View style={styles.tripleIndicators}>
            {[0, 1, 2].map((idx) => (
              <View
                key={idx}
                style={[
                  styles.tripleIndicator,
                  idx < tripleIndex && styles.tripleIndicatorCompleted,
                  idx === tripleIndex && styles.tripleIndicatorCurrent,
                ]}
              />
            ))}
          </View>
        </View>
      )}

      <View style={styles.valueContainer}>
        <Text style={styles.forLabel}>FOR</Text>
        <Text style={styles.valueText}>{formattedValue}</Text>
      </View>
    </Animated.View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: MODERN_THEME.colors.danger,
    borderRadius: MODERN_THEME.borderRadius.lg,
    padding: MODERN_THEME.spacing[4],
    alignItems: 'center',
    ...MODERN_THEME.shadows.large,
    borderWidth: 2,
    borderColor: MODERN_THEME.colors.gold,
  },
  tripleHeader: {
    alignItems: 'center',
    marginBottom: MODERN_THEME.spacing[2],
  },
  tripleLabel: {
    fontSize: MODERN_THEME.typography.fontSize.sm,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    letterSpacing: MODERN_THEME.typography.letterSpacing.wider,
    marginBottom: MODERN_THEME.spacing[1],
  },
  tripleIndicators: {
    flexDirection: 'row',
    gap: MODERN_THEME.spacing[2],
  },
  tripleIndicator: {
    width: 16,
    height: 16,
    borderRadius: 8,
    backgroundColor: 'rgba(255, 255, 255, 0.3)',
    borderWidth: 2,
    borderColor: MODERN_THEME.colors.text,
  },
  tripleIndicatorCompleted: {
    backgroundColor: MODERN_THEME.colors.success,
    borderColor: MODERN_THEME.colors.success,
  },
  tripleIndicatorCurrent: {
    backgroundColor: MODERN_THEME.colors.gold,
    borderColor: MODERN_THEME.colors.gold,
  },
  valueContainer: {
    alignItems: 'center',
  },
  forLabel: {
    fontSize: MODERN_THEME.typography.fontSize.sm,
    color: MODERN_THEME.colors.text,
    opacity: 0.8,
  },
  valueText: {
    fontSize: MODERN_THEME.typography.fontSize['3xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    textShadowColor: 'rgba(0, 0, 0, 0.5)',
    textShadowOffset: { width: 2, height: 2 },
    textShadowRadius: 4,
  },
});

export default TossupValueDisplay;
