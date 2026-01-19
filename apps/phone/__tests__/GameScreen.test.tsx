import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Alert } from 'react-native';
import { GameScreen } from '../src/screens/GameScreen';
import {
  useGameStore,
  useAuthStore,
  configService,
} from '@holiday-wheel/shared';

// Mock Alert
jest.spyOn(Alert, 'alert');

// Mock react-native-svg with proper default export
jest.mock('react-native-svg', () => {
  const ReactLib = require('react');
  const mockComponent = (name: string) => {
    const Component = ({ children, ...props }: { children?: ReactLib.ReactNode }) =>
      ReactLib.createElement(name, props, children);
    Component.displayName = name;
    return Component;
  };
  const Svg = mockComponent('Svg');
  return {
    __esModule: true,
    default: Svg,
    Svg,
    Path: mockComponent('Path'),
    G: mockComponent('G'),
    Circle: mockComponent('Circle'),
    Text: mockComponent('SvgText'),
    Rect: mockComponent('Rect'),
    Line: mockComponent('Line'),
    Defs: mockComponent('Defs'),
    LinearGradient: mockComponent('LinearGradient'),
    Stop: mockComponent('Stop'),
  };
});

// Mock logout function
const mockLogout = jest.fn();

// Mock shared services and stores
jest.mock('@holiday-wheel/shared', () => {
  const mockUseAuthStore = jest.fn();
  mockUseAuthStore.getState = jest.fn(() => ({ logout: mockLogout }));
  return {
    useGameStore: jest.fn(),
    useAuthStore: mockUseAuthStore,
    socketService: {
      connect: jest.fn(),
      disconnect: jest.fn(),
      joinRoom: jest.fn(),
      joinGame: jest.fn(),
      setToastCallback: jest.fn(),
      setSessionInvalidatedCallback: jest.fn(),
      spin: jest.fn(),
      guess: jest.fn(),
      buyVowel: jest.fn(),
      solve: jest.fn(),
      buzz: jest.fn(),
    },
    configService: {
      getServerUrl: jest.fn(() => Promise.resolve('http://localhost:5000')),
    },
    selectIsMyTurn: jest.fn((state) => state.isMyTurn),
    selectCanBuzz: jest.fn((state) => state.canBuzz),
    VOWELS: ['A', 'E', 'I', 'O', 'U'],
  };
});

// Create mock navigation and route
const createMockNavigation = () => ({
  navigate: jest.fn(),
  replace: jest.fn(),
  goBack: jest.fn(),
  reset: jest.fn(),
});

const createMockRoute = (room: string = 'test-room') => ({
  params: { room },
  key: 'game-screen',
  name: 'Game' as const,
});

// Default game state
const defaultGameState = {
  connected: true,
  phase: 'normal',
  puzzle: { id: 1, category: 'PHRASE', answer: 'HELLO WORLD' },
  revealed: new Set(['H', 'E', 'L', 'O']),
  players: [
    { id: 1, name: 'Player 1', total: 1000, round_bank: 500, prizes: [] },
    { id: 2, name: 'Player 2', total: 500, round_bank: 200, prizes: [] },
  ],
  activeIdx: 0,
  myPlayerIdx: 0,
  currentWedge: 500,
  wheelSlots: [500, 600, 700, 'BANKRUPT', 'LOSE A TURN'],
  lastSpinIndex: null,
  isMyTurn: true,
  canBuzz: false,
};

describe('GameScreen', () => {
  let mockNavigation: ReturnType<typeof createMockNavigation>;
  let mockRoute: ReturnType<typeof createMockRoute>;

  beforeEach(() => {
    jest.clearAllMocks();
    mockNavigation = createMockNavigation();
    mockRoute = createMockRoute();

    // Mock useGameStore with selector support
    (useGameStore as unknown as jest.Mock).mockImplementation((selector) => {
      if (typeof selector === 'function') {
        return selector(defaultGameState);
      }
      return defaultGameState;
    });

    // Mock useAuthStore
    (useAuthStore as unknown as jest.Mock).mockImplementation((selector) => {
      if (typeof selector === 'function') {
        return selector({ token: 'test-token' });
      }
      return { token: 'test-token' };
    });
  });

  describe('rendering', () => {
    it('renders the game screen', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      expect(tree?.toJSON()).not.toBeNull();
    }, 10000);

    it('shows connection status', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents?.some((t) =>
        typeof t === 'string' && t.includes('Connected')
      )).toBe(true);
    });

    it('shows game phase', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('NORMAL');
    });

    it('shows puzzle category', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('PHRASE');
    });

    it('shows player names', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Player 1');
      expect(textContents).toContain('Player 2');
    });

    it('shows player scores', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      // Deep flatten to handle nested arrays
      const flatten = (arr: unknown[]): string[] => {
        const result: string[] = [];
        const recurse = (val: unknown) => {
          if (Array.isArray(val)) {
            val.forEach(recurse);
          } else if (typeof val === 'string') {
            result.push(val);
          } else if (typeof val === 'number') {
            result.push(String(val));
          }
        };
        arr.forEach(recurse);
        return result;
      };
      const textContents = flatten(texts?.map((t) => t.props.children) ?? []);

      // Check that player scores are displayed as formatted cash
      const hasPlayer1Score = textContents.some(t => t.includes('$1,000') || t.includes('1,000'));
      const hasPlayer2Score = textContents.some(t => t.includes('$500') || t.includes('500'));

      expect(hasPlayer1Score).toBe(true);
      expect(hasPlayer2Score).toBe(true);
    });

    it('shows "You" badge for current player', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('You');
    });

    it('shows current wedge value', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('$500');
    });
  });

  describe('normal phase controls', () => {
    it('shows SPIN button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('SPIN');
    });

    it('shows GUESS button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('GUESS');
    });

    it('shows SOLVE button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('SOLVE');
    });

    it('shows letter input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const letterInput = inputs?.find((i) => i.props.placeholder === 'Letter');

      expect(letterInput).toBeDefined();
    });

    it('shows solve input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const solveInput = inputs?.find((i) =>
        i.props.placeholder === 'Solve the puzzle...'
      );

      expect(solveInput).toBeDefined();
    });
  });

  describe('tossup phase controls', () => {
    beforeEach(() => {
      const tossupState = {
        ...defaultGameState,
        phase: 'tossup',
        canBuzz: true,
      };
      (useGameStore as unknown as jest.Mock).mockImplementation((selector) => {
        if (typeof selector === 'function') {
          return selector(tossupState);
        }
        return tossupState;
      });
    });

    it('shows BUZZ button in tossup phase', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('BUZZ!');
    });

    it('shows TOSSUP phase text', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('TOSSUP');
    });

    it('does not show SPIN button in tossup phase', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).not.toContain('SPIN');
    });
  });

  describe('disconnected state', () => {
    beforeEach(() => {
      const disconnectedState = {
        ...defaultGameState,
        connected: false,
      };
      (useGameStore as unknown as jest.Mock).mockImplementation((selector) => {
        if (typeof selector === 'function') {
          return selector(disconnectedState);
        }
        return disconnectedState;
      });
    });

    it('shows connecting status when disconnected', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents?.some((t) =>
        typeof t === 'string' && t.includes('Connecting')
      )).toBe(true);
    });
  });

  describe('service initialization', () => {
    it('loads server URL on mount', async () => {
      await act(async () => {
        create(
          <GameScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      expect(configService.getServerUrl).toHaveBeenCalled();
    });
  });
});
