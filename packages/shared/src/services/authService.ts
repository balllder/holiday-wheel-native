import type { AuthResponse, RoomInfo, User } from '../types';
import { passkeyService } from './passkeyService';
import { oauthService } from './oauthService';

class AuthService {
  private baseUrl: string = '';

  /**
   * Set the base URL for API calls
   * Also syncs the URL to passkey and oauth services
   */
  setBaseUrl(url: string): void {
    this.baseUrl = url.replace(/\/$/, ''); // Remove trailing slash
    // Sync to related services
    passkeyService.setBaseUrl(this.baseUrl);
    oauthService.setBaseUrl(this.baseUrl);
  }

  /**
   * Get the current base URL
   */
  getBaseUrl(): string {
    return this.baseUrl;
  }

  /**
   * Login with email and password
   */
  async login(email: string, password: string): Promise<AuthResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/login`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ email, password }),
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
   * Register a new account
   */
  async register(
    email: string,
    password: string,
    displayName: string
  ): Promise<AuthResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email,
          password,
          display_name: displayName,
        }),
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
   * Get list of active rooms
   */
  async getRooms(token: string): Promise<{ rooms: RoomInfo[] }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/rooms`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        return { rooms: [] };
      }

      const data = await response.json();
      return data;
    } catch (error) {
      console.error('Failed to fetch rooms:', error);
      return { rooms: [] };
    }
  }

  /**
   * Verify the current token is valid
   */
  async verifyToken(token: string): Promise<{ ok: boolean; user?: User }> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/verify`, {
        headers: {
          Authorization: `Bearer ${token}`,
        },
      });

      if (!response.ok) {
        return { ok: false };
      }

      const data = await response.json();
      return data;
    } catch (error) {
      return { ok: false };
    }
  }
}

// Singleton instance
export const authService = new AuthService();
