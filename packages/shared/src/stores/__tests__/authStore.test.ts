import { useAuthStore, selectIsAuthenticated } from '../authStore';
import type { User } from '../../types';

// Helper to create a user
const createUser = (overrides: Partial<User> = {}): User => ({
  id: 1,
  email: 'test@example.com',
  display_name: 'Test User',
  ...overrides,
});

describe('authStore', () => {
  beforeEach(() => {
    // Reset the store before each test
    useAuthStore.getState().clearAuth();
    useAuthStore.getState().setLoading(false);
    useAuthStore.getState().setError(null);
  });

  describe('initial state', () => {
    it('has null user', () => {
      expect(useAuthStore.getState().user).toBeNull();
    });

    it('has null token', () => {
      expect(useAuthStore.getState().token).toBeNull();
    });

    it('has isLoading false', () => {
      expect(useAuthStore.getState().isLoading).toBe(false);
    });

    it('has null error', () => {
      expect(useAuthStore.getState().error).toBeNull();
    });
  });

  describe('actions', () => {
    describe('setUser', () => {
      it('sets the user', () => {
        const user = createUser({ id: 42, display_name: 'Alice' });
        useAuthStore.getState().setUser(user);

        expect(useAuthStore.getState().user).toEqual(user);
      });

      it('sets user to null', () => {
        const user = createUser();
        useAuthStore.getState().setUser(user);
        useAuthStore.getState().setUser(null);

        expect(useAuthStore.getState().user).toBeNull();
      });
    });

    describe('setToken', () => {
      it('sets the token', () => {
        useAuthStore.getState().setToken('abc123');

        expect(useAuthStore.getState().token).toBe('abc123');
      });

      it('sets token to null', () => {
        useAuthStore.getState().setToken('abc123');
        useAuthStore.getState().setToken(null);

        expect(useAuthStore.getState().token).toBeNull();
      });
    });

    describe('setLoading', () => {
      it('sets isLoading to true', () => {
        useAuthStore.getState().setLoading(true);

        expect(useAuthStore.getState().isLoading).toBe(true);
      });

      it('sets isLoading to false', () => {
        useAuthStore.getState().setLoading(true);
        useAuthStore.getState().setLoading(false);

        expect(useAuthStore.getState().isLoading).toBe(false);
      });
    });

    describe('setError', () => {
      it('sets the error', () => {
        useAuthStore.getState().setError('Something went wrong');

        expect(useAuthStore.getState().error).toBe('Something went wrong');
      });

      it('clears the error with null', () => {
        useAuthStore.getState().setError('Something went wrong');
        useAuthStore.getState().setError(null);

        expect(useAuthStore.getState().error).toBeNull();
      });
    });

    describe('setAuth', () => {
      it('sets both user and token', () => {
        const user = createUser({ id: 10, email: 'auth@test.com' });
        useAuthStore.getState().setAuth(user, 'token123');

        expect(useAuthStore.getState().user).toEqual(user);
        expect(useAuthStore.getState().token).toBe('token123');
      });

      it('clears any existing error', () => {
        useAuthStore.getState().setError('Previous error');
        const user = createUser();
        useAuthStore.getState().setAuth(user, 'token123');

        expect(useAuthStore.getState().error).toBeNull();
      });
    });

    describe('clearAuth', () => {
      it('clears user and token', () => {
        const user = createUser();
        useAuthStore.getState().setAuth(user, 'token123');
        useAuthStore.getState().clearAuth();

        expect(useAuthStore.getState().user).toBeNull();
        expect(useAuthStore.getState().token).toBeNull();
      });

      it('clears any error', () => {
        useAuthStore.getState().setError('Some error');
        useAuthStore.getState().clearAuth();

        expect(useAuthStore.getState().error).toBeNull();
      });

      it('does not affect isLoading', () => {
        useAuthStore.getState().setLoading(true);
        useAuthStore.getState().clearAuth();

        // isLoading should remain unchanged
        expect(useAuthStore.getState().isLoading).toBe(true);
      });
    });
  });

  describe('selectors', () => {
    describe('selectIsAuthenticated', () => {
      it('returns true when both user and token are set', () => {
        const user = createUser();
        useAuthStore.getState().setAuth(user, 'token123');

        expect(selectIsAuthenticated(useAuthStore.getState())).toBe(true);
      });

      it('returns false when user is null', () => {
        useAuthStore.getState().setToken('token123');

        expect(selectIsAuthenticated(useAuthStore.getState())).toBe(false);
      });

      it('returns false when token is null', () => {
        const user = createUser();
        useAuthStore.getState().setUser(user);

        expect(selectIsAuthenticated(useAuthStore.getState())).toBe(false);
      });

      it('returns false when both are null', () => {
        expect(selectIsAuthenticated(useAuthStore.getState())).toBe(false);
      });
    });
  });

  describe('state persistence', () => {
    it('maintains state across multiple updates', () => {
      const user = createUser({ display_name: 'Persistent User' });
      useAuthStore.getState().setAuth(user, 'persistent-token');
      useAuthStore.getState().setLoading(true);

      const state = useAuthStore.getState();
      expect(state.user?.display_name).toBe('Persistent User');
      expect(state.token).toBe('persistent-token');
      expect(state.isLoading).toBe(true);
    });

    it('allows independent updates', () => {
      const user = createUser();
      useAuthStore.getState().setAuth(user, 'token');
      useAuthStore.getState().setError('Error occurred');

      const state = useAuthStore.getState();
      expect(state.user).toEqual(user);
      expect(state.token).toBe('token');
      expect(state.error).toBe('Error occurred');
    });
  });
});
