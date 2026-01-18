import type { OAuthResponse, AppleFullName } from '../types';

/**
 * Service for OAuth authentication (Google, Apple)
 * This service sends tokens from native SDKs to the backend for verification.
 */
class OAuthService {
  private baseUrl: string = '';

  /**
   * Set the base URL for API calls
   */
  setBaseUrl(url: string): void {
    this.baseUrl = url.replace(/\/$/, '');
  }

  /**
   * Authenticate with Google
   * @param idToken The ID token from Google Sign-In SDK
   */
  async googleAuth(idToken: string): Promise<OAuthResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/oauth/google`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ id_token: idToken }),
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
   * Authenticate with Apple
   * @param identityToken The identity token from Apple Sign-In
   * @param userIdentifier Optional user identifier (Apple user ID)
   * @param email Optional email (only provided on first sign-in)
   * @param fullName Optional full name (only provided on first sign-in)
   */
  async appleAuth(
    identityToken: string,
    userIdentifier?: string,
    email?: string,
    fullName?: AppleFullName
  ): Promise<OAuthResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/auth/api/oauth/apple`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          identity_token: identityToken,
          user_identifier: userIdentifier,
          email,
          full_name: fullName
            ? {
                given_name: fullName.givenName,
                family_name: fullName.familyName,
              }
            : undefined,
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
}

// Singleton instance
export const oauthService = new OAuthService();
