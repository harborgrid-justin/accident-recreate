/**
 * Authentication Service
 * Handles all SSO authentication API calls
 */

import {
  AuthenticationResult,
  LoginRequest,
  LoginResponse,
  CallbackParams,
  LDAPLoginRequest,
  Session,
  SSOUser,
  UserMFAConfig,
  TOTPSetup,
  WebAuthnCredential,
  APIResponse,
  SSOError,
  SSOErrorCode,
} from '../types';
import SessionManager from '../SessionManager';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080';
const SSO_API_PATH = '/api/v1/sso';

export class AuthService {
  private apiUrl: string;
  private sessionManager: SessionManager;

  constructor(apiUrl: string = API_BASE_URL) {
    this.apiUrl = apiUrl;
    this.sessionManager = SessionManager;
  }

  // ============================================================================
  // HTTP Request Helper
  // ============================================================================

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.apiUrl}${SSO_API_PATH}${endpoint}`;

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    // Add auth token if available
    const accessToken = this.sessionManager.getAccessToken();
    if (accessToken) {
      headers['Authorization'] = `Bearer ${accessToken}`;
    }

    try {
      const response = await fetch(url, {
        ...options,
        headers,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          code: SSOErrorCode.UNKNOWN_ERROR,
          message: response.statusText,
        }));

        throw new SSOError(
          error.message || 'Request failed',
          error.code || SSOErrorCode.UNKNOWN_ERROR,
          response.status,
          error.details
        );
      }

      const data = await response.json();
      return data as T;
    } catch (error) {
      if (error instanceof SSOError) {
        throw error;
      }

      throw new SSOError(
        'Network error',
        SSOErrorCode.NETWORK_ERROR,
        undefined,
        error
      );
    }
  }

  // ============================================================================
  // Authentication Methods
  // ============================================================================

  /**
   * Initiate SSO login flow
   */
  async initiateLogin(request: LoginRequest): Promise<LoginResponse> {
    const response = await this.request<LoginResponse>('/login', {
      method: 'POST',
      body: JSON.stringify(request),
    });

    // Save state for CSRF protection
    this.sessionManager.saveOAuthState(response.state, {
      provider: request.provider,
      redirect_uri: request.redirect_uri,
    });

    return response;
  }

  /**
   * Handle OAuth callback
   */
  async handleCallback(params: CallbackParams): Promise<AuthenticationResult> {
    // Check for errors
    if (params.error) {
      throw new SSOError(
        params.error_description || params.error,
        SSOErrorCode.AUTHENTICATION_FAILED
      );
    }

    // Validate state
    if (!params.state) {
      throw new SSOError('Missing state parameter', SSOErrorCode.AUTHENTICATION_FAILED);
    }

    const stateData = this.sessionManager.validateOAuthState(params.state);
    if (!stateData) {
      throw new SSOError('Invalid or expired state', SSOErrorCode.AUTHENTICATION_FAILED);
    }

    // Exchange code for tokens
    const result = await this.request<AuthenticationResult>('/callback', {
      method: 'POST',
      body: JSON.stringify({
        code: params.code,
        state: params.state,
        provider: stateData.provider,
      }),
    });

    // Clear state
    this.sessionManager.clearOAuthState(params.state);

    // Store session
    this.sessionManager.setCompleteSession(
      result.user,
      {
        id: result.session_id,
        user_id: result.user.id,
        email: result.user.email,
        created_at: new Date().toISOString(),
        last_accessed: new Date().toISOString(),
        expires_at: result.expires_at,
        mfa_verified: !result.mfa_required,
        metadata: {},
      },
      {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        id_token: result.id_token,
        expires_at: result.expires_at,
      }
    );

    return result;
  }

  /**
   * Login with LDAP credentials
   */
  async loginWithLDAP(request: LDAPLoginRequest): Promise<AuthenticationResult> {
    const result = await this.request<AuthenticationResult>('/login/ldap', {
      method: 'POST',
      body: JSON.stringify(request),
    });

    // Store session
    this.sessionManager.setCompleteSession(
      result.user,
      {
        id: result.session_id,
        user_id: result.user.id,
        email: result.user.email,
        created_at: new Date().toISOString(),
        last_accessed: new Date().toISOString(),
        expires_at: result.expires_at,
        mfa_verified: !result.mfa_required,
        metadata: {},
      },
      {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        id_token: result.id_token,
        expires_at: result.expires_at,
      }
    );

    return result;
  }

  /**
   * Logout
   */
  async logout(): Promise<void> {
    const session = this.sessionManager.getSession();
    if (session) {
      try {
        await this.request('/logout', {
          method: 'POST',
          body: JSON.stringify({ session_id: session.id }),
        });
      } catch (error) {
        console.error('Logout request failed:', error);
      }
    }

    this.sessionManager.clearCompleteSession();
  }

  // ============================================================================
  // Session Management
  // ============================================================================

  /**
   * Refresh access token
   */
  async refreshToken(): Promise<AuthenticationResult> {
    const refreshToken = this.sessionManager.getRefreshToken();
    if (!refreshToken) {
      throw new SSOError('No refresh token available', SSOErrorCode.INVALID_TOKEN);
    }

    const result = await this.request<AuthenticationResult>('/refresh', {
      method: 'POST',
      body: JSON.stringify({ refresh_token: refreshToken }),
    });

    // Update session
    this.sessionManager.setTokens({
      access_token: result.access_token,
      refresh_token: result.refresh_token,
      id_token: result.id_token,
      expires_at: result.expires_at,
    });

    return result;
  }

  /**
   * Validate current session
   */
  async validateSession(): Promise<Session> {
    return await this.request<Session>('/session/validate', {
      method: 'GET',
    });
  }

  /**
   * Get current session
   */
  async getSession(): Promise<Session> {
    const session = await this.request<Session>('/session', {
      method: 'GET',
    });

    this.sessionManager.setSession(session);
    return session;
  }

  // ============================================================================
  // MFA Methods
  // ============================================================================

  /**
   * Get user's MFA configuration
   */
  async getMFAConfig(): Promise<UserMFAConfig> {
    return await this.request<UserMFAConfig>('/mfa/config', {
      method: 'GET',
    });
  }

  /**
   * Setup TOTP authenticator
   */
  async setupTOTP(): Promise<TOTPSetup> {
    return await this.request<TOTPSetup>('/mfa/totp/setup', {
      method: 'POST',
    });
  }

  /**
   * Verify TOTP code and complete setup
   */
  async verifyTOTPSetup(code: string): Promise<{ success: boolean; backup_codes: string[] }> {
    return await this.request('/mfa/totp/verify-setup', {
      method: 'POST',
      body: JSON.stringify({ code }),
    });
  }

  /**
   * Verify TOTP code during login
   */
  async verifyTOTP(code: string, mfaToken: string): Promise<AuthenticationResult> {
    const result = await this.request<AuthenticationResult>('/mfa/totp/verify', {
      method: 'POST',
      body: JSON.stringify({ code, mfa_token: mfaToken }),
    });

    // Update session with MFA verification
    this.sessionManager.setCompleteSession(
      result.user,
      {
        id: result.session_id,
        user_id: result.user.id,
        email: result.user.email,
        created_at: new Date().toISOString(),
        last_accessed: new Date().toISOString(),
        expires_at: result.expires_at,
        mfa_verified: true,
        metadata: {},
      },
      {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        id_token: result.id_token,
        expires_at: result.expires_at,
      }
    );

    return result;
  }

  /**
   * Start WebAuthn registration
   */
  async startWebAuthnRegistration(name: string): Promise<any> {
    return await this.request('/mfa/webauthn/register/start', {
      method: 'POST',
      body: JSON.stringify({ name }),
    });
  }

  /**
   * Complete WebAuthn registration
   */
  async completeWebAuthnRegistration(credential: any): Promise<WebAuthnCredential> {
    return await this.request('/mfa/webauthn/register/complete', {
      method: 'POST',
      body: JSON.stringify({ credential }),
    });
  }

  /**
   * Start WebAuthn authentication
   */
  async startWebAuthnAuthentication(mfaToken: string): Promise<any> {
    return await this.request('/mfa/webauthn/authenticate/start', {
      method: 'POST',
      body: JSON.stringify({ mfa_token: mfaToken }),
    });
  }

  /**
   * Complete WebAuthn authentication
   */
  async completeWebAuthnAuthentication(
    credential: any,
    mfaToken: string
  ): Promise<AuthenticationResult> {
    const result = await this.request<AuthenticationResult>(
      '/mfa/webauthn/authenticate/complete',
      {
        method: 'POST',
        body: JSON.stringify({ credential, mfa_token: mfaToken }),
      }
    );

    // Update session with MFA verification
    this.sessionManager.setCompleteSession(
      result.user,
      {
        id: result.session_id,
        user_id: result.user.id,
        email: result.user.email,
        created_at: new Date().toISOString(),
        last_accessed: new Date().toISOString(),
        expires_at: result.expires_at,
        mfa_verified: true,
        metadata: {},
      },
      {
        access_token: result.access_token,
        refresh_token: result.refresh_token,
        id_token: result.id_token,
        expires_at: result.expires_at,
      }
    );

    return result;
  }

  /**
   * Disable MFA
   */
  async disableMFA(method: string): Promise<void> {
    await this.request(`/mfa/${method}/disable`, {
      method: 'POST',
    });
  }

  /**
   * Get WebAuthn credentials
   */
  async getWebAuthnCredentials(): Promise<WebAuthnCredential[]> {
    return await this.request('/mfa/webauthn/credentials', {
      method: 'GET',
    });
  }

  /**
   * Remove WebAuthn credential
   */
  async removeWebAuthnCredential(credentialId: string): Promise<void> {
    await this.request(`/mfa/webauthn/credentials/${credentialId}`, {
      method: 'DELETE',
    });
  }
}

export default new AuthService();
