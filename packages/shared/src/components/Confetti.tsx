import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import { View, StyleSheet, Animated, Dimensions, ViewStyle } from 'react-native';

export interface ConfettiProps {
  /** Whether confetti animation is active */
  active: boolean;
  /** Number of confetti pieces (default: 50) */
  count?: number;
  /** Duration of animation in ms (default: 3000) */
  duration?: number;
  /** Confetti colors (default: festive colors) */
  colors?: string[];
  /** Callback when animation completes */
  onComplete?: () => void;
  /** Style for the container */
  style?: ViewStyle;
  /** Test ID for testing */
  testID?: string;
}

interface ConfettiPiece {
  id: number;
  x: Animated.Value;
  y: Animated.Value;
  rotate: Animated.Value;
  opacity: Animated.Value;
  scale: Animated.Value;
  color: string;
  size: number;
  shape: 'square' | 'rectangle' | 'circle';
  startX: number;
  startY: number;
  endX: number;
  endY: number;
  rotations: number;
}

const DEFAULT_COLORS = [
  '#d4af37', // Gold
  '#ffd700', // Bright gold
  '#ff6b6b', // Red
  '#4ecdc4', // Teal
  '#45b7d1', // Blue
  '#96ceb4', // Green
  '#ff69b4', // Pink
  '#9b59b6', // Purple
  '#f39c12', // Orange
  '#1abc9c', // Aqua
];

const SHAPES: ConfettiPiece['shape'][] = ['square', 'rectangle', 'circle'];

/**
 * Confetti - Celebratory confetti animation component
 *
 * Features:
 * - Customizable number of confetti pieces
 * - Multiple shapes (squares, rectangles, circles)
 * - Physics-based falling animation
 * - Rotation and fade effects
 * - Customizable colors
 *
 * @example
 * <Confetti
 *   active={puzzleSolved}
 *   count={100}
 *   duration={3000}
 *   onComplete={() => setShowConfetti(false)}
 * />
 */
export function Confetti({
  active,
  count = 50,
  duration = 3000,
  colors = DEFAULT_COLORS,
  onComplete,
  style,
  testID,
}: ConfettiProps): React.JSX.Element | null {
  const [pieces, setPieces] = useState<ConfettiPiece[]>([]);
  const [isAnimating, setIsAnimating] = useState(false);
  const animationRef = useRef<Animated.CompositeAnimation | null>(null);
  const windowDimensions = useMemo(() => Dimensions.get('window'), []);

  // Create confetti pieces
  const createPieces = useCallback((): ConfettiPiece[] => {
    const newPieces: ConfettiPiece[] = [];
    const { width, height } = windowDimensions;

    for (let i = 0; i < count; i++) {
      const startX = Math.random() * width;
      const startY = -50 - Math.random() * 100; // Start above screen
      const endX = startX + (Math.random() - 0.5) * 200; // Drift sideways
      const endY = height + 100; // End below screen

      newPieces.push({
        id: i,
        x: new Animated.Value(startX),
        y: new Animated.Value(startY),
        rotate: new Animated.Value(0),
        opacity: new Animated.Value(1),
        scale: new Animated.Value(0),
        color: colors[Math.floor(Math.random() * colors.length)],
        size: 8 + Math.random() * 8,
        shape: SHAPES[Math.floor(Math.random() * SHAPES.length)],
        startX,
        startY,
        endX,
        endY,
        rotations: 2 + Math.floor(Math.random() * 4),
      });
    }

    return newPieces;
  }, [count, colors, windowDimensions]);

  // Start confetti animation
  useEffect(() => {
    if (!active) {
      // Clean up when deactivated
      if (animationRef.current) {
        animationRef.current.stop();
        animationRef.current = null;
      }
      setPieces([]);
      setIsAnimating(false);
      return;
    }

    if (isAnimating) return;

    setIsAnimating(true);
    const newPieces = createPieces();
    setPieces(newPieces);

    // Create animations for each piece
    const animations = newPieces.map((piece) => {
      const delay = Math.random() * (duration * 0.3);
      const pieceDuration = duration * (0.7 + Math.random() * 0.3);

      return Animated.sequence([
        Animated.delay(delay),
        Animated.parallel([
          // Vertical fall with gravity effect
          Animated.timing(piece.y, {
            toValue: piece.endY,
            duration: pieceDuration,
            useNativeDriver: true,
          }),
          // Horizontal drift
          Animated.timing(piece.x, {
            toValue: piece.endX,
            duration: pieceDuration,
            useNativeDriver: true,
          }),
          // Rotation
          Animated.timing(piece.rotate, {
            toValue: piece.rotations * 360,
            duration: pieceDuration,
            useNativeDriver: true,
          }),
          // Scale in
          Animated.sequence([
            Animated.timing(piece.scale, {
              toValue: 1,
              duration: 200,
              useNativeDriver: true,
            }),
            Animated.delay(pieceDuration - 700),
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
      onComplete?.();
    });

    return () => {
      if (animationRef.current) {
        animationRef.current.stop();
      }
    };
  }, [active, duration, createPieces, isAnimating, onComplete]);

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
}

interface ConfettiPieceViewProps {
  piece: ConfettiPiece;
}

function ConfettiPieceView({ piece }: ConfettiPieceViewProps): React.JSX.Element {
  const rotate = piece.rotate.interpolate({
    inputRange: [0, 360],
    outputRange: ['0deg', '360deg'],
  });

  const shapeStyle: ViewStyle = {
    backgroundColor: piece.color,
    width: piece.shape === 'rectangle' ? piece.size * 1.5 : piece.size,
    height: piece.shape === 'rectangle' ? piece.size * 0.6 : piece.size,
    borderRadius: piece.shape === 'circle' ? piece.size / 2 : 2,
  };

  return (
    <Animated.View
      style={[
        styles.piece,
        shapeStyle,
        {
          transform: [
            { translateX: piece.x },
            { translateY: piece.y },
            { rotate },
            { scale: piece.scale },
          ],
          opacity: piece.opacity,
        },
      ]}
    />
  );
}

/**
 * Hook to manage confetti state
 *
 * @example
 * const { showConfetti, triggerConfetti } = useConfetti();
 *
 * // Trigger on puzzle solve
 * triggerConfetti();
 *
 * // In render
 * <Confetti active={showConfetti} onComplete={() => {}} />
 */
export function useConfetti(autoHideDuration?: number) {
  const [showConfetti, setShowConfetti] = useState(false);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const triggerConfetti = useCallback(() => {
    setShowConfetti(true);

    if (autoHideDuration) {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
      timeoutRef.current = setTimeout(() => {
        setShowConfetti(false);
      }, autoHideDuration);
    }
  }, [autoHideDuration]);

  const hideConfetti = useCallback(() => {
    if (timeoutRef.current) {
      clearTimeout(timeoutRef.current);
    }
    setShowConfetti(false);
  }, []);

  useEffect(() => {
    return () => {
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }
    };
  }, []);

  return {
    showConfetti,
    triggerConfetti,
    hideConfetti,
  };
}

const styles = StyleSheet.create({
  container: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    overflow: 'hidden',
  },
  piece: {
    position: 'absolute',
  },
});

export default Confetti;
