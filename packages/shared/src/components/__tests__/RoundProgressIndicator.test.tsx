import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { RoundProgressIndicator } from '../RoundProgressIndicator';
import type { RoundConfig } from '../../types';

describe('RoundProgressIndicator', () => {
  describe('visibility', () => {
    it('renders when enabled and totalRounds > 0', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            enabled={true}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('returns null when not enabled', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            enabled={false}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });

    it('returns null when totalRounds is 0', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={0}
            totalRounds={0}
            enabled={true}
          />
        );
      });

      expect(tree?.toJSON()).toBeNull();
    });
  });

  describe('content', () => {
    it('displays current round number', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={2}
            totalRounds={4}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      // Text may be split into separate children: "ROUND " and "2"
      expect(json).toContain('"ROUND "');
      expect(json).toContain('"2"');
    });

    it('displays total rounds', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={5}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      // Text may be split into separate children: "of " and "5"
      expect(json).toContain('"of "');
      expect(json).toContain('"5"');
    });

    it('shows round type badge when provided', () => {
      const roundConfig: RoundConfig = {
        number: 1,
        type: 'tossup',
        value_multiplier: 1,
        has_mystery: false,
        has_express: false,
      };

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            roundConfig={roundConfig}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('TOSS-UP');
    });

    it('shows speed round type', () => {
      const roundConfig: RoundConfig = {
        number: 1,
        type: 'speed',
        value_multiplier: 1,
        has_mystery: false,
        has_express: false,
      };

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            roundConfig={roundConfig}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('SPEED');
    });

    it('shows bonus round type', () => {
      const roundConfig: RoundConfig = {
        number: 1,
        type: 'bonus',
        value_multiplier: 1,
        has_mystery: false,
        has_express: false,
      };

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            roundConfig={roundConfig}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('BONUS');
    });

    it('shows multiplier badge when value_multiplier > 1', () => {
      const roundConfig: RoundConfig = {
        number: 3,
        type: 'normal',
        value_multiplier: 2,
        has_mystery: true,
        has_express: false,
      };

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={3}
            totalRounds={4}
            roundConfig={roundConfig}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      // Text may be split into separate children: "2" and "x"
      expect(json).toContain('"2"');
      expect(json).toContain('"x"');
    });

    it('does not show multiplier when value_multiplier is 1', () => {
      const roundConfig: RoundConfig = {
        number: 1,
        type: 'normal',
        value_multiplier: 1,
        has_mystery: false,
        has_express: false,
      };

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            roundConfig={roundConfig}
            enabled={true}
          />
        );
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).not.toContain('1x');
    });
  });

  describe('props', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            enabled={true}
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
          <RoundProgressIndicator
            currentRound={1}
            totalRounds={4}
            enabled={true}
            testID="round-progress"
          />
        );
      });

      expect(tree).toBeDefined();
    });
  });
});
