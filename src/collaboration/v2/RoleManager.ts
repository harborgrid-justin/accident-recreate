/**
 * AccuScene Enterprise v0.3.0 - Role Manager
 *
 * Role-based access control (Viewer, Editor, Admin, Owner)
 */

import { EventEmitter } from 'events';
import { UserRole, UserId, User } from './types';

export class RoleManager extends EventEmitter {
  private userRoles: Map<UserId, UserRole> = new Map();
  private roleHierarchy: Map<UserRole, number> = new Map([
    [UserRole.VIEWER, 1],
    [UserRole.EDITOR, 2],
    [UserRole.ADMIN, 3],
    [UserRole.OWNER, 4]
  ]);

  assignRole(userId: UserId, role: UserRole): void {
    const previousRole = this.userRoles.get(userId);
    this.userRoles.set(userId, role);
    this.emit('roleAssigned', { userId, role, previousRole });
  }

  getRole(userId: UserId): UserRole {
    return this.userRoles.get(userId) || UserRole.VIEWER;
  }

  hasRole(userId: UserId, role: UserRole): boolean {
    return this.getRole(userId) === role;
  }

  hasMinimumRole(userId: UserId, minimumRole: UserRole): boolean {
    const userLevel = this.roleHierarchy.get(this.getRole(userId)) || 0;
    const requiredLevel = this.roleHierarchy.get(minimumRole) || 0;
    return userLevel >= requiredLevel;
  }

  canPromote(promoterId: UserId, targetUserId: UserId, newRole: UserRole): boolean {
    const promoterRole = this.getRole(promoterId);
    const targetRole = this.getRole(targetUserId);

    const promoterLevel = this.roleHierarchy.get(promoterRole) || 0;
    const newRoleLevel = this.roleHierarchy.get(newRole) || 0;

    // Can only promote to a role lower than your own
    return promoterLevel > newRoleLevel && promoterLevel > (this.roleHierarchy.get(targetRole) || 0);
  }

  getUsersByRole(role: UserRole): UserId[] {
    return Array.from(this.userRoles.entries())
      .filter(([_, r]) => r === role)
      .map(([userId]) => userId);
  }
}
