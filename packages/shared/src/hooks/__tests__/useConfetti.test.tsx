import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { View } from 'react-native';
import { useConfetti, useConfettiSimple } from '../useConfetti';
import { useGameStore } from '../../stores/gameStore';

// Mock the game store
jest.mock('../../stores/gameStore', () => ({
  useGameStore: jest.fn(),
}));

const mockUseGameStore = useGameStore as jest.MockedFunction<typeof useGameStore>;

// Default mock state
const createMockState = (overrides = {}) => ({
  revealed: new Set<string>(),
  phase: 'normal' as const,
  puzzle: { id: 0, category: '', answer: '' },
  round: { current_round: 0, total_rounds: 0, rounds: [], enabled: false },
  ...overrides,
});

// Test component for useConfetti
function TestConfettiComponent({
  options = {},
  onHookResult,
}: {
  options?: Parameters<typeof useConfetti>[0];
  onHookResult?: (result: ReturnType<typeof useConfetti>) => void;
}) {
  const hookResult = useConfetti(options);

  React.useEffect(() => {
    onHookResult?.(hookResult);
  }, [hookResult, onHookResult]);

  return (
    <View testID="confetti-container">
      <View testID="confetti" {...hookResult.confettiProps} />
    </View>
  );
}

// Test component for useConfettiSimple
function TestSimpleComponent({
  autoHideDuration,
  onHookResult,
}: {
  autoHideDuration?: number;
  onHookResult?: (result: ReturnType<typeof useConfettiSimple>) => void;
}) {
  const hookResult = useConfettiSimple(autoHideDuration);

  React.useEffect(() => {
    onHookResult?.(hookResult);
  }, [hookResult, onHookResult]);

  return <View testID="simple-container" />;
}

describe('useConfetti', () => {
  beforeEach(() => {
    jest.useFakeTimers();
    // Default mock implementation
    mockUseGameStore.mockImplementation((selector) => {
      const state = createMockState();
      return selector(state as never);
    });
  });

  afterEach(() => {
    jest.useRealTimers();
    jest.clearAllMocks();
  });

  describe('initial state', () => {
    it('returns isActive as false initially', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isActive).toBe(false);
    });

    it('returns currentVariant as null initially', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.currentVariant).toBeNull();
    });

    it('returns trigger function', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.trigger).toBe('function');
    });

    it('returns stop function', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.stop).toBe('function');
    });

    it('returns confettiRef', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.confettiRef).toBeDefined();
      expect(hookResult?.confettiRef.current).toBeNull();
    });

    it('returns confettiProps object', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.confettiProps).toBeDefined();
      expect(hookResult?.confettiProps.active).toBe(false);
      expect(hookResult?.confettiProps.variant).toBe('solve');
      expect(typeof hookResult?.confettiProps.onComplete).toBe('function');
    });
  });

  describe('trigger', () => {
    it('sets isActive to true', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isActive).toBe(true);
    });

    it('sets currentVariant to default "solve"', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.currentVariant).toBe('solve');
    });

    it('accepts custom variant', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger('gameWin');
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.currentVariant).toBe('gameWin');
    });

    it('calls onTrigger callback', () => {
      const onTrigger = jest.fn();
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            options={{ onTrigger }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger('roundWin');
      });

      expect(onTrigger).toHaveBeenCalledWith('roundWin');
    });
  });

  describe('stop', () => {
    it('sets isActive to false', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      // First trigger
      act(() => {
        hookResult?.trigger();
      });

      // Then stop
      act(() => {
        hookResult?.stop();
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isActive).toBe(false);
    });

    it('sets currentVariant to null', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger('gameWin');
      });

      act(() => {
        hookResult?.stop();
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.currentVariant).toBeNull();
    });
  });

  describe('confettiProps.onComplete', () => {
    it('sets isActive to false when called', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      act(() => {
        hookResult?.confettiProps.onComplete();
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isActive).toBe(false);
    });

    it('calls onComplete callback', () => {
      const onComplete = jest.fn();
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            options={{ onComplete }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      act(() => {
        hookResult?.confettiProps.onComplete();
      });

      expect(onComplete).toHaveBeenCalled();
    });
  });

  describe('autoHideDuration', () => {
    it('auto-hides after specified duration', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            options={{ autoHideDuration: 2000 }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      // Should still be active before timeout
      expect(hookResult?.isActive).toBe(true);

      // Fast forward past duration
      act(() => {
        jest.advanceTimersByTime(2500);
      });

      act(() => {
        tree?.update(
          <TestConfettiComponent
            options={{ autoHideDuration: 2000 }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isActive).toBe(false);
    });
  });

  describe('options', () => {
    it('accepts autoTriggerOnSolve option', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            options={{ autoTriggerOnSolve: false }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult).toBeDefined();
    });

    it('accepts autoTriggerOnRoundWin option', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            options={{ autoTriggerOnRoundWin: false }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult).toBeDefined();
    });

    it('accepts autoTriggerOnGameWin option', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestConfettiComponent
            options={{ autoTriggerOnGameWin: false }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult).toBeDefined();
    });
  });

  describe('cleanup', () => {
    it('clears timeout on unmount', () => {
      let hookResult: ReturnType<typeof useConfetti> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestConfettiComponent
            options={{ autoHideDuration: 5000 }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.trigger();
      });

      // Unmount should not throw
      expect(() => {
        act(() => {
          tree?.unmount();
        });
      }).not.toThrow();
    });
  });
});

describe('useConfettiSimple', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('initial state', () => {
    it('returns showConfetti as false initially', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.showConfetti).toBe(false);
    });

    it('returns default variant as "solve"', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.variant).toBe('solve');
    });

    it('returns triggerConfetti function', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.triggerConfetti).toBe('function');
    });

    it('returns hideConfetti function', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.hideConfetti).toBe('function');
    });
  });

  describe('triggerConfetti', () => {
    it('sets showConfetti to true', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti();
      });

      act(() => {
        tree?.update(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.showConfetti).toBe(true);
    });

    it('sets variant when provided', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti('gameWin');
      });

      act(() => {
        tree?.update(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.variant).toBe('gameWin');
    });
  });

  describe('hideConfetti', () => {
    it('sets showConfetti to false', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti();
      });

      act(() => {
        hookResult?.hideConfetti();
      });

      act(() => {
        tree?.update(
          <TestSimpleComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.showConfetti).toBe(false);
    });
  });

  describe('autoHideDuration', () => {
    it('auto-hides after specified duration', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            autoHideDuration={1000}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti();
      });

      expect(hookResult?.showConfetti).toBe(true);

      act(() => {
        jest.advanceTimersByTime(1500);
      });

      act(() => {
        tree?.update(
          <TestSimpleComponent
            autoHideDuration={1000}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.showConfetti).toBe(false);
    });

    it('clears previous timeout when triggered again', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            autoHideDuration={2000}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti();
      });

      // Advance halfway
      act(() => {
        jest.advanceTimersByTime(1000);
      });

      // Trigger again - should reset timeout
      act(() => {
        hookResult?.triggerConfetti('roundWin');
      });

      // Advance another 1500ms (total 2500 from first, 1500 from second)
      act(() => {
        jest.advanceTimersByTime(1500);
      });

      act(() => {
        tree?.update(
          <TestSimpleComponent
            autoHideDuration={2000}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      // Should still be visible since we reset timeout
      expect(hookResult?.showConfetti).toBe(true);
    });
  });

  describe('cleanup', () => {
    it('clears timeout on unmount', () => {
      let hookResult: ReturnType<typeof useConfettiSimple> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestSimpleComponent
            autoHideDuration={5000}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.triggerConfetti();
      });

      expect(() => {
        act(() => {
          tree?.unmount();
        });
      }).not.toThrow();
    });
  });
});
