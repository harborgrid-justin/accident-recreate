/**
 * Session Manager - User Session Management
 * AccuScene Enterprise Accident Recreation Platform
 */

import * as crypto from 'crypto';
import { Session, AuthError, AuthConfig } from './types';

export class SessionManager {
  private sessions: Map<string, Session>;
  private userSessions: Map<string, Set<string>>;
  private config: AuthConfig;
  private cleanupInterval: NodeJS.Timeout | null;

  constructor(config: AuthConfig) {
    this.config = config;
    this.sessions = new Map();
    this.userSessions = new Map();
    this.cleanupInterval = null;
    this.startCleanupSchedule();
  }

  /**
   * Create a new session for a user
   */
  async createSession(
    userId: string,
    refreshToken: string,
    metadata?: {
      userAgent?: string;
      ipAddress?: string;
    }
  ): Promise<Session> {
    // Check if user has reached max sessions limit
    const existingSessions = this.getUserSessions(userId);
    if (existingSessions.length >= this.config.maxSessionsPerUser) {
      // Remove oldest session
      const oldestSession = existingSessions.sort(
        (a, b) => a.createdAt.getTime() - b.createdAt.getTime()
      )[0];
      if (oldestSession) {
        await this.destroySession(oldestSession.id);
      }
    }

    const session: Session = {
      id: this.generateSessionId(),
      userId,
      refreshToken,
      userAgent: metadata?.userAgent,
      ipAddress: metadata?.ipAddress,
      createdAt: new Date(),
      expiresAt: new Date(Date.now() + this.config.sessionTimeout),
      lastActivity: new Date()
    };

    // Store session
    this.sessions.set(session.id, session);

    // Track user sessions
    if (!this.userSessions.has(userId)) {
      this.userSessions.set(userId, new Set());
    }
    this.userSessions.get(userId)!.add(session.id);

    return session;
  }

  /**
   * Validate a session by ID
   */
  async validateSession(sessionId: string): Promise<Session> {
    const session = this.sessions.get(sessionId);

    if (!session) {
      throw new AuthError('Session not found', 'SESSION_NOT_FOUND', 401);
    }

    // Check if session is expired
    if (session.expiresAt < new Date()) {
      await this.destroySession(sessionId);
      throw new AuthError('Session has expired', 'SESSION_EXPIRED', 401);
    }

    // Update last activity
    session.lastActivity = new Date();
    this.sessions.set(sessionId, session);

    return session;
  }

  /**
   * Validate a session by refresh token
   */
  async validateSessionByToken(refreshToken: string): Promise<Session> {
    for (const session of this.sessions.values()) {
      if (session.refreshToken === refreshToken) {
        return this.validateSession(session.id);
      }
    }

    throw new AuthError('Session not found for token', 'SESSION_NOT_FOUND', 401);
  }

  /**
   * Destroy a session by ID
   */
  async destroySession(sessionId: string): Promise<void> {
    const session = this.sessions.get(sessionId);

    if (!session) {
      return;
    }

    // Remove from sessions map
    this.sessions.delete(sessionId);

    // Remove from user sessions tracking
    const userSessionIds = this.userSessions.get(session.userId);
    if (userSessionIds) {
      userSessionIds.delete(sessionId);
      if (userSessionIds.size === 0) {
        this.userSessions.delete(session.userId);
      }
    }
  }

  /**
   * Destroy all sessions for a user
   */
  async destroyUserSessions(userId: string): Promise<void> {
    const sessionIds = this.userSessions.get(userId);

    if (!sessionIds) {
      return;
    }

    // Destroy each session
    for (const sessionId of Array.from(sessionIds)) {
      await this.destroySession(sessionId);
    }
  }

  /**
   * Get all active sessions for a user
   */
  getUserSessions(userId: string): Session[] {
    const sessionIds = this.userSessions.get(userId);

    if (!sessionIds) {
      return [];
    }

    return Array.from(sessionIds)
      .map(id => this.sessions.get(id))
      .filter((session): session is Session => session !== undefined);
  }

  /**
   * Extend session expiration (refresh activity)
   */
  async extendSession(sessionId: string): Promise<Session> {
    const session = await this.validateSession(sessionId);

    session.expiresAt = new Date(Date.now() + this.config.sessionTimeout);
    session.lastActivity = new Date();

    this.sessions.set(sessionId, session);

    return session;
  }

  /**
   * Update session metadata
   */
  async updateSessionMetadata(
    sessionId: string,
    metadata: {
      userAgent?: string;
      ipAddress?: string;
    }
  ): Promise<Session> {
    const session = await this.validateSession(sessionId);

    if (metadata.userAgent !== undefined) {
      session.userAgent = metadata.userAgent;
    }

    if (metadata.ipAddress !== undefined) {
      session.ipAddress = metadata.ipAddress;
    }

    session.lastActivity = new Date();
    this.sessions.set(sessionId, session);

    return session;
  }

  /**
   * Get session by ID (without validation)
   */
  getSession(sessionId: string): Session | undefined {
    return this.sessions.get(sessionId);
  }

  /**
   * Get total active sessions count
   */
  getTotalSessionsCount(): number {
    return this.sessions.size;
  }

  /**
   * Get user session count
   */
  getUserSessionCount(userId: string): number {
    return this.userSessions.get(userId)?.size || 0;
  }

  /**
   * Clean up expired sessions
   */
  async cleanupExpiredSessions(): Promise<number> {
    const now = new Date();
    let cleanedCount = 0;

    for (const [sessionId, session] of this.sessions.entries()) {
      if (session.expiresAt < now) {
        await this.destroySession(sessionId);
        cleanedCount++;
      }
    }

    return cleanedCount;
  }

  /**
   * Get session statistics
   */
  getSessionStats(): {
    totalSessions: number;
    totalUsers: number;
    expiredSessions: number;
    activeSessions: number;
  } {
    const now = new Date();
    let expiredSessions = 0;
    let activeSessions = 0;

    for (const session of this.sessions.values()) {
      if (session.expiresAt < now) {
        expiredSessions++;
      } else {
        activeSessions++;
      }
    }

    return {
      totalSessions: this.sessions.size,
      totalUsers: this.userSessions.size,
      expiredSessions,
      activeSessions
    };
  }

  /**
   * Check if session exists for a user with specific IP
   */
  hasSessionFromIP(userId: string, ipAddress: string): boolean {
    const sessions = this.getUserSessions(userId);
    return sessions.some(session => session.ipAddress === ipAddress);
  }

  /**
   * Get sessions from suspicious activity
   */
  getSuspiciousSessions(userId: string): Session[] {
    const sessions = this.getUserSessions(userId);
    const ipAddresses = sessions.map(s => s.ipAddress).filter(Boolean);
    const uniqueIPs = new Set(ipAddresses);

    // Flag if user has sessions from more than 3 different IPs
    if (uniqueIPs.size > 3) {
      return sessions;
    }

    return [];
  }

  /**
   * Generate a unique session ID
   */
  private generateSessionId(): string {
    return crypto.randomBytes(32).toString('hex');
  }

  /**
   * Start automatic cleanup of expired sessions
   */
  private startCleanupSchedule(): void {
    // Run cleanup every 15 minutes
    this.cleanupInterval = setInterval(async () => {
      await this.cleanupExpiredSessions();
    }, 15 * 60 * 1000);
  }

  /**
   * Stop automatic cleanup
   */
  stopCleanupSchedule(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
      this.cleanupInterval = null;
    }
  }

  /**
   * Destroy all sessions (use for testing or maintenance)
   */
  async destroyAllSessions(): Promise<void> {
    this.sessions.clear();
    this.userSessions.clear();
  }

  /**
   * Export session data (for persistence)
   */
  exportSessions(): Session[] {
    return Array.from(this.sessions.values());
  }

  /**
   * Import session data (for restoration)
   */
  importSessions(sessions: Session[]): void {
    for (const session of sessions) {
      this.sessions.set(session.id, session);

      if (!this.userSessions.has(session.userId)) {
        this.userSessions.set(session.userId, new Set());
      }
      this.userSessions.get(session.userId)!.add(session.id);
    }
  }

  /**
   * Cleanup on shutdown
   */
  async shutdown(): Promise<void> {
    this.stopCleanupSchedule();
    await this.cleanupExpiredSessions();
  }
}
