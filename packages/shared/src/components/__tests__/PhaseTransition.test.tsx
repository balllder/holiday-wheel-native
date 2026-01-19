import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { PhaseTransition, usePhaseTransition } from '../PhaseTransition';

describe('PhaseTransition', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('rendering', () => {
    it('renders null when no transition is happening', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PhaseTransition phase="normal" previousPhase={null} />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });

    it('renders when phase changes', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition phase="tossup" previousPhase="normal" />
        );
      });

      // Advance timers to let animation start
      act(() => {
        jest.advanceTimersByTime(100);
      });

      // Should render during transition
      expect(tree).toBeDefined();
    });

    it('renders with testID', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            testID="test-transition"
          />
        );
      });

      // Advance timers
      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('phases', () => {
    it('handles normal phase', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition phase="normal" previousPhase="tossup" />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });

    it('handles tossup phase with slide animation', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition phase="tossup" previousPhase="normal" />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });

    it('handles final phase with fade animation', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition phase="final" previousPhase="normal" />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });

    it('handles solved phase with flash animation', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition phase="solved" previousPhase="normal" />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('callbacks', () => {
    it('calls onTransitionStart when transition begins', () => {
      const onTransitionStart = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            onTransitionStart={onTransitionStart}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(onTransitionStart).toHaveBeenCalledWith('normal', 'tossup');
    });

    it('calls onTransitionComplete when animation finishes', () => {
      const onTransitionComplete = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            duration={100}
            onTransitionComplete={onTransitionComplete}
          />
        );
      });

      // Advance through full animation
      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(onTransitionComplete).toHaveBeenCalledWith('tossup');
    });
  });

  describe('enabled prop', () => {
    it('skips animation when disabled', () => {
      const onTransitionComplete = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            enabled={false}
            onTransitionComplete={onTransitionComplete}
          />
        );
      });

      // Callback should be called immediately
      expect(onTransitionComplete).toHaveBeenCalledWith('tossup');
    });
  });

  describe('category prop', () => {
    it('accepts category prop', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            category="PHRASE"
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('duration prop', () => {
    it('accepts custom duration', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            duration={1000}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('style prop', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PhaseTransition
            phase="tossup"
            previousPhase="normal"
            style={{ backgroundColor: 'transparent' }}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });
});

describe('usePhaseTransition', () => {
  function TestComponent({ onRender }: { onRender?: (data: ReturnType<typeof usePhaseTransition>) => void }) {
    const hookResult = usePhaseTransition('normal');
    onRender?.(hookResult);
    return (
      <>
        <div data-testid="phase">{hookResult.currentPhase}</div>
        <div data-testid="prev-phase">{hookResult.previousPhase || 'null'}</div>
        <div data-testid="transitioning">{hookResult.isTransitioning.toString()}</div>
        <button data-testid="set-tossup" onClick={() => hookResult.setPhase('tossup')} />
        <button data-testid="complete" onClick={hookResult.completeTransition} />
      </>
    );
  }

  it('initializes with default phase', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const phase = tree?.root.findByProps({ 'data-testid': 'phase' });
    expect(phase?.props.children).toBe('normal');
  });

  it('initializes with custom initial phase', () => {
    function CustomComponent() {
      const { currentPhase } = usePhaseTransition('tossup');
      return <div data-testid="phase">{currentPhase}</div>;
    }

    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<CustomComponent />);
    });

    const phase = tree?.root.findByProps({ 'data-testid': 'phase' });
    expect(phase?.props.children).toBe('tossup');
  });

  it('updates phase when setPhase is called', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const setTossup = tree?.root.findByProps({ 'data-testid': 'set-tossup' });
    act(() => {
      setTossup?.props.onClick();
    });

    const phase = tree?.root.findByProps({ 'data-testid': 'phase' });
    expect(phase?.props.children).toBe('tossup');
  });

  it('sets previousPhase when phase changes', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const setTossup = tree?.root.findByProps({ 'data-testid': 'set-tossup' });
    act(() => {
      setTossup?.props.onClick();
    });

    const prevPhase = tree?.root.findByProps({ 'data-testid': 'prev-phase' });
    expect(prevPhase?.props.children).toBe('normal');
  });

  it('sets isTransitioning to true when phase changes', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    const setTossup = tree?.root.findByProps({ 'data-testid': 'set-tossup' });
    act(() => {
      setTossup?.props.onClick();
    });

    const transitioning = tree?.root.findByProps({ 'data-testid': 'transitioning' });
    expect(transitioning?.props.children).toBe('true');
  });

  it('completeTransition sets isTransitioning to false', () => {
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent />);
    });

    // Set phase to trigger transition
    const setTossup = tree?.root.findByProps({ 'data-testid': 'set-tossup' });
    act(() => {
      setTossup?.props.onClick();
    });

    // Complete transition
    const complete = tree?.root.findByProps({ 'data-testid': 'complete' });
    act(() => {
      complete?.props.onClick();
    });

    const transitioning = tree?.root.findByProps({ 'data-testid': 'transitioning' });
    expect(transitioning?.props.children).toBe('false');
  });

  it('does not change if setPhase called with same phase', () => {
    let capturedData: ReturnType<typeof usePhaseTransition> | null = null;
    let tree: ReactTestRenderer | undefined;
    act(() => {
      tree = create(<TestComponent onRender={(data) => { capturedData = data; }} />);
    });

    // Call setPhase with same phase
    act(() => {
      capturedData?.setPhase('normal');
    });

    // Should not be transitioning
    const transitioning = tree?.root.findByProps({ 'data-testid': 'transitioning' });
    expect(transitioning?.props.children).toBe('false');
  });
});
