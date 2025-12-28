/**
 * AccuScene Enterprise v0.2.0 - Permission System
 * Permission management for plugins
 */

import {
  PluginPermission,
  PluginId,
  PermissionRequest,
  PermissionResult,
} from '../types';

export class PermissionManager {
  private granted = new Map<PluginId, Set<PluginPermission>>();
  private denied = new Map<PluginId, Set<PluginPermission>>();
  private requestHandlers = new Set<PermissionRequestHandler>();

  /**
   * Request a permission for a plugin
   */
  async request(request: PermissionRequest): Promise<PermissionResult> {
    const { pluginId, permission } = request;

    // Check if already granted
    if (this.isGranted(pluginId, permission)) {
      return { granted: true };
    }

    // Check if already denied
    if (this.isDenied(pluginId, permission)) {
      return { granted: false };
    }

    // Ask permission handlers
    for (const handler of this.requestHandlers) {
      const result = await handler(request);
      if (result !== undefined) {
        if (result.granted) {
          this.grant(pluginId, permission);
        } else {
          this.deny(pluginId, permission);
        }
        return result;
      }
    }

    // Default: deny
    return { granted: false };
  }

  /**
   * Grant a permission to a plugin
   */
  grant(pluginId: PluginId, permission: PluginPermission): void {
    if (!this.granted.has(pluginId)) {
      this.granted.set(pluginId, new Set());
    }

    this.granted.get(pluginId)!.add(permission);
    this.denied.get(pluginId)?.delete(permission);
  }

  /**
   * Deny a permission for a plugin
   */
  deny(pluginId: PluginId, permission: PluginPermission): void {
    if (!this.denied.has(pluginId)) {
      this.denied.set(pluginId, new Set());
    }

    this.denied.get(pluginId)!.add(permission);
    this.granted.get(pluginId)?.delete(permission);
  }

  /**
   * Revoke a permission from a plugin
   */
  revoke(pluginId: PluginId, permission: PluginPermission): void {
    this.granted.get(pluginId)?.delete(permission);
    this.denied.get(pluginId)?.delete(permission);
  }

  /**
   * Check if a plugin has a permission
   */
  isGranted(pluginId: PluginId, permission: PluginPermission): boolean {
    return this.granted.get(pluginId)?.has(permission) ?? false;
  }

  /**
   * Check if a permission is denied for a plugin
   */
  isDenied(pluginId: PluginId, permission: PluginPermission): boolean {
    return this.denied.get(pluginId)?.has(permission) ?? false;
  }

  /**
   * Get all granted permissions for a plugin
   */
  getGranted(pluginId: PluginId): PluginPermission[] {
    return Array.from(this.granted.get(pluginId) || []);
  }

  /**
   * Get all denied permissions for a plugin
   */
  getDenied(pluginId: PluginId): PluginPermission[] {
    return Array.from(this.denied.get(pluginId) || []);
  }

  /**
   * Check if a plugin has all required permissions
   */
  hasAll(pluginId: PluginId, permissions: PluginPermission[]): boolean {
    return permissions.every(p => this.isGranted(pluginId, p));
  }

  /**
   * Check if a plugin has any of the permissions
   */
  hasAny(pluginId: PluginId, permissions: PluginPermission[]): boolean {
    return permissions.some(p => this.isGranted(pluginId, p));
  }

  /**
   * Grant all permissions to a plugin
   */
  grantAll(pluginId: PluginId, permissions: PluginPermission[]): void {
    for (const permission of permissions) {
      this.grant(pluginId, permission);
    }
  }

  /**
   * Revoke all permissions from a plugin
   */
  revokeAll(pluginId: PluginId): void {
    this.granted.delete(pluginId);
    this.denied.delete(pluginId);
  }

  /**
   * Register a permission request handler
   */
  registerRequestHandler(handler: PermissionRequestHandler): () => void {
    this.requestHandlers.add(handler);

    return () => {
      this.requestHandlers.delete(handler);
    };
  }

  /**
   * Clear all permissions
   */
  clear(): void {
    this.granted.clear();
    this.denied.clear();
  }
}

export type PermissionRequestHandler = (
  request: PermissionRequest
) => Promise<PermissionResult | undefined> | PermissionResult | undefined;

export const createPermissionManager = (): PermissionManager => {
  return new PermissionManager();
};
