import React, { useEffect } from 'react';
import { View, Text, StyleSheet, ViewStyle, TextStyle } from 'react-native';
import Animated, {
  useSharedValue,
  useAnimatedStyle,
  withTiming,
  withSpring,
  withSequence,
  withDelay,
  runOnJS,
  interpolate,
  Easing,
} from 'react-native-reanimated';

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
 * LetterCell - Individual letter tile with reveal animation using react-native-reanimated
 *
 * Supports three animation types:
 * - pop: Scale from 0 to 1.15 to 1 with spring bounce
 * - flip: 3D rotate on Y-axis (card flip effect)
 * - fade: Simple fade in
 *
 * @example
 * <LetterCell
 *   char="A"
 *   state="revealing"
 *   animationType="flip"
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
  // Animation shared values
  const scale = useSharedValue(1);
  const opacity = useSharedValue(1);
  const rotateY = useSharedValue(0); // 0 = showing back (blank), 180 = showing front (letter)
  const glowOpacity = useSharedValue(0);

  const sizeConfig = SIZE_CONFIG[size];
  const isLetter = char !== null && char !== ' ' && /[A-Z]/i.test(char);
  const isPunctuation = char !== null && char !== ' ' && !isLetter;
  const isEmpty = char === null;
  const isSpace = char === ' ';

  // Helper to call completion callback on JS thread
  const handleComplete = () => {
    onAnimationComplete?.();
  };

  useEffect(() => {
    if (state === 'hidden') {
      // Reset to initial hidden state
      scale.value = 1;
      opacity.value = 1;
      rotateY.value = 0;
      glowOpacity.value = 0;
    } else if (state === 'revealing') {
      // Reset values for animation start
      if (animationType === 'flip') {
        rotateY.value = 0;
        opacity.value = 1;
        scale.value = 1;
      } else if (animationType === 'pop') {
        scale.value = 0;
        opacity.value = 0;
      } else {
        // fade
        scale.value = 1;
        opacity.value = 0;
      }
      glowOpacity.value = 0;

      // Start reveal animation after delay
      const startAnimation = () => {
        switch (animationType) {
          case 'flip':
            // Start glow
            glowOpacity.value = withTiming(1, { duration: 50 });

            // Flip animation - rotate 180 degrees on Y axis
            rotateY.value = withDelay(
              50,
              withTiming(180, {
                duration: animationDuration,
                easing: Easing.inOut(Easing.ease),
              }, (finished) => {
                if (finished) {
                  // Fade glow after flip completes
                  glowOpacity.value = withTiming(0, { duration: 200 });
                  runOnJS(handleComplete)();
                }
              })
            );
            break;

          case 'pop':
            // Start glow
            glowOpacity.value = withTiming(1, { duration: 50 });

            // Pop animation with spring
            opacity.value = withTiming(1, { duration: animationDuration * 0.3 });
            scale.value = withSequence(
              withSpring(1.15, { damping: 6, stiffness: 200 }),
              withSpring(1, { damping: 4, stiffness: 150 }, (finished) => {
                if (finished) {
                  glowOpacity.value = withTiming(0, { duration: 200 });
                  runOnJS(handleComplete)();
                }
              })
            );
            break;

          case 'fade':
          default:
            opacity.value = withTiming(1, { duration: animationDuration }, (finished) => {
              if (finished) {
                runOnJS(handleComplete)();
              }
            });
            scale.value = withTiming(1, { duration: animationDuration });
            break;
        }
      };

      if (animationDelay > 0) {
        const timeout = setTimeout(startAnimation, animationDelay);
        return () => clearTimeout(timeout);
      } else {
        startAnimation();
      }
    }
  }, [state, animationType, animationDuration, animationDelay]);

  // Animated style for the front face (shows the letter)
  const frontFaceStyle = useAnimatedStyle(() => {
    if (animationType !== 'flip') {
      return {
        opacity: opacity.value,
        transform: [{ scale: scale.value }],
      };
    }

    // For flip animation, front face starts hidden (rotated -180) and becomes visible as rotateY approaches 180
    const frontRotation = rotateY.value - 180;
    const frontOpacity = interpolate(
      rotateY.value,
      [0, 89, 90, 180],
      [0, 0, 1, 1]
    );

    return {
      opacity: frontOpacity,
      transform: [
        { perspective: 1000 },
        { rotateY: `${frontRotation}deg` },
      ],
      backfaceVisibility: 'hidden',
    };
  });

  // Animated style for the back face (blank tile)
  const backFaceStyle = useAnimatedStyle(() => {
    if (animationType !== 'flip') {
      // For non-flip animations, back face is visible when hidden
      return {
        opacity: state === 'hidden' ? 1 : 1 - opacity.value,
      };
    }

    // For flip animation, back face starts visible and hides when rotated past 90 degrees
    const backOpacity = interpolate(
      rotateY.value,
      [0, 89, 90, 180],
      [1, 1, 0, 0]
    );

    return {
      opacity: backOpacity,
      transform: [
        { perspective: 1000 },
        { rotateY: `${rotateY.value}deg` },
      ],
      backfaceVisibility: 'hidden',
    };
  });

  // Animated style for glow overlay
  const glowAnimatedStyle = useAnimatedStyle(() => ({
    opacity: glowOpacity.value,
  }));

  const getCellStyle = (): ViewStyle[] => {
    const baseStyle: ViewStyle = {
      width: sizeConfig.width,
      height: sizeConfig.height,
      borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
    };

    const styleArray: ViewStyle[] = isEmpty || isSpace
      ? [styles.cell, baseStyle, styles.emptyCell]
      : [styles.cell, baseStyle, styles.letterCell];

    if (style) {
      styleArray.push(style);
    }

    return styleArray;
  };

  const showLetter = (state === 'revealing' || state === 'revealed') && (isLetter || isPunctuation);

  // For non-letter cells (empty/space), render simple view
  if (isEmpty || isSpace) {
    return <View style={getCellStyle()} testID={testID} />;
  }

  return (
    <View style={getCellStyle()} testID={testID}>
      {/* Back face - blank white tile */}
      <Animated.View
        style={[
          styles.face,
          styles.backFace,
          {
            borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
          },
          backFaceStyle,
        ]}
      />

      {/* Glow overlay for reveal effect */}
      <Animated.View
        style={[
          styles.glowOverlay,
          {
            borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
          },
          glowAnimatedStyle,
        ]}
      />

      {/* Front face - letter content */}
      {showLetter && (
        <Animated.View
          style={[
            styles.face,
            styles.frontFace,
            {
              borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
            },
            frontFaceStyle,
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

      {/* Hidden state - show blank tile */}
      {state === 'hidden' && (
        <View
          style={[
            styles.face,
            styles.hiddenFace,
            {
              borderRadius: size === 'large' ? 6 : size === 'medium' ? 4 : 3,
            },
          ]}
        />
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
  letterCell: {
    backgroundColor: 'transparent',
  },
  face: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    alignItems: 'center',
    justifyContent: 'center',
  },
  backFace: {
    backgroundColor: '#fff',
  },
  frontFace: {
    backgroundColor: '#fff',
  },
  hiddenFace: {
    backgroundColor: '#fff',
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
