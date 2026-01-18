import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Platform } from 'react-native';
import { RegisterScreen } from '../src/screens/RegisterScreen';
import {
  authService,
  passkeyService,
  oauthService,
  useAuthStore,
} from '@holiday-wheel/shared';

// Mock shared services
jest.mock('@holiday-wheel/shared', () => ({
  useAuthStore: jest.fn(),
  authService: {
    setBaseUrl: jest.fn(),
    register: jest.fn(() => Promise.resolve({ ok: true })),
  },
  passkeyService: {
    setBaseUrl: jest.fn(),
    registerStart: jest.fn(() =>
      Promise.resolve({ ok: true, options: { challenge: 'test' } })
    ),
    registerFinish: jest.fn(() =>
      Promise.resolve({
        ok: true,
        token: 'test-token',
        user: { id: 1, email: 'test@example.com', display_name: 'Test' },
      })
    ),
  },
  oauthService: {
    setBaseUrl: jest.fn(),
    googleAuth: jest.fn(() =>
      Promise.resolve({
        ok: true,
        token: 'google-token',
        user: { id: 1, email: 'google@example.com', display_name: 'Google User' },
      })
    ),
    appleAuth: jest.fn(() =>
      Promise.resolve({
        ok: true,
        token: 'apple-token',
        user: { id: 1, email: 'apple@example.com', display_name: 'Apple User' },
      })
    ),
  },
}));

// Mock react-native-passkeys
jest.mock('react-native-passkeys', () => ({
  create: jest.fn(() => Promise.resolve({ id: 'credential-id' })),
}));

// Mock Google Sign-In
jest.mock('@react-native-google-signin/google-signin', () => ({
  GoogleSignin: {
    configure: jest.fn(),
    hasPlayServices: jest.fn(() => Promise.resolve(true)),
    signIn: jest.fn(() =>
      Promise.resolve({ data: { idToken: 'google-id-token' } })
    ),
  },
  statusCodes: {
    SIGN_IN_CANCELLED: 'SIGN_IN_CANCELLED',
    IN_PROGRESS: 'IN_PROGRESS',
    PLAY_SERVICES_NOT_AVAILABLE: 'PLAY_SERVICES_NOT_AVAILABLE',
  },
  isErrorWithCode: jest.fn(() => false),
}));

// Mock Apple Auth
jest.mock('@invertase/react-native-apple-authentication', () => ({
  __esModule: true,
  default: {
    performRequest: jest.fn(() =>
      Promise.resolve({
        user: 'apple-user-id',
        identityToken: 'apple-identity-token',
        email: 'apple@example.com',
        fullName: { givenName: 'Apple', familyName: 'User' },
      })
    ),
    getCredentialStateForUser: jest.fn(() => Promise.resolve(1)), // AUTHORIZED
    Operation: { LOGIN: 0 },
    Scope: { EMAIL: 0, FULL_NAME: 1 },
    State: { AUTHORIZED: 1 },
    Error: { CANCELED: '1001' },
  },
}));

// Create mock navigation
const createMockNavigation = () => ({
  navigate: jest.fn(),
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

describe('RegisterScreen', () => {
  let mockNavigation: ReturnType<typeof createMockNavigation>;
  let mockSetAuth: jest.Mock;

  beforeEach(() => {
    jest.clearAllMocks();
    jest.useFakeTimers();
    mockNavigation = createMockNavigation();
    mockSetAuth = jest.fn();

    // Mock useAuthStore
    (useAuthStore as unknown as jest.Mock).mockImplementation((selector) => {
      const state = {
        setAuth: mockSetAuth,
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
    it('renders the register screen', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
      });

      await act(async () => {
        jest.runAllTimers();
      });

      expect(tree?.toJSON()).not.toBeNull();
    }, 10000);

    it('shows Create Account header', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('Create Account');
    });

    it('shows Google sign-in button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasGoogle = textContents.some((t) => t.includes('Google'));
      expect(hasGoogle).toBe(true);
    });

    it('shows Apple sign-in button on iOS', async () => {
      Platform.OS = 'ios';
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasApple = textContents.some((t) => t.includes('Apple'));
      expect(hasApple).toBe(true);
    });

    it('shows passkey button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasPasskey = textContents.some((t) => t.includes('Passkey'));
      expect(hasPasskey).toBe(true);
    });

    it('shows email input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const emailInput = inputs?.find(
        (i) => i.props.placeholder === 'Email'
      );

      expect(emailInput).toBeDefined();
    });

    it('shows display name input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const nameInput = inputs?.find(
        (i) => i.props.placeholder === 'Display Name'
      );

      expect(nameInput).toBeDefined();
    });

    it('shows password input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const passwordInput = inputs?.find(
        (i) => i.props.placeholder === 'Password'
      );

      expect(passwordInput).toBeDefined();
    });

    it('shows confirm password input', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const confirmInput = inputs?.find(
        (i) => i.props.placeholder === 'Confirm Password'
      );

      expect(confirmInput).toBeDefined();
    });

    it('shows Register button', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      expect(textContents).toContain('Register');
    });

    it('shows login link', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasLoginLink = textContents.some((t) =>
        t.includes('Already have an account')
      );
      expect(hasLoginLink).toBe(true);
    });

    it('shows divider text', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasDivider = textContents.some((t) =>
        t.includes('or register with email')
      );
      expect(hasDivider).toBe(true);
    });
  });

  describe('service initialization', () => {
    it('sets base URL on authService', async () => {
      await act(async () => {
        create(<RegisterScreen navigation={mockNavigation as never} />);
        jest.runAllTimers();
      });

      expect(authService.setBaseUrl).toHaveBeenCalled();
    });

    it('sets base URL on oauthService', async () => {
      await act(async () => {
        create(<RegisterScreen navigation={mockNavigation as never} />);
        jest.runAllTimers();
      });

      expect(oauthService.setBaseUrl).toHaveBeenCalled();
    });
  });

  describe('form inputs', () => {
    it('has four text inputs', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      expect(inputs?.length).toBe(4);
    });

    it('password inputs have secureTextEntry', async () => {
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const inputs = tree?.root.findAllByType('TextInput' as never);
      const passwordInputs = inputs?.filter((i) => i.props.secureTextEntry);

      expect(passwordInputs?.length).toBe(2);
    });
  });

  describe('platform-specific', () => {
    it('shows passkey button on iOS', async () => {
      Platform.OS = 'ios';
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasPasskey = textContents.some((t) => t.includes('Passkey'));
      expect(hasPasskey).toBe(true);
    });

    it('shows passkey button on Android', async () => {
      Platform.OS = 'android';
      let tree: ReactTestRenderer | undefined;
      await act(async () => {
        tree = create(
          <RegisterScreen navigation={mockNavigation as never} />
        );
        jest.runAllTimers();
      });

      const textContents = getAllTextContent(tree!);
      const hasPasskey = textContents.some((t) => t.includes('Passkey'));
      expect(hasPasskey).toBe(true);
    });
  });
});
