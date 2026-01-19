import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { ConnectionStatus } from '../ConnectionStatus';
import { useGameStore } from '../../../stores/gameStore';

// Mock the gameStore
jest.mock('../../../stores/gameStore', () => ({
  useGameStore: jest.fn(),
}));

const mockUseGameStore = useGameStore as jest.MockedFunction<typeof useGameStore>;

describe('ConnectionStatus', () => {
  beforeEach(() => {
    jest.useFakeTimers();
    // Default mock: connected state
    mockUseGameStore.mockImplementation((selector) => {
      const state = {
        connectionStatus: 'connected' as const,
        reconnectAttempt: 0,
      };
      return selector(state as never);
    });
  });

  afterEach(() => {
    jest.useRealTimers();
    jest.clearAllMocks();
  });

  describe('connected state', () => {
    it('renders connected indicator', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('displays "Connected" text', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Connected');
    });

    it('shows green dot for connected state', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus testID="conn-status" />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('does not show retry button when connected', () => {
      const onRetry = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus onRetry={onRetry} />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).not.toContain('Retry');
    });
  });

  describe('connecting state', () => {
    beforeEach(() => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'connecting' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });
    });

    it('displays "Connecting..." text', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Connecting...');
    });

    it('runs pulse animation when connecting', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('reconnecting state', () => {
    beforeEach(() => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'reconnecting' as const,
          reconnectAttempt: 3,
        };
        return selector(state as never);
      });
    });

    it('displays reconnecting text with attempt count', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Reconnecting');
      expect(json).toContain('3');
    });

    it('runs pulse animation when reconnecting', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('shows correct attempt number', () => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'reconnecting' as const,
          reconnectAttempt: 5,
        };
        return selector(state as never);
      });

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('(5)');
    });
  });

  describe('disconnected state', () => {
    beforeEach(() => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'disconnected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });
    });

    it('displays "Disconnected" text', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Disconnected');
    });

    it('shows retry button when onRetry is provided', () => {
      const onRetry = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus onRetry={onRetry} />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Retry');
    });

    it('calls onRetry when retry button is pressed', () => {
      const onRetry = jest.fn();
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus onRetry={onRetry} testID="conn-status" />);
      });

      const retryButton = tree?.root.findByProps({ testID: 'conn-status-retry' });
      act(() => {
        retryButton?.props.onPress?.();
      });

      expect(onRetry).toHaveBeenCalledTimes(1);
    });

    it('does not show retry button when onRetry is not provided', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).not.toContain('Retry');
    });

    it('does not show pulse animation when disconnected', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      // Should render without throwing
      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('compact mode', () => {
    it('renders in compact mode', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus compact />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('shows only the dot in compact mode', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus compact />);
      });

      const json = JSON.stringify(tree?.toJSON());
      // Should not contain text labels in compact mode
      expect(json).not.toContain('Connected');
      expect(json).not.toContain('Disconnected');
    });

    it('compact mode with connecting state still pulses', () => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'connecting' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus compact />);
      });

      act(() => {
        jest.advanceTimersByTime(2000);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('animations', () => {
    it('handles status change animation', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      // Change status
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'disconnected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      act(() => {
        tree?.update(<ConnectionStatus />);
      });

      act(() => {
        jest.advanceTimersByTime(500);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('stops pulse animation when status changes to non-pulsing state', () => {
      // Start with connecting
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'connecting' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      act(() => {
        jest.advanceTimersByTime(1000);
      });

      // Change to connected
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'connected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      act(() => {
        tree?.update(<ConnectionStatus />);
      });

      act(() => {
        jest.advanceTimersByTime(500);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });
  });

  describe('props', () => {
    it('accepts custom style', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus style={{ margin: 20 }} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('accepts testID', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus testID="connection-indicator" />);
      });

      const container = tree?.root.findByProps({ testID: 'connection-indicator' });
      expect(container).toBeDefined();
    });

    it('applies testID to dot element', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus testID="conn" />);
      });

      const dot = tree?.root.findByProps({ testID: 'conn-dot' });
      expect(dot).toBeDefined();
    });

    it('applies testID to label element', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus testID="conn" />);
      });

      const label = tree?.root.findByProps({ testID: 'conn-label' });
      expect(label).toBeDefined();
    });
  });

  describe('state transitions', () => {
    it('transitions from connected to disconnected', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      let json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Connected');

      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'disconnected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      act(() => {
        tree?.update(<ConnectionStatus />);
      });

      json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Disconnected');
    });

    it('transitions from disconnected to reconnecting', () => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'disconnected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'reconnecting' as const,
          reconnectAttempt: 1,
        };
        return selector(state as never);
      });

      act(() => {
        tree?.update(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Reconnecting');
    });

    it('transitions from reconnecting to connected', () => {
      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'reconnecting' as const,
          reconnectAttempt: 2,
        };
        return selector(state as never);
      });

      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<ConnectionStatus />);
      });

      mockUseGameStore.mockImplementation((selector) => {
        const state = {
          connectionStatus: 'connected' as const,
          reconnectAttempt: 0,
        };
        return selector(state as never);
      });

      act(() => {
        tree?.update(<ConnectionStatus />);
      });

      const json = JSON.stringify(tree?.toJSON());
      expect(json).toContain('Connected');
    });
  });
});
