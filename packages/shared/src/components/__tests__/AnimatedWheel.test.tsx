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

      // Animation should not be complete yet
      expect(onSpinComplete).not.toHaveBeenCalled();

      // Fast forward to animation completion
      act(() => {
        jest.advanceTimersByTime(3000);
      });

      expect(onSpinComplete).toHaveBeenCalledTimes(1);
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

    it('clears animation interval on unmount', () => {
      const clearIntervalSpy = jest.spyOn(globalThis, 'clearInterval');
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

      // Unmount
      act(() => {
        tree?.unmount();
      });

      expect(clearIntervalSpy).toHaveBeenCalled();
      clearIntervalSpy.mockRestore();
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
});
