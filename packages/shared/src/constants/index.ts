// Constants matching the Flask backend

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
