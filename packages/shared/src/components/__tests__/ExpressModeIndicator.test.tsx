import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { ExpressModeIndicator } from '../ExpressModeIndicator';

describe('ExpressModeIndicator', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('visibility', () => {
    it('renders when active is true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('returns null when active is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={false}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });

  describe('content', () => {
    it('displays EXPRESS MODE text', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={5}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('EXPRESS MODE');
    });

    it('displays correct count', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={5}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('5');
    });

    it('calculates and displays total earnings', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={3}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('3,000');
    });

    it('displays player name when provided', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
            playerName="John"
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('John');
    });

    it('shows warning text about bankrupt', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('BANKRUPT');
    });
  });

  describe('animations', () => {
    it('runs slide-in animation when activated', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(500);
      });

      expect(tree).toBeDefined();
    });

    it('runs pulse animation when active', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(tree).toBeDefined();
    });

    it('returns null when deactivated', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      act(() => {
        tree?.update(
          <ExpressModeIndicator
            active={false}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });

  describe('props', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
            style={{ margin: 10 }}
          />
        );
      });

      expect(tree).toBeDefined();
    });

    it('accepts testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
            testID="express-indicator"
          />
        );
      });

      expect(tree).toBeDefined();
    });
  });

  describe('earnings calculation', () => {
    it('handles zero correct count', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={0}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      // $ and 0 are separate children in the render tree
      expect(json).toContain('"$"');
      expect(json).toContain('"0"');
    });

    it('handles custom value per consonant', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={2}
            valuePerConsonant={2000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('4,000');
    });

    it('handles large earnings', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <ExpressModeIndicator
            active={true}
            correctCount={10}
            valuePerConsonant={1000}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('10,000');
    });
  });
});
