import { passkeyService } from '../passkeyService';

// Mock fetch globally
const mockFetch = jest.fn();
global.fetch = mockFetch;

describe('passkeyService', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    passkeyService.setBaseUrl('http://localhost:5000');
  });

  describe('setBaseUrl', () => {
    it('sets the base URL', () => {
      passkeyService.setBaseUrl('http://example.com:3000');
      mockFetch.mockResolvedValueOnce({
        json: () => Promise.resolve({ ok: true }),
      });

      passkeyService.loginStart();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://example.com:3000/auth/api/passkey/login/start',
        expect.any(Object)
      );
    });

    it('removes trailing slash from URL', () => {
      passkeyService.setBaseUrl('http://example.com:3000/');
      mockFetch.mockResolvedValueOnce({
        json: () => Promise.resolve({ ok: true }),
      });

      passkeyService.loginStart();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://example.com:3000/auth/api/passkey/login/start',
        expect.any(Object)
      );
    });
  });

  describe('registerStart', () => {
    it('sends registration start request with email and display name', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            options: { challenge: 'test-challenge' },
          }),
      });

      const result = await passkeyService.registerStart(
        'user@example.com',
        'Test User'
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/register/start',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'user@example.com',
            display_name: 'Test User',
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.options).toEqual({ challenge: 'test-challenge' });
    });

    it('returns error on failed request', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Email already registered',
          }),
      });

      const result = await passkeyService.registerStart(
        'existing@example.com',
        'User'
      );

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Email already registered');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network failure'));

      const result = await passkeyService.registerStart('user@example.com', 'User');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network failure');
    });

    it('handles non-Error exceptions', async () => {
      mockFetch.mockRejectedValueOnce('Unknown error');

      const result = await passkeyService.registerStart('user@example.com', 'User');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('registerFinish', () => {
    it('sends registration finish request with credential', async () => {
      const mockCredential = {
        id: 'credential-id',
        rawId: 'raw-id',
        response: { attestationObject: 'attestation', clientDataJSON: 'client-data' },
      };

      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'user@example.com', display_name: 'User' },
          }),
      });

      const result = await passkeyService.registerFinish(
        'user@example.com',
        mockCredential
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/register/finish',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            email: 'user@example.com',
            credential: mockCredential,
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.token).toBe('auth-token');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Connection refused'));

      const result = await passkeyService.registerFinish('user@example.com', {});

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Connection refused');
    });
  });

  describe('loginStart', () => {
    it('sends login start request without email', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            options: { challenge: 'login-challenge' },
          }),
      });

      const result = await passkeyService.loginStart();

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/login/start',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email: undefined }),
        }
      );

      expect(result.ok).toBe(true);
    });

    it('sends login start request with email', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            options: { challenge: 'login-challenge', allowCredentials: [] },
          }),
      });

      const result = await passkeyService.loginStart('user@example.com');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/login/start',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email: 'user@example.com' }),
        }
      );

      expect(result.ok).toBe(true);
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Timeout'));

      const result = await passkeyService.loginStart();

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Timeout');
    });
  });

  describe('loginFinish', () => {
    it('sends login finish request with credential', async () => {
      const mockCredential = {
        id: 'credential-id',
        rawId: 'raw-id',
        response: { authenticatorData: 'auth-data', signature: 'sig' },
      };

      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            token: 'auth-token',
            user: { id: 1, email: 'user@example.com', display_name: 'User' },
          }),
      });

      const result = await passkeyService.loginFinish(mockCredential);

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/login/finish',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ credential: mockCredential }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.token).toBe('auth-token');
    });

    it('returns error on invalid credential', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Invalid credential',
          }),
      });

      const result = await passkeyService.loginFinish({});

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Invalid credential');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Server unavailable'));

      const result = await passkeyService.loginFinish({});

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Server unavailable');
    });
  });

  describe('listPasskeys', () => {
    it('sends list request with auth token', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            passkeys: [
              { id: '1', device_name: 'iPhone', created_at: '2024-01-01' },
              { id: '2', device_name: 'MacBook', created_at: '2024-01-02' },
            ],
          }),
      });

      const result = await passkeyService.listPasskeys('auth-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/list',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: 'Bearer auth-token',
          },
        }
      );

      expect(result.ok).toBe(true);
      expect(result.passkeys).toHaveLength(2);
    });

    it('returns error on unauthorized', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Unauthorized',
          }),
      });

      const result = await passkeyService.listPasskeys('invalid-token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Unauthorized');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'));

      const result = await passkeyService.listPasskeys('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('addPasskeyStart', () => {
    it('sends add passkey start request with token', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            options: { challenge: 'add-challenge' },
          }),
      });

      const result = await passkeyService.addPasskeyStart('auth-token');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/add/start',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: 'Bearer auth-token',
          },
          body: JSON.stringify({ device_name: undefined }),
        }
      );

      expect(result.ok).toBe(true);
    });

    it('sends add passkey start request with device name', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            options: { challenge: 'add-challenge' },
          }),
      });

      const result = await passkeyService.addPasskeyStart('auth-token', 'My iPhone');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/add/start',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: 'Bearer auth-token',
          },
          body: JSON.stringify({ device_name: 'My iPhone' }),
        }
      );

      expect(result.ok).toBe(true);
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Timeout'));

      const result = await passkeyService.addPasskeyStart('token');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Timeout');
    });
  });

  describe('addPasskeyFinish', () => {
    it('sends add passkey finish request', async () => {
      const mockCredential = { id: 'new-credential' };

      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            message: 'Passkey added successfully',
          }),
      });

      const result = await passkeyService.addPasskeyFinish(
        'auth-token',
        'user@example.com',
        mockCredential
      );

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/add/finish',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: 'Bearer auth-token',
          },
          body: JSON.stringify({
            email: 'user@example.com',
            credential: mockCredential,
          }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.message).toBe('Passkey added successfully');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Connection failed'));

      const result = await passkeyService.addPasskeyFinish('token', 'email', {});

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Connection failed');
    });
  });

  describe('deletePasskey', () => {
    it('sends delete passkey request', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: true,
            message: 'Passkey deleted',
          }),
      });

      const result = await passkeyService.deletePasskey('auth-token', 'credential-123');

      expect(mockFetch).toHaveBeenCalledWith(
        'http://localhost:5000/auth/api/passkey/delete',
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: 'Bearer auth-token',
          },
          body: JSON.stringify({ credential_id: 'credential-123' }),
        }
      );

      expect(result.ok).toBe(true);
      expect(result.message).toBe('Passkey deleted');
    });

    it('returns error when passkey not found', async () => {
      mockFetch.mockResolvedValueOnce({
        json: () =>
          Promise.resolve({
            ok: false,
            error: 'Passkey not found',
          }),
      });

      const result = await passkeyService.deletePasskey('token', 'invalid-id');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Passkey not found');
    });

    it('handles network error', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Server error'));

      const result = await passkeyService.deletePasskey('token', 'id');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Server error');
    });

    it('handles non-Error exceptions', async () => {
      mockFetch.mockRejectedValueOnce({ status: 500 });

      const result = await passkeyService.deletePasskey('token', 'id');

      expect(result.ok).toBe(false);
      expect(result.error).toBe('Network error');
    });
  });

  describe('URL construction', () => {
    it('uses baseUrl for all endpoints', async () => {
      passkeyService.setBaseUrl('https://api.example.com');

      mockFetch.mockResolvedValue({
        json: () => Promise.resolve({ ok: true }),
      });

      await passkeyService.registerStart('email', 'name');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/register/start',
        expect.any(Object)
      );

      await passkeyService.registerFinish('email', {});
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/register/finish',
        expect.any(Object)
      );

      await passkeyService.loginStart();
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/login/start',
        expect.any(Object)
      );

      await passkeyService.loginFinish({});
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/login/finish',
        expect.any(Object)
      );

      await passkeyService.listPasskeys('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/list',
        expect.any(Object)
      );

      await passkeyService.addPasskeyStart('token');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/add/start',
        expect.any(Object)
      );

      await passkeyService.addPasskeyFinish('token', 'email', {});
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/add/finish',
        expect.any(Object)
      );

      await passkeyService.deletePasskey('token', 'id');
      expect(mockFetch).toHaveBeenCalledWith(
        'https://api.example.com/auth/api/passkey/delete',
        expect.any(Object)
      );
    });
  });
});
