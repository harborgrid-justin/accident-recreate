/**
 * AccuScene Enterprise - Room Permissions
 * v0.2.0
 *
 * Permission management for collaboration rooms
 */

import { ClientId, UserId, RoomId, RoomPermission, PermissionError } from '../types';

export interface PermissionRule {
  userId: UserId;
  permissions: RoomPermission[];
}

/**
 * Room Permission Manager
 */
export class PermissionManager {
  private permissions: Map<RoomId, Map<UserId, Set<RoomPermission>>>;
  private defaultPermissions: RoomPermission[];

  constructor(defaultPermissions: RoomPermission[] = [RoomPermission.READ]) {
    this.permissions = new Map();
    this.defaultPermissions = defaultPermissions;
  }

  /**
   * Initialize permissions for a room
   */
  initializeRoom(roomId: RoomId, ownerId: UserId): void {
    if (!this.permissions.has(roomId)) {
      this.permissions.set(roomId, new Map());
    }

    // Owner gets all permissions
    this.setPermissions(roomId, ownerId, [
      RoomPermission.READ,
      RoomPermission.WRITE,
      RoomPermission.ADMIN,
      RoomPermission.OWNER,
    ]);
  }

  /**
   * Set permissions for a user in a room
   */
  setPermissions(roomId: RoomId, userId: UserId, permissions: RoomPermission[]): void {
    if (!this.permissions.has(roomId)) {
      this.permissions.set(roomId, new Map());
    }

    this.permissions.get(roomId)!.set(userId, new Set(permissions));
  }

  /**
   * Add permission for a user
   */
  addPermission(roomId: RoomId, userId: UserId, permission: RoomPermission): void {
    if (!this.permissions.has(roomId)) {
      this.permissions.set(roomId, new Map());
    }

    const roomPerms = this.permissions.get(roomId)!;
    if (!roomPerms.has(userId)) {
      roomPerms.set(userId, new Set(this.defaultPermissions));
    }

    roomPerms.get(userId)!.add(permission);
  }

  /**
   * Remove permission from a user
   */
  removePermission(roomId: RoomId, userId: UserId, permission: RoomPermission): void {
    const roomPerms = this.permissions.get(roomId);
    if (!roomPerms) {
      return;
    }

    const userPerms = roomPerms.get(userId);
    if (userPerms) {
      userPerms.delete(permission);
    }
  }

  /**
   * Check if user has a specific permission
   */
  hasPermission(roomId: RoomId, userId: UserId, permission: RoomPermission): boolean {
    const roomPerms = this.permissions.get(roomId);
    if (!roomPerms) {
      return this.defaultPermissions.includes(permission);
    }

    const userPerms = roomPerms.get(userId);
    if (!userPerms) {
      return this.defaultPermissions.includes(permission);
    }

    // Owner and Admin have all permissions
    if (userPerms.has(RoomPermission.OWNER) || userPerms.has(RoomPermission.ADMIN)) {
      return true;
    }

    return userPerms.has(permission);
  }

  /**
   * Check if user has any of the specified permissions
   */
  hasAnyPermission(
    roomId: RoomId,
    userId: UserId,
    permissions: RoomPermission[]
  ): boolean {
    return permissions.some(perm => this.hasPermission(roomId, userId, perm));
  }

  /**
   * Check if user has all of the specified permissions
   */
  hasAllPermissions(
    roomId: RoomId,
    userId: UserId,
    permissions: RoomPermission[]
  ): boolean {
    return permissions.every(perm => this.hasPermission(roomId, userId, perm));
  }

  /**
   * Get all permissions for a user
   */
  getPermissions(roomId: RoomId, userId: UserId): RoomPermission[] {
    const roomPerms = this.permissions.get(roomId);
    if (!roomPerms) {
      return [...this.defaultPermissions];
    }

    const userPerms = roomPerms.get(userId);
    if (!userPerms) {
      return [...this.defaultPermissions];
    }

    return Array.from(userPerms);
  }

  /**
   * Check if user can read
   */
  canRead(roomId: RoomId, userId: UserId): boolean {
    return this.hasPermission(roomId, userId, RoomPermission.READ);
  }

  /**
   * Check if user can write
   */
  canWrite(roomId: RoomId, userId: UserId): boolean {
    return this.hasPermission(roomId, userId, RoomPermission.WRITE);
  }

  /**
   * Check if user is admin
   */
  isAdmin(roomId: RoomId, userId: UserId): boolean {
    return this.hasPermission(roomId, userId, RoomPermission.ADMIN);
  }

  /**
   * Check if user is owner
   */
  isOwner(roomId: RoomId, userId: UserId): boolean {
    return this.hasPermission(roomId, userId, RoomPermission.OWNER);
  }

  /**
   * Require permission (throws if not authorized)
   */
  requirePermission(
    roomId: RoomId,
    userId: UserId,
    permission: RoomPermission
  ): void {
    if (!this.hasPermission(roomId, userId, permission)) {
      throw new PermissionError(
        `User ${userId} does not have ${permission} permission in room ${roomId}`
      );
    }
  }

  /**
   * Get all users with a specific permission
   */
  getUsersWithPermission(roomId: RoomId, permission: RoomPermission): UserId[] {
    const roomPerms = this.permissions.get(roomId);
    if (!roomPerms) {
      return [];
    }

    const users: UserId[] = [];

    for (const [userId, perms] of roomPerms.entries()) {
      if (
        perms.has(permission) ||
        perms.has(RoomPermission.OWNER) ||
        perms.has(RoomPermission.ADMIN)
      ) {
        users.push(userId);
      }
    }

    return users;
  }

  /**
   * Get room owner
   */
  getOwner(roomId: RoomId): UserId | null {
    const owners = this.getUsersWithPermission(roomId, RoomPermission.OWNER);
    return owners.length > 0 ? owners[0] : null;
  }

  /**
   * Transfer ownership
   */
  transferOwnership(
    roomId: RoomId,
    fromUserId: UserId,
    toUserId: UserId
  ): void {
    // Verify current owner
    if (!this.isOwner(roomId, fromUserId)) {
      throw new PermissionError(`User ${fromUserId} is not the owner`);
    }

    // Remove owner permission from current owner
    this.removePermission(roomId, fromUserId, RoomPermission.OWNER);

    // Make them admin instead
    this.addPermission(roomId, fromUserId, RoomPermission.ADMIN);

    // Give owner permission to new owner
    this.setPermissions(roomId, toUserId, [
      RoomPermission.READ,
      RoomPermission.WRITE,
      RoomPermission.ADMIN,
      RoomPermission.OWNER,
    ]);
  }

  /**
   * Remove all permissions for a user
   */
  revokeAllPermissions(roomId: RoomId, userId: UserId): void {
    const roomPerms = this.permissions.get(roomId);
    if (roomPerms) {
      roomPerms.delete(userId);
    }
  }

  /**
   * Remove all permissions for a room
   */
  removeRoom(roomId: RoomId): void {
    this.permissions.delete(roomId);
  }

  /**
   * Export state for serialization
   */
  toJSON(): Record<string, unknown> {
    return {
      permissions: Array.from(this.permissions.entries()).map(([roomId, userPerms]) => [
        roomId,
        Array.from(userPerms.entries()).map(([userId, perms]) => [
          userId,
          Array.from(perms),
        ]),
      ]),
      defaultPermissions: this.defaultPermissions,
    };
  }

  /**
   * Import state from serialized data
   */
  fromJSON(data: {
    permissions: Array<[RoomId, Array<[UserId, RoomPermission[]]>]>;
    defaultPermissions: RoomPermission[];
  }): void {
    this.permissions = new Map(
      data.permissions.map(([roomId, userPerms]) => [
        roomId,
        new Map(userPerms.map(([userId, perms]) => [userId, new Set(perms)])),
      ])
    );
    this.defaultPermissions = data.defaultPermissions;
  }
}
