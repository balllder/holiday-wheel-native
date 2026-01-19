import React from 'react';
import { View, StyleSheet, ViewStyle, StyleProp } from 'react-native';
import { theme } from '../../constants/theme';

export interface GradientBackgroundProps {
  /** Child components to render inside the gradient */
  children: React.ReactNode;
  /** Custom container style */
  style?: StyleProp<ViewStyle>;
  /** Custom start color (overrides theme) */
  startColor?: string;
  /** Custom end color (overrides theme) */
  endColor?: string;
  /** Whether to fill the screen (flex: 1) */
  fillScreen?: boolean;
  /** Test ID for testing */
  testID?: string;
}

/**
 * GradientBackground - Linear gradient background wrapper
 *
 * Note: This component uses a simple View-based gradient approximation
 * since expo-linear-gradient or react-native-linear-gradient may not
 * be available. For true gradient support, consider using those libraries.
 *
 * This implementation creates a layered background effect using
 * absolute positioned views with opacity.
 *
 * @example
 * <GradientBackground fillScreen>
 *   <Text>Content on gradient</Text>
 * </GradientBackground>
 *
 * <GradientBackground
 *   startColor="#1a0a3e"
 *   endColor="#0d0628"
 *   style={{ padding: 16 }}
 * >
 *   <Card>...</Card>
 * </GradientBackground>
 */
export function GradientBackground({
  children,
  style,
  startColor = theme.colors.background.start,
  endColor = theme.colors.background.end,
  fillScreen = false,
  testID,
}: GradientBackgroundProps): React.JSX.Element {
  return (
    <View
      style={[styles.container, fillScreen && styles.fillScreen, style]}
      testID={testID}
    >
      {/* Background layers to simulate gradient */}
      <View style={[styles.gradientLayer, styles.baseLayer, { backgroundColor: endColor }]} />
      <View
        style={[
          styles.gradientLayer,
          styles.topLayer,
          { backgroundColor: startColor },
        ]}
      />
      {/* Content */}
      <View style={styles.content}>{children}</View>
    </View>
  );
}

/**
 * SimpleGradient - A simpler gradient using just the end color
 * Use this when you need a solid background matching the theme
 */
export function SimpleGradient({
  children,
  style,
  color = theme.colors.background.end,
  fillScreen = false,
  testID,
}: Omit<GradientBackgroundProps, 'startColor' | 'endColor'> & {
  color?: string;
}): React.JSX.Element {
  return (
    <View
      style={[
        styles.container,
        fillScreen && styles.fillScreen,
        { backgroundColor: color },
        style,
      ]}
      testID={testID}
    >
      {children}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    position: 'relative',
    overflow: 'hidden',
  },
  fillScreen: {
    flex: 1,
  },
  gradientLayer: {
    ...StyleSheet.absoluteFillObject,
  },
  baseLayer: {
    zIndex: 0,
  },
  topLayer: {
    zIndex: 1,
    opacity: 0.7,
    // Create a fade effect from top
    transform: [{ translateY: -100 }],
  },
  content: {
    position: 'relative',
    zIndex: 2,
    flex: 1,
  },
});

export default GradientBackground;
