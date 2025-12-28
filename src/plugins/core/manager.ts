/**
 * AccuScene Enterprise v0.2.0 - Plugin Manager
 * Central manager for the plugin system
 */

import {
  Plugin,
  PluginId,
  PluginState,
  PluginContext,
  PluginManagerConfig,
  PluginManifest,
  SystemEvent,
  Disposable,
  DisposableStore,
} from '../types';
import { PluginRegistryImpl } from './registry';
import { PluginLoaderImpl } from './loader';
import { PluginValidatorImpl } from './validator';
import { PluginSandbox } from './sandbox';
import { createPluginContext } from '../api/context';
import { PluginLifecycleManager } from '../lifecycle/hooks';

export class PluginManager {
  private registry: PluginRegistryImpl;
  private loader: PluginLoaderImpl;
  private validator: PluginValidatorImpl;
  private lifecycleManager: PluginLifecycleManager;
  private sandboxes = new Map<PluginId, PluginSandbox>();
  private contexts = new Map<PluginId, PluginContext>();
  private disposables = new Map<PluginId, DisposableStore>();
  private eventHandlers = new Map<string, Set<Function>>();
  private config: Required<PluginManagerConfig>;

  constructor(config: PluginManagerConfig) {
    this.config = {
      autoLoad: config.autoLoad ?? false,
      autoActivate: config.autoActivate ?? false,
      hotReload: config.hotReload ?? true,
      maxPlugins: config.maxPlugins ?? 100,
      timeout: config.timeout ?? 30000,
      ...config,
    };

    this.registry = new PluginRegistryImpl();
    this.loader = new PluginLoaderImpl({
      hotReload: this.config.hotReload,
      timeout: this.config.timeout,
    });
    this.validator = new PluginValidatorImpl();
    this.lifecycleManager = new PluginLifecycleManager();
  }

  /**
   * Initialize the plugin manager
   */
  async initialize(): Promise<void> {
    // Load built-in plugins if auto-load is enabled
    if (this.config.autoLoad) {
      await this.loadBuiltinPlugins();
    }

    this.emit(SystemEvent.PLUGIN_LOADED, { initialized: true });
  }

  /**
   * Install a plugin from a path or URL
   */
  async install(path: string, autoActivate?: boolean): Promise<PluginId> {
    try {
      // Check plugin limit
      if (this.registry.getAllEntries().length >= this.config.maxPlugins) {
        throw new Error(`Maximum number of plugins (${this.config.maxPlugins}) reached`);
      }

      // Load the manifest first to get the plugin ID
      const manifestPath = path.endsWith('.json') ? path : `${path}/manifest.json`;
      const response = await fetch(manifestPath);
      const manifest: PluginManifest = await response.json();

      // Validate the manifest
      const validationResult = this.validator.validate(manifest);
      if (!validationResult.valid) {
        throw new Error(
          `Plugin validation failed:\n${validationResult.errors.map(e => e.message).join('\n')}`
        );
      }

      // Check if plugin is already installed
      if (this.registry.has(manifest.id)) {
        throw new Error(`Plugin ${manifest.id} is already installed`);
      }

      // Create plugin context
      const context = createPluginContext(manifest, this);

      // Load the plugin
      this.registry.setState(manifest.id, PluginState.LOADING);
      const plugin = await this.loader.load(path, context);

      // Register the plugin
      this.registry.register(plugin);
      this.contexts.set(manifest.id, context);
      this.disposables.set(manifest.id, new DisposableStore());

      // Register with validator for dependency checking
      this.validator.registerKnownPlugin(manifest.id, manifest.version);

      // Create sandbox if needed
      if (manifest.permissions || manifest.capabilities) {
        const sandbox = new PluginSandbox(
          manifest.id,
          {
            permissions: manifest.permissions || [],
            capabilities: manifest.capabilities || [],
            isolated: true,
            sandbox: true,
          }
        );
        this.sandboxes.set(manifest.id, sandbox);
      }

      this.emit(SystemEvent.PLUGIN_LOADED, { pluginId: manifest.id, manifest });

      // Auto-activate if requested
      const shouldActivate = autoActivate ?? this.config.autoActivate;
      if (shouldActivate) {
        await this.activate(manifest.id);
      }

      return manifest.id;
    } catch (error) {
      throw new Error(`Failed to install plugin from ${path}: ${error}`);
    }
  }

  /**
   * Uninstall a plugin
   */
  async uninstall(pluginId: PluginId): Promise<void> {
    if (!this.registry.has(pluginId)) {
      throw new Error(`Plugin ${pluginId} is not installed`);
    }

    const state = this.registry.getState(pluginId);

    // Deactivate if active
    if (state === PluginState.ACTIVE) {
      await this.deactivate(pluginId);
    }

    try {
      // Unload the plugin
      this.registry.setState(pluginId, PluginState.UNLOADING);
      await this.loader.unload(pluginId);

      // Cleanup resources
      this.disposables.get(pluginId)?.dispose();
      this.disposables.delete(pluginId);
      this.contexts.delete(pluginId);
      this.sandboxes.get(pluginId)?.dispose();
      this.sandboxes.delete(pluginId);

      // Unregister
      this.registry.unregister(pluginId);
      this.validator.unregisterKnownPlugin(pluginId);

      this.emit(SystemEvent.PLUGIN_UNLOADED, { pluginId });
    } catch (error) {
      this.registry.setState(pluginId, PluginState.ERROR, error as Error);
      throw error;
    }
  }

  /**
   * Activate a plugin
   */
  async activate(pluginId: PluginId): Promise<void> {
    const plugin = this.registry.get(pluginId);
    if (!plugin) {
      throw new Error(`Plugin ${pluginId} is not installed`);
    }

    const state = this.registry.getState(pluginId);
    if (state === PluginState.ACTIVE) {
      return; // Already active
    }

    try {
      this.registry.setState(pluginId, PluginState.INITIALIZING);

      // Run pre-activation hooks
      await this.lifecycleManager.runPreActivate(pluginId);

      // Get plugin context
      const context = this.contexts.get(pluginId);
      if (!context) {
        throw new Error(`Context not found for plugin ${pluginId}`);
      }

      // Activate the plugin
      if (plugin.activate) {
        await plugin.activate(context);
      }

      // Run post-activation hooks
      await this.lifecycleManager.runPostActivate(pluginId);

      this.registry.setState(pluginId, PluginState.ACTIVE);
      this.emit(SystemEvent.PLUGIN_ACTIVATED, { pluginId });
    } catch (error) {
      this.registry.setState(pluginId, PluginState.ERROR, error as Error);
      this.emit(SystemEvent.PLUGIN_ERROR, { pluginId, error });
      throw error;
    }
  }

  /**
   * Deactivate a plugin
   */
  async deactivate(pluginId: PluginId): Promise<void> {
    const plugin = this.registry.get(pluginId);
    if (!plugin) {
      throw new Error(`Plugin ${pluginId} is not installed`);
    }

    const state = this.registry.getState(pluginId);
    if (state !== PluginState.ACTIVE) {
      return; // Not active
    }

    try {
      // Run pre-deactivation hooks
      await this.lifecycleManager.runPreDeactivate(pluginId);

      // Get plugin context
      const context = this.contexts.get(pluginId);
      if (!context) {
        throw new Error(`Context not found for plugin ${pluginId}`);
      }

      // Deactivate the plugin
      if (plugin.deactivate) {
        await plugin.deactivate(context);
      }

      // Dispose plugin resources
      this.disposables.get(pluginId)?.dispose();

      // Run post-deactivation hooks
      await this.lifecycleManager.runPostDeactivate(pluginId);

      this.registry.setState(pluginId, PluginState.LOADED);
      this.emit(SystemEvent.PLUGIN_DEACTIVATED, { pluginId });
    } catch (error) {
      this.registry.setState(pluginId, PluginState.ERROR, error as Error);
      this.emit(SystemEvent.PLUGIN_ERROR, { pluginId, error });
      throw error;
    }
  }

  /**
   * Reload a plugin (hot reload)
   */
  async reload(pluginId: PluginId): Promise<void> {
    const wasActive = this.registry.getState(pluginId) === PluginState.ACTIVE;

    // Deactivate if active
    if (wasActive) {
      await this.deactivate(pluginId);
    }

    try {
      // Get context
      const context = this.contexts.get(pluginId);
      if (!context) {
        throw new Error(`Context not found for plugin ${pluginId}`);
      }

      // Reload the plugin
      const plugin = await this.loader.reload(pluginId, context);

      // Update registry
      this.registry.unregister(pluginId);
      this.registry.register(plugin);

      // Reactivate if it was active
      if (wasActive) {
        await this.activate(pluginId);
      }

      this.emit(SystemEvent.PLUGIN_UPDATED, { pluginId });
    } catch (error) {
      this.registry.setState(pluginId, PluginState.ERROR, error as Error);
      throw error;
    }
  }

  /**
   * Get a plugin by ID
   */
  getPlugin(pluginId: PluginId): Plugin | undefined {
    return this.registry.get(pluginId);
  }

  /**
   * Get all installed plugins
   */
  getAllPlugins(): Plugin[] {
    return this.registry.getAll();
  }

  /**
   * Get active plugins
   */
  getActivePlugins(): Plugin[] {
    return this.registry.getByState(PluginState.ACTIVE);
  }

  /**
   * Get plugin context
   */
  getContext(pluginId: PluginId): PluginContext | undefined {
    return this.contexts.get(pluginId);
  }

  /**
   * Get plugin sandbox
   */
  getSandbox(pluginId: PluginId): PluginSandbox | undefined {
    return this.sandboxes.get(pluginId);
  }

  /**
   * Listen to system events
   */
  on(event: string, handler: Function): Disposable {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }

    this.eventHandlers.get(event)!.add(handler);

    return {
      dispose: () => {
        this.eventHandlers.get(event)?.delete(handler);
      },
    };
  }

  /**
   * Emit a system event
   */
  private emit(event: string, data: any): void {
    const handlers = this.eventHandlers.get(event);
    if (handlers) {
      handlers.forEach(handler => {
        try {
          handler(data);
        } catch (error) {
          console.error(`Error in event handler for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Load built-in plugins
   */
  private async loadBuiltinPlugins(): Promise<void> {
    // This would load plugins from the builtin directory
    // Implementation depends on the bundler and deployment strategy
  }

  /**
   * Get manager statistics
   */
  getStats() {
    return {
      registry: this.registry.getStats(),
      loader: this.loader.getStats(),
      contexts: this.contexts.size,
      sandboxes: this.sandboxes.size,
    };
  }

  /**
   * Cleanup all resources
   */
  async dispose(): Promise<void> {
    // Deactivate all active plugins
    const activePlugins = this.getActivePlugins();
    for (const plugin of activePlugins) {
      await this.deactivate(plugin.manifest.id);
    }

    // Cleanup all resources
    this.disposables.forEach(store => store.dispose());
    this.sandboxes.forEach(sandbox => sandbox.dispose());

    this.disposables.clear();
    this.contexts.clear();
    this.sandboxes.clear();
    this.registry.clear();
    this.eventHandlers.clear();
  }
}

export const createPluginManager = (config: PluginManagerConfig): PluginManager => {
  return new PluginManager(config);
};
