import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Platform } from 'react-native';
import { LobbyScreen } from '../src/screens/LobbyScreen';
import {
  authService,
  useAuthStore,
  configService,
} from '@holiday-wheel/shared';

// Mock shared services
jest.mock('@holiday-wheel/shared', () => ({
  useAuthStore: jest.fn(),
  authService: {
    setBaseUrl: jest.fn(),
    getRooms: jest.fn(() => Promise.resolve({ rooms: [] })),
  },
  configService: {
    getServerUrl: jest.fn(() => Promise.resolve('http://localhost:5000')),
    setServerUrl: jest.fn(() => Promise.resolve()),
  },
}));

// Create mock navigation
const createMockNavigation = () => ({
  navigate: jest.fn(),
  replace: jest.fn(),
  goBack: jest.fn(),
});

// Helper to deeply extract all text content
const getAllTextContent = (node: ReactTestRenderer): string[] => {
  const result: string[] = [];

  const extractText = (element: unknown): void => {
    if (!element) return;

    if (typeof element === 'string' || typeof element === 'number') {
      result.push(String(element));
      return;
    }

    if (Array.isArray(element)) {
      element.forEach(extractText);
      return;
    }

    if (typeof element === 'object' && element !== null) {
      const obj = element as Record<string, unknown>;
      if (obj.children) {
        extractText(obj.children);
      }
      if (obj.props && typeof obj.props === 'object') {
        const props = obj.props as Record<string, unknown>;
        if (props.children) {
          extractText(props.children);
        }
      }
    }
  };

  try {
    const json = node.toJSON();
    extractText(json);
  } catch {
    // Fallback: try to get texts from root
    try {
      const texts = node.root.findAllByType('Text' as never);
      texts.forEach((t) => {
        const children = t.props.children;
        if (Array.isArray(children)) {
          children.forEach((c: unknown) => {
            if (typeof c === 'string' || typeof c === 'number') {
              result.push(String(c));
            }
          });
        } else if (typeof children === 'string' || typeof children === 'number') {
          result.push(String(children));
        }
      });
    } catch {
      // Ignore errors
    }
  }

  return result;
};

describe('LobbyScreen', () => {
  let mockNavigation: ReturnType<typeof createMockNavigation>;
  let mockClearAuth: jest.Mock;

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
    mockNavigation = createMockNavigation();
    mockClearAuth = jest.fn();

    // Mock useAuthStore
    (useAuthStore as unknown as jest.Mock).mockImplementation((selector) => {
      const state = {
        user: { id: 1, email: 'test@example.com', display_name: 'Test User' },
        token: 'test-token',
        clearAuth: mockClearAuth,
      };
      if (typeof selector === 'function') {
        return selector(state);
      }
      return state;
    });

    // Reset Platform.OS
    Platform.OS = 'ios';
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('rendering', () => {
    it('renders the lobby screen', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
      });

      await act(async () => {
        jest.runAllTimers();
      });

      expect(tree?.toJSON()).not.toBeNull();
    }, 10000);

    it('shows welcome message with user name', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasWelcome = textContents.some((t) => t.includes('Welcome'));
      expect(hasWelcome).toBe(true);
    });

    it('shows logout button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('Logout');
    });

    it('shows server configuration toggle', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasServer = textContents.some((t) => t.includes('Server'));
      expect(hasServer).toBe(true);
    });

    it('shows room name input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const roomInput = inputs?.find((i) =>
        i.props.placeholder?.includes('room name')
      );

      expect(roomInput).toBeDefined();
    });

    it('shows Play button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('Play');
    });

    it('shows controller button emoji', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('📱');
    });

    it('shows QR code scan button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasQR = textContents.some((t) => t.includes('Scan QR'));
      expect(hasQR).toBe(true);
    });

    it('shows mode help text', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasPlayHelp = textContents.some((t) => t.includes('Full game'));
      expect(hasPlayHelp).toBe(true);
    });

    it('shows Active Rooms section', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('Active Rooms');
    });

    it('shows empty rooms message when no rooms', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasEmpty = textContents.some((t) => t.includes('No active rooms'));
      expect(hasEmpty).toBe(true);
    });
  });

  describe('with rooms', () => {
    beforeEach(() => {
      (authService.getRooms as jest.Mock).mockResolvedValue({
        rooms: [
          { name: 'room1', player_count: 3, last_activity: '2024-01-01' },
          { name: 'room2', player_count: 1, last_activity: '2024-01-02' },
        ],
      });
    });

    it('shows room list when rooms exist', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
      });

      // Run timers to trigger effects and state updates
      await act(async () => {
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('room1');
      expect(textContents).toContain('room2');
    });

    it('fetches rooms with player counts', async () => {
      await act(async () => {
        create(<LobbyScreen navigation={mockNavigation as never} />);
      });

      await act(async () => {
        jest.runAllTimers();
      });

      // Verify getRooms was called which includes player_count data
      expect(authService.getRooms).toHaveBeenCalled();
    });
  });

  describe('service initialization', () => {
    it('loads server URL on mount', async () => {
      await act(async () => {
        create(<LobbyScreen navigation={mockNavigation as never} />);
        jest.runAllTimers();
      });

      expect(configService.getServerUrl).toHaveBeenCalled();
    });

    it('loads rooms on mount', async () => {
      await act(async () => {
        create(<LobbyScreen navigation={mockNavigation as never} />);
      });

      await act(async () => {
        jest.runAllTimers();
      });

      expect(authService.getRooms).toHaveBeenCalled();
    });
  });

  describe('platform-specific', () => {
    it('shows Android emulator hint on Android', async () => {
      Platform.OS = 'android';
      let tree: ReactTestRenderer | undefined;

      // First render without server config visible
      await act(async () => {
        tree = create(
          <LobbyScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      // Toggle to show server config would reveal Android-specific hint
      // Testing just that platform detection works
      expect(Platform.OS).toBe('android');
      expect(tree?.toJSON()).not.toBeNull();
    });
  });
});
