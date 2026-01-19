import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Text } from 'react-native';
import {
  GlassCard,
  GlassCardHeader,
  GlassCardContent,
  GlassCardFooter,
} from '../ui/GlassCard';

describe('GlassCard', () => {
  describe('rendering', () => {
    it('renders with children', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard>
            <Text>Card Content</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays children correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard>
            <Text>Test Content</Text>
          </GlassCard>
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const content = texts?.find((t) => t.props.children === 'Test Content');

      expect(content).toBeDefined();
    });

    it('passes testID correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard testID="test-card">
            <Text>Content</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByProps({ testID: 'test-card' });
      expect(view).toBeDefined();
    });
  });

  describe('padding variants', () => {
    it('renders with no padding', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard padding="none">
            <Text>No Padding</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByType('View' as never);
      const style = Array.isArray(view?.props.style)
        ? view?.props.style.flat()
        : [view?.props.style];

      const hasPadding = style.some((s: { padding?: number }) => s?.padding === 0);
      expect(hasPadding).toBe(true);
    });

    it('renders with small padding', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard padding="sm">
            <Text>Small Padding</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with medium padding (default)', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard padding="md">
            <Text>Medium Padding</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with large padding', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard padding="lg">
            <Text>Large Padding</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with extra large padding', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard padding="xl">
            <Text>Extra Large Padding</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('border radius variants', () => {
    it('renders with small radius', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard radius="sm">
            <Text>Small Radius</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with medium radius (default)', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard radius="md">
            <Text>Medium Radius</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with large radius', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard radius="lg">
            <Text>Large Radius</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with extra large radius', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard radius="xl">
            <Text>Extra Large Radius</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('border styling', () => {
    it('shows border by default', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard>
            <Text>With Border</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByType('View' as never);
      const style = Array.isArray(view?.props.style)
        ? view?.props.style.flat()
        : [view?.props.style];

      const hasBorder = style.some((s: { borderWidth?: number }) => s?.borderWidth);
      expect(hasBorder).toBe(true);
    });

    it('hides border when showBorder is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard showBorder={false}>
            <Text>No Border</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts custom border color', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard borderColor="#ff0000">
            <Text>Custom Border</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByType('View' as never);
      const style = Array.isArray(view?.props.style)
        ? view?.props.style.flat()
        : [view?.props.style];

      const hasBorderColor = style.some(
        (s: { borderColor?: string }) => s?.borderColor === '#ff0000'
      );
      expect(hasBorderColor).toBe(true);
    });
  });

  describe('background color', () => {
    it('uses default background color', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard>
            <Text>Default Background</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts custom background color', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard backgroundColor="#00ff00">
            <Text>Custom Background</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByType('View' as never);
      const style = Array.isArray(view?.props.style)
        ? view?.props.style.flat()
        : [view?.props.style];

      const hasBackgroundColor = style.some(
        (s: { backgroundColor?: string }) => s?.backgroundColor === '#00ff00'
      );
      expect(hasBackgroundColor).toBe(true);
    });
  });

  describe('shadow styling', () => {
    it('shows shadow by default', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard>
            <Text>With Shadow</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('hides shadow when showShadow is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard showShadow={false}>
            <Text>No Shadow</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts small shadow size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard shadowSize="sm">
            <Text>Small Shadow</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts medium shadow size (default)', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard shadowSize="md">
            <Text>Medium Shadow</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts large shadow size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard shadowSize="lg">
            <Text>Large Shadow</Text>
          </GlassCard>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('custom styling', () => {
    it('accepts custom style prop', () => {
      const customStyle = { margin: 20 };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GlassCard style={customStyle}>
            <Text>Custom Style</Text>
          </GlassCard>
        );
      });

      const view = tree?.root.findByType('View' as never);
      const style = Array.isArray(view?.props.style)
        ? view?.props.style.flat()
        : [view?.props.style];

      const hasMargin = style.some((s: { margin?: number }) => s?.margin === 20);
      expect(hasMargin).toBe(true);
    });
  });
});

describe('GlassCardHeader', () => {
  it('renders with children', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardHeader>
          <Text>Header Content</Text>
        </GlassCardHeader>
      );
    });

    expect(tree?.toJSON()).not.toBeNull();
  });

  it('displays children correctly', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardHeader>
          <Text>Header Title</Text>
        </GlassCardHeader>
      );
    });

    const texts = tree?.root.findAllByType('Text' as never);
    const header = texts?.find((t) => t.props.children === 'Header Title');

    expect(header).toBeDefined();
  });

  it('accepts custom style', () => {
    const customStyle = { paddingTop: 20 };
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardHeader style={customStyle}>
          <Text>Styled Header</Text>
        </GlassCardHeader>
      );
    });

    const view = tree?.root.findByType('View' as never);
    const style = Array.isArray(view?.props.style)
      ? view?.props.style.flat()
      : [view?.props.style];

    const hasPadding = style.some((s: { paddingTop?: number }) => s?.paddingTop === 20);
    expect(hasPadding).toBe(true);
  });
});

describe('GlassCardContent', () => {
  it('renders with children', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardContent>
          <Text>Content</Text>
        </GlassCardContent>
      );
    });

    expect(tree?.toJSON()).not.toBeNull();
  });

  it('displays children correctly', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardContent>
          <Text>Main Content</Text>
        </GlassCardContent>
      );
    });

    const texts = tree?.root.findAllByType('Text' as never);
    const content = texts?.find((t) => t.props.children === 'Main Content');

    expect(content).toBeDefined();
  });

  it('accepts custom style', () => {
    const customStyle = { padding: 10 };
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardContent style={customStyle}>
          <Text>Styled Content</Text>
        </GlassCardContent>
      );
    });

    const view = tree?.root.findByType('View' as never);
    const style = Array.isArray(view?.props.style)
      ? view?.props.style.flat()
      : [view?.props.style];

    const hasPadding = style.some((s: { padding?: number }) => s?.padding === 10);
    expect(hasPadding).toBe(true);
  });

  it('has flex: 1 style', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardContent>
          <Text>Content</Text>
        </GlassCardContent>
      );
    });

    const view = tree?.root.findByType('View' as never);
    const style = Array.isArray(view?.props.style)
      ? view?.props.style.flat()
      : [view?.props.style];

    const hasFlex = style.some((s: { flex?: number }) => s?.flex === 1);
    expect(hasFlex).toBe(true);
  });
});

describe('GlassCardFooter', () => {
  it('renders with children', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardFooter>
          <Text>Footer</Text>
        </GlassCardFooter>
      );
    });

    expect(tree?.toJSON()).not.toBeNull();
  });

  it('displays children correctly', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardFooter>
          <Text>Footer Content</Text>
        </GlassCardFooter>
      );
    });

    const texts = tree?.root.findAllByType('Text' as never);
    const footer = texts?.find((t) => t.props.children === 'Footer Content');

    expect(footer).toBeDefined();
  });

  it('accepts custom style', () => {
    const customStyle = { paddingBottom: 20 };
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardFooter style={customStyle}>
          <Text>Styled Footer</Text>
        </GlassCardFooter>
      );
    });

    const view = tree?.root.findByType('View' as never);
    const style = Array.isArray(view?.props.style)
      ? view?.props.style.flat()
      : [view?.props.style];

    const hasPadding = style.some((s: { paddingBottom?: number }) => s?.paddingBottom === 20);
    expect(hasPadding).toBe(true);
  });

  it('has border top styling', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCardFooter>
          <Text>Footer</Text>
        </GlassCardFooter>
      );
    });

    const view = tree?.root.findByType('View' as never);
    const style = Array.isArray(view?.props.style)
      ? view?.props.style.flat()
      : [view?.props.style];

    const hasBorderTop = style.some(
      (s: { borderTopWidth?: number }) => s?.borderTopWidth === 1
    );
    expect(hasBorderTop).toBe(true);
  });
});

describe('GlassCard composition', () => {
  it('renders card with header, content, and footer', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(
        <GlassCard>
          <GlassCardHeader>
            <Text>Header</Text>
          </GlassCardHeader>
          <GlassCardContent>
            <Text>Content</Text>
          </GlassCardContent>
          <GlassCardFooter>
            <Text>Footer</Text>
          </GlassCardFooter>
        </GlassCard>
      );
    });

    const texts = tree?.root.findAllByType('Text' as never);
    expect(texts?.length).toBeGreaterThanOrEqual(3);

    const hasHeader = texts?.some((t) => t.props.children === 'Header');
    const hasContent = texts?.some((t) => t.props.children === 'Content');
    const hasFooter = texts?.some((t) => t.props.children === 'Footer');

    expect(hasHeader).toBe(true);
    expect(hasContent).toBe(true);
    expect(hasFooter).toBe(true);
  });
});
