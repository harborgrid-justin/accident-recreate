/**
 * Authentication and Authorization Type Definitions
 * AccuScene Enterprise Accident Recreation Platform
 */

export enum UserRole {
  ADMIN = 'ADMIN',
  INVESTIGATOR = 'INVESTIGATOR',
  ADJUSTER = 'ADJUSTER',
  VIEWER = 'VIEWER'
}

export interface User {
  id: string;
  email: string;
  passwordHash: string;
  role: UserRole;
  firstName?: string;
  lastName?: string;
  createdAt: Date;
  updatedAt: Date;
  lastLogin?: Date;
  isActive: boolean;
  emailVerified: boolean;
  failedLoginAttempts: number;
  lockedUntil?: Date;
}

export interface AuthTokenPayload {
  userId: string;
  email: string;
  role: UserRole;
  type: 'access' | 'refresh';
  iat?: number;
  exp?: number;
}

export interface AuthTokens {
  accessToken: string;
  refreshToken: string;
  expiresIn: number;
}

export interface LoginCredentials {
  email: string;
  password: string;
}

export interface RegisterData {
  email: string;
  password: string;
  role: UserRole;
  firstName?: string;
  lastName?: string;
}

export interface Session {
  id: string;
  userId: string;
  refreshToken: string;
  userAgent?: string;
  ipAddress?: string;
  createdAt: Date;
  expiresAt: Date;
  lastActivity: Date;
}

export interface PasswordResetToken {
  id: string;
  userId: string;
  token: string;
  expiresAt: Date;
  used: boolean;
}

export interface PasswordStrengthResult {
  isValid: boolean;
  score: number;
  errors: string[];
  suggestions: string[];
}

export interface AuthResponse {
  success: boolean;
  message?: string;
  user?: Omit<User, 'passwordHash'>;
  tokens?: AuthTokens;
}

export interface RateLimitConfig {
  windowMs: number;
  maxAttempts: number;
  blockDuration: number;
}

export interface AuthConfig {
  jwtSecret: string;
  jwtRefreshSecret: string;
  accessTokenExpiry: string;
  refreshTokenExpiry: string;
  bcryptRounds: number;
  maxLoginAttempts: number;
  lockoutDuration: number;
  passwordMinLength: number;
  passwordRequireUppercase: boolean;
  passwordRequireLowercase: boolean;
  passwordRequireNumbers: boolean;
  passwordRequireSpecialChars: boolean;
  sessionTimeout: number;
  maxSessionsPerUser: number;
  rateLimitConfig: RateLimitConfig;
}

export interface DecodedToken {
  userId: string;
  email: string;
  role: UserRole;
  type: 'access' | 'refresh';
  iat: number;
  exp: number;
}

export type SafeUser = Omit<User, 'passwordHash'>;

export class AuthError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 401
  ) {
    super(message);
    this.name = 'AuthError';
    Object.setPrototypeOf(this, AuthError.prototype);
  }
}

export class ValidationError extends Error {
  constructor(
    message: string,
    public fields: Record<string, string> = {}
  ) {
    super(message);
    this.name = 'ValidationError';
    Object.setPrototypeOf(this, ValidationError.prototype);
  }
}
