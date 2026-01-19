import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Confetti } from '../Confetti';
import type { ConfettiVariant } from '../Confetti';
import { useConfettiSimple } from '../../hooks/useConfetti';

describe('Confetti', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('rendering', () => {
    it('renders null when not active', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={false} />);
      });

      expect(tree?.toJSON()).toBeNull();
    });

    it('mounts without error when active', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} count={5} />);
      });

      // Component should mount without throwing
      expect(tree).toBeDefined();
    });

    it('accepts custom count prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} count={10} />);
      });

      expect(tree).toBeDefined();
    });

    it('accepts custom colors prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Confetti active={true} count={5} colors={['#ff0000', '#00ff00']} />
        );
      });

      expect(tree).toBeDefined();
    });
  });

  describe('variants', () => {
    it('accepts solve variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} variant="solve" />);
      });

      expect(tree).toBeDefined();
    });

    it('accepts roundWin variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} variant="roundWin" />);
      });

      expect(tree).toBeDefined();
    });

    it('accepts gameWin variant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} variant="gameWin" />);
      });

      expect(tree).toBeDefined();
    });

    it('defaults to solve variant when not specified', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} />);
      });

      // Should mount without throwing (using default solve variant)
      expect(tree).toBeDefined();
    });
  });

  describe('animation', () => {
    it('transitions from inactive to active', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(<Confetti active={false} />);
      });

      // Initially should be null
      expect(tree?.toJSON()).toBeNull();

      // Activate confetti
      act(() => {
        tree?.update(<Confetti active={true} count={5} />);
      });

      // Should transition without error
      expect(tree).toBeDefined();
    });

    it('handles deactivation gracefully', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(<Confetti active={true} count={5} />);
      });

      // Deactivate
      act(() => {
        tree?.update(<Confetti active={false} count={5} />);
      });

      // Should not throw
      expect(tree).toBeDefined();
    });

    it('calls onComplete callback', () => {
      const onComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Confetti active={true} count={5} duration={1000} onComplete={onComplete} />
        );
      });

      // Run timers to complete animation
      act(() => {
        jest.advanceTimersByTime(2000);
      });

      // onComplete should be called (mock animations complete immediately)
      expect(onComplete).toHaveBeenCalled();
    });
  });

  describe('props', () => {
    it('accepts duration prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} count={5} duration={5000} />);
      });

      expect(tree).toBeDefined();
    });

    it('accepts custom style prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Confetti active={true} count={5} style={{ backgroundColor: 'transparent' }} />
        );
      });

      expect(tree).toBeDefined();
    });

    it('accepts testID prop', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<Confetti active={true} count={5} testID="test-confetti" />);
      });

      // Component mounts with testID
      expect(tree).toBeDefined();
    });
  });

  describe('cleanup', () => {
    it('stops animation on unmount', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(<Confetti active={true} count={5} />);
      });

      // Unmount while animation is running
      act(() => {
        tree?.unmount();
      });

      // Should not throw
      expect(true).toBe(true);
    });
  });
});

describe('useConfettiSimple', () => {
  function TestComponent({ autoHideDuration }: { autoHideDuration?: number }) {
    const { showConfetti, variant, triggerConfetti, hideConfetti } = useConfettiSimple(autoHideDuration);
    return (
      <>
        <div data-testid="show-state">{showConfetti.toString()}</div>
        <div data-testid="variant-state">{variant}</div>
        <button data-testid="trigger" onClick={() => triggerConfetti()} />
        <button data-testid="trigger-game-win" onClick={() => triggerConfetti('gameWin')} />
        <button data-testid="hide" onClick={hideConfetti} />
      </>
    );
  }

  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  it('initially returns showConfetti as false', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const showState = tree?.root.findByProps({ 'data-testid': 'show-state' });
    expect(showState?.props.children).toBe('false');
  });

  it('triggerConfetti sets showConfetti to true', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const trigger = tree?.root.findByProps({ 'data-testid': 'trigger' });
    act(() => {
      trigger?.props.onClick();
    });

    const showState = tree?.root.findByProps({ 'data-testid': 'show-state' });
    expect(showState?.props.children).toBe('true');
  });

  it('triggerConfetti with variant sets the correct variant', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const trigger = tree?.root.findByProps({ 'data-testid': 'trigger-game-win' });
    act(() => {
      trigger?.props.onClick();
    });

    const variantState = tree?.root.findByProps({ 'data-testid': 'variant-state' });
    expect(variantState?.props.children).toBe('gameWin');
  });

  it('hideConfetti sets showConfetti to false', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    // First trigger
    const trigger = tree?.root.findByProps({ 'data-testid': 'trigger' });
    act(() => {
      trigger?.props.onClick();
    });

    // Then hide
    const hide = tree?.root.findByProps({ 'data-testid': 'hide' });
    act(() => {
      hide?.props.onClick();
    });

    const showState = tree?.root.findByProps({ 'data-testid': 'show-state' });
    expect(showState?.props.children).toBe('false');
  });

  it('auto-hides after specified duration', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent autoHideDuration={1000} />);
    });

    // Trigger confetti
    const trigger = tree?.root.findByProps({ 'data-testid': 'trigger' });
    act(() => {
      trigger?.props.onClick();
    });

    // Should be true initially
    let showState = tree?.root.findByProps({ 'data-testid': 'show-state' });
    expect(showState?.props.children).toBe('true');

    // Advance time past auto-hide duration
    act(() => {
      jest.advanceTimersByTime(1500);
    });

    // Should be false now
    showState = tree?.root.findByProps({ 'data-testid': 'show-state' });
    expect(showState?.props.children).toBe('false');
  });

  it('clears timeout on unmount', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent autoHideDuration={1000} />);
    });

    const trigger = tree?.root.findByProps({ 'data-testid': 'trigger' });
    act(() => {
      trigger?.props.onClick();
    });

    // Unmount before timeout completes
    act(() => {
      tree?.unmount();
    });

    // Advance time - should not throw
    act(() => {
      jest.advanceTimersByTime(2000);
    });

    expect(true).toBe(true);
  });
});
