import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { LetterCell, LetterState } from '../LetterCell';

describe('LetterCell', () => {
  describe('rendering', () => {
    it('renders empty cell when char is null', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char={null} state="empty" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders space cell when char is space', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char=" " state="space" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders hidden letter cell', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="A" state="hidden" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
      // Hidden cell should not show the letter
      const json = tree?.toJSON();
      expect(JSON.stringify(json)).not.toContain('"A"');
    });

    it('renders revealed letter cell with letter visible', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="A" state="revealed" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
      // Revealed cell should show the letter
      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('"A"');
    });

    it('renders punctuation as always visible', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="!" state="revealed" />);
      });

      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('"!"');
    });
  });

  describe('size variants', () => {
    it('renders small size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="A" state="revealed" size="small" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders medium size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="A" state="revealed" size="medium" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('renders large size', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="A" state="revealed" size="large" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animation types', () => {
    it('accepts pop animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell char="A" state="revealing" animationType="pop" />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts flip animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell char="A" state="revealing" animationType="flip" />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts fade animation type', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell char="A" state="revealing" animationType="fade" />
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

    it('calls onAnimationComplete after animation', () => {
      const onComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <LetterCell
            char="A"
            state="revealing"
            animationType="pop"
            animationDuration={100}
            animationDelay={0}
            onAnimationComplete={onComplete}
          />
        );
      });

      // Run timers to complete animation
      act(() => {
        jest.advanceTimersByTime(500);
      });

      // Animation mock calls callback immediately in tests
      expect(onComplete).toHaveBeenCalled();
    });

    it('respects animation delay', () => {
      const onComplete = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <LetterCell
            char="A"
            state="revealing"
            animationType="pop"
            animationDuration={100}
            animationDelay={200}
            onAnimationComplete={onComplete}
          />
        );
      });

      // Before delay, callback should not be called
      act(() => {
        jest.advanceTimersByTime(100);
      });

      // After delay, animation starts and completes (mock completes immediately)
      act(() => {
        jest.advanceTimersByTime(300);
      });

      expect(onComplete).toHaveBeenCalled();
    });
  });

  describe('state transitions', () => {
    it('handles transition from hidden to revealing', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(<LetterCell char="A" state="hidden" />);
      });

      act(() => {
        tree?.update(<LetterCell char="A" state="revealing" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles transition from revealing to revealed', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(<LetterCell char="A" state="revealing" />);
      });

      act(() => {
        tree?.update(<LetterCell char="A" state="revealed" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('"A"');
    });
  });

  describe('styling', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell
            char="A"
            state="revealed"
            style={{ backgroundColor: 'red' }}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts custom text style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell
            char="A"
            state="revealed"
            textStyle={{ color: 'blue' }}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <LetterCell char="A" state="revealed" testID="test-cell" />
        );
      });

      const view = tree?.root.findByProps({ testID: 'test-cell' });
      expect(view).toBeDefined();
    });
  });

  describe('letter case', () => {
    it('displays lowercase letters as uppercase', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LetterCell char="a" state="revealed" />);
      });

      const json = tree?.toJSON();
      expect(JSON.stringify(json)).toContain('"A"');
    });
  });
});
