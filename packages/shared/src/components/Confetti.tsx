import React, { useEffect, useRef, useState, useCallback, useMemo, useImperativeHandle, forwardRef } from 'react';
import { View, StyleSheet, Animated, Dimensions, ViewStyle } from 'react-native';

/**
 * Confetti celebration variants with different intensities
 */
export type ConfettiVariant = 'solve' | 'roundWin' | 'gameWin';

/**
 * Configuration presets for each confetti variant
 */
interface VariantConfig {
  count: number;
  duration: number;
  colors: string[];
  /** Spread angle for confetti burst */
  spread: number;
  /** Initial velocity multiplier */
  velocity: number;
  /** Size range for pieces */
  sizeRange: [number, number];
}

const THEME_COLORS = {
  gold: '#d4af37',
  brightGold: '#ffd700',
  purple: '#9b59b6',
  deepPurple: '#6c5ce7',
  royalPurple: '#663399',
};

/**
 * Variant configurations for different celebration intensities
 */
const VARIANT_CONFIGS: Record<ConfettiVariant, VariantConfig> = {
  // Normal puzzle solve - moderate celebration
  solve: {
    count: 50,
    duration: 3000,
    colors: [
      THEME_COLORS.gold,
      THEME_COLORS.brightGold,
      '#ff6b6b', // Red
      '#4ecdc4', // Teal
      '#45b7d1', // Blue
      THEME_COLORS.purple,
    ],
    spread: 0.6,
    velocity: 1.0,
    sizeRange: [8, 14],
  },
  // Round win - bigger celebration
  roundWin: {
    count: 100,
    duration: 4000,
    colors: [
      THEME_COLORS.gold,
      THEME_COLORS.brightGold,
      THEME_COLORS.purple,
      THEME_COLORS.deepPurple,
      '#ff69b4', // Pink
      '#f39c12', // Orange
      '#1abc9c', // Aqua
    ],
    spread: 0.8,
    velocity: 1.3,
    sizeRange: [10, 18],
  },
  // Game win - dramatic celebration with more gold/purple theme
  gameWin: {
    count: 200,
    duration: 6000,
    colors: [
      THEME_COLORS.gold,
      THEME_COLORS.brightGold,
      THEME_COLORS.gold, // Extra gold weight
      THEME_COLORS.purple,
      THEME_COLORS.deepPurple,
      THEME_COLORS.royalPurple,
      '#fff', // White sparkles
      '#ff6b6b', // Red
      '#00ff88', // Green
    ],
    spread: 1.0,
    velocity: 1.6,
    sizeRange: [10, 22],
  },
};

export interface ConfettiProps {
  /** Whether confetti animation is active */
  active?: boolean;
  /** Celebration variant (determines intensity) */
  variant?: ConfettiVariant;
  /** Override number of confetti pieces */
  count?: number;
  /** Override duration of animation in ms */
  duration?: number;
  /** Override confetti colors */
  colors?: string[];
  /** Callback when animation completes */
  onComplete?: () => void;
  /** Style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

/**
 * Ref handle for imperatively triggering confetti
 */
export interface ConfettiRef {
  /** Trigger confetti with optional variant */
  trigger: (variant?: ConfettiVariant) => void;
  /** Stop current animation */
  stop: () => void;
  /** Check if animation is running */
  isAnimating: () => boolean;
}

interface ConfettiPiece {
  id: number;
  x: Animated.Value;
  y: Animated.Value;
  rotate: Animated.Value;
  rotateY: Animated.Value;
  opacity: Animated.Value;
  scale: Animated.Value;
  color: string;
  size: number;
  shape: 'square' | 'rectangle' | 'circle' | 'star';
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  rotations: number;
  delay: number;
}

const SHAPES: ConfettiPiece['shape'][] = ['square', 'rectangle', 'circle', 'star'];

/**
 * Confetti - Celebratory confetti animation component
 *
 * Features:
 * - Three variants: 'solve' (normal), 'roundWin' (bigger), 'gameWin' (dramatic)
 * - Gold and purple theme colors matching the game
 * - Multiple shapes (squares, rectangles, circles, stars)
 * - Physics-based falling animation with realistic flutter
 * - Rotation and fade effects
 * - Customizable colors
 * - Imperative trigger via ref
 *
 * @example
 * // Controlled by active prop
 * <Confetti active={puzzleSolved} variant="solve" />
 *
 * // Or via ref
 * const confettiRef = useRef<ConfettiRef>(null);
 * <Confetti ref={confettiRef} />
 * // Later: confettiRef.current?.trigger('gameWin');
 */
export const Confetti = forwardRef<ConfettiRef, ConfettiProps>(function Confetti(
  {
    active = false,
    variant = 'solve',
    count,
    duration,
    colors,
    onComplete,
    style,
    testID,
  },
  ref
) {
  const [pieces, setPieces] = useState<ConfettiPiece[]>([]);
  const [isAnimating, setIsAnimating] = useState(false);
  const [currentVariant, setCurrentVariant] = useState<ConfettiVariant>(variant);
  const animationRef = useRef<Animated.CompositeAnimation | null>(null);
  const windowDimensions = useMemo(() => Dimensions.get('window'), []);

  // Get config for current variant with optional overrides
  const getConfig = useCallback(
    (v: ConfettiVariant): VariantConfig => {
      const baseConfig = VARIANT_CONFIGS[v];
      return {
        ...baseConfig,
        count: count ?? baseConfig.count,
        duration: duration ?? baseConfig.duration,
        colors: colors ?? baseConfig.colors,
      };
    },
    [count, duration, colors]
  );

  // Create confetti pieces based on variant
  const createPieces = useCallback(
    (v: ConfettiVariant): ConfettiPiece[] => {
      const config = getConfig(v);
      const newPieces: ConfettiPiece[] = [];
      const { width, height } = windowDimensions;

      for (let i = 0; i < config.count; i++) {
        // Random start position across top (with some variation)
        const startX = Math.random() * width;
        const startY = -50 - Math.random() * 150; // Staggered start

        // End position with spread based on variant
        const spreadX = (Math.random() - 0.5) * width * config.spread;
        const endX = startX + spreadX;
        const endY = height + 100 + Math.random() * 100;

        // Random size within variant's range
        const size =
          config.sizeRange[0] +
          Math.random() * (config.sizeRange[1] - config.sizeRange[0]);

        // More rotations for dramatic variants
        const rotations = Math.floor(2 + Math.random() * 4 * config.velocity);

        newPieces.push({
          id: i,
          x: new Animated.Value(startX),
          y: new Animated.Value(startY),
          rotate: new Animated.Value(0),
          rotateY: new Animated.Value(0),
          opacity: new Animated.Value(1),
          scale: new Animated.Value(0),
          color: config.colors[Math.floor(Math.random() * config.colors.length)],
          size,
          shape: SHAPES[Math.floor(Math.random() * SHAPES.length)],
          startX,
          startY,
          endX,
          endY,
          rotations,
          delay: Math.random() * (config.duration * 0.3),
        });
      }

      return newPieces;
    },
    [getConfig, windowDimensions]
  );

  // Start confetti animation
  const startAnimation = useCallback(
    (v: ConfettiVariant) => {
      if (isAnimating) {
        // Stop current animation first
        if (animationRef.current) {
          animationRef.current.stop();
          animationRef.current = null;
        }
      }

      setCurrentVariant(v);
      setIsAnimating(true);
      const config = getConfig(v);
      const newPieces = createPieces(v);
      setPieces(newPieces);

      // Create animations for each piece
      const animations = newPieces.map((piece) => {
        const pieceDuration =
          config.duration * (0.6 + Math.random() * 0.4) * (1 / config.velocity);

        return Animated.sequence([
          Animated.delay(piece.delay),
          Animated.parallel([
            // Vertical fall with gravity effect (ease-in for acceleration)
            Animated.timing(piece.y, {
              toValue: piece.endY,
              duration: pieceDuration,
              useNativeDriver: true,
            }),
            // Horizontal drift with wobble
            Animated.timing(piece.x, {
              toValue: piece.endX,
              duration: pieceDuration,
              useNativeDriver: true,
            }),
            // Z-axis rotation (spin)
            Animated.timing(piece.rotate, {
              toValue: piece.rotations * 360,
              duration: pieceDuration,
              useNativeDriver: true,
            }),
            // Y-axis rotation (flutter effect)
            Animated.loop(
              Animated.sequence([
                Animated.timing(piece.rotateY, {
                  toValue: 180,
                  duration: 300 + Math.random() * 200,
                  useNativeDriver: true,
                }),
                Animated.timing(piece.rotateY, {
                  toValue: 0,
                  duration: 300 + Math.random() * 200,
                  useNativeDriver: true,
                }),
              ]),
              { iterations: Math.ceil(pieceDuration / 600) }
            ),
            // Scale and fade sequence
            Animated.sequence([
              // Pop in
              Animated.spring(piece.scale, {
                toValue: 1,
                tension: 100,
                friction: 8,
                useNativeDriver: true,
              }),
              Animated.delay(pieceDuration - 800),
              // Fade out at end
              Animated.timing(piece.opacity, {
                toValue: 0,
                duration: 500,
                useNativeDriver: true,
              }),
            ]),
          ]),
        ]);
      });

      animationRef.current = Animated.parallel(animations);
      animationRef.current.start(() => {
        setIsAnimating(false);
        setPieces([]);
        animationRef.current = null;
        onComplete?.();
      });
    },
    [createPieces, getConfig, isAnimating, onComplete]
  );

  // Stop animation
  const stopAnimation = useCallback(() => {
    if (animationRef.current) {
      animationRef.current.stop();
      animationRef.current = null;
    }
    setIsAnimating(false);
    setPieces([]);
  }, []);

  // Expose imperative methods via ref
  useImperativeHandle(
    ref,
    () => ({
      trigger: (v?: ConfettiVariant) => startAnimation(v ?? variant),
      stop: stopAnimation,
      isAnimating: () => isAnimating,
    }),
    [startAnimation, stopAnimation, isAnimating, variant]
  );

  // Handle active prop changes
  useEffect(() => {
    if (active && !isAnimating) {
      startAnimation(variant);
    } else if (!active && isAnimating) {
      stopAnimation();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [active]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        animationRef.current.stop();
      }
    };
  }, []);

  if (pieces.length === 0) {
    return null;
  }

  return (
    <View style={[styles.container, style]} pointerEvents="none" testID={testID}>
      {pieces.map((piece) => (
        <ConfettiPieceView key={piece.id} piece={piece} />
      ))}
    </View>
  );
});

interface ConfettiPieceViewProps {
  piece: ConfettiPiece;
}

function ConfettiPieceView({ piece }: ConfettiPieceViewProps): React.JSX.Element {
  const rotate = piece.rotate.interpolate({
    inputRange: [0, 360],
    outputRange: ['0deg', '360deg'],
  });

  const rotateY = piece.rotateY.interpolate({
    inputRange: [0, 180],
    outputRange: ['0deg', '180deg'],
  });

  const getShapeStyle = (): ViewStyle => {
    const baseStyle: ViewStyle = {
      backgroundColor: piece.color,
    };

    switch (piece.shape) {
      case 'rectangle':
        return {
          ...baseStyle,
          width: piece.size * 1.5,
          height: piece.size * 0.6,
          borderRadius: 2,
        };
      case 'circle':
        return {
          ...baseStyle,
          width: piece.size,
          height: piece.size,
          borderRadius: piece.size / 2,
        };
      case 'star':
        // Simplified star as a rotated square (diamond shape)
        return {
          ...baseStyle,
          width: piece.size,
          height: piece.size,
          borderRadius: 2,
        };
      case 'square':
      default:
        return {
          ...baseStyle,
          width: piece.size,
          height: piece.size,
          borderRadius: 2,
        };
    }
  };

  return (
    <Animated.View
      style={[
        styles.piece,
        getShapeStyle(),
        {
          transform: [
            { translateX: piece.x },
            { translateY: piece.y },
            { rotate },
            { rotateY },
            { scale: piece.scale },
          ],
          opacity: piece.opacity,
        },
      ]}
    />
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: 'hidden',
    zIndex: 9999,
  },
  piece: {
    position: 'absolute',
  },
});

export default Confetti;
