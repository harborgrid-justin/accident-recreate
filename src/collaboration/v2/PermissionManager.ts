/**
 * AccuScene Enterprise v0.3.0 - Permission Manager
 *
 * Fine-grained permission system for collaboration features
 */

import { EventEmitter } from 'events';
import {
  Permission,
  Resource,
  Action,
  UserId,
  UserRole
} from './types';

export class PermissionManager extends EventEmitter {
  private permissions: Map<string, Permission> = new Map();
  private userRoles: Map<UserId, UserRole> = new Map();

  // Default role permissions
  private rolePermissions: Record<UserRole, Permission[]> = {
    [UserRole.VIEWER]: [
      { resource: Resource.SCENE, action: Action.VIEW, granted: true },
      { resource: Resource.ANNOTATION, action: Action.VIEW, granted: true }
    ],
    [UserRole.EDITOR]: [
      { resource: Resource.SCENE, action: Action.VIEW, granted: true },
      { resource: Resource.SCENE, action: Action.EDIT, granted: true },
      { resource: Resource.OBJECT, action: Action.CREATE, granted: true },
      { resource: Resource.OBJECT, action: Action.EDIT, granted: true },
      { resource: Resource.ANNOTATION, action: Action.CREATE, granted: true },
      { resource: Resource.ANNOTATION, action: Action.EDIT, granted: true },
      { resource: Resource.CHAT, action: Action.CREATE, granted: true }
    ],
    [UserRole.ADMIN]: [
      { resource: Resource.SCENE, action: Action.VIEW, granted: true },
      { resource: Resource.SCENE, action: Action.EDIT, granted: true },
      { resource: Resource.SCENE, action: Action.DELETE, granted: true },
      { resource: Resource.OBJECT, action: Action.CREATE, granted: true },
      { resource: Resource.OBJECT, action: Action.EDIT, granted: true },
      { resource: Resource.OBJECT, action: Action.DELETE, granted: true },
      { resource: Resource.BRANCH, action: Action.CREATE, granted: true },
      { resource: Resource.BRANCH, action: Action.EDIT, granted: true },
      { resource: Resource.BRANCH, action: Action.DELETE, granted: true },
      { resource: Resource.ANNOTATION, action: Action.MANAGE, granted: true },
      { resource: Resource.CHAT, action: Action.MANAGE, granted: true },
      { resource: Resource.SETTINGS, action: Action.EDIT, granted: true }
    ],
    [UserRole.OWNER]: [
      { resource: Resource.SCENE, action: Action.MANAGE, granted: true },
      { resource: Resource.BRANCH, action: Action.MANAGE, granted: true },
      { resource: Resource.OBJECT, action: Action.MANAGE, granted: true },
      { resource: Resource.ANNOTATION, action: Action.MANAGE, granted: true },
      { resource: Resource.CHAT, action: Action.MANAGE, granted: true },
      { resource: Resource.VOICE, action: Action.MANAGE, granted: true },
      { resource: Resource.VIDEO, action: Action.MANAGE, granted: true },
      { resource: Resource.SETTINGS, action: Action.MANAGE, granted: true }
    ]
  };

  setUserRole(userId: UserId, role: UserRole): void {
    this.userRoles.set(userId, role);
    this.emit('roleChanged', { userId, role });
  }

  getUserRole(userId: UserId): UserRole {
    return this.userRoles.get(userId) || UserRole.VIEWER;
  }

  async checkPermission(userId: UserId, resource: Resource, action: Action): Promise<boolean> {
    const role = this.getUserRole(userId);
    const permissions = this.rolePermissions[role];

    return permissions.some(p =>
      p.resource === resource &&
      (p.action === action || p.action === Action.MANAGE) &&
      p.granted
    );
  }

  async requirePermission(userId: UserId, resource: Resource, action: Action): Promise<void> {
    const hasPermission = await this.checkPermission(userId, resource, action);
    if (!hasPermission) {
      throw new Error(`Permission denied: ${action} on ${resource}`);
    }
  }

  grantPermission(userId: UserId, permission: Permission): void {
    const key = `${userId}-${permission.resource}-${permission.action}`;
    this.permissions.set(key, { ...permission, granted: true });
    this.emit('permissionGranted', { userId, permission });
  }

  revokePermission(userId: UserId, resource: Resource, action: Action): void {
    const key = `${userId}-${resource}-${action}`;
    this.permissions.delete(key);
    this.emit('permissionRevoked', { userId, resource, action });
  }
}
