/**
 * SSO Authentication Types and Interfaces
 * AccuScene Enterprise v0.2.5
 */

// ============================================================================
// User and Authentication Types
// ============================================================================

export interface SSOUser {
  id: string;
  email: string;
  name?: string;
  given_name?: string;
  family_name?: string;
  picture?: string;
  metadata: Record<string, any>;
  provider: string;
}

export interface AuthTokens {
  access_token: string;
  refresh_token?: string;
  id_token?: string;
  expires_at: string;
  token_type?: string;
}

export interface AuthenticationResult {
  user: SSOUser;
  access_token: string;
  refresh_token?: string;
  expires_at: string;
  id_token?: string;
  session_id: string;
  mfa_required: boolean;
  mfa_token?: string;
}

// ============================================================================
// Session Types
// ============================================================================

export interface Session {
  id: string;
  user_id: string;
  email: string;
  created_at: string;
  last_accessed: string;
  expires_at: string;
  ip_address?: string;
  user_agent?: string;
  mfa_verified: boolean;
  metadata: Record<string, any>;
}

export interface SessionState {
  user: SSOUser | null;
  session: Session | null;
  tokens: AuthTokens | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

// ============================================================================
// SSO Provider Types
// ============================================================================

export enum SSOProviderType {
  SAML = 'saml',
  OIDC = 'oidc',
  OAUTH2 = 'oauth2',
  LDAP = 'ldap',
}

export interface SSOProvider {
  name: string;
  type: SSOProviderType;
  display_name: string;
  icon_url?: string;
  enabled: boolean;
}

export interface SSOConfig {
  providers: SSOProvider[];
  default_provider?: string;
  enable_mfa: boolean;
  enable_remember_me: boolean;
}

// ============================================================================
// MFA Types
// ============================================================================

export enum MFAMethod {
  TOTP = 'totp',
  WEBAUTHN = 'webauthn',
  SMS = 'sms',
  EMAIL = 'email',
}

export interface MFAChallenge {
  challenge_id: string;
  method: MFAMethod;
  expires_at: string;
  qr_code?: string; // For TOTP
  challenge?: any; // For WebAuthn
}

export interface MFAVerification {
  verified: boolean;
  method: MFAMethod;
  timestamp: string;
  device_id?: string;
}

export interface UserMFAConfig {
  user_id: string;
  enabled_methods: MFAMethod[];
  totp_enabled: boolean;
  webauthn_enabled: boolean;
  mfa_enabled: boolean;
  backup_codes_remaining: number;
}

export interface TOTPSetup {
  secret: string;
  qr_code: string;
  provisioning_uri: string;
  backup_codes: string[];
}

export interface WebAuthnCredential {
  id: string;
  name: string;
  registered_at: string;
  last_used?: string;
}

// ============================================================================
// Login and Callback Types
// ============================================================================

export interface LoginRequest {
  provider: string;
  redirect_uri?: string;
  state?: string;
  remember_me?: boolean;
}

export interface LoginResponse {
  authorization_url: string;
  state: string;
}

export interface CallbackParams {
  code?: string;
  state?: string;
  error?: string;
  error_description?: string;
}

export interface LDAPLoginRequest {
  provider: string;
  username: string;
  password: string;
}

// ============================================================================
// Error Types
// ============================================================================

export class SSOError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode?: number,
    public details?: any
  ) {
    super(message);
    this.name = 'SSOError';
  }
}

export enum SSOErrorCode {
  PROVIDER_NOT_FOUND = 'PROVIDER_NOT_FOUND',
  INVALID_CREDENTIALS = 'INVALID_CREDENTIALS',
  AUTHENTICATION_FAILED = 'AUTHENTICATION_FAILED',
  TOKEN_VALIDATION_FAILED = 'TOKEN_VALIDATION_FAILED',
  TOKEN_EXPIRED = 'TOKEN_EXPIRED',
  INVALID_TOKEN = 'INVALID_TOKEN',
  SESSION_NOT_FOUND = 'SESSION_NOT_FOUND',
  SESSION_EXPIRED = 'SESSION_EXPIRED',
  MFA_REQUIRED = 'MFA_REQUIRED',
  INVALID_MFA_CODE = 'INVALID_MFA_CODE',
  MFA_NOT_CONFIGURED = 'MFA_NOT_CONFIGURED',
  NETWORK_ERROR = 'NETWORK_ERROR',
  UNKNOWN_ERROR = 'UNKNOWN_ERROR',
}

// ============================================================================
// API Response Types
// ============================================================================

export interface APIResponse<T> {
  success: boolean;
  data?: T;
  error?: {
    code: string;
    message: string;
    details?: any;
  };
}

// ============================================================================
// Audit Types
// ============================================================================

export interface AuditEvent {
  id: string;
  event_type: string;
  timestamp: string;
  user_id?: string;
  ip_address?: string;
  user_agent?: string;
  metadata: Record<string, any>;
}

// ============================================================================
// Auth Context Types
// ============================================================================

export interface AuthContextValue {
  // State
  user: SSOUser | null;
  session: Session | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;

  // Auth methods
  login: (provider: string, redirectUri?: string) => Promise<void>;
  loginWithLDAP: (provider: string, username: string, password: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshSession: () => Promise<void>;

  // MFA methods
  setupTOTP: () => Promise<TOTPSetup>;
  verifyTOTP: (code: string, mfaToken?: string) => Promise<void>;
  setupWebAuthn: (name: string) => Promise<any>;
  verifyWebAuthn: (credential: any, mfaToken?: string) => Promise<void>;
  getMFAConfig: () => Promise<UserMFAConfig>;

  // Session methods
  validateSession: () => Promise<boolean>;
  getSession: () => Promise<Session | null>;
}

// ============================================================================
// Route Protection Types
// ============================================================================

export interface ProtectedRouteProps {
  children: React.ReactNode;
  requireAuth?: boolean;
  requireMFA?: boolean;
  redirectTo?: string;
  fallback?: React.ReactNode;
}

// ============================================================================
// Storage Types
// ============================================================================

export interface TokenStorage {
  getAccessToken: () => string | null;
  setAccessToken: (token: string) => void;
  getRefreshToken: () => string | null;
  setRefreshToken: (token: string) => void;
  clearTokens: () => void;
}

// ============================================================================
// Utility Types
// ============================================================================

export type AsyncFunction<T = void> = (...args: any[]) => Promise<T>;

export interface RequestConfig {
  headers?: Record<string, string>;
  timeout?: number;
  retries?: number;
}
