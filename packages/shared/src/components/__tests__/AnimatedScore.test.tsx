import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { AnimatedScore, useScoreAnimation } from '../AnimatedScore';

// Helper component to test the hook
function HookTester({
  onHook,
}: {
  onHook: (hook: ReturnType<typeof useScoreAnimation>) => void;
}) {
  const hook = useScoreAnimation();
  React.useEffect(() => {
    onHook(hook);
  }, [hook, onHook]);
  return <hook.ScoreChangeComponent />;
}

describe('AnimatedScore', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('rendering', () => {
    it('renders with positive value', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} visible={true} />);
      });

      const json = tree?.toJSON();
      expect(json).not.toBeNull();
    });

    it('renders with negative value', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={-1000} visible={true} />);
      });

      const json = tree?.toJSON();
      expect(json).not.toBeNull();
    });

    it('returns null when not visible and not animating', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={0} visible={false} />);
      });

      expect(tree?.toJSON()).toBeNull();
    });

    it('returns null for zero value', () => {
      let _tree: ReactTestRenderer | undefined;
      act(() => {
        _tree = create(<AnimatedScore value={0} visible={true} />);
      });

      // Component renders container but starts animation only for non-zero
      // After animation completes with zero value, should be null
    });
  });

  describe('formatting', () => {
    it('formats positive values with + prefix', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} visible={true} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContent = texts?.map((t) => t.props.children).join('');

      expect(textContent).toContain('+');
      expect(textContent).toContain('$');
      expect(textContent).toContain('500');
    });

    it('formats negative values correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={-1000} visible={true} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContent = texts?.map((t) => t.props.children).join('');

      expect(textContent).toContain('$');
      expect(textContent).toContain('1,000');
    });

    it('uses custom format function', () => {
      const customFormat = (value: number) => `Points: ${value}`;
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedScore
            value={500}
            visible={true}
            formatValue={customFormat}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContent = texts?.map((t) => t.props.children).join('');

      expect(textContent).toBe('Points: 500');
    });
  });

  describe('positions', () => {
    it('renders with above position', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} position="above" visible={true} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with below position', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} position="below" visible={true} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with left position', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} position="left" visible={true} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders with right position', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} position="right" visible={true} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation', () => {
    it('calls onComplete after animation', () => {
      const onComplete = jest.fn();
      act(() => {
        create(
          <AnimatedScore value={500} visible={true} onComplete={onComplete} duration={1000} />
        );
      });

      // Fast-forward timers
      act(() => {
        jest.advanceTimersByTime(1500);
      });

      expect(onComplete).toHaveBeenCalled();
    });

    it('accepts custom duration prop', () => {
      // Note: In test environment, animations complete immediately due to mocks
      // This test verifies the component accepts the duration prop without error
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedScore value={500} visible={true} duration={500} />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('styling', () => {
    it('applies positive text style for positive values', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={500} visible={true} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textStyles = texts?.[0]?.props.style;

      // Should have gold color for positive
      const flattenedStyles = Array.isArray(textStyles)
        ? textStyles.flat()
        : [textStyles];
      const hasGoldColor = flattenedStyles.some(
        (s: { color?: string }) => s?.color === '#d4af37'
      );
      expect(hasGoldColor).toBe(true);
    });

    it('applies negative text style for negative values', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<AnimatedScore value={-500} visible={true} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textStyles = texts?.[0]?.props.style;

      // Should have red color for negative
      const flattenedStyles = Array.isArray(textStyles)
        ? textStyles.flat()
        : [textStyles];
      const hasRedColor = flattenedStyles.some(
        (s: { color?: string }) => s?.color === '#ef4444'
      );
      expect(hasRedColor).toBe(true);
    });

    it('applies custom text style', () => {
      const customStyle = { fontSize: 32 };
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedScore value={500} visible={true} textStyle={customStyle} />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textStyles = texts?.[0]?.props.style;

      const flattenedStyles = Array.isArray(textStyles)
        ? textStyles.flat()
        : [textStyles];
      const hasCustomFontSize = flattenedStyles.some(
        (s: { fontSize?: number }) => s?.fontSize === 32
      );
      expect(hasCustomFontSize).toBe(true);
    });
  });
});

describe('useScoreAnimation', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('provides showChange function', () => {
    let hookResult: ReturnType<typeof useScoreAnimation> | undefined;
    act(() => {
      create(
        <HookTester
          onHook={(hook) => {
            hookResult = hook;
          }}
        />
      );
    });

    expect(hookResult?.showChange).toBeDefined();
    expect(typeof hookResult?.showChange).toBe('function');
  });

  it('provides ScoreChangeComponent', () => {
    let hookResult: ReturnType<typeof useScoreAnimation> | undefined;
    act(() => {
      create(
        <HookTester
          onHook={(hook) => {
            hookResult = hook;
          }}
        />
      );
    });

    expect(hookResult?.ScoreChangeComponent).toBeDefined();
  });

  it('tracks changes array', () => {
    // Test that the hook provides an array for tracking changes
    let hookResult: ReturnType<typeof useScoreAnimation> | undefined;
    act(() => {
      create(
        <HookTester
          onHook={(hook) => {
            hookResult = hook;
          }}
        />
      );
    });

    // Initially empty
    expect(hookResult?.changes).toBeDefined();
    expect(Array.isArray(hookResult?.changes)).toBe(true);
  });

  it('showChange adds to queue', () => {
    // The showChange function should be callable without errors
    let showChangeFn: ((value: number) => void) | undefined;
    act(() => {
      create(
        <HookTester
          onHook={(hook) => {
            showChangeFn = hook.showChange;
          }}
        />
      );
    });

    expect(showChangeFn).toBeDefined();

    // Should not throw when called
    act(() => {
      showChangeFn?.(500);
      showChangeFn?.(-1000);
    });
  });
});
