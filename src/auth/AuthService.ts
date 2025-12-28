/**
 * Authentication Service - Main Authentication Logic
 * AccuScene Enterprise Accident Recreation Platform
 */

import * as crypto from 'crypto';
import { PasswordService } from './PasswordService';
import { JwtService } from './JwtService';
import { SessionManager } from './SessionManager';
import {
  User,
  UserRole,
  LoginCredentials,
  RegisterData,
  AuthTokens,
  AuthResponse,
  AuthError,
  ValidationError,
  AuthConfig,
  SafeUser,
  PasswordResetToken
} from './types';

export class AuthService {
  private passwordService: PasswordService;
  private jwtService: JwtService;
  private sessionManager: SessionManager;
  private config: AuthConfig;

  // In-memory storage (replace with actual database in production)
  private users: Map<string, User>;
  private usersByEmail: Map<string, User>;
  private resetTokens: Map<string, PasswordResetToken>;

  constructor(config: AuthConfig) {
    this.config = config;
    this.passwordService = new PasswordService(config);
    this.jwtService = new JwtService(config);
    this.sessionManager = new SessionManager(config);

    this.users = new Map();
    this.usersByEmail = new Map();
    this.resetTokens = new Map();

    // Create default admin user if none exists
    this.initializeDefaultAdmin();
  }

  /**
   * Register a new user
   */
  async register(data: RegisterData): Promise<AuthResponse> {
    try {
      // Validate email
      if (!this.isValidEmail(data.email)) {
        throw new ValidationError('Invalid email address', { email: 'Invalid format' });
      }

      // Check if email already exists
      if (this.usersByEmail.has(data.email.toLowerCase())) {
        throw new ValidationError('Email already registered', { email: 'Already exists' });
      }

      // Validate password strength
      const passwordValidation = this.passwordService.validatePasswordStrength(data.password);
      if (!passwordValidation.isValid) {
        throw new ValidationError('Password does not meet requirements', {
          password: passwordValidation.errors.join(', ')
        });
      }

      // Check for compromised password
      const isCompromised = await this.passwordService.isPasswordCompromised(data.password);
      if (isCompromised) {
        throw new ValidationError('Password has been compromised in data breaches', {
          password: 'Please choose a different password'
        });
      }

      // Hash password
      const passwordHash = await this.passwordService.hash(data.password);

      // Create user
      const user: User = {
        id: this.generateUserId(),
        email: data.email.toLowerCase(),
        passwordHash,
        role: data.role,
        firstName: data.firstName,
        lastName: data.lastName,
        createdAt: new Date(),
        updatedAt: new Date(),
        isActive: true,
        emailVerified: false,
        failedLoginAttempts: 0
      };

      // Store user
      this.users.set(user.id, user);
      this.usersByEmail.set(user.email, user);

      // Return safe user data
      const safeUser = this.sanitizeUser(user);

      return {
        success: true,
        message: 'User registered successfully',
        user: safeUser
      };
    } catch (error) {
      if (error instanceof ValidationError || error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Registration failed', 'REGISTRATION_FAILED', 500);
    }
  }

  /**
   * Login with email and password
   */
  async login(credentials: LoginCredentials, metadata?: {
    userAgent?: string;
    ipAddress?: string;
  }): Promise<AuthResponse> {
    try {
      // Find user by email
      const user = this.usersByEmail.get(credentials.email.toLowerCase());

      if (!user) {
        throw new AuthError('Invalid email or password', 'INVALID_CREDENTIALS', 401);
      }

      // Check if account is locked
      if (user.lockedUntil && user.lockedUntil > new Date()) {
        const remainingTime = Math.ceil((user.lockedUntil.getTime() - Date.now()) / 1000 / 60);
        throw new AuthError(
          `Account is locked. Try again in ${remainingTime} minutes`,
          'ACCOUNT_LOCKED',
          423
        );
      }

      // Check if account is active
      if (!user.isActive) {
        throw new AuthError('Account is deactivated', 'ACCOUNT_DEACTIVATED', 403);
      }

      // Verify password
      const isPasswordValid = await this.passwordService.verify(
        credentials.password,
        user.passwordHash
      );

      if (!isPasswordValid) {
        // Increment failed login attempts
        user.failedLoginAttempts++;

        // Lock account if max attempts exceeded
        if (user.failedLoginAttempts >= this.config.maxLoginAttempts) {
          user.lockedUntil = new Date(Date.now() + this.config.lockoutDuration);
          user.failedLoginAttempts = 0;

          this.users.set(user.id, user);

          throw new AuthError(
            'Account locked due to multiple failed login attempts',
            'ACCOUNT_LOCKED',
            423
          );
        }

        this.users.set(user.id, user);

        throw new AuthError('Invalid email or password', 'INVALID_CREDENTIALS', 401);
      }

      // Reset failed login attempts
      user.failedLoginAttempts = 0;
      user.lockedUntil = undefined;
      user.lastLogin = new Date();
      this.users.set(user.id, user);

      // Generate tokens
      const tokens = this.generateTokens(user);

      // Create session
      await this.sessionManager.createSession(
        user.id,
        tokens.refreshToken,
        metadata
      );

      // Return success response
      return {
        success: true,
        message: 'Login successful',
        user: this.sanitizeUser(user),
        tokens
      };
    } catch (error) {
      if (error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Login failed', 'LOGIN_FAILED', 500);
    }
  }

  /**
   * Validate an access token
   */
  async validateToken(token: string): Promise<SafeUser> {
    try {
      const decoded = this.jwtService.verifyToken(token, false);

      const user = this.users.get(decoded.userId);

      if (!user) {
        throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
      }

      if (!user.isActive) {
        throw new AuthError('Account is deactivated', 'ACCOUNT_DEACTIVATED', 403);
      }

      return this.sanitizeUser(user);
    } catch (error) {
      if (error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Token validation failed', 'INVALID_TOKEN', 401);
    }
  }

  /**
   * Refresh access token using refresh token
   */
  async refreshToken(refreshToken: string): Promise<AuthTokens> {
    try {
      // Verify refresh token
      const decoded = this.jwtService.verifyToken(refreshToken, true);

      // Validate session
      await this.sessionManager.validateSessionByToken(refreshToken);

      // Get user
      const user = this.users.get(decoded.userId);

      if (!user) {
        throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
      }

      if (!user.isActive) {
        throw new AuthError('Account is deactivated', 'ACCOUNT_DEACTIVATED', 403);
      }

      // Generate new tokens
      return this.generateTokens(user);
    } catch (error) {
      if (error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Token refresh failed', 'REFRESH_FAILED', 401);
    }
  }

  /**
   * Logout user by destroying session
   */
  async logout(userId: string, sessionId?: string): Promise<void> {
    try {
      if (sessionId) {
        await this.sessionManager.destroySession(sessionId);
      } else {
        await this.sessionManager.destroyUserSessions(userId);
      }
    } catch (error) {
      throw new AuthError('Logout failed', 'LOGOUT_FAILED', 500);
    }
  }

  /**
   * Change user password
   */
  async changePassword(
    userId: string,
    oldPassword: string,
    newPassword: string
  ): Promise<void> {
    try {
      const user = this.users.get(userId);

      if (!user) {
        throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
      }

      // Verify old password
      const isOldPasswordValid = await this.passwordService.verify(
        oldPassword,
        user.passwordHash
      );

      if (!isOldPasswordValid) {
        throw new AuthError('Current password is incorrect', 'INVALID_PASSWORD', 401);
      }

      // Validate new password
      const passwordValidation = this.passwordService.validatePasswordStrength(newPassword);
      if (!passwordValidation.isValid) {
        throw new ValidationError('New password does not meet requirements', {
          password: passwordValidation.errors.join(', ')
        });
      }

      // Ensure new password is different from old
      const isSamePassword = await this.passwordService.verify(newPassword, user.passwordHash);
      if (isSamePassword) {
        throw new ValidationError('New password must be different from current password', {
          password: 'Password must be different'
        });
      }

      // Check for compromised password
      const isCompromised = await this.passwordService.isPasswordCompromised(newPassword);
      if (isCompromised) {
        throw new ValidationError('Password has been compromised in data breaches', {
          password: 'Please choose a different password'
        });
      }

      // Hash new password
      const newPasswordHash = await this.passwordService.hash(newPassword);

      // Update user
      user.passwordHash = newPasswordHash;
      user.updatedAt = new Date();
      this.users.set(user.id, user);

      // Destroy all existing sessions for security
      await this.sessionManager.destroyUserSessions(userId);
    } catch (error) {
      if (error instanceof ValidationError || error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Password change failed', 'PASSWORD_CHANGE_FAILED', 500);
    }
  }

  /**
   * Initiate password reset
   */
  async resetPassword(email: string): Promise<string> {
    try {
      const user = this.usersByEmail.get(email.toLowerCase());

      if (!user) {
        // Don't reveal if email exists for security
        return 'If the email exists, a reset link will be sent';
      }

      // Generate reset token
      const token = this.passwordService.generateResetToken();
      const hashedToken = await this.passwordService.hashResetToken(token);

      // Store reset token
      const resetToken: PasswordResetToken = {
        id: this.generateResetTokenId(),
        userId: user.id,
        token: hashedToken,
        expiresAt: new Date(Date.now() + 60 * 60 * 1000), // 1 hour
        used: false
      };

      this.resetTokens.set(resetToken.id, resetToken);

      // In production, send email with reset link containing token
      // For now, return token for testing
      return token;
    } catch (error) {
      // Don't reveal errors for security
      return 'If the email exists, a reset link will be sent';
    }
  }

  /**
   * Complete password reset with token
   */
  async completePasswordReset(token: string, newPassword: string): Promise<void> {
    try {
      // Find valid reset token
      let resetTokenRecord: PasswordResetToken | undefined;

      for (const record of this.resetTokens.values()) {
        if (!record.used && record.expiresAt > new Date()) {
          const isValid = await this.passwordService.verifyResetToken(token, record.token);
          if (isValid) {
            resetTokenRecord = record;
            break;
          }
        }
      }

      if (!resetTokenRecord) {
        throw new AuthError('Invalid or expired reset token', 'INVALID_RESET_TOKEN', 400);
      }

      // Get user
      const user = this.users.get(resetTokenRecord.userId);

      if (!user) {
        throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
      }

      // Validate new password
      const passwordValidation = this.passwordService.validatePasswordStrength(newPassword);
      if (!passwordValidation.isValid) {
        throw new ValidationError('Password does not meet requirements', {
          password: passwordValidation.errors.join(', ')
        });
      }

      // Hash new password
      const newPasswordHash = await this.passwordService.hash(newPassword);

      // Update user
      user.passwordHash = newPasswordHash;
      user.updatedAt = new Date();
      user.failedLoginAttempts = 0;
      user.lockedUntil = undefined;
      this.users.set(user.id, user);

      // Mark token as used
      resetTokenRecord.used = true;
      this.resetTokens.set(resetTokenRecord.id, resetTokenRecord);

      // Destroy all existing sessions
      await this.sessionManager.destroyUserSessions(user.id);
    } catch (error) {
      if (error instanceof ValidationError || error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Password reset failed', 'RESET_FAILED', 500);
    }
  }

  /**
   * Get user by ID
   */
  async getUserById(userId: string): Promise<SafeUser | null> {
    const user = this.users.get(userId);
    return user ? this.sanitizeUser(user) : null;
  }

  /**
   * Get user by email
   */
  async getUserByEmail(email: string): Promise<SafeUser | null> {
    const user = this.usersByEmail.get(email.toLowerCase());
    return user ? this.sanitizeUser(user) : null;
  }

  /**
   * Update user role (admin only)
   */
  async updateUserRole(userId: string, newRole: UserRole): Promise<SafeUser> {
    const user = this.users.get(userId);

    if (!user) {
      throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
    }

    user.role = newRole;
    user.updatedAt = new Date();
    this.users.set(user.id, user);

    // Destroy sessions to force re-authentication with new role
    await this.sessionManager.destroyUserSessions(userId);

    return this.sanitizeUser(user);
  }

  /**
   * Deactivate user account
   */
  async deactivateUser(userId: string): Promise<void> {
    const user = this.users.get(userId);

    if (!user) {
      throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
    }

    user.isActive = false;
    user.updatedAt = new Date();
    this.users.set(user.id, user);

    // Destroy all sessions
    await this.sessionManager.destroyUserSessions(userId);
  }

  /**
   * Activate user account
   */
  async activateUser(userId: string): Promise<void> {
    const user = this.users.get(userId);

    if (!user) {
      throw new AuthError('User not found', 'USER_NOT_FOUND', 404);
    }

    user.isActive = true;
    user.updatedAt = new Date();
    this.users.set(user.id, user);
  }

  /**
   * Generate JWT tokens for a user
   */
  private generateTokens(user: User): AuthTokens {
    const payload = {
      userId: user.id,
      email: user.email,
      role: user.role
    };

    const accessToken = this.jwtService.generateAccessToken(payload);
    const refreshToken = this.jwtService.generateRefreshToken(payload);

    // Calculate expiration time
    const expiresIn = this.parseExpiry(this.config.accessTokenExpiry);

    return {
      accessToken,
      refreshToken,
      expiresIn
    };
  }

  /**
   * Remove sensitive data from user object
   */
  private sanitizeUser(user: User): SafeUser {
    const { passwordHash, ...safeUser } = user;
    return safeUser;
  }

  /**
   * Validate email format
   */
  private isValidEmail(email: string): boolean {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    return emailRegex.test(email);
  }

  /**
   * Generate unique user ID
   */
  private generateUserId(): string {
    return `user_${crypto.randomBytes(16).toString('hex')}`;
  }

  /**
   * Generate unique reset token ID
   */
  private generateResetTokenId(): string {
    return `reset_${crypto.randomBytes(16).toString('hex')}`;
  }

  /**
   * Parse expiry string to seconds
   */
  private parseExpiry(expiry: string): number {
    const units: Record<string, number> = {
      s: 1,
      m: 60,
      h: 3600,
      d: 86400
    };

    const match = expiry.match(/^(\d+)([smhd])$/);
    if (!match || !match[1] || !match[2]) {
      return 900; // Default 15 minutes
    }

    const value = match[1];
    const unit = match[2];
    return parseInt(value, 10) * (units[unit as keyof typeof units] ?? 1);
  }

  /**
   * Initialize default admin user
   */
  private async initializeDefaultAdmin(): Promise<void> {
    const adminEmail = 'admin@accuscene.com';

    if (!this.usersByEmail.has(adminEmail)) {
      const defaultPassword = 'Admin123!@#';
      const passwordHash = await this.passwordService.hash(defaultPassword);

      const admin: User = {
        id: 'admin_default',
        email: adminEmail,
        passwordHash,
        role: UserRole.ADMIN,
        firstName: 'System',
        lastName: 'Administrator',
        createdAt: new Date(),
        updatedAt: new Date(),
        isActive: true,
        emailVerified: true,
        failedLoginAttempts: 0
      };

      this.users.set(admin.id, admin);
      this.usersByEmail.set(admin.email, admin);

      console.log('Default admin user created:');
      console.log(`Email: ${adminEmail}`);
      console.log(`Password: ${defaultPassword}`);
      console.log('IMPORTANT: Change this password immediately in production!');
    }
  }

  /**
   * Get session manager instance
   */
  getSessionManager(): SessionManager {
    return this.sessionManager;
  }

  /**
   * Get JWT service instance
   */
  getJwtService(): JwtService {
    return this.jwtService;
  }

  /**
   * Get password service instance
   */
  getPasswordService(): PasswordService {
    return this.passwordService;
  }

  /**
   * Cleanup and shutdown
   */
  async shutdown(): Promise<void> {
    await this.sessionManager.shutdown();
  }
}
