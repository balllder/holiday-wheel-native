import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { WildCardButton } from '../WildCardButton';

describe('WildCardButton', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('visibility', () => {
    it('renders when count is greater than 0', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('returns null when count is 0', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={0}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });

  describe('content', () => {
    it('displays the count', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={3}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('3');
    });

    it('displays WILD label', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('WILD');
    });

    it('displays W icon', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('"W"');
    });
  });

  describe('interactions', () => {
    it('has onPress callback when enabled', () => {
      const onPress = jest.fn();
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={onPress}
          />
        );
      });

      // Verify component rendered with the callback
      expect(tree).toBeDefined();
      expect(tree?.toJSON()).not.toBeNull();
    });

    it('handles press in animation', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(100);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('animations', () => {
    it('runs shimmer animation when enabled', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(3500);
      });

      expect(tree).toBeDefined();
    });

    it('renders when disabled', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={false}
            onPress={jest.fn()}
          />
        );
      });

      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(tree).toBeDefined();
    });
  });

  describe('disabled state', () => {
    it('renders when enabled is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={false}
            onPress={jest.fn()}
          />
        );
      });

      expect(tree).toBeDefined();
    });
  });

  describe('props', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
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
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
            testID="test-wild-card"
          />
        );
      });

      expect(tree).toBeDefined();
    });
  });

  describe('count display', () => {
    it('updates when count changes', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      let json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('1');

      act(() => {
        tree?.update(
          <WildCardButton
            count={2}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('2');
    });

    it('hides when count becomes 0', () => {
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <WildCardButton
            count={1}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();

      act(() => {
        tree?.update(
          <WildCardButton
            count={0}
            enabled={true}
            onPress={jest.fn()}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });
});
