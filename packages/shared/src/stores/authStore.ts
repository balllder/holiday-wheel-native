import { create } from 'zustand';
import type { User } from '../types';

interface AuthStore {
  // State
  user: User | null;
  token: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setUser: (user: User | null) => void;
  setToken: (token: string | null) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  setAuth: (user: User, token: string) => void;
  clearAuth: () => void;
}

export const useAuthStore = create<AuthStore>((set) => ({
  user: null,
  token: null,
  isLoading: false,
  error: null,

  setUser: (user) => set({ user }),

  setToken: (token) => set({ token }),

  setLoading: (isLoading) => set({ isLoading }),

  setError: (error) => set({ error }),

  setAuth: (user, token) => set({ user, token, error: null }),

  clearAuth: () => set({ user: null, token: null, error: null }),
}));

// Selectors
export const selectIsAuthenticated = (state: AuthStore): boolean => {
  return state.user !== null && state.token !== null;
};
