import React, { useEffect, useRef } from 'react';
import { View, Text, StyleSheet, Animated, ViewStyle, TextStyle } from 'react-native';

export type LetterState = 'hidden' | 'revealing' | 'revealed' | 'empty' | 'space';

export interface LetterCellProps {
  /** The character to display */
  char: string | null;
  /** Current state of the letter */
  state: LetterState;
  /** Size variant for different displays */
  size?: 'small' | 'medium' | 'large';
  /** Animation type for reveal */
  animationType?: 'pop' | 'flip' | 'fade';
  /** Duration of reveal animation in ms */
  animationDuration?: number;
  /** Delay before starting animation (for staggering) */
  animationDelay?: number;
  /** Callback when animation completes */
  onAnimationComplete?: () => void;
  /** Custom container style */
  style?: ViewStyle;
  /** Custom text style */
  textStyle?: TextStyle;
  /** Test ID for testing */
  testID?: string;
}

const SIZE_CONFIG = {
  small: { width: 22, height: 28, fontSize: 16 },
  medium: { width: 36, height: 44, fontSize: 28 },
  large: { width: 56, height: 68, fontSize: 42 },
};

/**
 * LetterCell - Individual letter tile with reveal animation
 *
 * Supports three animation types:
 * - pop: Scale from 0 to 1.1 to 1 with bounce
 * - flip: Rotate on Y axis (simulated with scaleX)
 * - fade: Simple fade in
 *
 * @example
 * <LetterCell
 *   char="A"
 *   state="revealing"
 *   animationType="pop"
 *   onAnimationComplete={() => setRevealed(true)}
 * />
 */
export function LetterCell({
  char,
  state,
  size = 'medium',
  animationType = 'pop',
  animationDuration = 300,
  animationDelay = 0,
  onAnimationComplete,
  style,
  textStyle,
  testID,
}: LetterCellProps): React.JSX.Element {
  const scale = useRef(new Animated.Value(state === 'revealing' ? 0 : 1)).current;
  const opacity = useRef(new Animated.Value(state === 'revealing' ? 0 : 1)).current;
  const scaleX = useRef(new Animated.Value(state === 'revealing' ? 0 : 1)).current;
  const glowOpacity = useRef(new Animated.Value(0)).current;

  const sizeConfig = SIZE_CONFIG[size];
  const isLetter = char !== null && char !== ' ' && /[A-Z]/i.test(char);
  const isPunctuation = char !== null && char !== ' ' && !isLetter;
  const isEmpty = char === null;
  const isSpace = char === ' ';

  useEffect(() => {
    if (state !== 'revealing') return;

    const animationTimeout = setTimeout(() => {
      let animation: Animated.CompositeAnimation;

      switch (animationType) {
        case 'pop':
          animation = Animated.sequence([
            // Start glow
            Animated.timing(glowOpacity, {
              toValue: 1,
              duration: 50,
              useNativeDriver: true,
            }),
            // Pop effect
            Animated.parallel([
              Animated.timing(opacity, {
                toValue: 1,
                duration: animationDuration * 0.3,
                useNativeDriver: true,
              }),
              Animated.sequence([
                // Scale up past 1
                Animated.spring(scale, {
                  toValue: 1.15,
                  friction: 6,
                  tension: 200,
                  useNativeDriver: true,
                }),
                // Settle to 1
                Animated.spring(scale, {
                  toValue: 1,
                  friction: 4,
                  tension: 150,
                  useNativeDriver: true,
                }),
              ]),
            ]),
            // Fade glow
            Animated.timing(glowOpacity, {
              toValue: 0,
              duration: 200,
              useNativeDriver: true,
            }),
          ]);
          break;

        case 'flip':
          animation = Animated.sequence([
            Animated.timing(glowOpacity, {
              toValue: 1,
              duration: 50,
              useNativeDriver: true,
            }),
            Animated.parallel([
              Animated.timing(opacity, {
                toValue: 1,
                duration: animationDuration * 0.5,
                useNativeDriver: true,
              }),
              Animated.timing(scaleX, {
                toValue: 1,
                duration: animationDuration,
                useNativeDriver: true,
              }),
            ]),
            Animated.timing(glowOpacity, {
              toValue: 0,
              duration: 200,
              useNativeDriver: true,
            }),
          ]);
          break;

        case 'fade':
        default:
          animation = Animated.parallel([
            Animated.timing(opacity, {
              toValue: 1,
              duration: animationDuration,
              useNativeDriver: true,
            }),
            Animated.timing(scale, {
              toValue: 1,
              duration: animationDuration,
              useNativeDriver: true,
            }),
          ]);
          break;
      }

      animation.start(() => {
        onAnimationComplete?.();
      });
    }, animationDelay);

    return () => clearTimeout(animationTimeout);
  }, [state, animationType, animationDuration, animationDelay, onAnimationComplete, scale, opacity, scaleX, glowOpacity]);

  // Reset animation values when state changes back to hidden
  useEffect(() => {
    if (state === 'hidden') {
      scale.setValue(1);
      opacity.setValue(1);
      scaleX.setValue(1);
      glowOpacity.setValue(0);
    } else if (state === 'revealing') {
      // Reset to initial revealing state
      if (animationType === 'flip') {
        scale.setValue(1);
        opacity.setValue(0);
        scaleX.setValue(0);
      } else {
        scale.setValue(0);
        opacity.setValue(0);
        scaleX.setValue(1);
      }
      glowOpacity.setValue(0);
    }
  }, [state, animationType, scale, opacity, scaleX, glowOpacity]);

  const getCellStyle = (): ViewStyle[] => {
    const baseStyle: ViewStyle = {
      width: sizeConfig.width,
      height: sizeConfig.height,
      borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
    };

    if (isEmpty || isSpace) {
      return [styles.cell, baseStyle, styles.emptyCell, style];
    }

    if (state === 'hidden') {
      return [styles.cell, baseStyle, styles.hiddenCell, style];
    }

    return [styles.cell, baseStyle, styles.revealedCell, style];
  };

  const getTransform = () => {
    const transforms: { scale: Animated.Value }[] | { scaleX: Animated.Value }[] = [];

    if (animationType === 'flip') {
      return [{ scaleX }];
    }

    return [{ scale }];
  };

  const showLetter = (state === 'revealing' || state === 'revealed') && (isLetter || isPunctuation);

  return (
    <View style={getCellStyle()} testID={testID}>
      {/* Glow overlay for reveal effect */}
      {state === 'revealing' && (
        <Animated.View
          style={[
            styles.glowOverlay,
            {
              opacity: glowOpacity,
              borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
            },
          ]}
        />
      )}

      {/* Letter content */}
      {showLetter && (
        <Animated.View
          style={[
            styles.letterContainer,
            {
              opacity,
              transform: getTransform(),
            },
          ]}
        >
          <Text
            style={[
              styles.letter,
              { fontSize: sizeConfig.fontSize },
              textStyle,
            ]}
          >
            {char?.toUpperCase()}
          </Text>
        </Animated.View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  cell: {
    margin: 2,
    alignItems: 'center',
    justifyContent: 'center',
    overflow: 'hidden',
  },
  emptyCell: {
    backgroundColor: '#228b22',
  },
  hiddenCell: {
    backgroundColor: '#fff',
  },
  revealedCell: {
    backgroundColor: '#fff',
  },
  letterContainer: {
    alignItems: 'center',
    justifyContent: 'center',
  },
  letter: {
    fontWeight: 'bold',
    color: '#1a1a2e',
  },
  glowOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(255, 255, 255, 0.8)',
  },
});
