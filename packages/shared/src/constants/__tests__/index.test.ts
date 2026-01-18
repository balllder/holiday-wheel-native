import {
  ALPHABET,
  VOWELS,
  CONSONANTS,
  ROW_WIDTHS,
  DEFAULT_VOWEL_COST,
  DEFAULT_FINAL_SECONDS,
  DEFAULT_FINAL_JACKPOT,
  FINAL_RSTLNE,
  WHEEL_COLORS,
  SPECIAL_COLORS,
  THEME,
  MODERN_THEME,
} from '../index';

describe('Constants', () => {
  describe('ALPHABET', () => {
    it('contains all 26 letters', () => {
      expect(ALPHABET).toBe('ABCDEFGHIJKLMNOPQRSTUVWXYZ');
      expect(ALPHABET.length).toBe(26);
    });
  });

  describe('VOWELS', () => {
    it('contains AEIOU', () => {
      expect(VOWELS).toBe('AEIOU');
      expect(VOWELS.length).toBe(5);
    });
  });

  describe('CONSONANTS', () => {
    it('contains 21 consonants', () => {
      expect(CONSONANTS.length).toBe(21);
      expect(CONSONANTS).not.toContain('A');
      expect(CONSONANTS).not.toContain('E');
      expect(CONSONANTS).not.toContain('I');
      expect(CONSONANTS).not.toContain('O');
      expect(CONSONANTS).not.toContain('U');
    });
  });

  describe('ROW_WIDTHS', () => {
    it('defines 4 rows', () => {
      expect(ROW_WIDTHS).toHaveLength(4);
      expect(ROW_WIDTHS).toEqual([12, 14, 14, 12]);
    });
  });

  describe('Default values', () => {
    it('defines vowel cost', () => {
      expect(DEFAULT_VOWEL_COST).toBe(250);
    });

    it('defines final round seconds', () => {
      expect(DEFAULT_FINAL_SECONDS).toBe(30);
    });

    it('defines final jackpot', () => {
      expect(DEFAULT_FINAL_JACKPOT).toBe(10000);
    });
  });

  describe('FINAL_RSTLNE', () => {
    it('contains the free letters R, S, T, L, N, E', () => {
      expect(FINAL_RSTLNE).toEqual(['R', 'S', 'T', 'L', 'N', 'E']);
    });
  });

  describe('WHEEL_COLORS', () => {
    it('defines 20 colors', () => {
      expect(WHEEL_COLORS).toHaveLength(20);
    });

    it('contains valid hex colors', () => {
      WHEEL_COLORS.forEach((color) => {
        expect(color).toMatch(/^#[0-9a-fA-F]{6}$/);
      });
    });
  });

  describe('SPECIAL_COLORS', () => {
    it('defines special wedge colors', () => {
      expect(SPECIAL_COLORS.BANKRUPT).toBe('#000000');
      expect(SPECIAL_COLORS['LOSE A TURN']).toBe('#ffffff');
      expect(SPECIAL_COLORS['FREE PLAY']).toBe('#39ff14');
      expect(SPECIAL_COLORS.PRIZE).toBe('#c0c0c0');
    });
  });
});

describe('THEME', () => {
  it('has dark and light themes', () => {
    expect(THEME.dark).toBeDefined();
    expect(THEME.light).toBeDefined();
  });

  describe('dark theme', () => {
    it('has required color properties', () => {
      expect(THEME.dark.bg).toBeDefined();
      expect(THEME.dark.card).toBeDefined();
      expect(THEME.dark.text).toBeDefined();
      expect(THEME.dark.gold).toBeDefined();
      expect(THEME.dark.accent).toBeDefined();
      expect(THEME.dark.boardBg).toBeDefined();
    });
  });

  describe('light theme', () => {
    it('has required color properties', () => {
      expect(THEME.light.bg).toBeDefined();
      expect(THEME.light.card).toBeDefined();
      expect(THEME.light.text).toBeDefined();
      expect(THEME.light.gold).toBeDefined();
      expect(THEME.light.accent).toBeDefined();
      expect(THEME.light.boardBg).toBeDefined();
    });
  });
});

describe('MODERN_THEME', () => {
  describe('colors', () => {
    it('has primary colors', () => {
      expect(MODERN_THEME.colors.primary).toBe('#d4af37');
      expect(MODERN_THEME.colors.primaryLight).toBeDefined();
      expect(MODERN_THEME.colors.primaryDark).toBeDefined();
      expect(MODERN_THEME.colors.primaryGlow).toBeDefined();
    });

    it('has accent colors', () => {
      expect(MODERN_THEME.colors.accent).toBe('#6366f1');
      expect(MODERN_THEME.colors.accentLight).toBeDefined();
      expect(MODERN_THEME.colors.accentDark).toBeDefined();
    });

    it('has semantic colors', () => {
      expect(MODERN_THEME.colors.success).toBeDefined();
      expect(MODERN_THEME.colors.danger).toBeDefined();
      expect(MODERN_THEME.colors.warning).toBeDefined();
      expect(MODERN_THEME.colors.info).toBeDefined();
    });

    it('has neutral colors', () => {
      expect(MODERN_THEME.colors.background).toBeDefined();
      expect(MODERN_THEME.colors.surface).toBeDefined();
      expect(MODERN_THEME.colors.border).toBeDefined();
    });

    it('has text colors', () => {
      expect(MODERN_THEME.colors.text).toBeDefined();
      expect(MODERN_THEME.colors.textSecondary).toBeDefined();
      expect(MODERN_THEME.colors.textMuted).toBeDefined();
    });

    it('has game-specific colors', () => {
      expect(MODERN_THEME.colors.boardBg).toBeDefined();
      expect(MODERN_THEME.colors.boardFrame).toBeDefined();
      expect(MODERN_THEME.colors.emptyCell).toBeDefined();
      expect(MODERN_THEME.colors.letterTile).toBeDefined();
    });
  });

  describe('gradients', () => {
    it('has background gradients', () => {
      expect(MODERN_THEME.gradients.backgroundPrimary).toBeDefined();
      expect(MODERN_THEME.gradients.backgroundPrimary.colors).toHaveLength(2);
      expect(MODERN_THEME.gradients.backgroundPrimary.start).toBeDefined();
      expect(MODERN_THEME.gradients.backgroundPrimary.end).toBeDefined();
    });

    it('has button gradients', () => {
      expect(MODERN_THEME.gradients.buttonPrimary).toBeDefined();
      expect(MODERN_THEME.gradients.buttonSecondary).toBeDefined();
      expect(MODERN_THEME.gradients.buttonDanger).toBeDefined();
      expect(MODERN_THEME.gradients.buttonSuccess).toBeDefined();
    });
  });

  describe('glass', () => {
    it('has glassmorphism properties', () => {
      expect(MODERN_THEME.glass.background).toBeDefined();
      expect(MODERN_THEME.glass.blur).toBe(10);
      expect(MODERN_THEME.glass.borderColor).toBeDefined();
      expect(MODERN_THEME.glass.borderWidth).toBe(1);
    });
  });

  describe('shadows', () => {
    it('has elevation shadows', () => {
      expect(MODERN_THEME.shadows.small).toBeDefined();
      expect(MODERN_THEME.shadows.medium).toBeDefined();
      expect(MODERN_THEME.shadows.large).toBeDefined();
    });

    it('has glow shadows', () => {
      expect(MODERN_THEME.shadows.primaryGlow).toBeDefined();
      expect(MODERN_THEME.shadows.accentGlow).toBeDefined();
      expect(MODERN_THEME.shadows.dangerGlow).toBeDefined();
    });

    it('shadow objects have required properties', () => {
      const shadow = MODERN_THEME.shadows.small;
      expect(shadow.shadowColor).toBeDefined();
      expect(shadow.shadowOffset).toBeDefined();
      expect(shadow.shadowOpacity).toBeDefined();
      expect(shadow.shadowRadius).toBeDefined();
      expect(shadow.elevation).toBeDefined();
    });
  });

  describe('typography', () => {
    it('has font size scales', () => {
      expect(MODERN_THEME.typography.fontSize.xs).toBe(10);
      expect(MODERN_THEME.typography.fontSize['6xl']).toBe(60);
    });

    it('has TV font size scales', () => {
      expect(MODERN_THEME.typography.fontSizeTV.xs).toBeGreaterThan(
        MODERN_THEME.typography.fontSize.xs
      );
    });

    it('has font weights', () => {
      expect(MODERN_THEME.typography.fontWeight.normal).toBe('400');
      expect(MODERN_THEME.typography.fontWeight.bold).toBe('700');
    });
  });

  describe('spacing', () => {
    it('has spacing scale', () => {
      expect(MODERN_THEME.spacing[0]).toBe(0);
      expect(MODERN_THEME.spacing[1]).toBe(4);
      expect(MODERN_THEME.spacing[4]).toBe(16);
    });
  });

  describe('borderRadius', () => {
    it('has border radius scale', () => {
      expect(MODERN_THEME.borderRadius.none).toBe(0);
      expect(MODERN_THEME.borderRadius.md).toBe(8);
      expect(MODERN_THEME.borderRadius.full).toBe(9999);
    });
  });

  describe('animation', () => {
    it('has animation duration scale', () => {
      expect(MODERN_THEME.animation.fast).toBeLessThan(MODERN_THEME.animation.normal);
      expect(MODERN_THEME.animation.normal).toBeLessThan(MODERN_THEME.animation.slow);
    });
  });

  describe('zIndex', () => {
    it('has z-index scale', () => {
      expect(MODERN_THEME.zIndex.base).toBe(0);
      expect(MODERN_THEME.zIndex.modal).toBeGreaterThan(MODERN_THEME.zIndex.dropdown);
      expect(MODERN_THEME.zIndex.toast).toBeGreaterThan(MODERN_THEME.zIndex.modal);
    });
  });
});
