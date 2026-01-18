import type {
  PasskeyStartResponse,
  PasskeyFinishResponse,
  PasskeyListResponse,
} from '../types';

/**
 * Service for passkey/WebAuthn authentication
 * This service communicates with the backend API.
 * The actual credential creation/verification uses native platform APIs.
 */
class PasskeyService {
  private baseUrl: string = '';

  /**
   * Set the base URL for API calls
   */
  setBaseUrl(url: string): void {
    this.baseUrl = url.replace(/\/$/, '');
  }

  // ========== Registration ==========

  /**
   * Start passkey registration for a new user
   */
  async registerStart(
    email: string,
    displayName: string
  ): Promise<PasskeyStartResponse> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/register/start`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email, display_name: displayName }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Complete passkey registration
   * @param email The user's email
   * @param credential The credential response from the platform authenticator
   */
  async registerFinish(
    email: string,
    credential: unknown
  ): Promise<PasskeyFinishResponse> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/register/finish`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email, credential }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // ========== Authentication ==========

  /**
   * Start passkey authentication
   * @param email Optional email to get user-specific passkeys
   */
  async loginStart(email?: string): Promise<PasskeyStartResponse> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/login/start`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ email }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Complete passkey authentication
   * @param credential The credential response from the platform authenticator
   */
  async loginFinish(credential: unknown): Promise<PasskeyFinishResponse> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/login/finish`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ credential }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  // ========== Passkey Management ==========

  /**
   * List user's passkeys (requires auth)
   */
  async listPasskeys(token: string): Promise<PasskeyListResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/passkey/list`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
      });

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Start adding a new passkey to existing account (requires auth)
   */
  async addPasskeyStart(
    token: string,
    deviceName?: string
  ): Promise<PasskeyStartResponse> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/add/start`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({ device_name: deviceName }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Complete adding a new passkey (requires auth)
   */
  async addPasskeyFinish(
    token: string,
    email: string,
    credential: unknown
  ): Promise<{ ok: boolean; message?: string; error?: string }> {
    try {
      const response = await fetch(
        `${this.baseUrl}/auth/api/passkey/add/finish`,
        {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            Authorization: `Bearer ${token}`,
          },
          body: JSON.stringify({ email, credential }),
        }
      );

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }

  /**
   * Delete a passkey (requires auth)
   */
  async deletePasskey(
    token: string,
    credentialId: string
  ): Promise<{ ok: boolean; message?: string; error?: string }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/passkey/delete`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: `Bearer ${token}`,
        },
        body: JSON.stringify({ credential_id: credentialId }),
      });

      const data = await response.json();
      return data;
    } catch (error) {
      return {
        ok: false,
        error: error instanceof Error ? error.message : 'Network error',
      };
    }
  }
}

// Singleton instance
export const passkeyService = new PasskeyService();
