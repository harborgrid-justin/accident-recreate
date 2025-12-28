/**
 * AccuScene Enterprise v0.2.0 - Plugin Registry
 * Central registry for managing plugin instances
 */

import { Plugin, PluginId, PluginState } from '../types';

export interface PluginRegistryEntry {
  plugin: Plugin;
  state: PluginState;
  loadedAt: number;
  activatedAt?: number;
  error?: Error;
}

export class PluginRegistryImpl {
  private plugins = new Map<PluginId, PluginRegistryEntry>();
  private stateListeners = new Map<PluginId, Set<StateChangeListener>>();

  /**
   * Register a plugin in the registry
   */
  register(plugin: Plugin): void {
    const { id } = plugin.manifest;

    if (this.plugins.has(id)) {
      throw new Error(`Plugin ${id} is already registered`);
    }

    this.plugins.set(id, {
      plugin,
      state: PluginState.LOADED,
      loadedAt: Date.now(),
    });

    this.notifyStateChange(id, PluginState.LOADED);
  }

  /**
   * Unregister a plugin from the registry
   */
  unregister(pluginId: PluginId): void {
    const entry = this.plugins.get(pluginId);

    if (!entry) {
      throw new Error(`Plugin ${pluginId} is not registered`);
    }

    this.plugins.delete(pluginId);
    this.stateListeners.delete(pluginId);
    this.notifyStateChange(pluginId, PluginState.UNLOADED);
  }

  /**
   * Get a plugin by ID
   */
  get(pluginId: PluginId): Plugin | undefined {
    return this.plugins.get(pluginId)?.plugin;
  }

  /**
   * Get plugin entry with metadata
   */
  getEntry(pluginId: PluginId): PluginRegistryEntry | undefined {
    return this.plugins.get(pluginId);
  }

  /**
   * Get all registered plugins
   */
  getAll(): Plugin[] {
    return Array.from(this.plugins.values()).map(entry => entry.plugin);
  }

  /**
   * Get all plugin entries
   */
  getAllEntries(): PluginRegistryEntry[] {
    return Array.from(this.plugins.values());
  }

  /**
   * Check if a plugin is registered
   */
  has(pluginId: PluginId): boolean {
    return this.plugins.has(pluginId);
  }

  /**
   * Get the current state of a plugin
   */
  getState(pluginId: PluginId): PluginState | undefined {
    return this.plugins.get(pluginId)?.state;
  }

  /**
   * Update the state of a plugin
   */
  setState(pluginId: PluginId, state: PluginState, error?: Error): void {
    const entry = this.plugins.get(pluginId);

    if (!entry) {
      throw new Error(`Plugin ${pluginId} is not registered`);
    }

    entry.state = state;
    entry.error = error;

    if (state === PluginState.ACTIVE && !entry.activatedAt) {
      entry.activatedAt = Date.now();
    }

    this.notifyStateChange(pluginId, state);
  }

  /**
   * Get plugins by state
   */
  getByState(state: PluginState): Plugin[] {
    return Array.from(this.plugins.values())
      .filter(entry => entry.state === state)
      .map(entry => entry.plugin);
  }

  /**
   * Listen to state changes for a plugin
   */
  onStateChange(pluginId: PluginId, listener: StateChangeListener): () => void {
    if (!this.stateListeners.has(pluginId)) {
      this.stateListeners.set(pluginId, new Set());
    }

    this.stateListeners.get(pluginId)!.add(listener);

    return () => {
      this.stateListeners.get(pluginId)?.delete(listener);
    };
  }

  /**
   * Clear all plugins
   */
  clear(): void {
    this.plugins.clear();
    this.stateListeners.clear();
  }

  /**
   * Get statistics about registered plugins
   */
  getStats(): PluginRegistryStats {
    const entries = Array.from(this.plugins.values());

    return {
      total: entries.length,
      byState: {
        [PluginState.UNLOADED]: entries.filter(e => e.state === PluginState.UNLOADED).length,
        [PluginState.LOADING]: entries.filter(e => e.state === PluginState.LOADING).length,
        [PluginState.LOADED]: entries.filter(e => e.state === PluginState.LOADED).length,
        [PluginState.INITIALIZING]: entries.filter(e => e.state === PluginState.INITIALIZING).length,
        [PluginState.ACTIVE]: entries.filter(e => e.state === PluginState.ACTIVE).length,
        [PluginState.PAUSED]: entries.filter(e => e.state === PluginState.PAUSED).length,
        [PluginState.ERROR]: entries.filter(e => e.state === PluginState.ERROR).length,
        [PluginState.UNLOADING]: entries.filter(e => e.state === PluginState.UNLOADING).length,
      },
      errors: entries.filter(e => e.error).length,
    };
  }

  private notifyStateChange(pluginId: PluginId, state: PluginState): void {
    const listeners = this.stateListeners.get(pluginId);

    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(state);
        } catch (error) {
          console.error(`Error in state change listener for ${pluginId}:`, error);
        }
      });
    }
  }
}

export type StateChangeListener = (state: PluginState) => void;

export interface PluginRegistryStats {
  total: number;
  byState: Record<PluginState, number>;
  errors: number;
}

export const createPluginRegistry = (): PluginRegistryImpl => {
  return new PluginRegistryImpl();
};
