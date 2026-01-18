import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { Alert, Platform } from 'react-native';
import { LoginScreen } from '../src/screens/LoginScreen';
import {
  authService,
  oauthService,
  useAuthStore,
} from '@holiday-wheel/shared';
import { GoogleSignin } from '@react-native-google-signin/google-signin';

// Mock Alert
jest.spyOn(Alert, 'alert');
jest.spyOn(Alert, 'prompt');

// Mock shared services
jest.mock('@holiday-wheel/shared', () => ({
  useAuthStore: jest.fn(),
  authService: {
    setBaseUrl: jest.fn(),
    login: jest.fn(),
  },
  passkeyService: {
    setBaseUrl: jest.fn(),
    loginStart: jest.fn(),
    loginFinish: jest.fn(),
  },
  oauthService: {
    setBaseUrl: jest.fn(),
    googleAuth: jest.fn(),
    appleAuth: jest.fn(),
  },
}));

// Create mock navigation
const createMockNavigation = () => ({
  navigate: jest.fn(),
  replace: jest.fn(),
  goBack: jest.fn(),
});

describe('LoginScreen', () => {
  let mockNavigation: ReturnType<typeof createMockNavigation>;
  let mockSetAuth: jest.Mock;

  beforeEach(() => {
    jest.clearAllMocks();
    mockNavigation = createMockNavigation();
    mockSetAuth = jest.fn();

    // Mock useAuthStore to return setAuth function
    (useAuthStore as unknown as jest.Mock).mockImplementation((selector) => {
      if (typeof selector === 'function') {
        return selector({ setAuth: mockSetAuth });
      }
      return { setAuth: mockSetAuth };
    });

    // Reset Platform.OS for each test
    Platform.OS = 'ios';
  });

  describe('rendering', () => {
    it('renders the login screen', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('shows app title and logo', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Holiday Wheel');
      expect(textContents).toContain('of Fortune');
    });

    it('shows wheel emoji logo', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const emojiText = texts?.find((t) => t.props.children === '🎡');

      expect(emojiText).toBeDefined();
    });

    it('shows passkey button on iOS', () => {
      Platform.OS = 'ios';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Sign in with Passkey');
    });

    it('shows passkey button on Android', () => {
      Platform.OS = 'android';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Sign in with Passkey');
    });

    it('shows Google sign-in button', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Sign in with Google');
    });

    it('shows Apple sign-in button on iOS', () => {
      Platform.OS = 'ios';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Sign in with Apple');
    });

    it('hides Apple sign-in button on Android', () => {
      Platform.OS = 'android';
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).not.toContain('Sign in with Apple');
    });

    it('shows email sign-in toggle', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('Sign in with email');
    });

    it('shows "or" divider', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => t.props.children).flat();

      expect(textContents).toContain('or');
    });

    it('shows register link', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const textContents = texts?.map((t) => {
        const children = t.props.children;
        if (Array.isArray(children)) {
          return children.map(c => typeof c === 'string' ? c : '').join('');
        }
        return typeof children === 'string' ? children : '';
      });

      const hasRegisterPrompt = textContents.some((t) =>
        t.includes("Don't have an account?")
      );
      const hasRegisterLink = textContents.some((t) => t.includes('Register'));

      expect(hasRegisterPrompt).toBe(true);
      expect(hasRegisterLink).toBe(true);
    });
  });

  describe('service initialization', () => {
    it('initializes authService on mount', () => {
      act(() => {
        create(<LoginScreen navigation={mockNavigation as never} />);
      });

      expect(authService.setBaseUrl).toHaveBeenCalled();
    });

    it('initializes oauthService on mount', () => {
      act(() => {
        create(<LoginScreen navigation={mockNavigation as never} />);
      });

      expect(oauthService.setBaseUrl).toHaveBeenCalled();
    });

    it('configures GoogleSignin on mount', () => {
      act(() => {
        create(<LoginScreen navigation={mockNavigation as never} />);
      });

      expect(GoogleSignin.configure).toHaveBeenCalled();
    });
  });

  describe('platform-specific behavior', () => {
    it('shows passkey emoji icon', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const passkeyEmoji = texts?.find((t) => t.props.children === '🔐');

      expect(passkeyEmoji).toBeDefined();
    });

    it('shows Google G icon', () => {
      let tree: ReactTestRenderer | undefined;
      act(() => {
        tree = create(<LoginScreen navigation={mockNavigation as never} />);
      });

      const texts = tree?.root.findAllByType('Text' as never);
      const googleIcon = texts?.find((t) => t.props.children === 'G');

      expect(googleIcon).toBeDefined();
    });
  });
});
