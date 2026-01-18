import { authService } from '../authService';
import { passkeyService } from '../passkeyService';
import { oauthService } from '../oauthService';

// Mock the dependent services
jest.mock('../passkeyService', () => ({
  passkeyService: {
    setBaseUrl: jest.fn(),
  },
}));

jest.mock('../oauthService', () => ({
  oauthService: {
    setBaseUrl: jest.fn(),
  },
}));

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('authService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    authService.setBaseUrl('http://localhost:5000');
  });

  describe('setBaseUrl', () => {
    it('sets the base URL', () => {
      authService.setBaseUrl('http://example.com:3000');
      expect(authService.getBaseUrl()).toBe('http://example.com:3000');
    });

    it('removes trailing slash from URL', () => {
      authService.setBaseUrl('http://example.com:3000/');
      expect(authService.getBaseUrl()).toBe('http://example.com:3000');
    });

    it('syncs URL to passkeyService', () => {
      authService.setBaseUrl('http://example.com:3000');
      expect(passkeyService.setBaseUrl).toHaveBeenCalledWith(
        'http://example.com:3000'
      );
    });

    it('syncs URL to oauthService', () => {
      authService.setBaseUrl('http://example.com:3000');
      expect(oauthService.setBaseUrl).toHaveBeenCalledWith(
        'http://example.com:3000'
      );
    });
  });

  describe('getBaseUrl', () => {
    it('returns the current base URL', () => {
      authService.setBaseUrl('http://test.com');
      expect(authService.getBaseUrl()).toBe('http://test.com');
    });
  });

  describe('login', () => {
    it('sends login request with email and password', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'test@example.com', display_name: 'Test' },
          }),
      });

      const result = await authService.login('test@example.com', 'password123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/login',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'test@example.com',
            password: 'password123',
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.token).toBe('auth-token');
      expect(result.user?.email).toBe('test@example.com');
    });

    it('returns error response on failed login', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Invalid credentials',
          }),
      });

      const result = await authService.login('test@example.com', 'wrongpass');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Invalid credentials');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network failure'));

      const result = await authService.login('test@example.com', 'password');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network failure');
    });

    it('handles non-Error exceptions', async () => {
      mockFetch.mockRejectedValueOnce('Unknown error');

      const result = await authService.login('test@example.com', 'password');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('register', () => {
    it('sends registration request with all fields', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'new-token',
            user: { id: 2, email: 'new@example.com', display_name: 'New User' },
          }),
      });

      const result = await authService.register(
        'new@example.com',
        'securepass',
        'New User'
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/register',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'new@example.com',
            password: 'securepass',
            display_name: 'New User',
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.user?.display_name).toBe('New User');
    });

    it('returns error on duplicate email', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Email already registered',
          }),
      });

      const result = await authService.register(
        'existing@example.com',
        'password',
        'User'
      );

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Email already registered');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Connection refused'));

      const result = await authService.register(
        'test@example.com',
        'password',
        'Test'
      );

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Connection refused');
    });
  });

  describe('getRooms', () => {
    it('fetches rooms with authorization header', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            rooms: [
              { name: 'room1', player_count: 3, last_activity: '2024-01-01' },
              { name: 'room2', player_count: 1, last_activity: '2024-01-02' },
            ],
          }),
      });

      const result = await authService.getRooms('my-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/rooms',
        {
          headers: {
            Authorization: 'Bearer my-token',
          },
        }
      );

      expect(result.rooms).toHaveLength(2);
      expect(result.rooms[0].name).toBe('room1');
    });

    it('returns empty array on non-ok response', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
      });

      const result = await authService.getRooms('invalid-token');

      expect(result.rooms).toEqual([]);
    });

    it('returns empty array on network error', async () => {
      const consoleSpy = jest.spyOn(console, 'error').mockImplementation();
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const result = await authService.getRooms('token');

      expect(result.rooms).toEqual([]);
      expect(consoleSpy).toHaveBeenCalled();
      consoleSpy.mockRestore();
    });
  });

  describe('verifyToken', () => {
    it('verifies valid token and returns user', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: () =>
          Promise.resolve({
            ok: true,
            user: { id: 1, email: 'verified@example.com', display_name: 'User' },
          }),
      });

      const result = await authService.verifyToken('valid-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/verify',
        {
          headers: {
            Authorization: 'Bearer valid-token',
          },
        }
      );

      expect(result.ok).toBe(true);
      expect(result.user?.email).toBe('verified@example.com');
    });

    it('returns ok: false for invalid token', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        status: 401,
      });

      const result = await authService.verifyToken('expired-token');

      expect(result.ok).toBe(false);
      expect(result.user).toBeUndefined();
    });

    it('returns ok: false on network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const result = await authService.verifyToken('token');

      expect(result.ok).toBe(false);
    });
  });

  describe('URL construction', () => {
    it('uses baseUrl for all endpoints', async () => {
      authService.setBaseUrl('https://api.example.com');

      mockFetch.mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ ok: true }),
      });

      await authService.login('test@example.com', 'pass');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/login',
        expect.any(Object)
      );

      await authService.register('test@example.com', 'pass', 'Test');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/register',
        expect.any(Object)
      );

      await authService.getRooms('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/rooms',
        expect.any(Object)
      );

      await authService.verifyToken('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/verify',
        expect.any(Object)
      );
    });
  });
});
