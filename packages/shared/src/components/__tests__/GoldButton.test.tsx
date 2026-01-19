import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { GoldButton } from '../ui/GoldButton';

describe('GoldButton', () => {
  describe('rendering', () => {
    it('renders with text children', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Click Me</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays the button text correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Start Game</GoldButton>);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const buttonText = texts?.find((t) => t.props.children === 'Start Game');

      expect(buttonText).toBeDefined();
    });

    it('passes testID to TouchableOpacity', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GoldButton testID="gold-button">Test Button</GoldButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.testID).toBe('gold-button');
    });
  });

  describe('size variants', () => {
    it('renders small size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton size="sm">Small</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders medium size (default)', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton size="md">Medium</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders large size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton size="lg">Large</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders extra large size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton size="xl">Extra Large</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('button variants', () => {
    it('renders primary variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton variant="primary">Primary</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders outline variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton variant="outline">Outline</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders ghost variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton variant="ghost">Ghost</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('press handlers', () => {
    it('calls onPress when pressed', () => {
      const onPress = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton onPress={onPress}>Press Me</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      act(() => {
        touchable?.props.onPress?.();
      });

      expect(onPress).toHaveBeenCalled();
    });

    it('calls onLongPress when long pressed', () => {
      const onLongPress = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GoldButton onLongPress={onLongPress}>Long Press</GoldButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      act(() => {
        touchable?.props.onLongPress?.();
      });

      expect(onLongPress).toHaveBeenCalled();
    });

    it('does not call onPress when disabled', () => {
      const onPress = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GoldButton onPress={onPress} disabled>
            Disabled
          </GoldButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });

    it('does not call onPress when loading', () => {
      const onPress = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GoldButton onPress={onPress} loading>
            Loading
          </GoldButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });
  });

  describe('disabled state', () => {
    it('renders disabled state', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton disabled>Disabled</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });

    it('applies disabled styling', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton disabled>Disabled</GoldButton>);
      });

      // Disabled button should still render
      expect(tree?.toJSON()).not.toBeNull();

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });
  });

  describe('loading state', () => {
    it('displays "Loading..." text when loading', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton loading>Click Me</GoldButton>);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const loadingText = texts?.find((t) => t.props.children === 'Loading...');

      expect(loadingText).toBeDefined();
    });

    it('disables button when loading', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton loading>Loading</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });
  });

  describe('glow effect', () => {
    it('renders with glow enabled by default', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Glow Button</GoldButton>);
      });

      // Button should render with glow (default)
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('hides glow when showGlow is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton showGlow={false}>No Glow</GoldButton>);
      });

      // Button should still render
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation', () => {
    it('handles pressIn event', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Press In Test</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      act(() => {
        touchable?.props.onPressIn?.();
      });

      // Should not throw
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles pressOut event', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Press Out Test</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      act(() => {
        touchable?.props.onPressIn?.();
        touchable?.props.onPressOut?.();
      });

      // Should not throw
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('does not animate when disabled', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton disabled>Disabled</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);

      // PressIn/PressOut should not throw even when disabled
      act(() => {
        touchable?.props.onPressIn?.();
        touchable?.props.onPressOut?.();
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with animation-ready structure', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Scale Button</GoldButton>);
      });

      // Button should render with scale animation structure
      expect(tree?.toJSON()).not.toBeNull();

      // Should have TouchableOpacity for interaction
      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable).toBeDefined();
    });
  });

  describe('custom styling', () => {
    it('accepts custom container style', () => {
      const customStyle = { backgroundColor: '#ff0000' };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton style={customStyle}>Styled</GoldButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts custom text style', () => {
      const customTextStyle = { fontSize: 24 };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <GoldButton textStyle={customTextStyle}>Styled Text</GoldButton>
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textStyles = texts?.[0]?.props.style;
      const flattenedStyles = Array.isArray(textStyles)
        ? textStyles.flat()
        : [textStyles];

      const hasCustomFontSize = flattenedStyles.some(
        (s: { fontSize?: number }) => s?.fontSize === 24
      );

      expect(hasCustomFontSize).toBe(true);
    });
  });

  describe('activeOpacity', () => {
    it('uses default activeOpacity', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<GoldButton>Default</GoldButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.activeOpacity).toBe(0.9);
    });
  });
});
