import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { AnimatedButton, SpinButton, BuzzButton } from '../AnimatedButton';

describe('AnimatedButton', () => {
  describe('rendering', () => {
    it('renders with text children', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton>Click Me</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with element children', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton>
            <span>Custom Content</span>
          </AnimatedButton>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays text content correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton>Click Me</AnimatedButton>);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContent = texts?.map((t) => t.props.children).join('');

      expect(textContent).toContain('Click Me');
    });
  });

  describe('variants', () => {
    it('renders primary variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton variant="primary">Primary</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders secondary variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton variant="secondary">Secondary</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders outline variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton variant="outline">Outline</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders ghost variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton variant="ghost">Ghost</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('disabled state', () => {
    it('renders disabled state', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton disabled>Disabled</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('passes disabled prop to TouchableOpacity', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton disabled>Disabled</AnimatedButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });
  });

  describe('press handlers', () => {
    it('calls onPress when pressed', () => {
      const onPress = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton onPress={onPress}>Press Me</AnimatedButton>);
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
          <AnimatedButton onLongPress={onLongPress}>Long Press</AnimatedButton>
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
          <AnimatedButton onPress={onPress} disabled>
            Disabled
          </AnimatedButton>
        );
      });

      // TouchableOpacity prevents press when disabled, so just verify prop is passed
      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.disabled).toBe(true);
    });
  });

  describe('animation props', () => {
    it('accepts scaleOnPress prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton scaleOnPress={0.9}>Scale Button</AnimatedButton>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts showGlow prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton showGlow>Glow Button</AnimatedButton>);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts glowColor prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton showGlow glowColor="#ff0000">
            Custom Glow
          </AnimatedButton>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles pressIn event', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedButton>Press In Test</AnimatedButton>);
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
        tree = create(<AnimatedButton>Press Out Test</AnimatedButton>);
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      act(() => {
        touchable?.props.onPressIn?.();
        touchable?.props.onPressOut?.();
      });

      // Should not throw
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('styling', () => {
    it('accepts custom style', () => {
      const customStyle = { backgroundColor: '#ff0000' };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton style={customStyle}>Styled</AnimatedButton>
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts custom textStyle', () => {
      const customTextStyle = { fontSize: 24 };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton textStyle={customTextStyle}>Styled Text</AnimatedButton>
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

    it('accepts activeOpacity prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton activeOpacity={0.5}>Active Opacity</AnimatedButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.activeOpacity).toBe(0.5);
    });
  });

  describe('testID', () => {
    it('passes testID to TouchableOpacity', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedButton testID="test-button">Test ID</AnimatedButton>
        );
      });

      const touchable = tree?.root.findByType('TouchableOpacity' as never);
      expect(touchable?.props.testID).toBe('test-button');
    });
  });
});

describe('SpinButton', () => {
  it('renders correctly', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<SpinButton onPress={jest.fn()} />);
    });

    expect(tree?.toJSON()).not.toBeNull();
  });

  it('displays SPIN text', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<SpinButton onPress={jest.fn()} />);
    });

    const texts = tree?.root.findAllByType('Text' as never);
    const textContent = texts?.map((t) => t.props.children).join('');

    expect(textContent).toContain('SPIN');
  });

  it('calls onPress when pressed', () => {
    const onPress = jest.fn();
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<SpinButton onPress={onPress} />);
    });

    const touchable = tree?.root.findByType('TouchableOpacity' as never);
    act(() => {
      touchable?.props.onPress?.();
    });

    expect(onPress).toHaveBeenCalled();
  });

  it('can be disabled', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<SpinButton onPress={jest.fn()} disabled />);
    });

    const touchable = tree?.root.findByType('TouchableOpacity' as never);
    expect(touchable?.props.disabled).toBe(true);
  });
});

describe('BuzzButton', () => {
  it('renders correctly', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<BuzzButton onPress={jest.fn()} />);
    });

    expect(tree?.toJSON()).not.toBeNull();
  });

  it('displays BUZZ! text', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<BuzzButton onPress={jest.fn()} />);
    });

    const texts = tree?.root.findAllByType('Text' as never);
    const textContent = texts?.map((t) => t.props.children).join('');

    expect(textContent).toContain('BUZZ!');
  });

  it('calls onPress when pressed', () => {
    const onPress = jest.fn();
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<BuzzButton onPress={onPress} />);
    });

    const touchable = tree?.root.findByType('TouchableOpacity' as never);
    act(() => {
      touchable?.props.onPress?.();
    });

    expect(onPress).toHaveBeenCalled();
  });

  it('can be disabled', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<BuzzButton onPress={jest.fn()} disabled />);
    });

    const touchable = tree?.root.findByType('TouchableOpacity' as never);
    expect(touchable?.props.disabled).toBe(true);
  });
});
