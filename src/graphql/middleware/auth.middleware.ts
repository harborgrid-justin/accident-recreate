/**
 * Authentication Middleware
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { GraphQLError } from 'graphql';
import { GraphQLContext, Permission } from '../types';

/**
 * Checks if user is authenticated
 */
export function requireAuth(context: GraphQLContext): void {
  if (!context.user) {
    throw new GraphQLError('Authentication required', {
      extensions: {
        code: 'UNAUTHENTICATED',
        http: { status: 401 },
      },
    });
  }
}

/**
 * Checks if user has specific permission
 */
export function requirePermission(context: GraphQLContext, permission: Permission): void {
  requireAuth(context);

  if (!context.user!.permissions.includes(permission)) {
    throw new GraphQLError('Insufficient permissions', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
        required: permission,
        actual: context.user!.permissions,
      },
    });
  }
}

/**
 * Checks if user has any of the specified permissions
 */
export function requireAnyPermission(
  context: GraphQLContext,
  permissions: Permission[]
): void {
  requireAuth(context);

  const hasPermission = permissions.some((permission) =>
    context.user!.permissions.includes(permission)
  );

  if (!hasPermission) {
    throw new GraphQLError('Insufficient permissions', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
        required: permissions,
        actual: context.user!.permissions,
      },
    });
  }
}

/**
 * Checks if user has all of the specified permissions
 */
export function requireAllPermissions(
  context: GraphQLContext,
  permissions: Permission[]
): void {
  requireAuth(context);

  const hasAllPermissions = permissions.every((permission) =>
    context.user!.permissions.includes(permission)
  );

  if (!hasAllPermissions) {
    throw new GraphQLError('Insufficient permissions', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
        required: permissions,
        actual: context.user!.permissions,
      },
    });
  }
}

/**
 * Checks if user belongs to the specified organization
 */
export function requireOrganization(
  context: GraphQLContext,
  organizationId: string
): void {
  requireAuth(context);

  if (context.user!.organizationId !== organizationId) {
    throw new GraphQLError('Access denied to this organization', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
      },
    });
  }
}

/**
 * Checks if user is admin
 */
export function requireAdmin(context: GraphQLContext): void {
  requireAuth(context);

  if (context.user!.role !== 'ADMIN') {
    throw new GraphQLError('Admin access required', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
      },
    });
  }
}

/**
 * Checks if user owns the resource
 */
export function requireOwnership(
  context: GraphQLContext,
  resourceOwnerId: string
): void {
  requireAuth(context);

  if (context.user!.id !== resourceOwnerId) {
    throw new GraphQLError('Access denied to this resource', {
      extensions: {
        code: 'UNAUTHORIZED',
        http: { status: 403 },
      },
    });
  }
}
