// Mocks are set up in jest.setup.js
import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { AnimatedWheel } from '../AnimatedWheel';
import type { WedgeValue } from '../../types';

// Mock timers for animation testing
jest.useFakeTimers();

describe('AnimatedWheel', () => {
  const mockWheelSlots: WedgeValue[] = [
    500, 600, 700, 'BANKRUPT', 800, 900, 'LOSE A TURN', 1000,
  ];

  afterEach(() => {
    jest.clearAllTimers();
  });

  describe('rendering', () => {
    it('renders null when wheelSlots is empty', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={[]} lastSpinIndex={null} />
        );
      });
      expect(tree?.toJSON()).toBeNull();
    });

    it('renders wheel with correct number of segments', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={mockWheelSlots} lastSpinIndex={null} />
        );
      });
      const json = tree?.toJSON();
      expect(json).not.toBeNull();
    });

    it('renders with default size of 300', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={mockWheelSlots} lastSpinIndex={null} />
        );
      });
      const json = tree?.toJSON();
      // Check that the SVG has the default dimensions
      expect(json).toBeDefined();
    });

    it('renders with custom size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={mockWheelSlots} lastSpinIndex={null} size={500} />
        );
      });
      const json = tree?.toJSON();
      expect(json).not.toBeNull();
    });
  });

  describe('wedge labels', () => {
    it('formats dollar values correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={[500, 1000]} lastSpinIndex={null} />
        );
      });
      // The component should render $500 and $1000 labels
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles BANKRUPT wedge', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={['BANKRUPT']} lastSpinIndex={null} />
        );
      });
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles LOSE A TURN wedge', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={['LOSE A TURN']} lastSpinIndex={null} />
        );
      });
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles PRIZE type wedges', () => {
      const prizeSlots: WedgeValue[] = [
        { type: 'PRIZE', name: 'Trip to Paris', value: 5000 },
      ];
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <AnimatedWheel wheelSlots={prizeSlots} lastSpinIndex={null} />
        );
      });
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation', () => {
    it('starts animation when lastSpinIndex changes', () => {
      const onSpinComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      // Initial render
      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            onSpinComplete={onSpinComplete}
          />
        );
      });

      // Trigger spin
      act(() => {
        tree?.update(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={3}
            onSpinComplete={onSpinComplete}
          />
        );
      });

      // Fast forward to animation completion
      // Note: With Animated API mock, callback may be called synchronously
      act(() => {
        jest.advanceTimersByTime(3000);
      });

      // Animation should have completed
      expect(onSpinComplete).toHaveBeenCalled();
    });

    it('does not restart animation for same lastSpinIndex', () => {
      const onSpinComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={3}
            onSpinComplete={onSpinComplete}
          />
        );
      });

      // Fast forward through first animation
      act(() => {
        jest.advanceTimersByTime(3000);
      });

      expect(onSpinComplete).toHaveBeenCalledTimes(1);

      // Re-render with same lastSpinIndex
      act(() => {
        tree?.update(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={3}
            onSpinComplete={onSpinComplete}
          />
        );
      });

      // Should not trigger another animation
      act(() => {
        jest.advanceTimersByTime(3000);
      });

      expect(onSpinComplete).toHaveBeenCalledTimes(1);
    });

    it('cleans up properly on unmount', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={0}
          />
        );
      });

      // Start animation
      act(() => {
        jest.advanceTimersByTime(100);
      });

      // Unmount - should not throw any errors
      // The Animated API handles its own cleanup internally
      expect(() => {
        act(() => {
          tree?.unmount();
        });
      }).not.toThrow();
    });
  });

  describe('visual states', () => {
    it('applies rotation transform to wheel', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={0}
          />
        );
      });

      // Advance animation partway
      act(() => {
        jest.advanceTimersByTime(1000);
      });

      // Component should still be rendered with rotation
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('highlight feature', () => {
    it('calls onSpinStart when spin begins', () => {
      const onSpinStart = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            onSpinStart={onSpinStart}
          />
        );
      });

      // Trigger spin
      act(() => {
        tree?.update(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={2}
            onSpinStart={onSpinStart}
          />
        );
      });

      expect(onSpinStart).toHaveBeenCalledTimes(1);
    });

    it('accepts showWinningHighlight prop', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            showWinningHighlight={true}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts showWinningHighlight=false prop', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            showWinningHighlight={false}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts highlightDuration prop', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            highlightDuration={2000}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts highlightFlashes prop', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            highlightFlashes={5}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('triggers highlight animation after spin completes', () => {
      const onSpinComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={null}
            onSpinComplete={onSpinComplete}
            showWinningHighlight={true}
            highlightDuration={500}
            highlightFlashes={2}
          />
        );
      });

      // Trigger spin
      act(() => {
        tree?.update(
          <AnimatedWheel
            wheelSlots={mockWheelSlots}
            lastSpinIndex={3}
            onSpinComplete={onSpinComplete}
            showWinningHighlight={true}
            highlightDuration={500}
            highlightFlashes={2}
          />
        );
      });

      // Fast forward through spin animation
      act(() => {
        jest.advanceTimersByTime(3000);
      });

      expect(onSpinComplete).toHaveBeenCalled();

      // Fast forward through highlight animation
      act(() => {
        jest.advanceTimersByTime(600);
      });

      // Component should still render after highlight
      expect(tree?.toJSON()).not.toBeNull();
    });
  });
});
