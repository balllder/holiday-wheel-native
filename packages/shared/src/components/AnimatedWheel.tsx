import React, { useEffect, useState, useRef, useCallback, useMemo } from 'react';
import { View, StyleSheet, Animated, Easing } from 'react-native';
import Svg, { Path, Text as SvgText } from 'react-native-svg';
import type { WedgeValue } from '../types';

// Wheel colors matching the TV show
const WHEEL_COLORS = [
  '#c41e3a', '#0047ab', '#ff8c00', '#ffcc00', '#9932cc', '#ff1493',
  '#008b8b', '#dc143c', '#4169e1', '#ff4500', '#32cd32', '#9400d3',
  '#ff69b4', '#1e90ff', '#ffd700', '#00ced1', '#ff6347', '#8a2be2',
];

// Highlight colors for winning wedge
const HIGHLIGHT_COLOR = 'rgba(255, 255, 255, 0.6)';

interface AnimatedWheelProps {
  wheelSlots: WedgeValue[];
  lastSpinIndex: number | null;
  size?: number;
  onSpinComplete?: () => void;
  /** Called when spin starts */
  onSpinStart?: () => void;
  /** Whether to show highlight on the winning wedge */
  showWinningHighlight?: boolean;
  /** Duration of highlight animation in ms (default: 1500) */
  highlightDuration?: number;
  /** Number of highlight flashes (default: 3) */
  highlightFlashes?: number;
}

const AnimatedWheelComponent = ({
  wheelSlots,
  lastSpinIndex,
  size = 300,
  onSpinComplete,
  onSpinStart,
  showWinningHighlight = true,
  highlightDuration = 1500,
  highlightFlashes = 3,
}: AnimatedWheelProps): React.JSX.Element | null => {
  // Use Animated.Value for native driver support
  const rotationAnim = useRef(new Animated.Value(0)).current;
  const [_isSpinning, setIsSpinning] = useState(false);
  const [showHighlight, setShowHighlight] = useState(false);
  const highlightOpacityAnim = useRef(new Animated.Value(0)).current;
  const prevSpinIdx = useRef<number | null>(null);
  const targetRotationRef = useRef(0);

  // Highlight animation using Animated API for better performance
  const startHighlightAnimation = useCallback(() => {
    if (!showWinningHighlight) return;

    setShowHighlight(true);
    highlightOpacityAnim.setValue(0);

    // Create flash sequence
    const flashDuration = highlightDuration / (highlightFlashes * 2);
    const animations = [];

    for (let i = 0; i < highlightFlashes; i++) {
      // Flash on
      animations.push(
        Animated.timing(highlightOpacityAnim, {
          toValue: 1,
          duration: flashDuration,
          useNativeDriver: true,
        })
      );
      // Flash off
      animations.push(
        Animated.timing(highlightOpacityAnim, {
          toValue: 0,
          duration: flashDuration,
          useNativeDriver: true,
        })
      );
    }

    Animated.sequence(animations).start(() => {
      setShowHighlight(false);
      highlightOpacityAnim.setValue(0);
    });
  }, [showWinningHighlight, highlightDuration, highlightFlashes, highlightOpacityAnim]);

  // Animate wheel when spin index changes - using Animated API with native driver
  useEffect(() => {
    if (lastSpinIndex !== null && lastSpinIndex !== prevSpinIdx.current && wheelSlots.length > 0) {
      prevSpinIdx.current = lastSpinIndex;

      // Notify spin start
      setIsSpinning(true);
      setShowHighlight(false);
      onSpinStart?.();

      // Calculate target rotation - center the slot under the pointer
      const anglePerSlot = 360 / wheelSlots.length;
      const targetSlotAngle = lastSpinIndex * anglePerSlot + anglePerSlot / 2;
      const spins = 3;
      const currentRotation = targetRotationRef.current;
      const targetRotation = currentRotation + spins * 360 + (360 - targetSlotAngle - (currentRotation % 360));

      targetRotationRef.current = targetRotation;

      // Use Animated.timing with native driver for smooth 60fps animation
      Animated.timing(rotationAnim, {
        toValue: targetRotation,
        duration: 2500,
        easing: Easing.out(Easing.cubic),
        useNativeDriver: true,
      }).start((result) => {
        // Handle both real animations and Jest mocks (which may not provide result)
        if (!result || result.finished) {
          setIsSpinning(false);
          startHighlightAnimation();
          onSpinComplete?.();
        }
      });
    }
  }, [lastSpinIndex, wheelSlots.length, onSpinComplete, onSpinStart, rotationAnim, startHighlightAnimation]);

  // Memoize expensive calculations
  const radius = useMemo(() => size / 2 - 5, [size]);
  const centerX = useMemo(() => size / 2, [size]);
  const centerY = useMemo(() => size / 2, [size]);
  const pointerSize = useMemo(() => size / 300, [size]);

  // Memoize label formatter
  const getWedgeLabel = useCallback((slot: WedgeValue): string => {
    if (typeof slot === 'number') return `$${slot}`;
    if (typeof slot === 'string') return slot;
    if (slot?.type === 'PRIZE') return slot.name || 'PRIZE';
    if (slot?.type) return slot.type;
    return '';
  }, []);

  // Memoize wedge elements to avoid recalculation on every render
  const wedgeElements = useMemo(() => {
    if (wheelSlots.length === 0) return [];

    const elements: React.ReactElement[] = [];

    wheelSlots.forEach((slot, idx) => {
      const anglePerSlot = 360 / wheelSlots.length;
      const startAngle = idx * anglePerSlot - 90;
      const endAngle = startAngle + anglePerSlot;
      const startRad = (startAngle * Math.PI) / 180;
      const endRad = (endAngle * Math.PI) / 180;

      const x1 = centerX + radius * Math.cos(startRad);
      const y1 = centerY + radius * Math.sin(startRad);
      const x2 = centerX + radius * Math.cos(endRad);
      const y2 = centerY + radius * Math.sin(endRad);

      const largeArc = anglePerSlot > 180 ? 1 : 0;
      const pathD = `M ${centerX} ${centerY} L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2} Z`;

      const color = WHEEL_COLORS[idx % WHEEL_COLORS.length];
      const label = getWedgeLabel(slot);
      const isBankrupt = label === 'BANKRUPT';
      const isLoseTurn = label === 'LOSE A TURN';

      // Add wedge path
      elements.push(
        <Path
          key={`wedge-${idx}`}
          d={pathD}
          fill={isBankrupt ? '#000' : isLoseTurn ? '#fff' : color}
          stroke="#222"
          strokeWidth={1}
        />
      );

      // Add text label - position near outer edge where wedge is wider
      const midAngle = (startAngle + endAngle) / 2;
      const midRad = (midAngle * Math.PI) / 180;
      const textRadius = radius * 0.78;
      const textX = centerX + textRadius * Math.cos(midRad);
      const textY = centerY + textRadius * Math.sin(midRad);

      // Rotate text to read outward from center, flip if on left side of wheel
      const normalizedAngle = ((midAngle % 360) + 360) % 360;
      let rotation = midAngle;
      if (normalizedAngle > 90 && normalizedAngle < 270) {
        rotation = midAngle + 180;
      }

      // Scale font size based on wheel size and label length
      const baseSize = size / 300;
      const textColor = isBankrupt ? '#fff' : isLoseTurn ? '#000' : '#000';
      const fontSize = (label.length > 8 ? 6 : label.length > 5 ? 7 : 9) * baseSize;

      elements.push(
        <SvgText
          key={`text-${idx}`}
          x={textX}
          y={textY}
          fill={textColor}
          fontSize={fontSize}
          fontWeight="bold"
          textAnchor="middle"
          alignmentBaseline="middle"
          transform={`rotate(${rotation}, ${textX}, ${textY})`}
        >
          {label}
        </SvgText>
      );
    });

    return elements;
  }, [wheelSlots, size, centerX, centerY, radius, getWedgeLabel]);

  // Separate highlight overlay to avoid re-rendering wedges
  const highlightOverlay = useMemo(() => {
    if (!showHighlight || lastSpinIndex === null || wheelSlots.length === 0) return null;

    const anglePerSlot = 360 / wheelSlots.length;
    const startAngle = lastSpinIndex * anglePerSlot - 90;
    const endAngle = startAngle + anglePerSlot;
    const startRad = (startAngle * Math.PI) / 180;
    const endRad = (endAngle * Math.PI) / 180;

    const x1 = centerX + radius * Math.cos(startRad);
    const y1 = centerY + radius * Math.sin(startRad);
    const x2 = centerX + radius * Math.cos(endRad);
    const y2 = centerY + radius * Math.sin(endRad);

    const largeArc = anglePerSlot > 180 ? 1 : 0;
    const pathD = `M ${centerX} ${centerY} L ${x1} ${y1} A ${radius} ${radius} 0 ${largeArc} 1 ${x2} ${y2} Z`;

    return (
      <Animated.View
        style={{
          position: 'absolute',
          opacity: highlightOpacityAnim,
        }}
      >
        <Svg width={size} height={size}>
          <Path d={pathD} fill={HIGHLIGHT_COLOR} />
        </Svg>
      </Animated.View>
    );
  }, [showHighlight, lastSpinIndex, wheelSlots.length, centerX, centerY, radius, size, highlightOpacityAnim]);

  if (wheelSlots.length === 0) return null;

  return (
    <View style={styles.wheelContainer}>
      <View
        style={[
          styles.wheelPointer,
          {
            borderLeftWidth: 12 * pointerSize,
            borderRightWidth: 12 * pointerSize,
            borderTopWidth: 20 * pointerSize,
          },
        ]}
      />
      <Animated.View
        style={{
          width: size,
          height: size,
          transform: [
            {
              rotate: rotationAnim.interpolate({
                inputRange: [0, 360],
                outputRange: ['0deg', '360deg'],
              }),
            },
          ],
        }}
      >
        <Svg width={size} height={size}>
          {wedgeElements}
        </Svg>
        {highlightOverlay}
      </Animated.View>
    </View>
  );
};

// Wrap with React.memo to prevent unnecessary re-renders
export const AnimatedWheel = React.memo(AnimatedWheelComponent);

const styles = StyleSheet.create({
  wheelContainer: {
    alignItems: 'center',
    position: 'relative',
  },
  wheelPointer: {
    position: 'absolute',
    top: -10,
    zIndex: 10,
    width: 0,
    height: 0,
    borderLeftColor: 'transparent',
    borderRightColor: 'transparent',
    borderTopColor: '#d4af37',
  },
});
