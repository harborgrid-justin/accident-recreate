/**
 * AccuScene Enterprise v0.2.0 - Lifecycle Hooks
 * Hooks for plugin lifecycle events
 */

import {
  PluginId,
  LifecycleHook,
  LifecycleHookType,
  LifecycleHookHandler,
  PluginContext,
} from '../types';

export class PluginLifecycleManager {
  private hooks = new Map<LifecycleHookType, Set<LifecycleHook>>();

  /**
   * Register a lifecycle hook
   */
  registerHook(hook: LifecycleHook): () => void {
    if (!this.hooks.has(hook.type)) {
      this.hooks.set(hook.type, new Set());
    }

    this.hooks.get(hook.type)!.add(hook);

    return () => {
      this.hooks.get(hook.type)?.delete(hook);
    };
  }

  /**
   * Unregister a lifecycle hook
   */
  unregisterHook(hook: LifecycleHook): void {
    this.hooks.get(hook.type)?.delete(hook);
  }

  /**
   * Run pre-load hooks
   */
  async runPreLoad(pluginId: PluginId): Promise<void> {
    await this.runHooks(LifecycleHookType.PRE_LOAD, pluginId);
  }

  /**
   * Run post-load hooks
   */
  async runPostLoad(pluginId: PluginId): Promise<void> {
    await this.runHooks(LifecycleHookType.POST_LOAD, pluginId);
  }

  /**
   * Run pre-activate hooks
   */
  async runPreActivate(pluginId: PluginId, context?: PluginContext): Promise<void> {
    await this.runHooks(LifecycleHookType.PRE_ACTIVATE, pluginId, context);
  }

  /**
   * Run post-activate hooks
   */
  async runPostActivate(pluginId: PluginId, context?: PluginContext): Promise<void> {
    await this.runHooks(LifecycleHookType.POST_ACTIVATE, pluginId, context);
  }

  /**
   * Run pre-deactivate hooks
   */
  async runPreDeactivate(pluginId: PluginId, context?: PluginContext): Promise<void> {
    await this.runHooks(LifecycleHookType.PRE_DEACTIVATE, pluginId, context);
  }

  /**
   * Run post-deactivate hooks
   */
  async runPostDeactivate(pluginId: PluginId, context?: PluginContext): Promise<void> {
    await this.runHooks(LifecycleHookType.POST_DEACTIVATE, pluginId, context);
  }

  /**
   * Run pre-unload hooks
   */
  async runPreUnload(pluginId: PluginId): Promise<void> {
    await this.runHooks(LifecycleHookType.PRE_UNLOAD, pluginId);
  }

  /**
   * Run post-unload hooks
   */
  async runPostUnload(pluginId: PluginId): Promise<void> {
    await this.runHooks(LifecycleHookType.POST_UNLOAD, pluginId);
  }

  /**
   * Run all hooks of a specific type
   */
  private async runHooks(
    type: LifecycleHookType,
    pluginId: PluginId,
    context?: PluginContext
  ): Promise<void> {
    const hooks = this.hooks.get(type);

    if (!hooks || hooks.size === 0) {
      return;
    }

    // Sort hooks by priority (higher priority runs first)
    const sortedHooks = Array.from(hooks).sort(
      (a, b) => (b.priority || 0) - (a.priority || 0)
    );

    // Run hooks sequentially
    for (const hook of sortedHooks) {
      try {
        await hook.handler(pluginId, context);
      } catch (error) {
        console.error(`Error in ${type} hook for plugin ${pluginId}:`, error);
        throw error;
      }
    }
  }

  /**
   * Clear all hooks
   */
  clear(): void {
    this.hooks.clear();
  }

  /**
   * Get all hooks of a specific type
   */
  getHooks(type: LifecycleHookType): LifecycleHook[] {
    return Array.from(this.hooks.get(type) || []);
  }
}

export const createLifecycleManager = (): PluginLifecycleManager => {
  return new PluginLifecycleManager();
};
