/**
 * GraphQL Context Factory
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { Request, Response } from 'express';
import { v4 as uuidv4 } from 'uuid';
import jwt from 'jsonwebtoken';
import { GraphQLContext, AuthenticatedUser, UserRole, Permission } from './types';
import { createDataLoaders } from './dataloaders';
import { PubSub } from 'graphql-subscriptions';

// ============================================================================
// PubSub Instance (In production, use Redis or other distributed solution)
// ============================================================================

const pubsub = new PubSub();

// ============================================================================
// JWT Configuration
// ============================================================================

const JWT_SECRET = process.env.JWT_SECRET || 'accuscene-enterprise-secret-key-change-in-production';
const JWT_ALGORITHM = 'HS256';

// ============================================================================
// Context Factory
// ============================================================================

export async function createContext({
  req,
  res,
}: {
  req: Request;
  res: Response;
}): Promise<GraphQLContext> {
  const requestId = uuidv4();
  const startTime = Date.now();

  // Extract user from JWT token
  const user = await extractUserFromRequest(req);

  // Create dataloaders
  const dataloaders = createDataLoaders();

  return {
    req,
    res,
    user,
    dataloaders,
    pubsub,
    requestId,
    startTime,
  };
}

// ============================================================================
// WebSocket Context Factory (for subscriptions)
// ============================================================================

export async function createWebSocketContext(
  connectionParams: any,
  webSocket: any
): Promise<GraphQLContext> {
  const requestId = uuidv4();
  const startTime = Date.now();

  // Extract user from connection params
  const user = await extractUserFromConnectionParams(connectionParams);

  // Create dataloaders
  const dataloaders = createDataLoaders();

  // Create mock request/response for WebSocket context
  const mockReq = {
    headers: connectionParams?.headers || {},
    connection: webSocket,
  } as Request;

  const mockRes = {} as Response;

  return {
    req: mockReq,
    res: mockRes,
    user,
    dataloaders,
    pubsub,
    requestId,
    startTime,
  };
}

// ============================================================================
// User Extraction from HTTP Request
// ============================================================================

async function extractUserFromRequest(req: Request): Promise<AuthenticatedUser | undefined> {
  try {
    // Extract token from Authorization header
    const authHeader = req.headers.authorization;
    if (!authHeader) {
      return undefined;
    }

    // Support both "Bearer token" and "token" formats
    const token = authHeader.startsWith('Bearer ')
      ? authHeader.substring(7)
      : authHeader;

    // Verify and decode JWT
    const decoded = jwt.verify(token, JWT_SECRET, {
      algorithms: [JWT_ALGORITHM],
    }) as any;

    // Extract user information from JWT payload
    const user: AuthenticatedUser = {
      id: decoded.sub || decoded.userId,
      email: decoded.email,
      role: decoded.role as UserRole,
      organizationId: decoded.organizationId,
      permissions: decoded.permissions || getDefaultPermissions(decoded.role),
    };

    return user;
  } catch (error) {
    // Invalid or expired token
    console.error('Failed to extract user from request:', error);
    return undefined;
  }
}

// ============================================================================
// User Extraction from WebSocket Connection Params
// ============================================================================

async function extractUserFromConnectionParams(
  connectionParams: any
): Promise<AuthenticatedUser | undefined> {
  try {
    if (!connectionParams?.authorization && !connectionParams?.Authorization) {
      return undefined;
    }

    const authHeader = connectionParams.authorization || connectionParams.Authorization;
    const token = authHeader.startsWith('Bearer ')
      ? authHeader.substring(7)
      : authHeader;

    const decoded = jwt.verify(token, JWT_SECRET, {
      algorithms: [JWT_ALGORITHM],
    }) as any;

    const user: AuthenticatedUser = {
      id: decoded.sub || decoded.userId,
      email: decoded.email,
      role: decoded.role as UserRole,
      organizationId: decoded.organizationId,
      permissions: decoded.permissions || getDefaultPermissions(decoded.role),
    };

    return user;
  } catch (error) {
    console.error('Failed to extract user from connection params:', error);
    return undefined;
  }
}

// ============================================================================
// Default Permissions Based on Role
// ============================================================================

function getDefaultPermissions(role: UserRole): Permission[] {
  switch (role) {
    case UserRole.ADMIN:
      return [
        Permission.CASE_CREATE,
        Permission.CASE_READ,
        Permission.CASE_UPDATE,
        Permission.CASE_DELETE,
        Permission.SIMULATION_RUN,
        Permission.REPORT_GENERATE,
        Permission.USER_MANAGE,
      ];
    case UserRole.INVESTIGATOR:
      return [
        Permission.CASE_CREATE,
        Permission.CASE_READ,
        Permission.CASE_UPDATE,
        Permission.SIMULATION_RUN,
        Permission.REPORT_GENERATE,
      ];
    case UserRole.ANALYST:
      return [
        Permission.CASE_READ,
        Permission.CASE_UPDATE,
        Permission.SIMULATION_RUN,
        Permission.REPORT_GENERATE,
      ];
    case UserRole.VIEWER:
      return [Permission.CASE_READ];
    default:
      return [];
  }
}

// ============================================================================
// Helper Functions
// ============================================================================

export function createJWT(user: AuthenticatedUser, expiresIn: string = '7d'): string {
  return jwt.sign(
    {
      sub: user.id,
      userId: user.id,
      email: user.email,
      role: user.role,
      organizationId: user.organizationId,
      permissions: user.permissions,
    },
    JWT_SECRET,
    {
      algorithm: JWT_ALGORITHM,
      expiresIn,
    }
  );
}

export function verifyJWT(token: string): AuthenticatedUser {
  const decoded = jwt.verify(token, JWT_SECRET, {
    algorithms: [JWT_ALGORITHM],
  }) as any;

  return {
    id: decoded.sub || decoded.userId,
    email: decoded.email,
    role: decoded.role as UserRole,
    organizationId: decoded.organizationId,
    permissions: decoded.permissions,
  };
}

// ============================================================================
// Export PubSub instance
// ============================================================================

export { pubsub };
