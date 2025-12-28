/**
 * Password Service - Secure Password Operations
 * AccuScene Enterprise Accident Recreation Platform
 */

import * as crypto from 'crypto';
import { PasswordStrengthResult, ValidationError, AuthConfig } from './types';

export class PasswordService {
  private config: AuthConfig;

  constructor(config: AuthConfig) {
    this.config = config;
  }

  /**
   * Hash a password using bcrypt-compatible PBKDF2
   * Note: Using Node.js crypto module for zero external dependencies
   */
  async hash(password: string): Promise<string> {
    return new Promise((resolve, reject) => {
      // Generate a cryptographically secure salt
      const salt = crypto.randomBytes(16).toString('hex');

      // Use PBKDF2 with SHA-512 (secure alternative to bcrypt)
      crypto.pbkdf2(
        password,
        salt,
        this.config.bcryptRounds || 100000,
        64,
        'sha512',
        (err, derivedKey) => {
          if (err) reject(err);
          // Store salt and hash together
          resolve(`${salt}:${derivedKey.toString('hex')}`);
        }
      );
    });
  }

  /**
   * Verify a password against its hash
   */
  async verify(password: string, storedHash: string): Promise<boolean> {
    return new Promise((resolve, reject) => {
      const [salt, hash] = storedHash.split(':');

      if (!salt || !hash) {
        return resolve(false);
      }

      crypto.pbkdf2(
        password,
        salt,
        this.config.bcryptRounds || 100000,
        64,
        'sha512',
        (err, derivedKey) => {
          if (err) reject(err);
          resolve(hash === derivedKey.toString('hex'));
        }
      );
    });
  }

  /**
   * Generate a cryptographically secure random password
   */
  generateSecurePassword(length: number = 16): string {
    const uppercase = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ';
    const lowercase = 'abcdefghijklmnopqrstuvwxyz';
    const numbers = '0123456789';
    const special = '!@#$%^&*()_+-=[]{}|;:,.<>?';

    const allChars = uppercase + lowercase + numbers + special;

    // Ensure at least one of each required character type
    let password = '';
    password += uppercase[crypto.randomInt(0, uppercase.length)];
    password += lowercase[crypto.randomInt(0, lowercase.length)];
    password += numbers[crypto.randomInt(0, numbers.length)];
    password += special[crypto.randomInt(0, special.length)];

    // Fill the rest with random characters
    for (let i = password.length; i < length; i++) {
      password += allChars[crypto.randomInt(0, allChars.length)];
    }

    // Shuffle the password
    return password.split('').sort(() => crypto.randomInt(-1, 2)).join('');
  }

  /**
   * Validate password strength against configured requirements
   */
  validatePasswordStrength(password: string): PasswordStrengthResult {
    const errors: string[] = [];
    const suggestions: string[] = [];
    let score = 0;

    // Check minimum length
    if (password.length < this.config.passwordMinLength) {
      errors.push(`Password must be at least ${this.config.passwordMinLength} characters long`);
    } else {
      score += 20;
    }

    // Check for uppercase letters
    if (this.config.passwordRequireUppercase) {
      if (!/[A-Z]/.test(password)) {
        errors.push('Password must contain at least one uppercase letter');
      } else {
        score += 20;
      }
    }

    // Check for lowercase letters
    if (this.config.passwordRequireLowercase) {
      if (!/[a-z]/.test(password)) {
        errors.push('Password must contain at least one lowercase letter');
      } else {
        score += 20;
      }
    }

    // Check for numbers
    if (this.config.passwordRequireNumbers) {
      if (!/\d/.test(password)) {
        errors.push('Password must contain at least one number');
      } else {
        score += 20;
      }
    }

    // Check for special characters
    if (this.config.passwordRequireSpecialChars) {
      if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(password)) {
        errors.push('Password must contain at least one special character');
      } else {
        score += 20;
      }
    }

    // Additional strength checks
    if (password.length >= 12) score += 10;
    if (password.length >= 16) score += 10;

    // Check for common patterns
    if (/^(.)\1+$/.test(password)) {
      errors.push('Password cannot consist of repeated characters');
      score = 0;
    }

    if (/^(012|123|234|345|456|567|678|789|890)/.test(password)) {
      suggestions.push('Avoid sequential patterns');
      score -= 10;
    }

    // Common passwords check (basic)
    const commonPasswords = ['password', 'Password123', '12345678', 'qwerty', 'admin'];
    if (commonPasswords.some(common => password.toLowerCase().includes(common.toLowerCase()))) {
      errors.push('Password is too common or predictable');
      score = 0;
    }

    // Generate suggestions
    if (password.length < 12) {
      suggestions.push('Consider using a longer password (12+ characters)');
    }

    if (!/[!@#$%^&*()_+\-=\[\]{}|;:,.<>?]/.test(password) && !this.config.passwordRequireSpecialChars) {
      suggestions.push('Adding special characters increases security');
    }

    const isValid = errors.length === 0;
    score = Math.max(0, Math.min(100, score));

    return {
      isValid,
      score,
      errors,
      suggestions
    };
  }

  /**
   * Generate a secure password reset token
   */
  generateResetToken(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Hash a reset token for secure storage
   */
  async hashResetToken(token: string): Promise<string> {
    return crypto.createHash('sha256').update(token).digest('hex');
  }

  /**
   * Verify a reset token against its hash
   */
  async verifyResetToken(token: string, hashedToken: string): Promise<boolean> {
    const tokenHash = crypto.createHash('sha256').update(token).digest('hex');
    return crypto.timingSafeEqual(
      Buffer.from(tokenHash),
      Buffer.from(hashedToken)
    );
  }

  /**
   * Check if password has been compromised (basic implementation)
   * In production, integrate with Have I Been Pwned API
   */
  async isPasswordCompromised(password: string): Promise<boolean> {
    // Basic implementation - check against known compromised patterns
    const weakPatterns = [
      /password/i,
      /123456/,
      /qwerty/i,
      /admin/i,
      /letmein/i,
      /welcome/i,
      /monkey/i,
      /dragon/i
    ];

    return weakPatterns.some(pattern => pattern.test(password));
  }

  /**
   * Calculate entropy of a password
   */
  calculateEntropy(password: string): number {
    let charset = 0;

    if (/[a-z]/.test(password)) charset += 26;
    if (/[A-Z]/.test(password)) charset += 26;
    if (/\d/.test(password)) charset += 10;
    if (/[^a-zA-Z0-9]/.test(password)) charset += 32;

    return Math.log2(Math.pow(charset, password.length));
  }
}

/**
 * Default password configuration
 */
export const defaultPasswordConfig = {
  passwordMinLength: 8,
  passwordRequireUppercase: true,
  passwordRequireLowercase: true,
  passwordRequireNumbers: true,
  passwordRequireSpecialChars: true,
  bcryptRounds: 100000
};
