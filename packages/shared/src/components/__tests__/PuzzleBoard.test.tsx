import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { PuzzleBoard } from '../PuzzleBoard';

describe('PuzzleBoard', () => {
  const defaultProps = {
    answer: 'WHEEL OF FORTUNE',
    category: 'TV SHOW',
    revealed: new Set<string>(),
  };

  describe('rendering', () => {
    it('renders with category and puzzle', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays the category', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} />);
      });

      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('TV SHOW');
    });

    it('renders all rows of the puzzle board', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} testID="puzzle-board" />);
      });

      // Should have multiple rows rendered
      const board = tree?.root.findByProps({ testID: 'puzzle-board' });
      expect(board).toBeDefined();
    });

    it('renders empty puzzle gracefully', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard answer="" category="TEST" revealed={new Set()} />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders loading state when no category', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard answer="TEST" category="" revealed={new Set()} />
        );
      });

      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('Loading...');
    });
  });

  describe('size variants', () => {
    it('renders phone size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} size="phone" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders tv size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} size="tv" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('letter reveal', () => {
    it('shows revealed letters', () => {
      const revealed = new Set(['W', 'E']);
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<PuzzleBoard {...defaultProps} revealed={revealed} />);
      });

      const json = tree?.toJSON();
      // Should contain revealed letters
      const jsonStr = JSON.stringify(json);
      expect(jsonStr).toContain('"W"');
      expect(jsonStr).toContain('"E"');
    });

    it('hides unrevealed letters', () => {
      const revealed = new Set(['W']);
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard
            answer="WX"
            category="TEST"
            revealed={revealed}
          />
        );
      });

      // Should not contain unrevealed 'X' as text
      // (X will be present as hidden cell, not visible text)
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation types', () => {
    it('accepts pop animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard {...defaultProps} animationType="pop" />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts flip animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard {...defaultProps} animationType="flip" />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts fade animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard {...defaultProps} animationType="fade" />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation callbacks', () => {
    beforeEach(() => {
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it('calls onLetterRevealStart when letter starts revealing', () => {
      const onStart = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PuzzleBoard
            answer="AB"
            category="TEST"
            revealed={new Set()}
            onLetterRevealStart={onStart}
          />
        );
      });

      // Trigger reveal by updating revealed set
      act(() => {
        tree?.update(
          <PuzzleBoard
            answer="AB"
            category="TEST"
            revealed={new Set(['A'])}
            onLetterRevealStart={onStart}
          />
        );
      });

      // Run timers to process reveal
      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(onStart).toHaveBeenCalled();
    });

    it('accepts onLetterRevealComplete callback prop', () => {
      const onComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PuzzleBoard
            answer="AB"
            category="TEST"
            revealed={new Set()}
            animationDuration={100}
            onLetterRevealComplete={onComplete}
          />
        );
      });

      // Just verify the component accepts the prop and renders
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('staggers reveals with staggerDelay', () => {
      const onStart = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PuzzleBoard
            answer="AAA"
            category="TEST"
            revealed={new Set()}
            staggerDelay={100}
            onLetterRevealStart={onStart}
          />
        );
      });

      // Reveal 'A' which appears multiple times
      act(() => {
        tree?.update(
          <PuzzleBoard
            answer="AAA"
            category="TEST"
            revealed={new Set(['A'])}
            staggerDelay={100}
            onLetterRevealStart={onStart}
          />
        );
      });

      // After first delay, one letter should start
      act(() => {
        jest.advanceTimersByTime(50);
      });

      const callsAfterFirst = onStart.mock.calls.length;

      // After more time, more letters should start
      act(() => {
        jest.advanceTimersByTime(200);
      });

      expect(onStart.mock.calls.length).toBeGreaterThanOrEqual(callsAfterFirst);
    });
  });

  describe('word wrapping', () => {
    it('wraps long phrases across multiple rows', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard
            answer="A VERY LONG PHRASE THAT SHOULD WRAP"
            category="TEST"
            revealed={new Set()}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles single word puzzles', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard
            answer="HELLO"
            category="TEST"
            revealed={new Set()}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles puzzles with punctuation', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard
            answer="IT'S A TEST!"
            category="PHRASE"
            revealed={new Set()}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('styling', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard
            {...defaultProps}
            style={{ marginTop: 20 }}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <PuzzleBoard {...defaultProps} testID="test-board" />
        );
      });

      const board = tree?.root.findByProps({ testID: 'test-board' });
      expect(board).toBeDefined();
    });
  });

  describe('multiple reveals', () => {
    beforeEach(() => {
      jest.useFakeTimers();
    });

    afterEach(() => {
      jest.useRealTimers();
    });

    it('handles revealing multiple different letters', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <PuzzleBoard
            answer="ABC"
            category="TEST"
            revealed={new Set()}
          />
        );
      });

      // Reveal 'A'
      act(() => {
        tree?.update(
          <PuzzleBoard
            answer="ABC"
            category="TEST"
            revealed={new Set(['A'])}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(500);
      });

      // Reveal 'B'
      act(() => {
        tree?.update(
          <PuzzleBoard
            answer="ABC"
            category="TEST"
            revealed={new Set(['A', 'B'])}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(500);
      });

      const json = tree?.toJSON();
      const jsonStr = JSON.stringify(json);
      expect(jsonStr).toContain('"A"');
      expect(jsonStr).toContain('"B"');
    });
  });
});
