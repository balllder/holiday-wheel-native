import React, { useEffect, useRef, useState } from 'react';
import {
  Animated,
  StyleSheet,
  Text,
  View,
  ViewStyle,
  TextStyle,
} from 'react-native';

export interface AnimatedScoreProps {
  /** The score change value (positive or negative) */
  value: number;
  /** Position relative to the component */
  position?: 'above' | 'below' | 'left' | 'right';
  /** Duration of the animation in milliseconds */
  duration?: number;
  /** Custom container style */
  style?: ViewStyle;
  /** Custom text style */
  textStyle?: TextStyle;
  /** Callback when animation completes */
  onComplete?: () => void;
  /** Format the value (default: adds +/- prefix and $ sign) */
  formatValue?: (value: number) => string;
  /** Whether to show the animation */
  visible?: boolean;
}

const defaultFormatValue = (value: number): string => {
  const prefix = value >= 0 ? '+' : '';
  return `${prefix}$${Math.abs(value).toLocaleString()}`;
};

/**
 * AnimatedScore - Displays floating score change animation
 *
 * Shows a value that fades in, floats up/down, and fades out.
 * Used to show score changes like "+$500" or "-$1000" (bankrupt).
 *
 * @example
 * <AnimatedScore value={500} position="above" />
 * <AnimatedScore value={-1000} position="above" textStyle={{ fontSize: 24 }} />
 */
export function AnimatedScore({
  value,
  position = 'above',
  duration = 1500,
  style,
  textStyle,
  onComplete,
  formatValue = defaultFormatValue,
  visible = true,
}: AnimatedScoreProps): React.JSX.Element | null {
  const [isAnimating, setIsAnimating] = useState(false);
  const opacity = useRef(new Animated.Value(0)).current;
  const translateY = useRef(new Animated.Value(0)).current;
  const scale = useRef(new Animated.Value(0.5)).current;

  useEffect(() => {
    if (!visible || value === 0) return;

    setIsAnimating(true);

    // Reset values
    opacity.setValue(0);
    translateY.setValue(0);
    scale.setValue(0.5);

    // Calculate direction
    const direction = position === 'below' || position === 'right' ? 1 : -1;
    const isVertical = position === 'above' || position === 'below';

    // Animation sequence
    Animated.parallel([
      // Fade in then out
      Animated.sequence([
        Animated.timing(opacity, {
          toValue: 1,
          duration: duration * 0.2,
          useNativeDriver: true,
        }),
        Animated.timing(opacity, {
          toValue: 1,
          duration: duration * 0.5,
          useNativeDriver: true,
        }),
        Animated.timing(opacity, {
          toValue: 0,
          duration: duration * 0.3,
          useNativeDriver: true,
        }),
      ]),
      // Float up/down
      Animated.timing(isVertical ? translateY : translateY, {
        toValue: direction * 60,
        duration: duration,
        useNativeDriver: true,
      }),
      // Scale pop
      Animated.sequence([
        Animated.spring(scale, {
          toValue: 1.2,
          friction: 4,
          tension: 100,
          useNativeDriver: true,
        }),
        Animated.timing(scale, {
          toValue: 1,
          duration: duration * 0.3,
          useNativeDriver: true,
        }),
      ]),
    ]).start(() => {
      setIsAnimating(false);
      onComplete?.();
    });
  }, [value, visible, position, duration, opacity, translateY, scale, onComplete]);

  if (!isAnimating && !visible) return null;

  const isPositive = value >= 0;
  const formattedValue = formatValue(value);

  const positionStyle = getPositionStyle(position);

  return (
    <View style={[styles.container, positionStyle, style]} pointerEvents="none">
      <Animated.View
        style={[
          styles.animatedContainer,
          {
            opacity,
            transform: [
              { translateY },
              { scale },
            ],
          },
        ]}
      >
        <Text
          style={[
            styles.text,
            isPositive ? styles.positiveText : styles.negativeText,
            textStyle,
          ]}
        >
          {formattedValue}
        </Text>
      </Animated.View>
    </View>
  );
}

const getPositionStyle = (position: AnimatedScoreProps['position']): ViewStyle => {
  switch (position) {
    case 'above':
      return { bottom: '100%', left: 0, right: 0 };
    case 'below':
      return { top: '100%', left: 0, right: 0 };
    case 'left':
      return { right: '100%', top: 0, bottom: 0 };
    case 'right':
      return { left: '100%', top: 0, bottom: 0 };
    default:
      return { bottom: '100%', left: 0, right: 0 };
  }
};

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    alignItems: 'center',
    justifyContent: 'center',
    zIndex: 100,
  },
  animatedContainer: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  text: {
    fontSize: 28,
    fontWeight: 'bold',
    textShadowColor: 'rgba(0, 0, 0, 0.75)',
    textShadowOffset: { width: 1, height: 1 },
    textShadowRadius: 3,
  },
  positiveText: {
    color: '#d4af37', // Gold for gains
  },
  negativeText: {
    color: '#ef4444', // Red for losses
  },
});

/**
 * Hook to manage score change animations
 *
 * @example
 * const { showChange, ScoreChangeComponent } = useScoreAnimation();
 *
 * // When score changes
 * showChange(500);
 *
 * // In render
 * <View>
 *   <ScoreChangeComponent />
 *   <Text>{score}</Text>
 * </View>
 */
export function useScoreAnimation(options?: Partial<AnimatedScoreProps>) {
  const [changes, setChanges] = useState<{ id: number; value: number }[]>([]);
  const nextId = useRef(0);

  const showChange = (value: number) => {
    const id = nextId.current++;
    setChanges((prev) => [...prev, { id, value }]);
  };

  const removeChange = (id: number) => {
    setChanges((prev) => prev.filter((c) => c.id !== id));
  };

  const ScoreChangeComponent = () => (
    <>
      {changes.map((change) => (
        <AnimatedScore
          key={change.id}
          value={change.value}
          visible={true}
          onComplete={() => removeChange(change.id)}
          {...options}
        />
      ))}
    </>
  );

  return { showChange, ScoreChangeComponent, changes };
}
