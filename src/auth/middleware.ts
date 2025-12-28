/**
 * Authentication Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';
import { JwtService } from './JwtService';
import { SessionManager } from './SessionManager';
import { UserRole, AuthError, DecodedToken, RateLimitConfig } from './types';

/**
 * Extend Express Request to include user information
 */
declare global {
  namespace Express {
    interface Request {
      user?: DecodedToken;
      sessionId?: string;
    }
  }
}

/**
 * Authentication middleware - Validates JWT tokens
 */
export function authMiddleware(jwtService: JwtService) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Extract token from Authorization header
      const authHeader = req.headers.authorization;

      if (!authHeader) {
        res.status(401).json({
          success: false,
          message: 'No authorization token provided',
          code: 'NO_TOKEN'
        });
        return;
      }

      // Check for Bearer token format
      const parts = authHeader.split(' ');
      if (parts.length !== 2 || parts[0] !== 'Bearer' || !parts[1]) {
        res.status(401).json({
          success: false,
          message: 'Invalid authorization header format. Expected: Bearer <token>',
          code: 'INVALID_HEADER_FORMAT'
        });
        return;
      }

      const token = parts[1];

      // Verify and decode token
      const decoded = jwtService.verifyToken(token, false);

      // Attach user info to request
      req.user = decoded;

      next();
    } catch (error) {
      if (error instanceof AuthError) {
        res.status(error.statusCode).json({
          success: false,
          message: error.message,
          code: error.code
        });
        return;
      }

      res.status(401).json({
        success: false,
        message: 'Invalid or expired token',
        code: 'UNAUTHORIZED'
      });
    }
  };
}

/**
 * Role-based access control middleware
 */
export function roleMiddleware(allowedRoles: UserRole[]) {
  return (req: Request, res: Response, next: NextFunction): void => {
    try {
      if (!req.user) {
        res.status(401).json({
          success: false,
          message: 'Authentication required',
          code: 'UNAUTHORIZED'
        });
        return;
      }

      const userRole = req.user.role;

      if (!allowedRoles.includes(userRole)) {
        res.status(403).json({
          success: false,
          message: 'Insufficient permissions',
          code: 'FORBIDDEN',
          requiredRoles: allowedRoles,
          userRole
        });
        return;
      }

      next();
    } catch (error) {
      res.status(500).json({
        success: false,
        message: 'Error checking permissions',
        code: 'INTERNAL_ERROR'
      });
    }
  };
}

/**
 * Rate limiter middleware - Prevents brute force attacks
 */
export class RateLimiter {
  private attempts: Map<string, { count: number; resetAt: number; blockedUntil?: number }>;
  private config: RateLimitConfig;

  constructor(config: RateLimitConfig) {
    this.attempts = new Map();
    this.config = config;

    // Clean up old entries every 5 minutes
    setInterval(() => {
      this.cleanup();
    }, 5 * 60 * 1000);
  }

  /**
   * Create rate limiter middleware
   */
  middleware() {
    return (req: Request, res: Response, next: NextFunction): void => {
      const identifier = this.getIdentifier(req);
      const now = Date.now();

      let record = this.attempts.get(identifier);

      // Check if currently blocked
      if (record && record.blockedUntil && record.blockedUntil > now) {
        const remainingTime = Math.ceil((record.blockedUntil - now) / 1000);
        res.status(429).json({
          success: false,
          message: 'Too many requests. Please try again later.',
          code: 'RATE_LIMIT_EXCEEDED',
          retryAfter: remainingTime
        });
        return;
      }

      // Initialize or reset record if window has passed
      if (!record || record.resetAt < now) {
        record = {
          count: 0,
          resetAt: now + this.config.windowMs
        };
      }

      // Increment attempt count
      record.count++;

      // Check if limit exceeded
      if (record.count > this.config.maxAttempts) {
        record.blockedUntil = now + this.config.blockDuration;
        this.attempts.set(identifier, record);

        const remainingTime = Math.ceil(this.config.blockDuration / 1000);
        res.status(429).json({
          success: false,
          message: 'Too many requests. Your access has been temporarily blocked.',
          code: 'RATE_LIMIT_EXCEEDED',
          retryAfter: remainingTime
        });
        return;
      }

      // Update record
      this.attempts.set(identifier, record);

      // Add rate limit headers
      res.setHeader('X-RateLimit-Limit', this.config.maxAttempts.toString());
      res.setHeader('X-RateLimit-Remaining', (this.config.maxAttempts - record.count).toString());
      res.setHeader('X-RateLimit-Reset', new Date(record.resetAt).toISOString());

      next();
    };
  }

  /**
   * Get identifier for rate limiting (IP + User-Agent)
   */
  private getIdentifier(req: Request): string {
    const ip = req.ip || req.socket.remoteAddress || 'unknown';
    const userAgent = req.headers['user-agent'] || 'unknown';
    return `${ip}:${userAgent}`;
  }

  /**
   * Clean up old entries
   */
  private cleanup(): void {
    const now = Date.now();
    for (const [key, record] of this.attempts.entries()) {
      if (record.resetAt < now && (!record.blockedUntil || record.blockedUntil < now)) {
        this.attempts.delete(key);
      }
    }
  }

  /**
   * Reset rate limit for an identifier
   */
  reset(identifier: string): void {
    this.attempts.delete(identifier);
  }

  /**
   * Get current attempt count for an identifier
   */
  getAttempts(identifier: string): number {
    return this.attempts.get(identifier)?.count || 0;
  }

  /**
   * Check if identifier is blocked
   */
  isBlocked(identifier: string): boolean {
    const record = this.attempts.get(identifier);
    if (!record || !record.blockedUntil) {
      return false;
    }
    return record.blockedUntil > Date.now();
  }
}

/**
 * Session validation middleware
 */
export function sessionMiddleware(sessionManager: SessionManager) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const sessionId = req.headers['x-session-id'] as string;

      if (!sessionId) {
        res.status(401).json({
          success: false,
          message: 'No session ID provided',
          code: 'NO_SESSION'
        });
        return;
      }

      // Validate session
      await sessionManager.validateSession(sessionId);

      // Attach session ID to request
      req.sessionId = sessionId;

      next();
    } catch (error) {
      if (error instanceof AuthError) {
        res.status(error.statusCode).json({
          success: false,
          message: error.message,
          code: error.code
        });
        return;
      }

      res.status(401).json({
        success: false,
        message: 'Invalid session',
        code: 'INVALID_SESSION'
      });
    }
  };
}

/**
 * Optional authentication middleware - Doesn't fail if no token provided
 */
export function optionalAuthMiddleware(jwtService: JwtService) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const authHeader = req.headers.authorization;

      if (!authHeader) {
        next();
        return;
      }

      const parts = authHeader.split(' ');
      if (parts.length === 2 && parts[0] === 'Bearer' && parts[1]) {
        const token = parts[1];
        const decoded = jwtService.verifyToken(token, false);
        req.user = decoded;
      }

      next();
    } catch (error) {
      // Ignore errors in optional auth
      next();
    }
  };
}

/**
 * Admin-only middleware (shorthand)
 */
export function adminOnly() {
  return roleMiddleware([UserRole.ADMIN]);
}

/**
 * Investigator and above middleware
 */
export function investigatorOrAbove() {
  return roleMiddleware([UserRole.ADMIN, UserRole.INVESTIGATOR]);
}

/**
 * Adjuster and above middleware
 */
export function adjusterOrAbove() {
  return roleMiddleware([UserRole.ADMIN, UserRole.INVESTIGATOR, UserRole.ADJUSTER]);
}

/**
 * Request logging middleware (for security auditing)
 */
export function auditMiddleware() {
  return (req: Request, res: Response, next: NextFunction): void => {
    const timestamp = new Date().toISOString();
    const ip = req.ip || req.socket.remoteAddress;
    const method = req.method;
    const path = req.path;
    const userId = req.user?.userId || 'anonymous';

    // Log request
    console.log(`[AUDIT] ${timestamp} | ${method} ${path} | User: ${userId} | IP: ${ip}`);

    next();
  };
}

/**
 * CORS middleware for API security
 */
export function corsMiddleware(allowedOrigins: string[] = ['http://localhost:3000']) {
  return (req: Request, res: Response, next: NextFunction): void => {
    const origin = req.headers.origin;

    if (origin && allowedOrigins.includes(origin)) {
      res.setHeader('Access-Control-Allow-Origin', origin);
      res.setHeader('Access-Control-Allow-Credentials', 'true');
      res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, PATCH, OPTIONS');
      res.setHeader('Access-Control-Allow-Headers', 'Content-Type, Authorization, X-Session-ID');
    }

    // Handle preflight
    if (req.method === 'OPTIONS') {
      res.status(204).send();
      return;
    }

    next();
  };
}

/**
 * Security headers middleware
 */
export function securityHeadersMiddleware() {
  return (req: Request, res: Response, next: NextFunction): void => {
    // Prevent clickjacking
    res.setHeader('X-Frame-Options', 'DENY');

    // Prevent MIME type sniffing
    res.setHeader('X-Content-Type-Options', 'nosniff');

    // Enable XSS protection
    res.setHeader('X-XSS-Protection', '1; mode=block');

    // Strict transport security
    res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

    // Content security policy
    res.setHeader(
      'Content-Security-Policy',
      "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: https:;"
    );

    next();
  };
}

/**
 * Error handling middleware for auth errors
 */
export function authErrorHandler() {
  return (err: Error, req: Request, res: Response, next: NextFunction): void => {
    if (err instanceof AuthError) {
      res.status(err.statusCode).json({
        success: false,
        message: err.message,
        code: err.code
      });
      return;
    }

    next(err);
  };
}
