/**
 * Authentication System Test & Demo Script
 * AccuScene Enterprise Accident Recreation Platform
 *
 * Run this to verify the authentication system is working correctly
 */

import { createAuthService, UserRole, AuthError, ValidationError } from './index';

async function runTests() {
  console.log('═══════════════════════════════════════════════════════════');
  console.log('  AccuScene Authentication System - Test & Demo');
  console.log('═══════════════════════════════════════════════════════════\n');

  const authService = createAuthService();
  let testsPassed = 0;
  let testsFailed = 0;

  // Helper function for test assertions
  const test = async (name: string, fn: () => Promise<void>) => {
    try {
      await fn();
      console.log(`✓ ${name}`);
      testsPassed++;
    } catch (error: any) {
      console.error(`✗ ${name}`);
      console.error(`  Error: ${error.message}`);
      testsFailed++;
    }
  };

  console.log('Testing Authentication System...\n');

  // Test 1: Default admin exists
  await test('Default admin user exists', async () => {
    const admin = await authService.getUserByEmail('admin@accuscene.com');
    if (!admin) throw new Error('Admin user not found');
  });

  // Test 2: Register new user
  let testUserId = '';
  await test('Register new user', async () => {
    const result = await authService.register({
      email: 'test.investigator@example.com',
      password: 'TestPass123!',
      role: UserRole.INVESTIGATOR,
      firstName: 'Test',
      lastName: 'Investigator'
    });
    if (!result.success || !result.user) {
      throw new Error('Registration failed');
    }
    testUserId = result.user.id;
  });

  // Test 3: Prevent duplicate registration
  await test('Prevent duplicate email registration', async () => {
    try {
      await authService.register({
        email: 'test.investigator@example.com',
        password: 'TestPass123!',
        role: UserRole.VIEWER
      });
      throw new Error('Should have failed with duplicate email');
    } catch (error) {
      if (!(error instanceof ValidationError)) {
        throw new Error('Expected ValidationError');
      }
    }
  });

  // Test 4: Reject weak passwords
  await test('Reject weak passwords', async () => {
    try {
      await authService.register({
        email: 'weak.password@example.com',
        password: 'weak',
        role: UserRole.VIEWER
      });
      throw new Error('Should have rejected weak password');
    } catch (error) {
      if (!(error instanceof ValidationError)) {
        throw new Error('Expected ValidationError for weak password');
      }
    }
  });

  // Test 5: Login with correct credentials
  let accessToken = '';
  let refreshToken = '';
  await test('Login with correct credentials', async () => {
    const result = await authService.login({
      email: 'test.investigator@example.com',
      password: 'TestPass123!'
    });
    if (!result.success || !result.tokens) {
      throw new Error('Login failed');
    }
    accessToken = result.tokens.accessToken;
    refreshToken = result.tokens.refreshToken;
  });

  // Test 6: Reject incorrect credentials
  await test('Reject incorrect credentials', async () => {
    try {
      await authService.login({
        email: 'test.investigator@example.com',
        password: 'WrongPassword123!'
      });
      throw new Error('Should have failed with invalid credentials');
    } catch (error) {
      if (!(error instanceof AuthError)) {
        throw new Error('Expected AuthError');
      }
    }
  });

  // Test 7: Validate access token
  await test('Validate access token', async () => {
    const user = await authService.validateToken(accessToken);
    if (user.email !== 'test.investigator@example.com') {
      throw new Error('Token validation failed');
    }
  });

  // Test 8: Refresh token
  await test('Refresh token', async () => {
    const newTokens = await authService.refreshToken(refreshToken);
    if (!newTokens.accessToken || !newTokens.refreshToken) {
      throw new Error('Token refresh failed');
    }
    accessToken = newTokens.accessToken;
    refreshToken = newTokens.refreshToken;
  });

  // Test 9: Change password
  await test('Change password', async () => {
    await authService.changePassword(
      testUserId,
      'TestPass123!',
      'NewTestPass456!'
    );
  });

  // Test 10: Login with new password
  await test('Login with new password', async () => {
    const result = await authService.login({
      email: 'test.investigator@example.com',
      password: 'NewTestPass456!'
    });
    if (!result.success) {
      throw new Error('Login with new password failed');
    }
  });

  // Test 11: Password reset flow
  await test('Password reset flow', async () => {
    const resetToken = await authService.resetPassword('test.investigator@example.com');
    await authService.completePasswordReset(resetToken, 'ResetPass789!');
  });

  // Test 12: Session management
  await test('Session management', async () => {
    const sessionManager = authService.getSessionManager();
    const loginResult = await authService.login({
      email: 'test.investigator@example.com',
      password: 'ResetPass789!'
    });

    const stats = sessionManager.getSessionStats();
    if (stats.activeSessions === 0) {
      throw new Error('No active sessions found');
    }
  });

  // Test 13: JWT service
  await test('JWT token generation and verification', async () => {
    const jwtService = authService.getJwtService();
    const token = jwtService.generateAccessToken({
      userId: 'test_user',
      email: 'test@example.com',
      role: UserRole.VIEWER
    });

    const decoded = jwtService.verifyToken(token, false);
    if (decoded.userId !== 'test_user') {
      throw new Error('Token verification failed');
    }
  });

  // Test 14: Password strength validation
  await test('Password strength validation', async () => {
    const passwordService = authService.getPasswordService();

    const weak = passwordService.validatePasswordStrength('weak');
    if (weak.isValid) {
      throw new Error('Should reject weak password');
    }

    const strong = passwordService.validatePasswordStrength('StrongPass123!');
    if (!strong.isValid) {
      throw new Error('Should accept strong password');
    }
  });

  // Test 15: Generate secure password
  await test('Generate secure password', async () => {
    const passwordService = authService.getPasswordService();
    const password = passwordService.generateSecurePassword(16);

    if (password.length !== 16) {
      throw new Error('Generated password has wrong length');
    }

    const validation = passwordService.validatePasswordStrength(password);
    if (!validation.isValid) {
      throw new Error('Generated password should be valid');
    }
  });

  // Test 16: Role hierarchy
  await test('Role hierarchy and permissions', async () => {
    const { hasPermissionLevel, UserRole } = require('./index');

    if (!hasPermissionLevel(UserRole.ADMIN, UserRole.VIEWER)) {
      throw new Error('Admin should have viewer permissions');
    }

    if (hasPermissionLevel(UserRole.VIEWER, UserRole.ADMIN)) {
      throw new Error('Viewer should not have admin permissions');
    }
  });

  // Test 17: Account lockout
  await test('Account lockout after failed attempts', async () => {
    // Register a new user for lockout test
    await authService.register({
      email: 'lockout.test@example.com',
      password: 'LockoutTest123!',
      role: UserRole.VIEWER
    });

    // Attempt multiple failed logins
    for (let i = 0; i < 5; i++) {
      try {
        await authService.login({
          email: 'lockout.test@example.com',
          password: 'WrongPassword'
        });
      } catch (error) {
        // Expected to fail
      }
    }

    // Next attempt should result in account lockout
    try {
      await authService.login({
        email: 'lockout.test@example.com',
        password: 'LockoutTest123!'
      });
      throw new Error('Should be locked out');
    } catch (error) {
      if (!(error instanceof AuthError) || error.code !== 'ACCOUNT_LOCKED') {
        throw new Error('Expected account lockout');
      }
    }
  });

  // Test 18: User deactivation
  await test('User deactivation and activation', async () => {
    await authService.deactivateUser(testUserId);

    try {
      await authService.login({
        email: 'test.investigator@example.com',
        password: 'ResetPass789!'
      });
      throw new Error('Should not allow deactivated user to login');
    } catch (error) {
      if (!(error instanceof AuthError)) {
        throw new Error('Expected AuthError for deactivated account');
      }
    }

    await authService.activateUser(testUserId);
  });

  // Test 19: Role update
  await test('Update user role', async () => {
    const updatedUser = await authService.updateUserRole(testUserId, UserRole.ADMIN);
    if (updatedUser.role !== UserRole.ADMIN) {
      throw new Error('Role update failed');
    }
  });

  // Test 20: Logout
  await test('Logout user', async () => {
    await authService.logout(testUserId);
    const sessionManager = authService.getSessionManager();
    const sessions = sessionManager.getUserSessions(testUserId);
    if (sessions.length > 0) {
      throw new Error('Sessions should be cleared after logout');
    }
  });

  // Print results
  console.log('\n═══════════════════════════════════════════════════════════');
  console.log(`  Test Results: ${testsPassed} passed, ${testsFailed} failed`);
  console.log('═══════════════════════════════════════════════════════════');

  if (testsFailed === 0) {
    console.log('\n✓ All tests passed! Authentication system is working correctly.\n');
  } else {
    console.log(`\n✗ ${testsFailed} test(s) failed. Please review the errors above.\n`);
    process.exit(1);
  }

  // Cleanup
  await authService.shutdown();
}

// Run tests if this file is executed directly
if (require.main === module) {
  runTests()
    .then(() => {
      console.log('Demo completed successfully!');
      process.exit(0);
    })
    .catch(error => {
      console.error('Error running tests:', error);
      process.exit(1);
    });
}

export { runTests };
