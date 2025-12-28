/**
 * JWT Service - JSON Web Token Operations
 * AccuScene Enterprise Accident Recreation Platform
 * Using Node.js crypto module for zero external dependencies
 */

import * as crypto from 'crypto';
import { AuthTokenPayload, DecodedToken, AuthError, AuthConfig } from './types';

export class JwtService {
  private config: AuthConfig;

  constructor(config: AuthConfig) {
    this.config = config;
  }

  /**
   * Generate an access token (short-lived)
   */
  generateAccessToken(payload: Omit<AuthTokenPayload, 'type' | 'iat' | 'exp'>): string {
    const now = Math.floor(Date.now() / 1000);
    const expiresIn = this.parseExpiry(this.config.accessTokenExpiry);

    const tokenPayload: AuthTokenPayload = {
      ...payload,
      type: 'access',
      iat: now,
      exp: now + expiresIn
    };

    return this.createToken(tokenPayload, this.config.jwtSecret);
  }

  /**
   * Generate a refresh token (long-lived)
   */
  generateRefreshToken(payload: Omit<AuthTokenPayload, 'type' | 'iat' | 'exp'>): string {
    const now = Math.floor(Date.now() / 1000);
    const expiresIn = this.parseExpiry(this.config.refreshTokenExpiry);

    const tokenPayload: AuthTokenPayload = {
      ...payload,
      type: 'refresh',
      iat: now,
      exp: now + expiresIn
    };

    return this.createToken(tokenPayload, this.config.jwtRefreshSecret);
  }

  /**
   * Verify and decode an access token
   */
  verifyToken(token: string, isRefreshToken: boolean = false): DecodedToken {
    try {
      const secret = isRefreshToken ? this.config.jwtRefreshSecret : this.config.jwtSecret;
      const decoded = this.verifyAndDecodeToken(token, secret);

      // Verify token type matches
      const expectedType = isRefreshToken ? 'refresh' : 'access';
      if (decoded.type !== expectedType) {
        throw new AuthError(
          `Invalid token type. Expected ${expectedType}, got ${decoded.type}`,
          'INVALID_TOKEN_TYPE',
          401
        );
      }

      return decoded;
    } catch (error) {
      if (error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Invalid or expired token', 'INVALID_TOKEN', 401);
    }
  }

  /**
   * Decode a token without verification (use with caution)
   */
  decodeToken(token: string): DecodedToken | null {
    try {
      const parts = token.split('.');
      if (parts.length !== 3 || !parts[1]) {
        return null;
      }

      const payload = JSON.parse(this.base64UrlDecode(parts[1]));
      return payload as DecodedToken;
    } catch (error) {
      return null;
    }
  }

  /**
   * Check if a token is expired
   */
  isTokenExpired(token: string): boolean {
    const decoded = this.decodeToken(token);
    if (!decoded || !decoded.exp) {
      return true;
    }

    const now = Math.floor(Date.now() / 1000);
    return decoded.exp < now;
  }

  /**
   * Get token expiration time in seconds
   */
  getTokenExpiration(token: string): number | null {
    const decoded = this.decodeToken(token);
    return decoded?.exp || null;
  }

  /**
   * Create a JWT token with HS256 signature
   */
  private createToken(payload: AuthTokenPayload, secret: string): string {
    const header = {
      alg: 'HS256',
      typ: 'JWT'
    };

    const encodedHeader = this.base64UrlEncode(JSON.stringify(header));
    const encodedPayload = this.base64UrlEncode(JSON.stringify(payload));

    const signature = this.createSignature(
      `${encodedHeader}.${encodedPayload}`,
      secret
    );

    return `${encodedHeader}.${encodedPayload}.${signature}`;
  }

  /**
   * Verify and decode a JWT token
   */
  private verifyAndDecodeToken(token: string, secret: string): DecodedToken {
    const parts = token.split('.');

    if (parts.length !== 3 || !parts[0] || !parts[1] || !parts[2]) {
      throw new AuthError('Invalid token format', 'INVALID_TOKEN_FORMAT', 401);
    }

    const encodedHeader = parts[0];
    const encodedPayload = parts[1];
    const signature = parts[2];

    // Verify signature
    const expectedSignature = this.createSignature(
      `${encodedHeader}.${encodedPayload}`,
      secret
    );

    if (!this.timingSafeEqual(signature, expectedSignature)) {
      throw new AuthError('Invalid token signature', 'INVALID_SIGNATURE', 401);
    }

    // Decode and validate payload
    try {
      const payload: DecodedToken = JSON.parse(this.base64UrlDecode(encodedPayload));

      // Check expiration
      const now = Math.floor(Date.now() / 1000);
      if (payload.exp && payload.exp < now) {
        throw new AuthError('Token has expired', 'TOKEN_EXPIRED', 401);
      }

      // Check issued at time (prevent tokens from the future)
      if (payload.iat && payload.iat > now + 60) {
        throw new AuthError('Token issued in the future', 'INVALID_IAT', 401);
      }

      return payload;
    } catch (error) {
      if (error instanceof AuthError) {
        throw error;
      }
      throw new AuthError('Invalid token payload', 'INVALID_PAYLOAD', 401);
    }
  }

  /**
   * Create HMAC-SHA256 signature
   */
  private createSignature(data: string, secret: string): string {
    const hmac = crypto.createHmac('sha256', secret);
    hmac.update(data);
    return this.base64UrlEncode(hmac.digest('base64'));
  }

  /**
   * Base64 URL encode
   */
  private base64UrlEncode(str: string): string {
    const base64 = Buffer.from(str).toString('base64');
    return base64
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  /**
   * Base64 URL decode
   */
  private base64UrlDecode(str: string): string {
    let base64 = str.replace(/-/g, '+').replace(/_/g, '/');

    // Add padding if necessary
    while (base64.length % 4) {
      base64 += '=';
    }

    return Buffer.from(base64, 'base64').toString('utf-8');
  }

  /**
   * Timing-safe string comparison
   */
  private timingSafeEqual(a: string, b: string): boolean {
    if (a.length !== b.length) {
      return false;
    }

    const bufA = Buffer.from(a);
    const bufB = Buffer.from(b);

    return crypto.timingSafeEqual(bufA, bufB);
  }

  /**
   * Parse expiry string to seconds
   * Supports: '15m', '1h', '7d', '30d'
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
      throw new Error(`Invalid expiry format: ${expiry}`);
    }

    const value = match[1];
    const unit = match[2];
    return parseInt(value, 10) * (units[unit as keyof typeof units] ?? 1);
  }

  /**
   * Generate a secure random token ID
   */
  generateTokenId(): string {
    return crypto.randomBytes(16).toString('hex');
  }

  /**
   * Revoke a token (implementation requires token blacklist)
   */
  async revokeToken(token: string): Promise<void> {
    // In production, add token to blacklist/revocation list
    // This would typically be stored in Redis or database
    const decoded = this.decodeToken(token);
    if (!decoded) {
      throw new AuthError('Cannot revoke invalid token', 'INVALID_TOKEN', 400);
    }

    // Token blacklist implementation would go here
    // For now, this is a placeholder
  }

  /**
   * Check if token is revoked (implementation requires token blacklist)
   */
  async isTokenRevoked(token: string): Promise<boolean> {
    // In production, check against blacklist/revocation list
    // This would typically be stored in Redis or database
    return false;
  }
}

/**
 * Default JWT configuration
 */
export const defaultJwtConfig = {
  jwtSecret: process.env.JWT_SECRET || 'your-secret-key-change-in-production',
  jwtRefreshSecret: process.env.JWT_REFRESH_SECRET || 'your-refresh-secret-change-in-production',
  accessTokenExpiry: '15m',
  refreshTokenExpiry: '7d'
};
