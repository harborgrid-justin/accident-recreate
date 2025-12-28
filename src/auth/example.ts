/**
 * Authentication System Usage Examples
 * AccuScene Enterprise Accident Recreation Platform
 */

import { createAuthService, createRateLimiter, UserRole } from './index';
import type { Express, Request, Response } from 'express';

/**
 * Example 1: Initialize the authentication system
 */
export function initializeAuth() {
  const authService = createAuthService({
    jwtSecret: process.env.JWT_SECRET || 'your-secret-key-change-in-production',
    jwtRefreshSecret: process.env.JWT_REFRESH_SECRET || 'your-refresh-secret-change-in-production',
    accessTokenExpiry: '15m',
    refreshTokenExpiry: '7d',
    maxLoginAttempts: 5,
    lockoutDuration: 15 * 60 * 1000, // 15 minutes
    sessionTimeout: 24 * 60 * 60 * 1000, // 24 hours
    maxSessionsPerUser: 5
  });

  return authService;
}

/**
 * Example 2: Set up Express routes with authentication
 */
export function setupAuthRoutes(app: Express, authService: ReturnType<typeof createAuthService>) {
  const jwtService = authService.getJwtService();
  const rateLimiter = createRateLimiter();

  // Register endpoint with rate limiting
  app.post('/api/auth/register',
    rateLimiter.middleware(),
    async (req: Request, res: Response) => {
      try {
        const result = await authService.register({
          email: req.body.email,
          password: req.body.password,
          role: req.body.role || UserRole.VIEWER,
          firstName: req.body.firstName,
          lastName: req.body.lastName
        });

        res.status(201).json(result);
      } catch (error: any) {
        res.status(error.statusCode || 400).json({
          success: false,
          message: error.message,
          code: error.code
        });
      }
    }
  );

  // Login endpoint with rate limiting
  app.post('/api/auth/login',
    rateLimiter.middleware(),
    async (req: Request, res: Response) => {
      try {
        const result = await authService.login(
          {
            email: req.body.email,
            password: req.body.password
          },
          {
            userAgent: req.headers['user-agent'],
            ipAddress: req.ip
          }
        );

        res.json(result);
      } catch (error: any) {
        res.status(error.statusCode || 401).json({
          success: false,
          message: error.message,
          code: error.code
        });
      }
    }
  );

  // Refresh token endpoint
  app.post('/api/auth/refresh', async (req: Request, res: Response) => {
    try {
      const { refreshToken } = req.body;
      const tokens = await authService.refreshToken(refreshToken);

      res.json({
        success: true,
        tokens
      });
    } catch (error: any) {
      res.status(error.statusCode || 401).json({
        success: false,
        message: error.message,
        code: error.code
      });
    }
  });

  // Logout endpoint
  app.post('/api/auth/logout', async (req: Request, res: Response) => {
    try {
      const userId = req.body.userId;
      const sessionId = req.body.sessionId;

      await authService.logout(userId, sessionId);

      res.json({
        success: true,
        message: 'Logged out successfully'
      });
    } catch (error: any) {
      res.status(error.statusCode || 500).json({
        success: false,
        message: error.message
      });
    }
  });

  // Password reset request
  app.post('/api/auth/password-reset', async (req: Request, res: Response) => {
    try {
      await authService.resetPassword(req.body.email);

      res.json({
        success: true,
        message: 'If the email exists, a reset link will be sent'
      });
    } catch (error: any) {
      res.status(500).json({
        success: false,
        message: 'Password reset request failed'
      });
    }
  });

  // Complete password reset
  app.post('/api/auth/password-reset/complete', async (req: Request, res: Response) => {
    try {
      const { token, newPassword } = req.body;
      await authService.completePasswordReset(token, newPassword);

      res.json({
        success: true,
        message: 'Password reset successfully'
      });
    } catch (error: any) {
      res.status(error.statusCode || 400).json({
        success: false,
        message: error.message,
        code: error.code
      });
    }
  });
}

/**
 * Example 3: Protect routes with middleware
 */
export function setupProtectedRoutes(app: Express, authService: ReturnType<typeof createAuthService>) {
  const {
    authMiddleware,
    roleMiddleware,
    adminOnly,
    investigatorOrAbove,
    adjusterOrAbove
  } = require('./middleware');

  const jwtService = authService.getJwtService();

  // Public route - no authentication required
  app.get('/api/public/info', (req: Request, res: Response) => {
    res.json({ message: 'Public information' });
  });

  // Protected route - authentication required
  app.get('/api/profile',
    authMiddleware(jwtService),
    async (req: Request, res: Response) => {
      const user = await authService.getUserById(req.user!.userId);
      res.json({ user });
    }
  );

  // Admin-only route
  app.get('/api/admin/users',
    authMiddleware(jwtService),
    adminOnly(),
    async (req: Request, res: Response) => {
      res.json({ message: 'Admin access granted' });
    }
  );

  // Investigator and above
  app.post('/api/cases',
    authMiddleware(jwtService),
    investigatorOrAbove(),
    async (req: Request, res: Response) => {
      res.json({ message: 'Case created by investigator' });
    }
  );

  // Adjuster and above
  app.put('/api/cases/:id',
    authMiddleware(jwtService),
    adjusterOrAbove(),
    async (req: Request, res: Response) => {
      res.json({ message: 'Case updated by adjuster' });
    }
  );

  // Multiple roles
  app.delete('/api/cases/:id',
    authMiddleware(jwtService),
    roleMiddleware([UserRole.ADMIN, UserRole.INVESTIGATOR]),
    async (req: Request, res: Response) => {
      res.json({ message: 'Case deleted' });
    }
  );
}

/**
 * Example 4: User management operations
 */
export async function userManagementExamples(authService: ReturnType<typeof createAuthService>) {
  // Register a new investigator
  const investigator = await authService.register({
    email: 'investigator@example.com',
    password: 'SecurePass123!',
    role: UserRole.INVESTIGATOR,
    firstName: 'Jane',
    lastName: 'Smith'
  });

  console.log('Registered investigator:', investigator.user);

  // Login the investigator
  const loginResult = await authService.login({
    email: 'investigator@example.com',
    password: 'SecurePass123!'
  });

  console.log('Access token:', loginResult.tokens?.accessToken);

  // Change password
  await authService.changePassword(
    investigator.user!.id,
    'SecurePass123!',
    'NewSecurePass456!'
  );

  // Update user role (admin only)
  await authService.updateUserRole(investigator.user!.id, UserRole.ADMIN);

  // Deactivate user
  await authService.deactivateUser(investigator.user!.id);

  // Reactivate user
  await authService.activateUser(investigator.user!.id);
}

/**
 * Example 5: Session management
 */
export async function sessionManagementExamples(authService: ReturnType<typeof createAuthService>) {
  const sessionManager = authService.getSessionManager();

  // Get all sessions for a user
  const sessions = sessionManager.getUserSessions('user_123');
  console.log('Active sessions:', sessions.length);

  // Get session statistics
  const stats = sessionManager.getSessionStats();
  console.log('Session stats:', stats);

  // Check for suspicious activity
  const suspiciousSessions = sessionManager.getSuspiciousSessions('user_123');
  if (suspiciousSessions.length > 0) {
    console.warn('Suspicious activity detected!');
  }

  // Destroy all sessions for a user
  await sessionManager.destroyUserSessions('user_123');

  // Cleanup expired sessions
  const cleanedCount = await sessionManager.cleanupExpiredSessions();
  console.log(`Cleaned up ${cleanedCount} expired sessions`);
}

/**
 * Example 6: Password validation
 */
export function passwordValidationExamples(authService: ReturnType<typeof createAuthService>) {
  const passwordService = authService.getPasswordService();

  // Validate password strength
  const validation = passwordService.validatePasswordStrength('MyPassword123!');

  if (validation.isValid) {
    console.log('Password is valid! Score:', validation.score);
  } else {
    console.error('Password validation failed:');
    validation.errors.forEach((error: string) => console.error('  -', error));
  }

  if (validation.suggestions.length > 0) {
    console.log('Suggestions:');
    validation.suggestions.forEach((suggestion: string) => console.log('  -', suggestion));
  }

  // Generate a secure password
  const securePassword = passwordService.generateSecurePassword(16);
  console.log('Generated secure password:', securePassword);

  // Calculate password entropy
  const entropy = passwordService.calculateEntropy('MyPassword123!');
  console.log('Password entropy:', entropy, 'bits');
}

/**
 * Example 7: Token operations
 */
export function tokenOperationExamples(authService: ReturnType<typeof createAuthService>) {
  const jwtService = authService.getJwtService();

  // Generate tokens
  const accessToken = jwtService.generateAccessToken({
    userId: 'user_123',
    email: 'user@example.com',
    role: UserRole.INVESTIGATOR
  });

  const refreshToken = jwtService.generateRefreshToken({
    userId: 'user_123',
    email: 'user@example.com',
    role: UserRole.INVESTIGATOR
  });

  // Verify token
  const decoded = jwtService.verifyToken(accessToken, false);
  console.log('Decoded token:', decoded);

  // Check if token is expired
  const isExpired = jwtService.isTokenExpired(accessToken);
  console.log('Token expired:', isExpired);

  // Get token expiration
  const expiration = jwtService.getTokenExpiration(accessToken);
  console.log('Token expires at:', new Date(expiration! * 1000));

  // Decode without verification (unsafe - for debugging only)
  const decodedUnsafe = jwtService.decodeToken(accessToken);
  console.log('Decoded (unsafe):', decodedUnsafe);
}

/**
 * Example 8: Complete authentication flow
 */
export async function completeAuthFlow() {
  // Step 1: Initialize
  const authService = initializeAuth();

  // Step 2: Register new user
  const registration = await authService.register({
    email: 'john.doe@example.com',
    password: 'SecurePass123!',
    role: UserRole.ADJUSTER,
    firstName: 'John',
    lastName: 'Doe'
  });

  console.log('✓ User registered:', registration.user?.email);

  // Step 3: Login
  const login = await authService.login({
    email: 'john.doe@example.com',
    password: 'SecurePass123!'
  });

  console.log('✓ Login successful');
  console.log('  Access Token:', login.tokens?.accessToken.substring(0, 20) + '...');
  console.log('  Refresh Token:', login.tokens?.refreshToken.substring(0, 20) + '...');

  // Step 4: Validate token
  const user = await authService.validateToken(login.tokens!.accessToken);
  console.log('✓ Token validated for:', user.email);

  // Step 5: Refresh token
  const newTokens = await authService.refreshToken(login.tokens!.refreshToken);
  console.log('✓ Tokens refreshed');

  // Step 6: Change password
  await authService.changePassword(
    user.id,
    'SecurePass123!',
    'NewSecurePass456!'
  );
  console.log('✓ Password changed');

  // Step 7: Logout
  await authService.logout(user.id);
  console.log('✓ Logged out');

  return authService;
}

/**
 * Example 9: Error handling
 */
export async function errorHandlingExample() {
  const authService = initializeAuth();

  try {
    await authService.login({
      email: 'nonexistent@example.com',
      password: 'wrong'
    });
  } catch (error: any) {
    if (error.code === 'INVALID_CREDENTIALS') {
      console.error('Invalid email or password');
    } else if (error.code === 'ACCOUNT_LOCKED') {
      console.error('Account is locked due to too many failed attempts');
    } else if (error.code === 'ACCOUNT_DEACTIVATED') {
      console.error('Account has been deactivated');
    } else {
      console.error('Login error:', error.message);
    }
  }
}

// Run examples if this file is executed directly
if (require.main === module) {
  completeAuthFlow()
    .then(() => console.log('\nAll examples completed successfully!'))
    .catch(error => console.error('Error:', error));
}
