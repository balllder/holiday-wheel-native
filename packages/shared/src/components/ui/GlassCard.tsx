import React from 'react';
import { View, StyleSheet, ViewStyle, StyleProp } from 'react-native';
import { theme } from '../../constants/theme';

export interface GlassCardProps {
  /** Child components to render inside the card */
  children: React.ReactNode;
  /** Custom container style */
  style?: StyleProp<ViewStyle>;
  /** Card padding size */
  padding?: 'none' | 'sm' | 'md' | 'lg' | 'xl';
  /** Border radius size */
  radius?: 'sm' | 'md' | 'lg' | 'xl';
  /** Whether to show the gold border */
  showBorder?: boolean;
  /** Custom border color */
  borderColor?: string;
  /** Custom background color */
  backgroundColor?: string;
  /** Whether to show shadow */
  showShadow?: boolean;
  /** Shadow intensity */
  shadowSize?: 'sm' | 'md' | 'lg';
  /** Test ID for testing */
  testID?: string;
}

/**
 * GlassCard - Glassmorphism card component
 *
 * Creates a semi-transparent card with blur effect (simulated),
 * subtle border, and optional shadow. Perfect for overlays and
 * content containers in the modern UI theme.
 *
 * Note: True blur effects require native libraries like
 * @react-native-community/blur. This implementation uses
 * opacity and colors to simulate the glassmorphism effect.
 *
 * @example
 * <GlassCard>
 *   <Text>Card content</Text>
 * </GlassCard>
 *
 * <GlassCard
 *   padding="lg"
 *   radius="xl"
 *   showBorder
 *   showShadow
 *   shadowSize="lg"
 * >
 *   <ScoreDisplay score={1000} />
 * </GlassCard>
 */
export function GlassCard({
  children,
  style,
  padding = 'md',
  radius = 'md',
  showBorder = true,
  borderColor = theme.colors.cardBorder,
  backgroundColor = theme.colors.card,
  showShadow = true,
  shadowSize = 'md',
  testID,
}: GlassCardProps): React.JSX.Element {
  const paddingValue = getPaddingValue(padding);
  const borderRadiusValue = theme.borderRadius[radius];
  const shadowStyle = showShadow ? theme.shadows[shadowSize] : {};

  return (
    <View
      style={[
        styles.container,
        {
          backgroundColor,
          borderRadius: borderRadiusValue,
          padding: paddingValue,
        },
        showBorder && {
          borderWidth: theme.glass.borderWidth,
          borderColor,
        },
        shadowStyle,
        style,
      ]}
      testID={testID}
    >
      {children}
    </View>
  );
}

/**
 * GlassCardHeader - Header section for GlassCard
 */
export function GlassCardHeader({
  children,
  style,
}: {
  children: React.ReactNode;
  style?: StyleProp<ViewStyle>;
}): React.JSX.Element {
  return <View style={[styles.header, style]}>{children}</View>;
}

/**
 * GlassCardContent - Content section for GlassCard
 */
export function GlassCardContent({
  children,
  style,
}: {
  children: React.ReactNode;
  style?: StyleProp<ViewStyle>;
}): React.JSX.Element {
  return <View style={[styles.content, style]}>{children}</View>;
}

/**
 * GlassCardFooter - Footer section for GlassCard
 */
export function GlassCardFooter({
  children,
  style,
}: {
  children: React.ReactNode;
  style?: StyleProp<ViewStyle>;
}): React.JSX.Element {
  return <View style={[styles.footer, style]}>{children}</View>;
}

const getPaddingValue = (padding: GlassCardProps['padding']): number => {
  switch (padding) {
    case 'none':
      return 0;
    case 'sm':
      return theme.spacing.sm;
    case 'md':
      return theme.spacing.md;
    case 'lg':
      return theme.spacing.lg;
    case 'xl':
      return theme.spacing.xl;
    default:
      return theme.spacing.md;
  }
};

const styles = StyleSheet.create({
  container: {
    overflow: 'hidden',
  },
  header: {
    marginBottom: theme.spacing.md,
  },
  content: {
    flex: 1,
  },
  footer: {
    marginTop: theme.spacing.md,
    paddingTop: theme.spacing.md,
    borderTopWidth: 1,
    borderTopColor: theme.colors.glassBorder,
  },
});

export default GlassCard;
