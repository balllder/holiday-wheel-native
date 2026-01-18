import React from 'react';
import { View, Text, StyleSheet, ViewStyle } from 'react-native';
import { MODERN_THEME } from '../constants';
import type { RoundConfig } from '../types';

export interface RoundProgressIndicatorProps {
  /** Current round number (1-indexed) */
  currentRound: number;
  /** Total number of rounds */
  totalRounds: number;
  /** Current round configuration */
  roundConfig?: RoundConfig | null;
  /** Whether multi-round mode is enabled */
  enabled: boolean;
  /** Custom style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * RoundProgressIndicator - Displays multi-round game progress
 *
 * Features:
 * - Shows current round out of total
 * - Displays round type indicator
 * - Value multiplier badge
 *
 * @example
 * <RoundProgressIndicator
 *   currentRound={2}
 *   totalRounds={4}
 *   roundConfig={config}
 *   enabled={true}
 * />
 */
export function RoundProgressIndicator({
  currentRound,
  totalRounds,
  roundConfig,
  enabled,
  style,
  testID,
}: RoundProgressIndicatorProps): React.JSX.Element | null {
  if (!enabled || totalRounds === 0) {
    return null;
  }

  const getRoundTypeLabel = (type: string): string => {
    switch (type) {
      case 'tossup':
        return 'TOSS-UP';
      case 'speed':
        return 'SPEED';
      case 'bonus':
        return 'BONUS';
      default:
        return '';
    }
  };

  const roundTypeLabel = roundConfig ? getRoundTypeLabel(roundConfig.type) : '';
  const showMultiplier = roundConfig && roundConfig.value_multiplier > 1;

  return (
    <View style={[styles.container, style]} testID={testID}>
      <View style={styles.progressContainer}>
        <Text style={styles.roundText}>
          ROUND {currentRound}
        </Text>
        <Text style={styles.ofText}>of {totalRounds}</Text>
      </View>

      <View style={styles.indicatorsContainer}>
        {/* Progress dots */}
        <View style={styles.dotsContainer}>
          {Array.from({ length: totalRounds }, (_, i) => (
            <View
              key={i}
              style={[
                styles.dot,
                i + 1 < currentRound && styles.dotCompleted,
                i + 1 === currentRound && styles.dotCurrent,
              ]}
            />
          ))}
        </View>

        {/* Round type badge */}
        {roundTypeLabel && (
          <View style={styles.typeBadge}>
            <Text style={styles.typeText}>{roundTypeLabel}</Text>
          </View>
        )}

        {/* Value multiplier badge */}
        {showMultiplier && (
          <View style={styles.multiplierBadge}>
            <Text style={styles.multiplierText}>
              {roundConfig.value_multiplier}x
            </Text>
          </View>
        )}
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: MODERN_THEME.colors.surface,
    borderRadius: MODERN_THEME.borderRadius.lg,
    padding: MODERN_THEME.spacing[3],
    ...MODERN_THEME.shadows.small,
  },
  progressContainer: {
    flexDirection: 'row',
    alignItems: 'baseline',
    justifyContent: 'center',
    marginBottom: MODERN_THEME.spacing[2],
  },
  roundText: {
    fontSize: MODERN_THEME.typography.fontSize.lg,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    letterSpacing: MODERN_THEME.typography.letterSpacing.wide,
  },
  ofText: {
    fontSize: MODERN_THEME.typography.fontSize.sm,
    color: MODERN_THEME.colors.textMuted,
    marginLeft: MODERN_THEME.spacing[2],
  },
  indicatorsContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    gap: MODERN_THEME.spacing[2],
  },
  dotsContainer: {
    flexDirection: 'row',
    gap: MODERN_THEME.spacing[1],
  },
  dot: {
    width: 8,
    height: 8,
    borderRadius: 4,
    backgroundColor: MODERN_THEME.colors.border,
  },
  dotCompleted: {
    backgroundColor: MODERN_THEME.colors.success,
  },
  dotCurrent: {
    backgroundColor: MODERN_THEME.colors.primary,
    width: 12,
    height: 12,
    borderRadius: 6,
  },
  typeBadge: {
    backgroundColor: MODERN_THEME.colors.accent,
    paddingHorizontal: MODERN_THEME.spacing[2],
    paddingVertical: MODERN_THEME.spacing[1],
    borderRadius: MODERN_THEME.borderRadius.sm,
  },
  typeText: {
    fontSize: MODERN_THEME.typography.fontSize.xs,
    fontWeight: MODERN_THEME.typography.fontWeight.semibold,
    color: MODERN_THEME.colors.text,
  },
  multiplierBadge: {
    backgroundColor: MODERN_THEME.colors.gold,
    paddingHorizontal: MODERN_THEME.spacing[2],
    paddingVertical: MODERN_THEME.spacing[1],
    borderRadius: MODERN_THEME.borderRadius.sm,
  },
  multiplierText: {
    fontSize: MODERN_THEME.typography.fontSize.xs,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.background,
  },
});

export default RoundProgressIndicator;
