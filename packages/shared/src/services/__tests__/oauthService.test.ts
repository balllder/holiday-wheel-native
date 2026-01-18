import { oauthService } from '../oauthService';

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('oauthService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    oauthService.setBaseUrl('http://localhost:5000');
  });

  describe('setBaseUrl', () => {
    it('sets the base URL', () => {
      oauthService.setBaseUrl('http://example.com:3000');
      // Verify by making a request and checking the URL
      mockFetch.mockResolvedValueOnce({
        json: () => Promise.resolve({ ok: true }),
      });

      oauthService.googleAuth('test-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://example.com:3000/auth/api/oauth/google',
        expect.any(Object)
      );
    });

    it('removes trailing slash from URL', () => {
      oauthService.setBaseUrl('http://example.com:3000/');
      mockFetch.mockResolvedValueOnce({
        json: () => Promise.resolve({ ok: true }),
      });

      oauthService.googleAuth('test-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://example.com:3000/auth/api/oauth/google',
        expect.any(Object)
      );
    });
  });

  describe('googleAuth', () => {
    it('sends Google auth request with id token', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'user@gmail.com', display_name: 'Google User' },
          }),
      });

      const result = await oauthService.googleAuth('google-id-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/oauth/google',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ id_token: 'google-id-token' }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.token).toBe('auth-token');
      expect(result.user?.email).toBe('user@gmail.com');
    });

    it('returns error response on failed auth', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Invalid token',
          }),
      });

      const result = await oauthService.googleAuth('invalid-token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Invalid token');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network failure'));

      const result = await oauthService.googleAuth('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network failure');
    });

    it('handles non-Error exceptions', async () => {
      mockFetch.mockRejectedValueOnce('Unknown error');

      const result = await oauthService.googleAuth('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('appleAuth', () => {
    it('sends Apple auth request with identity token only', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'user@icloud.com', display_name: 'Apple User' },
          }),
      });

      const result = await oauthService.appleAuth('apple-identity-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/oauth/apple',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            identity_token: 'apple-identity-token',
            user_identifier: undefined,
            email: undefined,
            full_name: undefined,
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.token).toBe('auth-token');
    });

    it('sends Apple auth request with all optional fields', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'john@icloud.com', display_name: 'John Doe' },
          }),
      });

      const result = await oauthService.appleAuth(
        'apple-identity-token',
        'apple-user-id-123',
        'john@icloud.com',
        { givenName: 'John', familyName: 'Doe' }
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/oauth/apple',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            identity_token: 'apple-identity-token',
            user_identifier: 'apple-user-id-123',
            email: 'john@icloud.com',
            full_name: {
              given_name: 'John',
              family_name: 'Doe',
            },
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.user?.display_name).toBe('John Doe');
    });

    it('sends Apple auth with partial full name', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'user@icloud.com', display_name: 'Jane' },
          }),
      });

      await oauthService.appleAuth(
        'apple-identity-token',
        'apple-user-id',
        'user@icloud.com',
        { givenName: 'Jane', familyName: undefined }
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/oauth/apple',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            identity_token: 'apple-identity-token',
            user_identifier: 'apple-user-id',
            email: 'user@icloud.com',
            full_name: {
              given_name: 'Jane',
              family_name: undefined,
            },
          }),
        }
      );
    });

    it('returns error response on failed auth', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Invalid Apple token',
          }),
      });

      const result = await oauthService.appleAuth('invalid-token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Invalid Apple token');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Connection refused'));

      const result = await oauthService.appleAuth('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Connection refused');
    });

    it('handles non-Error exceptions', async () => {
      mockFetch.mockRejectedValueOnce({ code: 'UNKNOWN' });

      const result = await oauthService.appleAuth('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('URL construction', () => {
    it('uses baseUrl for all endpoints', async () => {
      oauthService.setBaseUrl('https://api.example.com');

      mockFetch.mockResolvedValue({
        json: () => Promise.resolve({ ok: true }),
      });

      await oauthService.googleAuth('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/oauth/google',
        expect.any(Object)
      );

      await oauthService.appleAuth('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/oauth/apple',
        expect.any(Object)
      );
    });
  });
});
