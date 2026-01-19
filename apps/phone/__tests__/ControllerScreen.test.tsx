import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Alert, Vibration } from 'react-native';
import { ControllerScreen } from '../src/screens/ControllerScreen';
import {
  useGameStore,
  useAuthStore,
  configService,
} from '@holiday-wheel/shared';

// Mock Alert and Vibration
jest.spyOn(Alert, 'alert');
jest.spyOn(Vibration, 'vibrate');

// Mock logout function
const mockLogout = jest.fn();

// Mock shared services and stores
jest.mock('@holiday-wheel/shared', () => {
  const mockUseAuthStore = jest.fn();
  mockUseAuthStore.getState = jest.fn(() => ({ logout: jest.fn() }));
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
    selectMyPlayer: jest.fn((state) =>
      state.myPlayerIdx !== null ? state.players[state.myPlayerIdx] : null
    ),
    VOWELS: ['A', 'E', 'I', 'O', 'U'],
    useToast: jest.fn(() => ({
      showToast: jest.fn(),
      hideToast: jest.fn(),
      ToastComponent: () => null,
    })),
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
  key: 'controller-screen',
  name: 'Controller' as const,
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

describe('ControllerScreen', () => {
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
    it('renders the controller screen', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
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
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('●');
    });

    it('shows room name', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('test-room');
    });

    it('shows game phase', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
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
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('PHRASE');
    });

    it('shows player name when joined', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Player 1');
    });

    it('shows player round bank', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      // Score rendered as formatted cash (e.g., "$500")
      expect(textContents).toContain('$500');
    });
  });

  describe('normal phase controls', () => {
    it('shows SPIN button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
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
          <ControllerScreen
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
          <ControllerScreen
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
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const letterInput = inputs?.find((i) => i.props.placeholder === '?');

      expect(letterInput).toBeDefined();
    });

    it('shows solve input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
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
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('BUZZ!');
    });

    it('shows buzz subtext', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Tap to buzz in');
    });

    it('shows TOSSUP phase text', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
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
          <ControllerScreen
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

  describe('turn indicator', () => {
    it('shows YOUR TURN when it is my turn', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('YOUR TURN!');
    });

    it('shows waiting message when not my turn', async () => {
      const notMyTurnState = {
        ...defaultGameState,
        isMyTurn: false,
        activeIdx: 1,
      };
      (useGameStore as unknown as jest.Mock).mockImplementation((selector) => {
        if (typeof selector === 'function') {
          return selector(notMyTurnState);
        }
        return notMyTurnState;
      });

      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Waiting for your turn...');
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

    it('shows disconnected indicator', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('○');
    });
  });

  describe('service initialization', () => {
    it('loads server URL on mount', async () => {
      await act(async () => {
        create(
          <ControllerScreen
            navigation={mockNavigation as never}
            route={mockRoute as never}
          />
        );
      });

      expect(configService.getServerUrl).toHaveBeenCalled();
    });
  });
});
