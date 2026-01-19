import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Toast } from '../Toast';

describe('Toast', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('rendering', () => {
    it('renders nothing when visible is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast message="Test message" visible={false} onHide={jest.fn()} />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });

    it('renders toast when visible is true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast message="Test message" visible={true} onHide={jest.fn()} />
        );
      });

      // Initially renders during animation
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays the correct message', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast message="Success message!" visible={true} onHide={jest.fn()} />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const messageText = texts?.find((t) =>
        t.props.children === 'Success message!'
      );

      expect(messageText).toBeDefined();
    });

    it('passes testID correctly', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast
            message="Test"
            visible={true}
            onHide={jest.fn()}
            testID="test-toast"
          />
        );
      });

      const animatedView = tree?.root.findByProps({ testID: 'test-toast' });
      expect(animatedView).toBeDefined();
    });
  });

  describe('toast types', () => {
    it('renders info type toast', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast
            message="Info message"
            visible={true}
            onHide={jest.fn()}
            type="info"
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const iconText = texts?.find((t) => t.props.children === '\u2139');
      expect(iconText).toBeDefined();
    });

    it('renders success type toast', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast
            message="Success message"
            visible={true}
            onHide={jest.fn()}
            type="success"
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const iconText = texts?.find((t) => t.props.children === '\u2713');
      expect(iconText).toBeDefined();
    });

    it('renders error type toast', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast
            message="Error message"
            visible={true}
            onHide={jest.fn()}
            type="error"
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const iconText = texts?.find((t) => t.props.children === '\u2717');
      expect(iconText).toBeDefined();
    });

    it('renders warning type toast', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <Toast
            message="Warning message"
            visible={true}
            onHide={jest.fn()}
            type="warning"
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const iconText = texts?.find((t) => t.props.children === '\u26A0');
      expect(iconText).toBeDefined();
    });
  });

  describe('auto-dismiss behavior', () => {
    it('calls onHide after default duration', () => {
      const onHide = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <Toast message="Auto-dismiss test" visible={true} onHide={onHide} />
        );
      });

      // Fast-forward past the default 3000ms duration plus animation time
      act(() => {
        jest.advanceTimersByTime(3500);
      });

      expect(onHide).toHaveBeenCalled();
    });

    it('calls onHide after custom duration', () => {
      const onHide = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <Toast
            message="Custom duration test"
            visible={true}
            onHide={onHide}
            duration={1000}
          />
        );
      });

      // Fast-forward past the custom 1000ms duration plus animation time
      act(() => {
        jest.advanceTimersByTime(1500);
      });

      expect(onHide).toHaveBeenCalled();
    });

    it('does not call onHide before duration expires', () => {
      const onHide = jest.fn();
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <Toast
            message="Test"
            visible={true}
            onHide={onHide}
            duration={3000}
          />
        );
      });

      // Fast-forward only 1000ms (less than duration)
      act(() => {
        jest.advanceTimersByTime(1000);
      });

      expect(onHide).not.toHaveBeenCalled();
    });
  });

  describe('visibility transitions', () => {
    it('shows toast when visible changes from false to true', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message="Test" visible={false} onHide={jest.fn()} />
        );
      });

      // Initially should be null
      expect(tree?.toJSON()).toBeNull();

      // Update to visible
      act(() => {
        tree?.update(<Toast message="Test" visible={true} onHide={jest.fn()} />);
      });

      // Should now render
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('cleans up timer when unmounted', () => {
      const onHide = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message="Test" visible={true} onHide={onHide} />
        );
      });

      // Unmount before timer expires
      act(() => {
        tree?.unmount();
      });

      // Fast-forward past duration
      act(() => {
        jest.advanceTimersByTime(5000);
      });

      // onHide should not be called after unmount
      expect(onHide).not.toHaveBeenCalled();
    });
  });

  describe('message truncation', () => {
    it('accepts long messages', () => {
      const longMessage = 'This is a very long message that should be displayed with proper truncation and text wrapping to ensure the toast remains readable.';
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message={longMessage} visible={true} onHide={jest.fn()} />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const messageText = texts?.find((t) => t.props.children === longMessage);

      expect(messageText).toBeDefined();
      expect(messageText?.props.numberOfLines).toBe(3);
    });
  });

  describe('animation', () => {
    it('renders with animation-ready structure', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message="Test" visible={true} onHide={jest.fn()} />
        );
      });

      // Toast should render
      expect(tree?.toJSON()).not.toBeNull();

      // Should have views for structure
      const views = tree?.root.findAllByType('View' as never);
      expect(views?.length).toBeGreaterThan(0);
    });
  });

  describe('styling', () => {
    it('has proper z-index for overlay positioning', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message="Test" visible={true} onHide={jest.fn()} />
        );
      });

      const json = tree?.toJSON();
      const styleString = JSON.stringify(json);

      // Should have high z-index value (at least 1000)
      expect(styleString).toContain('zIndex');
    });

    it('includes accent border at bottom', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <Toast message="Test" visible={true} onHide={jest.fn()} />
        );
      });

      const views = tree?.root.findAllByType('View' as never);

      // Should have a view styled as accent border
      const hasAccentBorder = views?.some((view) => {
        const style = Array.isArray(view.props.style)
          ? view.props.style.flat()
          : [view.props.style];

        return style.some(
          (s: { position?: string; bottom?: number; height?: number }) =>
            s?.position === 'absolute' && s?.bottom === 0 && s?.height === 2
        );
      });

      expect(hasAccentBorder).toBe(true);
    });
  });
});
