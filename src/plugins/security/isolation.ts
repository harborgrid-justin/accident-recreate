/**
 * AccuScene Enterprise v0.2.0 - Isolation Policies
 * Security isolation for plugins
 */

import { PluginId, SecurityPolicy, PluginPermission, PluginCapability } from '../types';

export enum IsolationLevel {
  NONE = 'none',
  PARTIAL = 'partial',
  FULL = 'full',
}

export interface IsolationConfig {
  level: IsolationLevel;
  allowedGlobals?: string[];
  allowedModules?: string[];
  memoryLimitMb?: number;
  cpuQuotaMs?: number;
}

export class IsolationPolicyManager {
  private policies = new Map<PluginId, SecurityPolicy>();
  private isolationConfigs = new Map<PluginId, IsolationConfig>();

  /**
   * Set security policy for a plugin
   */
  setPolicy(pluginId: PluginId, policy: SecurityPolicy): void {
    this.policies.set(pluginId, policy);

    // Determine isolation config based on policy
    const config = this.determineIsolationConfig(policy);
    this.isolationConfigs.set(pluginId, config);
  }

  /**
   * Get security policy for a plugin
   */
  getPolicy(pluginId: PluginId): SecurityPolicy | undefined {
    return this.policies.get(pluginId);
  }

  /**
   * Get isolation config for a plugin
   */
  getIsolationConfig(pluginId: PluginId): IsolationConfig | undefined {
    return this.isolationConfigs.get(pluginId);
  }

  /**
   * Remove policy for a plugin
   */
  removePolicy(pluginId: PluginId): void {
    this.policies.delete(pluginId);
    this.isolationConfigs.delete(pluginId);
  }

  /**
   * Create a default security policy
   */
  createDefaultPolicy(): SecurityPolicy {
    return {
      permissions: [
        PluginPermission.READ_STORAGE,
        PluginPermission.NOTIFICATIONS,
      ],
      capabilities: [
        PluginCapability.BACKGROUND_TASK,
      ],
      isolated: true,
      sandbox: true,
    };
  }

  /**
   * Create a restrictive security policy
   */
  createRestrictivePolicy(): SecurityPolicy {
    return {
      permissions: [],
      capabilities: [],
      isolated: true,
      sandbox: true,
    };
  }

  /**
   * Create a permissive security policy (for trusted plugins)
   */
  createPermissivePolicy(): SecurityPolicy {
    return {
      permissions: Object.values(PluginPermission),
      capabilities: Object.values(PluginCapability),
      isolated: false,
      sandbox: false,
    };
  }

  /**
   * Determine isolation config based on security policy
   */
  private determineIsolationConfig(policy: SecurityPolicy): IsolationConfig {
    if (!policy.isolated) {
      return {
        level: IsolationLevel.NONE,
      };
    }

    const hasDangerousPermissions = [
      PluginPermission.WRITE_FILE,
      PluginPermission.EXECUTE_COMMAND,
      PluginPermission.NETWORK,
    ].some(p => policy.permissions.includes(p));

    const hasResourceIntensiveCapabilities = [
      PluginCapability.NATIVE_MODULE,
      PluginCapability.WORKER_THREAD,
      PluginCapability.WEB_WORKER,
    ].some(c => policy.capabilities.includes(c));

    if (hasDangerousPermissions || hasResourceIntensiveCapabilities) {
      return {
        level: IsolationLevel.FULL,
        allowedGlobals: ['console', 'Promise', 'Array', 'Object'],
        allowedModules: [],
        memoryLimitMb: 256,
        cpuQuotaMs: 1000,
      };
    }

    return {
      level: IsolationLevel.PARTIAL,
      allowedGlobals: [
        'console',
        'Promise',
        'Array',
        'Object',
        'String',
        'Number',
        'Boolean',
        'Math',
        'Date',
        'JSON',
      ],
      memoryLimitMb: 128,
    };
  }

  /**
   * Validate if a plugin's actions comply with its policy
   */
  validateAction(
    pluginId: PluginId,
    action: {
      type: 'permission' | 'capability' | 'resource';
      value: string | number;
    }
  ): { allowed: boolean; reason?: string } {
    const policy = this.policies.get(pluginId);

    if (!policy) {
      return {
        allowed: false,
        reason: 'No security policy found for plugin',
      };
    }

    switch (action.type) {
      case 'permission':
        const hasPermission = policy.permissions.includes(action.value as PluginPermission);
        return {
          allowed: hasPermission,
          reason: hasPermission ? undefined : `Permission ${action.value} not granted`,
        };

      case 'capability':
        const hasCapability = policy.capabilities.includes(action.value as PluginCapability);
        return {
          allowed: hasCapability,
          reason: hasCapability ? undefined : `Capability ${action.value} not available`,
        };

      case 'resource':
        const config = this.isolationConfigs.get(pluginId);
        if (!config) {
          return { allowed: true };
        }

        // In a real implementation, track resource usage
        return { allowed: true };

      default:
        return { allowed: false, reason: 'Unknown action type' };
    }
  }

  /**
   * Check if plugin is properly isolated
   */
  isIsolated(pluginId: PluginId): boolean {
    const policy = this.policies.get(pluginId);
    return policy?.isolated ?? false;
  }

  /**
   * Check if plugin is sandboxed
   */
  isSandboxed(pluginId: PluginId): boolean {
    const policy = this.policies.get(pluginId);
    return policy?.sandbox ?? false;
  }
}

export const createIsolationPolicyManager = (): IsolationPolicyManager => {
  return new IsolationPolicyManager();
};
