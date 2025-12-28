/**
 * AccuScene Enterprise v0.2.0 - Capability System
 * Capability checking for plugins
 */

import { PluginCapability, PluginId } from '../types';

export class CapabilityManager {
  private capabilities = new Map<PluginId, Set<PluginCapability>>();
  private systemCapabilities = new Set<PluginCapability>();

  constructor() {
    this.detectSystemCapabilities();
  }

  /**
   * Register capabilities for a plugin
   */
  register(pluginId: PluginId, capabilities: PluginCapability[]): void {
    if (!this.capabilities.has(pluginId)) {
      this.capabilities.set(pluginId, new Set());
    }

    for (const capability of capabilities) {
      // Check if system supports this capability
      if (!this.isSystemCapabilityAvailable(capability)) {
        console.warn(
          `Plugin ${pluginId} requires capability ${capability} which is not available on this system`
        );
        continue;
      }

      this.capabilities.get(pluginId)!.add(capability);
    }
  }

  /**
   * Unregister capabilities for a plugin
   */
  unregister(pluginId: PluginId): void {
    this.capabilities.delete(pluginId);
  }

  /**
   * Check if a plugin has a capability
   */
  has(pluginId: PluginId, capability: PluginCapability): boolean {
    return this.capabilities.get(pluginId)?.has(capability) ?? false;
  }

  /**
   * Get all capabilities for a plugin
   */
  get(pluginId: PluginId): PluginCapability[] {
    return Array.from(this.capabilities.get(pluginId) || []);
  }

  /**
   * Check if a plugin has all required capabilities
   */
  hasAll(pluginId: PluginId, capabilities: PluginCapability[]): boolean {
    const pluginCaps = this.capabilities.get(pluginId);
    if (!pluginCaps) return false;

    return capabilities.every(cap => pluginCaps.has(cap));
  }

  /**
   * Check if system supports a capability
   */
  isSystemCapabilityAvailable(capability: PluginCapability): boolean {
    return this.systemCapabilities.has(capability);
  }

  /**
   * Get all system capabilities
   */
  getSystemCapabilities(): PluginCapability[] {
    return Array.from(this.systemCapabilities);
  }

  /**
   * Detect what capabilities the system supports
   */
  private detectSystemCapabilities(): void {
    // Hot reload - always available in development
    if (process.env.NODE_ENV === 'development') {
      this.systemCapabilities.add(PluginCapability.HOT_RELOAD);
    }

    // Background tasks - check for async/Promise support
    if (typeof Promise !== 'undefined') {
      this.systemCapabilities.add(PluginCapability.BACKGROUND_TASK);
    }

    // Worker threads - check for Worker API
    if (typeof Worker !== 'undefined') {
      this.systemCapabilities.add(PluginCapability.WEB_WORKER);
    }

    // WebGL support
    const canvas = document.createElement('canvas');
    if (canvas.getContext('webgl') || canvas.getContext('webgl2')) {
      this.systemCapabilities.add(PluginCapability.WEBGL);
    }

    // WebGPU support
    if ('gpu' in navigator) {
      this.systemCapabilities.add(PluginCapability.WEBGPU);
    }

    // Native modules - only in Node.js environment
    if (typeof process !== 'undefined' && process.versions?.node) {
      this.systemCapabilities.add(PluginCapability.NATIVE_MODULE);
      this.systemCapabilities.add(PluginCapability.WORKER_THREAD);
    }
  }

  /**
   * Validate that a plugin can use its requested capabilities
   */
  validate(pluginId: PluginId, requestedCapabilities: PluginCapability[]): {
    valid: boolean;
    missing: PluginCapability[];
  } {
    const missing = requestedCapabilities.filter(
      cap => !this.isSystemCapabilityAvailable(cap)
    );

    return {
      valid: missing.length === 0,
      missing,
    };
  }
}

export const createCapabilityManager = (): CapabilityManager => {
  return new CapabilityManager();
};
