import React, { useEffect, useRef } from 'react';
import { Text, StyleSheet, Animated, Pressable, ViewStyle } from 'react-native';
import { MODERN_THEME } from '../constants';

export interface WildCardButtonProps {
  /** Number of wild cards the player has */
  count: number;
  /** Whether the button is enabled (player's turn, normal phase) */
  enabled: boolean;
  /** Callback when button is pressed */
  onPress: () => void;
  /** Custom style for the button */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * WildCardButton - Button to use a wild card token
 *
 * Features:
 * - Shows count of available wild cards
 * - Shimmer animation when available
 * - Disabled state when not player's turn
 *
 * @example
 * <WildCardButton
 *   count={2}
 *   enabled={isMyTurn}
 *   onPress={() => setShowLetterPicker(true)}
 * />
 */
export function WildCardButton({
  count,
  enabled,
  onPress,
  style,
  testID,
}: WildCardButtonProps): React.JSX.Element | null {
  const shimmerAnim = useRef(new Animated.Value(0)).current;
  const scaleAnim = useRef(new Animated.Value(1)).current;

  // Shimmer animation when enabled and has wild cards
  useEffect(() => {
    if (enabled && count > 0) {
      const shimmer = Animated.loop(
        Animated.sequence([
          Animated.timing(shimmerAnim, {
            toValue: 1,
            duration: 1500,
            useNativeDriver: true,
          }),
          Animated.timing(shimmerAnim, {
            toValue: 0,
            duration: 1500,
            useNativeDriver: true,
          }),
        ])
      );
      shimmer.start();
      return () => shimmer.stop();
    }
  }, [enabled, count, shimmerAnim]);

  const handlePressIn = () => {
    if (enabled && count > 0) {
      Animated.spring(scaleAnim, {
        toValue: 0.95,
        friction: 8,
        tension: 100,
        useNativeDriver: true,
      }).start();
    }
  };

  const handlePressOut = () => {
    Animated.spring(scaleAnim, {
      toValue: 1,
      friction: 8,
      tension: 100,
      useNativeDriver: true,
    }).start();
  };

  // Don't render if no wild cards
  if (count === 0) {
    return null;
  }

  const isActive = enabled && count > 0;

  const shimmerOpacity = shimmerAnim.interpolate({
    inputRange: [0, 0.5, 1],
    outputRange: [0.3, 0.7, 0.3],
  });

  return (
    <Pressable
      onPress={isActive ? onPress : undefined}
      onPressIn={handlePressIn}
      onPressOut={handlePressOut}
      testID={testID}
    >
      <Animated.View
        style={[
          styles.container,
          !isActive && styles.disabled,
          {
            transform: [{ scale: scaleAnim }],
          },
          style,
        ]}
      >
        {/* Shimmer overlay */}
        {isActive && (
          <Animated.View
            style={[
              styles.shimmer,
              { opacity: shimmerOpacity },
            ]}
          />
        )}

        <Text style={styles.icon}>W</Text>
        <Text style={styles.label}>WILD</Text>
        <Text style={styles.count}>{count}</Text>
      </Animated.View>
    </Pressable>
  );
}

const styles = StyleSheet.create({
  container: {
    backgroundColor: MODERN_THEME.colors.accent,
    borderRadius: MODERN_THEME.borderRadius.lg,
    padding: MODERN_THEME.spacing[3],
    alignItems: 'center',
    minWidth: 80,
    ...MODERN_THEME.shadows.medium,
    borderWidth: 2,
    borderColor: MODERN_THEME.colors.gold,
    overflow: 'hidden',
  },
  disabled: {
    opacity: 0.5,
  },
  shimmer: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: MODERN_THEME.colors.gold,
    borderRadius: MODERN_THEME.borderRadius.lg,
  },
  icon: {
    fontSize: MODERN_THEME.typography.fontSize['3xl'],
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    textShadowColor: 'rgba(0, 0, 0, 0.5)',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 2,
  },
  label: {
    fontSize: MODERN_THEME.typography.fontSize.xs,
    fontWeight: MODERN_THEME.typography.fontWeight.semibold,
    color: MODERN_THEME.colors.text,
    marginTop: MODERN_THEME.spacing[1],
    letterSpacing: MODERN_THEME.typography.letterSpacing.wide,
  },
  count: {
    position: 'absolute',
    top: -4,
    right: -4,
    backgroundColor: MODERN_THEME.colors.danger,
    borderRadius: MODERN_THEME.borderRadius.full,
    width: 24,
    height: 24,
    textAlign: 'center',
    lineHeight: 24,
    fontSize: MODERN_THEME.typography.fontSize.sm,
    fontWeight: MODERN_THEME.typography.fontWeight.bold,
    color: MODERN_THEME.colors.text,
    overflow: 'hidden',
  },
});

export default WildCardButton;
