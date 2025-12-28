/**
 * AccuScene Enterprise SSO Module
 * Version 0.2.5
 *
 * Comprehensive enterprise Single Sign-On authentication system
 */

// Types and interfaces
export * from './types';

// Core components
export { AuthProvider, useAuth } from './AuthProvider';
export { default as LoginPage } from './LoginPage';
export { default as SSOCallback } from './SSOCallback';
export { default as MFAChallenge } from './MFAChallenge';

// Session management
export { default as SessionManager } from './SessionManager';

// Services
export { default as AuthService } from './services/AuthService';

// Hooks
export { useAuth as useAuthHook } from './hooks/useAuth';
export { useSession } from './hooks/useSession';

// Guards and utilities
export { default as ProtectedRoute } from './guards/ProtectedRoute';

// UI Components
export { default as UserMenu } from './components/UserMenu';

// Module information
export const SSO_MODULE_VERSION = '0.2.5';

export const SSO_MODULE_INFO = {
  name: 'AccuScene Enterprise SSO',
  version: SSO_MODULE_VERSION,
  description: 'Enterprise Single Sign-On authentication system',
  features: [
    'SAML 2.0 authentication',
    'OpenID Connect (OIDC)',
    'OAuth 2.0',
    'LDAP/Active Directory',
    'Multi-factor authentication (TOTP, WebAuthn)',
    'Secure JWT token management',
    'PKCE flow support',
    'Session management',
    'Audit trail',
    'Route protection',
  ],
  supportedProviders: [
    'Google',
    'Microsoft Azure AD',
    'Okta',
    'Auth0',
    'LDAP/Active Directory',
    'Generic SAML 2.0',
    'Generic OIDC',
    'Generic OAuth 2.0',
  ],
  mfaMethods: ['TOTP (Authenticator apps)', 'WebAuthn/FIDO2 (Security keys)'],
};
