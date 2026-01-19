import React, { useRef, useCallback } from 'react';
import {
  Animated,
  TouchableOpacity,
  StyleSheet,
  ViewStyle,
  TextStyle,
  Text,
  GestureResponderEvent,
  StyleProp,
} from 'react-native';
import { theme } from '../../constants/theme';

export interface GoldButtonProps {
  /** Button press handler */
  onPress?: (event: GestureResponderEvent) => void;
  /** Long press handler */
  onLongPress?: (event: GestureResponderEvent) => void;
  /** Button label text */
  children: string;
  /** Button size variant */
  size?: 'sm' | 'md' | 'lg' | 'xl';
  /** Whether button is disabled */
  disabled?: boolean;
  /** Whether to show glow effect */
  showGlow?: boolean;
  /** Custom container style */
  style?: StyleProp<ViewStyle>;
  /** Custom text style */
  textStyle?: StyleProp<TextStyle>;
  /** Whether button is in loading state */
  loading?: boolean;
  /** Button variant */
  variant?: 'primary' | 'outline' | 'ghost';
  /** Test ID for testing */
  testID?: string;
}

/**
 * GoldButton - Styled button with gradient appearance and glow effect
 *
 * Features:
 * - Gold gradient appearance (simulated with solid color)
 * - Scale animation on press
 * - Optional glow effect
 * - Multiple size variants
 * - Disabled and loading states
 *
 * @example
 * <GoldButton onPress={handlePress}>
 *   Spin the Wheel
 * </GoldButton>
 *
 * <GoldButton
 *   onPress={handleAction}
 *   size="lg"
 *   showGlow
 * >
 *   Start Game
 * </GoldButton>
 *
 * <GoldButton
 *   variant="outline"
 *   onPress={handleCancel}
 * >
 *   Cancel
 * </GoldButton>
 */
export function GoldButton({
  onPress,
  onLongPress,
  children,
  size = 'md',
  disabled = false,
  showGlow = true,
  style,
  textStyle,
  loading = false,
  variant = 'primary',
  testID,
}: GoldButtonProps): React.JSX.Element {
  const scaleValue = useRef(new Animated.Value(1)).current;
  const glowOpacity = useRef(new Animated.Value(0)).current;

  const handlePressIn = useCallback(() => {
    if (disabled || loading) return;

    Animated.parallel([
      Animated.spring(scaleValue, {
        toValue: 0.95,
        friction: 5,
        tension: 100,
        useNativeDriver: true,
      }),
      ...(showGlow
        ? [
            Animated.timing(glowOpacity, {
              toValue: 1,
              duration: 100,
              useNativeDriver: true,
            }),
          ]
        : []),
    ]).start();
  }, [scaleValue, showGlow, glowOpacity, disabled, loading]);

  const handlePressOut = useCallback(() => {
    Animated.parallel([
      Animated.spring(scaleValue, {
        toValue: 1,
        friction: 5,
        tension: 100,
        useNativeDriver: true,
      }),
      ...(showGlow
        ? [
            Animated.timing(glowOpacity, {
              toValue: 0,
              duration: 200,
              useNativeDriver: true,
            }),
          ]
        : []),
    ]).start();
  }, [scaleValue, showGlow, glowOpacity]);

  const sizeStyles = getSizeStyles(size);
  const variantStyles = getVariantStyles(variant, disabled);
  const variantTextStyles = getVariantTextStyles(variant, disabled);

  const glowStyle: ViewStyle = showGlow
    ? {
        shadowColor: theme.colors.gold,
        shadowOffset: { width: 0, height: 0 },
        shadowRadius: 15,
        elevation: 10,
      }
    : {};

  return (
    <TouchableOpacity
      onPress={onPress}
      onLongPress={onLongPress}
      onPressIn={handlePressIn}
      onPressOut={handlePressOut}
      disabled={disabled || loading}
      activeOpacity={0.9}
      testID={testID}
    >
      <Animated.View
        style={[
          styles.container,
          sizeStyles.container,
          variantStyles,
          glowStyle,
          style,
          {
            transform: [{ scale: scaleValue }],
          },
          showGlow && {
            shadowOpacity: glowOpacity,
          },
        ]}
      >
        <Text style={[styles.text, sizeStyles.text, variantTextStyles, textStyle]}>
          {loading ? 'Loading...' : children}
        </Text>
      </Animated.View>
    </TouchableOpacity>
  );
}

const getSizeStyles = (
  size: GoldButtonProps['size']
): { container: ViewStyle; text: TextStyle } => {
  switch (size) {
    case 'sm':
      return {
        container: {
          paddingVertical: theme.spacing.sm,
          paddingHorizontal: theme.spacing.md,
          borderRadius: theme.borderRadius.sm,
          minHeight: 36,
        },
        text: {
          fontSize: theme.typography.fontSize.sm,
        },
      };
    case 'md':
      return {
        container: {
          paddingVertical: theme.spacing.sm + 4,
          paddingHorizontal: theme.spacing.lg,
          borderRadius: theme.borderRadius.md,
          minHeight: 48,
        },
        text: {
          fontSize: theme.typography.fontSize.md,
        },
      };
    case 'lg':
      return {
        container: {
          paddingVertical: theme.spacing.md,
          paddingHorizontal: theme.spacing.xl,
          borderRadius: theme.borderRadius.lg,
          minHeight: 56,
        },
        text: {
          fontSize: theme.typography.fontSize.lg,
        },
      };
    case 'xl':
      return {
        container: {
          paddingVertical: theme.spacing.lg,
          paddingHorizontal: theme.spacing['2xl'],
          borderRadius: theme.borderRadius.xl,
          minHeight: 64,
        },
        text: {
          fontSize: theme.typography.fontSize.xl,
        },
      };
    default:
      return {
        container: {},
        text: {},
      };
  }
};

const getVariantStyles = (
  variant: GoldButtonProps['variant'],
  disabled: boolean
): ViewStyle => {
  const disabledStyle: ViewStyle = disabled ? { opacity: 0.5 } : {};

  switch (variant) {
    case 'primary':
      return {
        backgroundColor: theme.colors.gold,
        borderWidth: 0,
        ...disabledStyle,
      };
    case 'outline':
      return {
        backgroundColor: 'transparent',
        borderWidth: 2,
        borderColor: theme.colors.gold,
        ...disabledStyle,
      };
    case 'ghost':
      return {
        backgroundColor: 'transparent',
        borderWidth: 0,
        ...disabledStyle,
      };
    default:
      return disabledStyle;
  }
};

const getVariantTextStyles = (
  variant: GoldButtonProps['variant'],
  disabled: boolean
): TextStyle => {
  const disabledStyle: TextStyle = disabled ? { opacity: 0.7 } : {};

  switch (variant) {
    case 'primary':
      return {
        color: theme.colors.textOnGold,
        ...disabledStyle,
      };
    case 'outline':
      return {
        color: theme.colors.gold,
        ...disabledStyle,
      };
    case 'ghost':
      return {
        color: theme.colors.gold,
        ...disabledStyle,
      };
    default:
      return disabledStyle;
  }
};

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  text: {
    fontWeight: theme.typography.fontWeight.bold,
    textAlign: 'center',
    letterSpacing: 1,
  },
});

export default GoldButton;
