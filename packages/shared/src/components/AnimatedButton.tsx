import React, { useRef, useCallback } from 'react';
import {
  Animated,
  TouchableOpacity,
  StyleSheet,
  ViewStyle,
  TextStyle,
  Text,
  GestureResponderEvent,
} from 'react-native';

export interface AnimatedButtonProps {
  /** Button press handler */
  onPress?: (event: GestureResponderEvent) => void;
  /** Long press handler */
  onLongPress?: (event: GestureResponderEvent) => void;
  /** Button content (string or custom content) */
  children: React.ReactNode;
  /** Scale multiplier when pressed (default: 0.95) */
  scaleOnPress?: number;
  /** Whether to show glow effect on press */
  showGlow?: boolean;
  /** Glow color (default: gold #d4af37) */
  glowColor?: string;
  /** Whether button is disabled */
  disabled?: boolean;
  /** Custom container style */
  style?: ViewStyle;
  /** Custom text style (when children is string) */
  textStyle?: TextStyle;
  /** Button variant */
  variant?: 'primary' | 'secondary' | 'outline' | 'ghost';
  /** Active opacity (default: 0.9) */
  activeOpacity?: number;
  /** Test ID for testing */
  testID?: string;
}

/**
 * AnimatedButton - Enhanced button with press animations
 *
 * Features:
 * - Scale animation on press (shrinks slightly)
 * - Optional glow effect
 * - Multiple style variants
 * - Supports both text and custom content
 *
 * @example
 * <AnimatedButton onPress={handleSpin} variant="primary">
 *   Spin the Wheel
 * </AnimatedButton>
 *
 * <AnimatedButton
 *   onPress={handleAction}
 *   scaleOnPress={0.9}
 *   showGlow
 *   glowColor="#d4af37"
 * >
 *   <Icon name="play" />
 * </AnimatedButton>
 */
export function AnimatedButton({
  onPress,
  onLongPress,
  children,
  scaleOnPress = 0.95,
  showGlow = false,
  glowColor = '#d4af37',
  disabled = false,
  style,
  textStyle,
  variant = 'primary',
  activeOpacity = 0.9,
  testID,
}: AnimatedButtonProps): React.JSX.Element {
  const scaleValue = useRef(new Animated.Value(1)).current;
  const glowOpacity = useRef(new Animated.Value(0)).current;

  const handlePressIn = useCallback(() => {
    Animated.parallel([
      Animated.spring(scaleValue, {
        toValue: scaleOnPress,
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
  }, [scaleValue, scaleOnPress, showGlow, glowOpacity]);

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

  const variantStyles = getVariantStyles(variant, disabled);
  const variantTextStyles = getVariantTextStyles(variant, disabled);

  const glowStyle: ViewStyle = showGlow
    ? {
        shadowColor: glowColor,
        shadowOffset: { width: 0, height: 0 },
        shadowRadius: 15,
        elevation: 10,
      }
    : {};

  const content =
    typeof children === 'string' ? (
      <Text style={[styles.text, variantTextStyles, textStyle]}>{children}</Text>
    ) : (
      children
    );

  return (
    <TouchableOpacity
      onPress={onPress}
      onLongPress={onLongPress}
      onPressIn={handlePressIn}
      onPressOut={handlePressOut}
      disabled={disabled}
      activeOpacity={activeOpacity}
      testID={testID}
    >
      <Animated.View
        style={[
          styles.container,
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
        {content}
      </Animated.View>
    </TouchableOpacity>
  );
}

const getVariantStyles = (
  variant: AnimatedButtonProps['variant'],
  disabled: boolean
): ViewStyle => {
  const disabledStyle: ViewStyle = disabled ? { opacity: 0.5 } : {};

  switch (variant) {
    case 'primary':
      return {
        backgroundColor: '#d4af37',
        borderWidth: 0,
        ...disabledStyle,
      };
    case 'secondary':
      return {
        backgroundColor: '#4a3b8c',
        borderWidth: 0,
        ...disabledStyle,
      };
    case 'outline':
      return {
        backgroundColor: 'transparent',
        borderWidth: 2,
        borderColor: '#d4af37',
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
  variant: AnimatedButtonProps['variant'],
  disabled: boolean
): TextStyle => {
  const disabledStyle: TextStyle = disabled ? { opacity: 0.7 } : {};

  switch (variant) {
    case 'primary':
      return {
        color: '#1a0a3e',
        ...disabledStyle,
      };
    case 'secondary':
      return {
        color: '#ffffff',
        ...disabledStyle,
      };
    case 'outline':
      return {
        color: '#d4af37',
        ...disabledStyle,
      };
    case 'ghost':
      return {
        color: '#d4af37',
        ...disabledStyle,
      };
    default:
      return disabledStyle;
  }
};

const styles = StyleSheet.create({
  container: {
    paddingVertical: 14,
    paddingHorizontal: 24,
    borderRadius: 8,
    alignItems: 'center',
    justifyContent: 'center',
    minHeight: 48,
  },
  text: {
    fontSize: 16,
    fontWeight: 'bold',
    textAlign: 'center',
  },
});

/**
 * Pre-configured spin button with appropriate styling
 */
export function SpinButton({
  onPress,
  disabled,
  ...props
}: Omit<AnimatedButtonProps, 'children' | 'variant'>): React.JSX.Element {
  return (
    <AnimatedButton
      onPress={onPress}
      disabled={disabled}
      variant="primary"
      scaleOnPress={0.92}
      showGlow
      glowColor="#d4af37"
      style={spinButtonStyles.button}
      textStyle={spinButtonStyles.text}
      {...props}
    >
      SPIN
    </AnimatedButton>
  );
}

/**
 * Pre-configured buzz button for toss-up mode
 */
export function BuzzButton({
  onPress,
  disabled,
  ...props
}: Omit<AnimatedButtonProps, 'children' | 'variant'>): React.JSX.Element {
  return (
    <AnimatedButton
      onPress={onPress}
      disabled={disabled}
      variant="secondary"
      scaleOnPress={0.88}
      showGlow
      glowColor="#ef4444"
      style={buzzButtonStyles.button}
      textStyle={buzzButtonStyles.text}
      {...props}
    >
      BUZZ!
    </AnimatedButton>
  );
}

const spinButtonStyles = StyleSheet.create({
  button: {
    paddingVertical: 18,
    paddingHorizontal: 48,
    borderRadius: 12,
  },
  text: {
    fontSize: 24,
    letterSpacing: 2,
  },
});

const buzzButtonStyles = StyleSheet.create({
  button: {
    paddingVertical: 24,
    paddingHorizontal: 48,
    borderRadius: 16,
    backgroundColor: '#ef4444',
  },
  text: {
    fontSize: 28,
    letterSpacing: 3,
    color: '#ffffff',
  },
});
