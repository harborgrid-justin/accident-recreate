/**
 * Session Manager
 * Handles client-side session state and token management
 */

import { AuthTokens, Session, SSOUser } from './types';

const ACCESS_TOKEN_KEY = 'accuscene_access_token';
const REFRESH_TOKEN_KEY = 'accuscene_refresh_token';
const SESSION_KEY = 'accuscene_session';
const USER_KEY = 'accuscene_user';

export class SessionManager {
  private static instance: SessionManager;

  private constructor() {}

  static getInstance(): SessionManager {
    if (!SessionManager.instance) {
      SessionManager.instance = new SessionManager();
    }
    return SessionManager.instance;
  }

  // ============================================================================
  // Token Management
  // ============================================================================

  setTokens(tokens: AuthTokens): void {
    if (tokens.access_token) {
      this.setAccessToken(tokens.access_token);
    }
    if (tokens.refresh_token) {
      this.setRefreshToken(tokens.refresh_token);
    }
  }

  setAccessToken(token: string): void {
    localStorage.setItem(ACCESS_TOKEN_KEY, token);
  }

  getAccessToken(): string | null {
    return localStorage.getItem(ACCESS_TOKEN_KEY);
  }

  setRefreshToken(token: string): void {
    localStorage.setItem(REFRESH_TOKEN_KEY, token);
  }

  getRefreshToken(): string | null {
    return localStorage.getItem(REFRESH_TOKEN_KEY);
  }

  clearTokens(): void {
    localStorage.removeItem(ACCESS_TOKEN_KEY);
    localStorage.removeItem(REFRESH_TOKEN_KEY);
  }

  // ============================================================================
  // Session Management
  // ============================================================================

  setSession(session: Session): void {
    localStorage.setItem(SESSION_KEY, JSON.stringify(session));
  }

  getSession(): Session | null {
    const sessionStr = localStorage.getItem(SESSION_KEY);
    if (!sessionStr) return null;

    try {
      return JSON.parse(sessionStr);
    } catch (e) {
      console.error('Failed to parse session:', e);
      return null;
    }
  }

  clearSession(): void {
    localStorage.removeItem(SESSION_KEY);
  }

  isSessionExpired(session: Session): boolean {
    const expiresAt = new Date(session.expires_at);
    return expiresAt <= new Date();
  }

  // ============================================================================
  // User Management
  // ============================================================================

  setUser(user: SSOUser): void {
    localStorage.setItem(USER_KEY, JSON.stringify(user));
  }

  getUser(): SSOUser | null {
    const userStr = localStorage.getItem(USER_KEY);
    if (!userStr) return null;

    try {
      return JSON.parse(userStr);
    } catch (e) {
      console.error('Failed to parse user:', e);
      return null;
    }
  }

  clearUser(): void {
    localStorage.removeItem(USER_KEY);
  }

  // ============================================================================
  // Complete Session Management
  // ============================================================================

  setCompleteSession(user: SSOUser, session: Session, tokens: AuthTokens): void {
    this.setUser(user);
    this.setSession(session);
    this.setTokens(tokens);
  }

  clearCompleteSession(): void {
    this.clearUser();
    this.clearSession();
    this.clearTokens();
  }

  // ============================================================================
  // Session Validation
  // ============================================================================

  hasValidSession(): boolean {
    const session = this.getSession();
    if (!session) return false;

    return !this.isSessionExpired(session);
  }

  hasValidTokens(): boolean {
    const accessToken = this.getAccessToken();
    return !!accessToken;
  }

  // ============================================================================
  // Token Decoding (JWT)
  // ============================================================================

  decodeToken(token: string): any {
    try {
      const base64Url = token.split('.')[1];
      const base64 = base64Url.replace(/-/g, '+').replace(/_/g, '/');
      const jsonPayload = decodeURIComponent(
        atob(base64)
          .split('')
          .map((c) => '%' + ('00' + c.charCodeAt(0).toString(16)).slice(-2))
          .join('')
      );
      return JSON.parse(jsonPayload);
    } catch (e) {
      console.error('Failed to decode token:', e);
      return null;
    }
  }

  isTokenExpired(token: string): boolean {
    const decoded = this.decodeToken(token);
    if (!decoded || !decoded.exp) return true;

    const expirationTime = decoded.exp * 1000; // Convert to milliseconds
    return Date.now() >= expirationTime;
  }

  getTokenExpirationTime(token: string): Date | null {
    const decoded = this.decodeToken(token);
    if (!decoded || !decoded.exp) return null;

    return new Date(decoded.exp * 1000);
  }

  // ============================================================================
  // Token Refresh Check
  // ============================================================================

  shouldRefreshToken(thresholdSeconds: number = 300): boolean {
    const accessToken = this.getAccessToken();
    if (!accessToken) return false;

    const expirationTime = this.getTokenExpirationTime(accessToken);
    if (!expirationTime) return false;

    const now = Date.now();
    const threshold = thresholdSeconds * 1000;

    return expirationTime.getTime() - now <= threshold;
  }

  // ============================================================================
  // State Persistence
  // ============================================================================

  saveState(key: string, value: any): void {
    try {
      localStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
      console.error('Failed to save state:', e);
    }
  }

  loadState<T>(key: string): T | null {
    try {
      const value = localStorage.getItem(key);
      return value ? JSON.parse(value) : null;
    } catch (e) {
      console.error('Failed to load state:', e);
      return null;
    }
  }

  clearState(key: string): void {
    localStorage.removeItem(key);
  }

  // ============================================================================
  // Session Storage (for temporary data)
  // ============================================================================

  setSessionStorage(key: string, value: any): void {
    try {
      sessionStorage.setItem(key, JSON.stringify(value));
    } catch (e) {
      console.error('Failed to save to session storage:', e);
    }
  }

  getSessionStorage<T>(key: string): T | null {
    try {
      const value = sessionStorage.getItem(key);
      return value ? JSON.parse(value) : null;
    } catch (e) {
      console.error('Failed to load from session storage:', e);
      return null;
    }
  }

  clearSessionStorage(key: string): void {
    sessionStorage.removeItem(key);
  }

  // ============================================================================
  // OAuth State Management (CSRF Protection)
  // ============================================================================

  generateState(): string {
    const array = new Uint8Array(32);
    crypto.getRandomValues(array);
    return Array.from(array, (byte) => byte.toString(16).padStart(2, '0')).join('');
  }

  saveOAuthState(state: string, data: any): void {
    this.setSessionStorage(`oauth_state_${state}`, {
      ...data,
      timestamp: Date.now(),
    });
  }

  validateOAuthState(state: string): any {
    const data = this.getSessionStorage(`oauth_state_${state}`);
    if (!data) return null;

    // Check if state is not older than 10 minutes
    const maxAge = 10 * 60 * 1000;
    if (Date.now() - data.timestamp > maxAge) {
      this.clearSessionStorage(`oauth_state_${state}`);
      return null;
    }

    return data;
  }

  clearOAuthState(state: string): void {
    this.clearSessionStorage(`oauth_state_${state}`);
  }
}

export default SessionManager.getInstance();
