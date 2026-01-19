// Constants matching the Flask backend

// Theme exports
export * from './theme';

export const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
export const VOWELS = 'AEIOU';
export const CONSONANTS = ALPHABET.split('').filter((c) => !VOWELS.includes(c)).join('');

// Puzzle board row widths (TV-authentic 4-row layout)
export const ROW_WIDTHS = [12, 14, 14, 12];

// Default config values
export const DEFAULT_VOWEL_COST = 250;
export const DEFAULT_FINAL_SECONDS = 30;
export const DEFAULT_FINAL_JACKPOT = 10000;

// Final round free letters
export const FINAL_RSTLNE = ['R', 'S', 'T', 'L', 'N', 'E'];

// Wheel colors (index-based, matching app.js WHEEL_COLORS)
export const WHEEL_COLORS = [
  '#e74c3c', // red
  '#3498db', // blue
  '#f39c12', // orange
  '#9b59b6', // purple
  '#1abc9c', // teal
  '#e91e63', // pink
  '#00bcd4', // cyan
  '#ff9800', // amber
  '#8bc34a', // light green
  '#795548', // brown
  '#607d8b', // blue grey
  '#ff5722', // deep orange
  '#673ab7', // deep purple
  '#009688', // dark teal
  '#ffc107', // yellow
  '#2196f3', // light blue
  '#4caf50', // green
  '#f44336', // bright red
  '#03a9f4', // sky blue
  '#cddc39', // lime
];

// Special wedge colors
export const SPECIAL_COLORS = {
  BANKRUPT: '#000000',
  'LOSE A TURN': '#ffffff',
  'FREE PLAY': '#39ff14', // neon green
  PRIZE: '#c0c0c0', // silver
};

// Theme colors (matching styles.css)
export const THEME = {
  dark: {
    bg: '#0d0628',
    card: '#1a0a3e',
    text: '#ffffff',
    muted: 'rgba(255,255,255,0.7)',
    gold: '#d4af37',
    goldLight: '#ffd700',
    accent: '#6c5ce7',
    boardBg: '#1a5cb8',
    boardFrame: '#d4af37',
    emptyCell: '#228b22',
    letterTile: '#ffffff',
    letterText: '#1a1a2e',
  },
  light: {
    bg: '#e8f4ff',
    card: '#ffffff',
    text: '#1a1a2e',
    muted: 'rgba(0,0,0,0.6)',
    gold: '#b8860b',
    goldLight: '#daa520',
    accent: '#5a4fcf',
    boardBg: '#1a5cb8',
    boardFrame: '#b8860b',
    emptyCell: '#228b22',
    letterTile: '#ffffff',
    letterText: '#1a1a2e',
  },
};

/**
 * Modern UI Theme - Enhanced visual design
 *
 * Features:
 * - Gradient backgrounds
 * - Glassmorphism effects
 * - Glow and shadow effects
 * - Better color hierarchy
 */
export const MODERN_THEME = {
  // Base colors
  colors: {
    // Primary palette
    primary: '#d4af37',
    primaryLight: '#ffd700',
    primaryDark: '#b8860b',
    primaryGlow: 'rgba(212, 175, 55, 0.4)',

    // Accent colors
    accent: '#6366f1', // Indigo
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

    // Neutral colors
    background: '#0d0628',
    backgroundLight: '#1a0a3e',
    surface: '#1a0a3e',
    surfaceLight: '#2a1a4e',
    border: '#333',
    borderLight: '#444',

    // Text colors
    text: '#ffffff',
    textSecondary: 'rgba(255, 255, 255, 0.7)',
    textMuted: 'rgba(255, 255, 255, 0.5)',
    textOnPrimary: '#1a0a3e',

    // Game-specific colors
    boardBg: '#1a5cb8',
    boardFrame: '#d4af37',
    emptyCell: '#228b22',
    letterTile: '#ffffff',
    letterText: '#1a1a2e',
    wheelPointer: '#d4af37',

    // Aliases for backward compatibility
    gold: '#d4af37',
    goldLight: '#ffd700',
  },

  // Gradient definitions (for use with LinearGradient components)
  gradients: {
    // Background gradients
    backgroundPrimary: {
      colors: ['#1a0a3e', '#0d0628'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },
    backgroundRadial: {
      colors: ['#2a1a4e', '#0d0628'],
      start: { x: 0.5, y: 0 },
      end: { x: 0.5, y: 1 },
    },

    // Button gradients
    buttonPrimary: {
      colors: ['#ffd700', '#d4af37'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },
    buttonSecondary: {
      colors: ['#6366f1', '#4f46e5'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },
    buttonDanger: {
      colors: ['#f87171', '#ef4444'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },
    buttonSuccess: {
      colors: ['#4ade80', '#22c55e'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },

    // Card gradients
    cardGlass: {
      colors: ['rgba(26, 10, 62, 0.8)', 'rgba(26, 10, 62, 0.6)'],
      start: { x: 0, y: 0 },
      end: { x: 0, y: 1 },
    },

    // Gold shimmer for highlights
    goldShimmer: {
      colors: ['#ffd700', '#d4af37', '#b8860b', '#d4af37', '#ffd700'],
      start: { x: 0, y: 0 },
      end: { x: 1, y: 0 },
    },
  },

  // Glassmorphism effects
  glass: {
    background: 'rgba(26, 10, 62, 0.7)',
    backgroundLight: 'rgba(42, 26, 78, 0.7)',
    blur: 10,
    borderColor: 'rgba(255, 255, 255, 0.1)',
    borderWidth: 1,
  },

  // Shadow definitions
  shadows: {
    // Elevation shadows
    small: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 2 },
      shadowOpacity: 0.25,
      shadowRadius: 4,
      elevation: 3,
    },
    medium: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 4 },
      shadowOpacity: 0.3,
      shadowRadius: 8,
      elevation: 6,
    },
    large: {
      shadowColor: '#000',
      shadowOffset: { width: 0, height: 8 },
      shadowOpacity: 0.4,
      shadowRadius: 16,
      elevation: 12,
    },

    // Glow shadows
    primaryGlow: {
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
    // Font sizes
    fontSize: {
      xs: 10,
      sm: 12,
      base: 14,
      md: 14,
      lg: 16,
      xl: 18,
      '2xl': 24,
      '3xl': 30,
      '4xl': 36,
      '5xl': 48,
      '6xl': 60,
    },
    // TV-specific larger sizes
    fontSizeTV: {
      xs: 14,
      sm: 18,
      md: 22,
      lg: 26,
      xl: 32,
      '2xl': 40,
      '3xl': 48,
      '4xl': 56,
      '5xl': 72,
      '6xl': 96,
    },
    // Font weights
    fontWeight: {
      normal: '400' as const,
      medium: '500' as const,
      semibold: '600' as const,
      bold: '700' as const,
      extrabold: '800' as const,
    },
    // Line heights
    lineHeight: {
      tight: 1.2,
      normal: 1.5,
      relaxed: 1.75,
    },
    // Letter spacing
    letterSpacing: {
      tight: -0.5,
      normal: 0,
      wide: 0.5,
      wider: 1,
      widest: 2,
    },
  },

  // Spacing scale (multiplier-based)
  spacing: {
    0: 0,
    1: 4,
    2: 8,
    3: 12,
    4: 16,
    5: 20,
    6: 24,
    8: 32,
    10: 40,
    12: 48,
    16: 64,
    20: 80,
    24: 96,
  },

  // Border radius scale
  borderRadius: {
    none: 0,
    sm: 4,
    md: 8,
    lg: 12,
    xl: 16,
    '2xl': 24,
    full: 9999,
  },

  // Animation durations
  animation: {
    fast: 150,
    normal: 300,
    slow: 500,
    slower: 800,
    slowest: 1000,
  },

  // Z-index scale
  zIndex: {
    base: 0,
    dropdown: 100,
    modal: 200,
    overlay: 300,
    tooltip: 400,
    toast: 500,
  },
};
