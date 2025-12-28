/**
 * AccuScene Enterprise v0.2.0 - Plugin Sandbox
 * Security sandbox for isolating plugin execution
 */

import {
  PluginId,
  PluginPermission,
  PluginCapability,
  SecurityPolicy,
  PluginContext,
} from '../types';

export interface SandboxOptions {
  isolated?: boolean;
  timeoutMs?: number;
  memoryLimitMb?: number;
  allowedGlobals?: string[];
}

export class PluginSandbox {
  private policy: SecurityPolicy;
  private options: Required<SandboxOptions>;
  private isolatedContext?: any;

  constructor(
    private pluginId: PluginId,
    policy: SecurityPolicy,
    options: SandboxOptions = {}
  ) {
    this.policy = policy;
    this.options = {
      isolated: options.isolated ?? policy.isolated,
      timeoutMs: options.timeoutMs ?? 30000,
      memoryLimitMb: options.memoryLimitMb ?? 128,
      allowedGlobals: options.allowedGlobals ?? [],
    };

    if (this.options.isolated) {
      this.createIsolatedContext();
    }
  }

  /**
   * Execute code in the sandbox
   */
  async execute<T = any>(code: string | Function, context?: any): Promise<T> {
    // Check if execution is allowed
    this.checkCapability(PluginCapability.BACKGROUND_TASK);

    try {
      if (this.options.isolated) {
        return await this.executeIsolated<T>(code, context);
      } else {
        return await this.executeNormal<T>(code, context);
      }
    } catch (error) {
      throw new Error(`Sandbox execution failed for plugin ${this.pluginId}: ${error}`);
    }
  }

  /**
   * Check if a permission is granted
   */
  hasPermission(permission: PluginPermission): boolean {
    return this.policy.permissions.includes(permission);
  }

  /**
   * Check if a capability is available
   */
  hasCapability(capability: PluginCapability): boolean {
    return this.policy.capabilities.includes(capability);
  }

  /**
   * Check permission and throw if not granted
   */
  checkPermission(permission: PluginPermission): void {
    if (!this.hasPermission(permission)) {
      throw new Error(
        `Permission denied: Plugin ${this.pluginId} does not have ${permission} permission`
      );
    }
  }

  /**
   * Check capability and throw if not available
   */
  checkCapability(capability: PluginCapability): void {
    if (!this.hasCapability(capability)) {
      throw new Error(
        `Capability unavailable: Plugin ${this.pluginId} does not have ${capability} capability`
      );
    }
  }

  /**
   * Create a proxied object with permission checks
   */
  createProxy<T extends object>(target: T, requiredPermission?: PluginPermission): T {
    return new Proxy(target, {
      get: (obj, prop) => {
        // Check permission before accessing
        if (requiredPermission) {
          this.checkPermission(requiredPermission);
        }

        const value = obj[prop as keyof T];

        // If it's a function, wrap it with permission check
        if (typeof value === 'function') {
          return (...args: any[]) => {
            if (requiredPermission) {
              this.checkPermission(requiredPermission);
            }
            return (value as Function).apply(obj, args);
          };
        }

        return value;
      },
      set: (obj, prop, value) => {
        // Check write permission
        if (requiredPermission) {
          this.checkPermission(requiredPermission);
        }

        obj[prop as keyof T] = value;
        return true;
      },
    });
  }

  /**
   * Create a sandboxed context for the plugin
   */
  createSandboxedContext(baseContext: PluginContext): PluginContext {
    // Wrap all context services with permission checks
    return {
      ...baseContext,
      storage: this.createProxy(baseContext.storage, PluginPermission.WRITE_STORAGE),
      commands: this.createProxy(baseContext.commands, PluginPermission.REGISTER_COMMAND),
      ui: this.createProxy(baseContext.ui, PluginPermission.UI_MODIFY),
    };
  }

  /**
   * Validate network access
   */
  validateNetworkAccess(url: string): void {
    this.checkPermission(PluginPermission.NETWORK);

    // Check against trusted origins if specified
    if (this.policy.trustedOrigins && this.policy.trustedOrigins.length > 0) {
      const urlObj = new URL(url);
      const allowed = this.policy.trustedOrigins.some(origin => {
        return urlObj.origin === origin || urlObj.hostname.endsWith(origin);
      });

      if (!allowed) {
        throw new Error(
          `Network access denied: ${url} is not in trusted origins for plugin ${this.pluginId}`
        );
      }
    }
  }

  /**
   * Validate file access
   */
  validateFileAccess(path: string, mode: 'read' | 'write'): void {
    const permission =
      mode === 'read' ? PluginPermission.READ_FILE : PluginPermission.WRITE_FILE;

    this.checkPermission(permission);

    // Additional path validation could be added here
    // e.g., prevent access to sensitive directories
  }

  /**
   * Execute in isolated context (using iframe or worker)
   */
  private async executeIsolated<T>(code: string | Function, context?: any): Promise<T> {
    if (!this.isolatedContext) {
      throw new Error('Isolated context not initialized');
    }

    // In a real implementation, this would use:
    // - iframe with sandbox attribute for browser
    // - vm module for Node.js
    // - Web Workers for heavy computation

    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`Execution timeout after ${this.options.timeoutMs}ms`));
      }, this.options.timeoutMs);

      try {
        const func = typeof code === 'function' ? code : new Function('context', code);
        const result = func(context);

        clearTimeout(timeout);
        resolve(result);
      } catch (error) {
        clearTimeout(timeout);
        reject(error);
      }
    });
  }

  /**
   * Execute in normal context with basic checks
   */
  private async executeNormal<T>(code: string | Function, context?: any): Promise<T> {
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        reject(new Error(`Execution timeout after ${this.options.timeoutMs}ms`));
      }, this.options.timeoutMs);

      try {
        const func = typeof code === 'function' ? code : new Function('context', code);
        const result = func(context);

        clearTimeout(timeout);
        resolve(result);
      } catch (error) {
        clearTimeout(timeout);
        reject(error);
      }
    });
  }

  /**
   * Create an isolated execution context
   */
  private createIsolatedContext(): void {
    // Create a minimal global context with only allowed globals
    const allowedGlobals = new Set([
      'Object',
      'Array',
      'String',
      'Number',
      'Boolean',
      'Date',
      'Math',
      'JSON',
      'Promise',
      'Set',
      'Map',
      'WeakSet',
      'WeakMap',
      ...this.options.allowedGlobals,
    ]);

    // In a real implementation, this would create a proper isolated context
    // using iframe sandbox, vm.createContext, or similar
    this.isolatedContext = {};
  }

  /**
   * Cleanup sandbox resources
   */
  dispose(): void {
    this.isolatedContext = undefined;
  }
}

export const createPluginSandbox = (
  pluginId: PluginId,
  policy: SecurityPolicy,
  options?: SandboxOptions
): PluginSandbox => {
  return new PluginSandbox(pluginId, policy, options);
};
