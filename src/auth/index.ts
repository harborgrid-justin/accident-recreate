/**
 * Authentication Module - Public API
 * AccuScene Enterprise Accident Recreation Platform
 *
 * Main entry point for all authentication and authorization functionality
 */

// Import for local use and re-export
import { AuthService } from './AuthService';
import { PasswordService } from './PasswordService';
import { JwtService } from './JwtService';
import { SessionManager } from './SessionManager';
import { RateLimiter } from './middleware';
import { UserRole, AuthConfig, RateLimitConfig } from './types';

// Core Services
export { AuthService } from './AuthService';
export { PasswordService } from './PasswordService';
export { JwtService } from './JwtService';
export { SessionManager } from './SessionManager';

// Middleware
export {
  authMiddleware,
  roleMiddleware,
  sessionMiddleware,
  optionalAuthMiddleware,
  adminOnly,
  investigatorOrAbove,
  adjusterOrAbove,
  auditMiddleware,
  corsMiddleware,
  securityHeadersMiddleware,
  authErrorHandler,
  RateLimiter
} from './middleware';

// Types and Interfaces
export { UserRole } from './types';
export type {
  User,
  SafeUser,
  AuthTokenPayload,
  AuthTokens,
  LoginCredentials,
  RegisterData,
  Session,
  PasswordResetToken,
  PasswordStrengthResult,
  AuthResponse,
  RateLimitConfig,
  AuthConfig,
  DecodedToken
} from './types';
export { AuthError, ValidationError } from './types';

// Default configurations
export { defaultPasswordConfig } from './PasswordService';
export { defaultJwtConfig } from './JwtService';

/**
 * Default authentication configuration
 */
export const defaultAuthConfig: AuthConfig = {
  // JWT Configuration
  jwtSecret: process.env.JWT_SECRET || 'your-secret-key-change-in-production',
  jwtRefreshSecret: process.env.JWT_REFRESH_SECRET || 'your-refresh-secret-change-in-production',
  accessTokenExpiry: process.env.ACCESS_TOKEN_EXPIRY || '15m',
  refreshTokenExpiry: process.env.REFRESH_TOKEN_EXPIRY || '7d',

  // Password Configuration
  bcryptRounds: parseInt(process.env.BCRYPT_ROUNDS || '100000', 10),
  passwordMinLength: 8,
  passwordRequireUppercase: true,
  passwordRequireLowercase: true,
  passwordRequireNumbers: true,
  passwordRequireSpecialChars: true,

  // Account Security
  maxLoginAttempts: 5,
  lockoutDuration: 15 * 60 * 1000, // 15 minutes

  // Session Configuration
  sessionTimeout: 24 * 60 * 60 * 1000, // 24 hours
  maxSessionsPerUser: 5,

  // Rate Limiting
  rateLimitConfig: {
    windowMs: 15 * 60 * 1000, // 15 minutes
    maxAttempts: 100,
    blockDuration: 15 * 60 * 1000 // 15 minutes
  }
};

/**
 * Create and configure authentication service instance
 */
export function createAuthService(customConfig?: Partial<AuthConfig>): AuthService {
  const config = {
    ...defaultAuthConfig,
    ...customConfig
  };

  return new AuthService(config);
}

/**
 * Create rate limiter instance
 */
export function createRateLimiter(customConfig?: Partial<RateLimitConfig>): RateLimiter {
  const config = {
    ...defaultAuthConfig.rateLimitConfig,
    ...customConfig
  };

  return new RateLimiter(config);
}

/**
 * Utility function to extract user ID from request
 */
export function getUserIdFromRequest(req: any): string | null {
  return req.user?.userId || null;
}

/**
 * Utility function to check if user has role
 */
export function hasRole(userRole: UserRole, requiredRoles: UserRole[]): boolean {
  return requiredRoles.includes(userRole);
}

/**
 * Utility function to check if user is admin
 */
export function isAdmin(userRole: UserRole): boolean {
  return userRole === UserRole.ADMIN;
}

/**
 * Role hierarchy helper
 */
export const roleHierarchy: Record<UserRole, number> = {
  [UserRole.VIEWER]: 1,
  [UserRole.ADJUSTER]: 2,
  [UserRole.INVESTIGATOR]: 3,
  [UserRole.ADMIN]: 4
};

/**
 * Check if user role has sufficient permissions
 */
export function hasPermissionLevel(userRole: UserRole, requiredRole: UserRole): boolean {
  return (roleHierarchy[userRole] ?? 0) >= (roleHierarchy[requiredRole] ?? 0);
}

/**
 * Get all roles with equal or lower permissions
 */
export function getRolesWithLowerOrEqualPermissions(role: UserRole): UserRole[] {
  const level = roleHierarchy[role] ?? 0;
  return (Object.keys(roleHierarchy) as UserRole[]).filter(
    r => (roleHierarchy[r] ?? 0) <= level
  );
}

/**
 * Get all roles with higher permissions
 */
export function getRolesWithHigherPermissions(role: UserRole): UserRole[] {
  const level = roleHierarchy[role] ?? 0;
  return (Object.keys(roleHierarchy) as UserRole[]).filter(
    r => (roleHierarchy[r] ?? 0) > level
  );
}

/**
 * Authentication module version
 */
export const AUTH_MODULE_VERSION = '1.0.0';

/**
 * Module metadata
 */
export const AUTH_MODULE_INFO = {
  name: 'AccuScene Authentication Module',
  version: AUTH_MODULE_VERSION,
  description: 'Enterprise-grade authentication and authorization system',
  features: [
    'JWT-based authentication',
    'Role-based access control (RBAC)',
    'Session management',
    'Password hashing with PBKDF2',
    'Rate limiting',
    'Account lockout protection',
    'Password reset functionality',
    'Security middleware',
    'Audit logging'
  ],
  roles: Object.values(UserRole),
  supportedTokenTypes: ['access', 'refresh']
};

/**
 * Export everything as a namespace for convenience
 */
export default {
  AuthService,
  PasswordService,
  JwtService,
  SessionManager,
  createAuthService,
  createRateLimiter,
  defaultAuthConfig,
  UserRole,
  AUTH_MODULE_VERSION,
  AUTH_MODULE_INFO
};
