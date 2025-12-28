/**
 * Request/Response Logging Middleware
 * AccuScene Enterprise Accident Recreation Platform
 */

import { Request, Response, NextFunction } from 'express';

export interface LogEntry {
  requestId: string;
  timestamp: string;
  method: string;
  path: string;
  query: any;
  ip: string;
  userAgent?: string;
  userId?: string;
  statusCode?: number;
  duration?: number;
  error?: string;
}

/**
 * Request logging middleware
 */
export function requestLogger(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Generate unique request ID
  const requestId = generateRequestId();
  (req as any).id = requestId;

  const startTime = Date.now();

  // Log request details
  const logEntry: LogEntry = {
    requestId,
    timestamp: new Date().toISOString(),
    method: req.method,
    path: req.path,
    query: req.query,
    ip: getClientIp(req),
    userAgent: req.get('user-agent'),
    userId: (req as any).user?.userId,
  };

  // Log incoming request
  console.log('[Request]', {
    requestId: logEntry.requestId,
    method: logEntry.method,
    path: logEntry.path,
    ip: logEntry.ip,
    userId: logEntry.userId,
  });

  // Capture response
  const originalSend = res.send;
  res.send = function (data: any): Response {
    res.send = originalSend;

    // Calculate request duration
    const duration = Date.now() - startTime;

    // Log response
    console.log('[Response]', {
      requestId: logEntry.requestId,
      method: logEntry.method,
      path: logEntry.path,
      statusCode: res.statusCode,
      duration: `${duration}ms`,
      userId: logEntry.userId,
    });

    return res.send(data);
  };

  next();
}

/**
 * Security headers middleware
 */
export function securityHeaders(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  // Remove X-Powered-By header
  res.removeHeader('X-Powered-By');

  // Add security headers
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  res.setHeader('Strict-Transport-Security', 'max-age=31536000; includeSubDomains');

  next();
}

/**
 * CORS configuration
 */
export function corsOptions() {
  return {
    origin: (origin: string | undefined, callback: Function) => {
      const allowedOrigins = process.env.ALLOWED_ORIGINS?.split(',') || [
        'http://localhost:3000',
        'http://localhost:5173',
      ];

      // Allow requests with no origin (mobile apps, Postman, etc.)
      if (!origin) {
        return callback(null, true);
      }

      if (allowedOrigins.includes(origin) || allowedOrigins.includes('*')) {
        callback(null, true);
      } else {
        callback(new Error('Not allowed by CORS'));
      }
    },
    credentials: true,
    methods: ['GET', 'POST', 'PUT', 'PATCH', 'DELETE', 'OPTIONS'],
    allowedHeaders: [
      'Origin',
      'X-Requested-With',
      'Content-Type',
      'Accept',
      'Authorization',
    ],
    exposedHeaders: ['X-Total-Count', 'X-Page', 'X-Per-Page'],
    maxAge: 86400, // 24 hours
  };
}

/**
 * Request size limiter middleware
 */
export function requestSizeLimiter(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const contentLength = req.get('content-length');
  const maxSize = 50 * 1024 * 1024; // 50MB

  if (contentLength && parseInt(contentLength) > maxSize) {
    res.status(413).json({
      success: false,
      error: {
        code: 'PAYLOAD_TOO_LARGE',
        message: 'Request payload too large',
      },
    });
    return;
  }

  next();
}

/**
 * Generate unique request ID
 */
function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
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

/**
 * Sanitize sensitive data from logs
 */
export function sanitizeForLog(data: any): any {
  if (!data || typeof data !== 'object') {
    return data;
  }

  const sensitiveFields = [
    'password',
    'passwordHash',
    'token',
    'accessToken',
    'refreshToken',
    'secret',
    'apiKey',
    'creditCard',
    'ssn',
  ];

  const sanitized = { ...data };

  for (const key in sanitized) {
    if (sensitiveFields.some(field => key.toLowerCase().includes(field))) {
      sanitized[key] = '[REDACTED]';
    } else if (typeof sanitized[key] === 'object') {
      sanitized[key] = sanitizeForLog(sanitized[key]);
    }
  }

  return sanitized;
}
