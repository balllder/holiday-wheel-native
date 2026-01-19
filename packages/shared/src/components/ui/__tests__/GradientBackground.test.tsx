import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Text } from 'react-native';
import { GradientBackground, SimpleGradient } from '../GradientBackground';
import { theme } from '../../../constants/theme';

describe('GradientBackground', () => {
  describe('rendering', () => {
    it('renders children correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground>
            <Text>Test Content</Text>
          </GradientBackground>
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      expect(texts?.some((t) => t.props.children === 'Test Content')).toBe(true);
    });

    it('renders with testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground testID="gradient-bg">
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const container = tree?.root.findByProps({ testID: 'gradient-bg' });
      expect(container).toBeDefined();
    });

    it('renders gradient layers', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      // Should have multiple View elements for gradient layers
      const views = tree?.root.findAllByType('View' as never);
      expect(views?.length).toBeGreaterThanOrEqual(3); // container + 2 layers + content
    });
  });

  describe('props', () => {
    it('uses default theme colors', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(theme.colors.background.end);
      expect(json).toContain(theme.colors.background.start);
    });

    it('accepts custom startColor', () => {
      const customColor = '#ff0000';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground startColor={customColor}>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(customColor);
    });

    it('accepts custom endColor', () => {
      const customColor = '#00ff00';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground endColor={customColor}>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(customColor);
    });

    it('accepts both custom colors', () => {
      const startColor = '#ff0000';
      const endColor = '#0000ff';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground startColor={startColor} endColor={endColor}>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(startColor);
      expect(json).toContain(endColor);
    });

    it('applies fillScreen style when true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground fillScreen>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      // fillScreen adds flex: 1
      expect(json).toContain('"flex":1');
    });

    it('does not apply fillScreen style when false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground fillScreen={false}>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      // Get root container - it should not have flex: 1 in its direct style
      const root = tree?.root;
      const container = root?.findAllByType('View' as never)[0];
      const style = Array.isArray(container?.props.style)
        ? container?.props.style.flat().filter(Boolean)
        : [container?.props.style];

      // fillScreen false means the fillScreen style object won't be included
      const hasFillScreenStyle = style.some(
        (s: { flex?: number }) => s?.flex === 1 && Object.keys(s).length === 1
      );
      expect(hasFillScreenStyle).toBe(false);
    });

    it('applies custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground style={{ padding: 20 }}>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('"padding":20');
    });
  });

  describe('structure', () => {
    it('has correct layer order (base, top, content)', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GradientBackground>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const views = tree?.root.findAllByType('View' as never);
      // First view is container, then layers
      expect(views?.length).toBeGreaterThanOrEqual(4);
    });
  });
});

describe('SimpleGradient', () => {
  describe('rendering', () => {
    it('renders children correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient>
            <Text>Simple Content</Text>
          </SimpleGradient>
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      expect(texts?.some((t) => t.props.children === 'Simple Content')).toBe(true);
    });

    it('renders with testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient testID="simple-gradient">
            <Text>Content</Text>
          </SimpleGradient>
        );
      });

      const container = tree?.root.findByProps({ testID: 'simple-gradient' });
      expect(container).toBeDefined();
    });

    it('renders simpler structure than GradientBackground', () => {
      let simpleTree: ReactTestRenderer | undefined;
      let gradientTree: ReactTestRenderer | undefined;

      act(() => {
        simpleTree = create(
          <SimpleGradient>
            <Text>Content</Text>
          </SimpleGradient>
        );
        gradientTree = create(
          <GradientBackground>
            <Text>Content</Text>
          </GradientBackground>
        );
      });

      const simpleViews = simpleTree?.root.findAllByType('View' as never);
      const gradientViews = gradientTree?.root.findAllByType('View' as never);

      // SimpleGradient should have fewer views (no gradient layers)
      expect(simpleViews?.length).toBeLessThan(gradientViews?.length || 0);
    });
  });

  describe('props', () => {
    it('uses default theme end color', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient>
            <Text>Content</Text>
          </SimpleGradient>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(theme.colors.background.end);
    });

    it('accepts custom color', () => {
      const customColor = '#123456';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient color={customColor}>
            <Text>Content</Text>
          </SimpleGradient>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain(customColor);
    });

    it('applies fillScreen style when true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient fillScreen>
            <Text>Content</Text>
          </SimpleGradient>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('"flex":1');
    });

    it('applies custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <SimpleGradient style={{ margin: 10 }}>
            <Text>Content</Text>
          </SimpleGradient>
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('"margin":10');
    });
  });
});
