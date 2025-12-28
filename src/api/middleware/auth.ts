/**
 * Authentication & Authorization Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';
import * as jwt from 'jsonwebtoken';
import { UserRole, DecodedToken, AuthError } from '../../auth/types';
import { ErrorCode } from '../responses';
import { ForbiddenError } from './errorHandler';

// Extend Express Request to include user
declare global {
  namespace Express {
    interface Request {
      user?: DecodedToken;
    }
  }
}

/**
 * Authentication middleware - verifies JWT token
 */
export function authenticate(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  try {
    // Get token from Authorization header
    const authHeader = req.get('Authorization');

    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      throw new AuthError(
        'No authentication token provided',
        ErrorCode.UNAUTHORIZED,
        401
      );
    }

    const token = authHeader.substring(7); // Remove 'Bearer ' prefix

    // Verify token
    const jwtSecret = process.env.JWT_SECRET || 'default-secret-change-in-production';
    const decoded = jwt.verify(token, jwtSecret) as DecodedToken;

    // Ensure it's an access token
    if (decoded.type !== 'access') {
      throw new AuthError(
        'Invalid token type',
        ErrorCode.TOKEN_INVALID,
        401
      );
    }

    // Attach user to request
    req.user = decoded;

    next();
  } catch (err) {
    if (err instanceof jwt.TokenExpiredError) {
      next(new AuthError('Token has expired', ErrorCode.TOKEN_EXPIRED, 401));
    } else if (err instanceof jwt.JsonWebTokenError) {
      next(new AuthError('Invalid token', ErrorCode.TOKEN_INVALID, 401));
    } else {
      next(err);
    }
  }
}

/**
 * Optional authentication - doesn't fail if token is missing
 */
export function optionalAuthenticate(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  try {
    const authHeader = req.get('Authorization');

    if (authHeader && authHeader.startsWith('Bearer ')) {
      const token = authHeader.substring(7);
      const jwtSecret = process.env.JWT_SECRET || 'default-secret-change-in-production';
      const decoded = jwt.verify(token, jwtSecret) as DecodedToken;

      if (decoded.type === 'access') {
        req.user = decoded;
      }
    }

    next();
  } catch (err) {
    // Silently ignore authentication errors in optional mode
    next();
  }
}

/**
 * Authorization middleware - checks user roles
 */
export function authorize(...allowedRoles: UserRole[]) {
  return (req: Request, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new AuthError(
        'Authentication required',
        ErrorCode.UNAUTHORIZED,
        401
      );
    }

    if (!allowedRoles.includes(req.user.role)) {
      throw new ForbiddenError(
        `Access denied. Required roles: ${allowedRoles.join(', ')}`
      );
    }

    next();
  };
}

/**
 * Check if user is admin
 */
export function requireAdmin(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  if (!req.user) {
    throw new AuthError(
      'Authentication required',
      ErrorCode.UNAUTHORIZED,
      401
    );
  }

  if (req.user.role !== UserRole.ADMIN) {
    throw new ForbiddenError('Admin access required');
  }

  next();
}

/**
 * Check if user owns the resource or is admin
 */
export function requireOwnershipOrAdmin(userIdParam: string = 'userId') {
  return (req: Request, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new AuthError(
        'Authentication required',
        ErrorCode.UNAUTHORIZED,
        401
      );
    }

    const resourceUserId = req.params[userIdParam] || req.body[userIdParam];

    if (req.user.role === UserRole.ADMIN || req.user.userId === resourceUserId) {
      next();
    } else {
      throw new ForbiddenError('You can only access your own resources');
    }
  };
}

/**
 * Rate limiting per user
 */
export function rateLimit(maxRequests: number, windowMs: number) {
  const requests = new Map<string, number[]>();

  return (req: Request, res: Response, next: NextFunction): void => {
    const userId = req.user?.userId || getClientIp(req);
    const now = Date.now();

    // Get user's request timestamps
    const userRequests = requests.get(userId) || [];

    // Filter out old requests outside the window
    const recentRequests = userRequests.filter(
      timestamp => now - timestamp < windowMs
    );

    // Check if limit exceeded
    if (recentRequests.length >= maxRequests) {
      const oldestRequest = Math.min(...recentRequests);
      const retryAfter = Math.ceil((oldestRequest + windowMs - now) / 1000);

      res.setHeader('Retry-After', retryAfter.toString());
      res.setHeader('X-RateLimit-Limit', maxRequests.toString());
      res.setHeader('X-RateLimit-Remaining', '0');
      res.setHeader('X-RateLimit-Reset', new Date(oldestRequest + windowMs).toISOString());

      throw new AuthError(
        'Rate limit exceeded. Please try again later.',
        ErrorCode.RATE_LIMIT_EXCEEDED,
        429
      );
    }

    // Add current request
    recentRequests.push(now);
    requests.set(userId, recentRequests);

    // Set rate limit headers
    res.setHeader('X-RateLimit-Limit', maxRequests.toString());
    res.setHeader('X-RateLimit-Remaining', (maxRequests - recentRequests.length).toString());

    next();
  };
}

/**
 * Get client IP address
 */
function getClientIp(req: Request): string {
  const forwarded = req.get('x-forwarded-for');
  if (forwarded) {
    return (forwarded.split(',')[0] || '').trim();
  }

  const realIp = req.get('x-real-ip');
  if (realIp) {
    return realIp;
  }

  return req.ip || req.socket.remoteAddress || 'unknown';
}
