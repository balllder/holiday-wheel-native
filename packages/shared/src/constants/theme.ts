/**
 * Modern UI Theme System
 *
 * Centralized theme constants for consistent styling across the app.
 * Provides colors, spacing, border radius, and shadow definitions.
 */

export const theme = {
  colors: {
    // Background gradient colors
    background: {
      start: '#1a0a3e',
      end: '#0d0628',
    },

    // Card/Surface colors
    card: 'rgba(26, 10, 62, 0.8)',
    cardBorder: 'rgba(212, 175, 55, 0.3)',
    cardBorderHover: 'rgba(212, 175, 55, 0.5)',

    // Primary gold colors
    gold: '#d4af37',
    goldLight: '#ffd700',
    goldDark: '#b8860b',
    goldGlow: 'rgba(212, 175, 55, 0.4)',

    // Accent colors
    accent: '#6366f1',
    accentLight: '#818cf8',
    accentDark: '#4f46e5',
    accentGlow: 'rgba(99, 102, 241, 0.4)',

    // Semantic colors
    success: '#22c55e',
    successGlow: 'rgba(34, 197, 94, 0.4)',
    danger: '#ef4444',
    dangerGlow: 'rgba(239, 68, 68, 0.4)',
    warning: '#f59e0b',
    warningGlow: 'rgba(245, 158, 11, 0.4)',
    info: '#3b82f6',
    infoGlow: 'rgba(59, 130, 246, 0.4)',

    // Text colors
    text: '#ffffff',
    textSecondary: 'rgba(255, 255, 255, 0.7)',
    textMuted: 'rgba(255, 255, 255, 0.5)',
    textOnGold: '#1a0a3e',

    // Glass effect colors
    glassBorder: 'rgba(255, 255, 255, 0.1)',
    glassBackground: 'rgba(26, 10, 62, 0.7)',
  },

  // Spacing scale (in pixels)
  spacing: {
    xs: 4,
    sm: 8,
    md: 16,
    lg: 24,
    xl: 32,
    '2xl': 48,
    '3xl': 64,
  },

  // Border radius scale
  borderRadius: {
    sm: 8,
    md: 12,
    lg: 16,
    xl: 24,
    '2xl': 32,
    full: 9999,
  },

  // Shadow definitions for React Native
  shadows: {
    sm: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 2 },
      shadowOpacity: 0.25,
      shadowRadius: 4,
      elevation: 3,
    },
    md: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 4 },
      shadowOpacity: 0.3,
      shadowRadius: 8,
      elevation: 6,
    },
    lg: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 8 },
      shadowOpacity: 0.4,
      shadowRadius: 16,
      elevation: 12,
    },
    // Glow shadows
    goldGlow: {
      shadowColor: '#d4af37',
      shadowOffset: { width: 0, height: 0 },
      shadowOpacity: 0.6,
      shadowRadius: 15,
      elevation: 10,
    },
    accentGlow: {
      shadowColor: '#6366f1',
      shadowOffset: { width: 0, height: 0 },
      shadowOpacity: 0.6,
      shadowRadius: 15,
      elevation: 10,
    },
    dangerGlow: {
      shadowColor: '#ef4444',
      shadowOffset: { width: 0, height: 0 },
      shadowOpacity: 0.6,
      shadowRadius: 15,
      elevation: 10,
    },
    successGlow: {
      shadowColor: '#22c55e',
      shadowOffset: { width: 0, height: 0 },
      shadowOpacity: 0.6,
      shadowRadius: 15,
      elevation: 10,
    },
  },

  // Typography scale
  typography: {
    fontSize: {
      xs: 10,
      sm: 12,
      md: 14,
      lg: 16,
      xl: 18,
      '2xl': 24,
      '3xl': 30,
      '4xl': 36,
    },
    fontWeight: {
      normal: '400' as const,
      medium: '500' as const,
      semibold: '600' as const,
      bold: '700' as const,
    },
  },

  // Animation durations (in ms)
  animation: {
    fast: 150,
    normal: 300,
    slow: 500,
  },

  // Glass effect configuration
  glass: {
    blur: 10,
    borderWidth: 1,
  },
} as const;

// Type exports for TypeScript consumers
export type Theme = typeof theme;
export type ThemeColors = typeof theme.colors;
export type ThemeSpacing = typeof theme.spacing;
export type ThemeBorderRadius = typeof theme.borderRadius;
export type ThemeShadows = typeof theme.shadows;
