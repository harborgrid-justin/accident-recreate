# AccuScene Authentication System

Enterprise-grade authentication and authorization system with JWT tokens, role-based access control, and comprehensive security features.

## Features

- **JWT Authentication**: Secure token-based authentication with access and refresh tokens
- **Role-Based Access Control (RBAC)**: Four role levels (ADMIN, INVESTIGATOR, ADJUSTER, VIEWER)
- **Session Management**: Track and manage user sessions across devices
- **Password Security**: PBKDF2 hashing, strength validation, breach detection
- **Rate Limiting**: Prevent brute force attacks
- **Account Lockout**: Automatic lockout after failed login attempts
- **Password Reset**: Secure password reset with time-limited tokens
- **Security Middleware**: Express middleware for route protection
- **Audit Logging**: Track authentication events

## Quick Start

### 1. Initialize Authentication Service

```typescript
import { createAuthService, UserRole } from './auth';

const authService = createAuthService({
  jwtSecret: 'your-secret-key',
  jwtRefreshSecret: 'your-refresh-secret',
  accessTokenExpiry: '15m',
  refreshTokenExpiry: '7d'
});
```

### 2. Register a New User

```typescript
const result = await authService.register({
  email: 'investigator@example.com',
  password: 'SecurePass123!',
  role: UserRole.INVESTIGATOR,
  firstName: 'John',
  lastName: 'Doe'
});

console.log('User registered:', result.user);
```

### 3. Login

```typescript
const loginResult = await authService.login(
  {
    email: 'investigator@example.com',
    password: 'SecurePass123!'
  },
  {
    userAgent: req.headers['user-agent'],
    ipAddress: req.ip
  }
);

console.log('Access Token:', loginResult.tokens.accessToken);
console.log('Refresh Token:', loginResult.tokens.refreshToken);
```

### 4. Protect Routes with Middleware

```typescript
import express from 'express';
import { authMiddleware, roleMiddleware, UserRole } from './auth';

const app = express();
const jwtService = authService.getJwtService();

// Protect all routes
app.use('/api', authMiddleware(jwtService));

// Admin-only route
app.get('/api/admin/users',
  roleMiddleware([UserRole.ADMIN]),
  (req, res) => {
    res.json({ users: [] });
  }
);

// Investigator and above
app.post('/api/cases',
  roleMiddleware([UserRole.ADMIN, UserRole.INVESTIGATOR]),
  (req, res) => {
    res.json({ message: 'Case created' });
  }
);
```

### 5. Rate Limiting

```typescript
import { createRateLimiter } from './auth';

const rateLimiter = createRateLimiter({
  windowMs: 15 * 60 * 1000, // 15 minutes
  maxAttempts: 100,
  blockDuration: 15 * 60 * 1000 // 15 minutes
});

app.use('/api/auth/login', rateLimiter.middleware());
```

### 6. Refresh Token

```typescript
const newTokens = await authService.refreshToken(refreshToken);
console.log('New Access Token:', newTokens.accessToken);
```

### 7. Change Password

```typescript
await authService.changePassword(
  userId,
  'OldPassword123!',
  'NewSecurePass456!'
);
```

### 8. Password Reset Flow

```typescript
// Step 1: Request reset token
const resetToken = await authService.resetPassword('user@example.com');
// In production, this would be sent via email

// Step 2: Complete reset with token
await authService.completePasswordReset(resetToken, 'NewPassword123!');
```

## User Roles

The system supports four hierarchical roles:

1. **VIEWER** (Level 1)
   - Read-only access
   - View cases and reports

2. **ADJUSTER** (Level 2)
   - All VIEWER permissions
   - Create and edit basic case information

3. **INVESTIGATOR** (Level 3)
   - All ADJUSTER permissions
   - Full case management
   - Advanced analysis tools
   - Report generation

4. **ADMIN** (Level 4)
   - Full system access
   - User management
   - System configuration
   - All permissions

## Security Features

### Password Requirements

- Minimum 8 characters
- At least one uppercase letter
- At least one lowercase letter
- At least one number
- At least one special character
- Not previously compromised in data breaches

### Account Protection

- Maximum 5 failed login attempts
- 15-minute account lockout after max failures
- Session timeout after 24 hours of inactivity
- Maximum 5 concurrent sessions per user

### Token Security

- Access tokens expire after 15 minutes
- Refresh tokens expire after 7 days
- HMAC-SHA256 signature
- Timing-safe comparisons

## API Reference

### AuthService

#### `register(data: RegisterData): Promise<AuthResponse>`
Register a new user account.

#### `login(credentials: LoginCredentials, metadata?): Promise<AuthResponse>`
Authenticate user and return tokens.

#### `validateToken(token: string): Promise<SafeUser>`
Validate an access token and return user data.

#### `refreshToken(refreshToken: string): Promise<AuthTokens>`
Generate new access token using refresh token.

#### `logout(userId: string, sessionId?: string): Promise<void>`
Destroy user session(s).

#### `changePassword(userId: string, oldPass: string, newPass: string): Promise<void>`
Change user password.

#### `resetPassword(email: string): Promise<string>`
Initiate password reset process.

#### `completePasswordReset(token: string, newPassword: string): Promise<void>`
Complete password reset with token.

### Middleware Functions

#### `authMiddleware(jwtService: JwtService)`
Validates JWT token on protected routes.

#### `roleMiddleware(allowedRoles: UserRole[])`
Restricts access based on user roles.

#### `sessionMiddleware(sessionManager: SessionManager)`
Validates session on each request.

#### `adminOnly()`
Shorthand for admin-only routes.

#### `investigatorOrAbove()`
Allows INVESTIGATOR and ADMIN roles.

#### `adjusterOrAbove()`
Allows ADJUSTER, INVESTIGATOR, and ADMIN roles.

## Default Admin Account

For initial setup, a default admin account is created:

- **Email**: admin@accuscene.com
- **Password**: Admin123!@#

**⚠️ IMPORTANT**: Change this password immediately in production!

## Error Handling

The system uses custom error classes:

```typescript
try {
  await authService.login(credentials);
} catch (error) {
  if (error instanceof AuthError) {
    console.error(`Auth Error [${error.code}]:`, error.message);
    // Handle auth error
  } else if (error instanceof ValidationError) {
    console.error('Validation Error:', error.fields);
    // Handle validation error
  }
}
```

## Environment Variables

```bash
JWT_SECRET=your-secret-key-change-in-production
JWT_REFRESH_SECRET=your-refresh-secret-change-in-production
ACCESS_TOKEN_EXPIRY=15m
REFRESH_TOKEN_EXPIRY=7d
BCRYPT_ROUNDS=100000
```

## Production Checklist

- [ ] Change default admin password
- [ ] Set strong JWT secrets via environment variables
- [ ] Enable HTTPS only
- [ ] Configure CORS properly
- [ ] Set up email service for password resets
- [ ] Implement token blacklist with Redis
- [ ] Enable audit logging to file/database
- [ ] Configure rate limiting based on traffic
- [ ] Set up monitoring and alerts
- [ ] Regular security audits

## License

Copyright © 2025 AccuScene Enterprise. All rights reserved.
