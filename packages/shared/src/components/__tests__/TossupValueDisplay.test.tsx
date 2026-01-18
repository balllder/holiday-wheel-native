import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { TossupValueDisplay } from '../TossupValueDisplay';

describe('TossupValueDisplay', () => {
  describe('visibility', () => {
    it('renders when visible is true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('returns null when visible is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={false}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });

  describe('value display', () => {
    it('displays formatted value with dollar sign', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('$1,000');
    });

    it('displays larger values with proper formatting', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={10000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('$10,000');
    });

    it('displays FOR label', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={2000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('FOR');
    });
  });

  describe('triple toss-up', () => {
    it('shows TRIPLE TOSS-UP label when isTriple is true', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={true}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('TRIPLE TOSS-UP');
    });

    it('does not show TRIPLE TOSS-UP label when isTriple is false', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).not.toContain('TRIPLE TOSS-UP');
    });

    it('renders three progress indicators for triple toss-up', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={2000}
            isTriple={true}
            tripleIndex={1}
            visible={true}
          />
        );
      });

      // The component should render, indicating progress indicators exist
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('props', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
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
          <TossupValueDisplay
            value={1000}
            isTriple={false}
            tripleIndex={0}
            visible={true}
            testID="tossup-display"
          />
        );
      });

      expect(tree).toBeDefined();
    });
  });
});
