/**
 * AccuScene Enterprise v0.2.0 - Plugin Loader
 * Dynamic loading and hot-reloading of plugins
 */

import {
  Plugin,
  PluginId,
  PluginFactory,
  PluginContext,
  PluginManifest,
  Disposable,
} from '../types';

export interface LoaderOptions {
  hotReload?: boolean;
  timeout?: number;
  cache?: boolean;
}

export class PluginLoaderImpl {
  private loadedModules = new Map<string, any>();
  private watchers = new Map<PluginId, Disposable>();
  private options: Required<LoaderOptions>;

  constructor(options: LoaderOptions = {}) {
    this.options = {
      hotReload: options.hotReload ?? true,
      timeout: options.timeout ?? 30000,
      cache: options.cache ?? true,
    };
  }

  /**
   * Load a plugin from a path or URL
   */
  async load(path: string, context: PluginContext): Promise<Plugin> {
    try {
      // Load the manifest first
      const manifest = await this.loadManifest(path);

      // Load the plugin module
      const module = await this.loadModule(path, manifest.main);

      // Create the plugin instance
      const plugin = await this.createPluginInstance(module, context);

      // Setup hot reload if enabled
      if (this.options.hotReload) {
        this.setupHotReload(path, manifest.id);
      }

      return plugin;
    } catch (error) {
      throw new Error(`Failed to load plugin from ${path}: ${error}`);
    }
  }

  /**
   * Unload a plugin and clean up resources
   */
  async unload(pluginId: PluginId): Promise<void> {
    // Stop watching for changes
    const watcher = this.watchers.get(pluginId);
    if (watcher) {
      watcher.dispose();
      this.watchers.delete(pluginId);
    }

    // Clear module cache if caching is disabled
    if (!this.options.cache) {
      this.clearModuleCache(pluginId);
    }
  }

  /**
   * Reload a plugin (hot reload)
   */
  async reload(pluginId: PluginId, context: PluginContext): Promise<Plugin> {
    // Find the plugin path from loaded modules
    const modulePath = this.findModulePath(pluginId);

    if (!modulePath) {
      throw new Error(`Cannot reload plugin ${pluginId}: module path not found`);
    }

    // Clear the module from cache
    this.clearModuleCache(pluginId);

    // Reload the plugin
    return this.load(modulePath, context);
  }

  /**
   * Load plugin manifest
   */
  private async loadManifest(path: string): Promise<PluginManifest> {
    const manifestPath = this.resolveManifestPath(path);

    try {
      const response = await fetch(manifestPath);

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}`);
      }

      return await response.json();
    } catch (error) {
      throw new Error(`Failed to load manifest from ${manifestPath}: ${error}`);
    }
  }

  /**
   * Load plugin module
   */
  private async loadModule(basePath: string, mainFile: string): Promise<any> {
    const modulePath = this.resolveModulePath(basePath, mainFile);

    // Check cache
    if (this.options.cache && this.loadedModules.has(modulePath)) {
      return this.loadedModules.get(modulePath);
    }

    try {
      // Dynamic import with timeout
      const module = await this.importWithTimeout(modulePath, this.options.timeout);

      // Cache the module
      if (this.options.cache) {
        this.loadedModules.set(modulePath, module);
      }

      return module;
    } catch (error) {
      throw new Error(`Failed to load module from ${modulePath}: ${error}`);
    }
  }

  /**
   * Create plugin instance from module
   */
  private async createPluginInstance(
    module: any,
    context: PluginContext
  ): Promise<Plugin> {
    // Support different export formats
    const factory: PluginFactory =
      module.default ||
      module.plugin ||
      module.createPlugin ||
      module;

    if (typeof factory !== 'function' && typeof factory !== 'object') {
      throw new Error('Plugin module must export a factory function or plugin object');
    }

    // If it's a function, call it with context
    if (typeof factory === 'function') {
      const plugin = await Promise.resolve(factory(context));
      return plugin;
    }

    // If it's already a plugin object, return it
    return factory as Plugin;
  }

  /**
   * Setup hot reload watching
   */
  private setupHotReload(path: string, pluginId: PluginId): void {
    // In a real implementation, this would use file system watchers
    // For web environments, this could use service workers or polling

    // Placeholder implementation
    const watcher: Disposable = {
      dispose: () => {
        // Clean up watcher
      },
    };

    this.watchers.set(pluginId, watcher);
  }

  /**
   * Import module with timeout
   */
  private async importWithTimeout(path: string, timeout: number): Promise<any> {
    return Promise.race([
      import(/* @vite-ignore */ path),
      new Promise((_, reject) =>
        setTimeout(() => reject(new Error('Module load timeout')), timeout)
      ),
    ]);
  }

  /**
   * Resolve manifest path
   */
  private resolveManifestPath(basePath: string): string {
    // Handle different path formats
    if (basePath.endsWith('.json')) {
      return basePath;
    }

    if (basePath.endsWith('/')) {
      return `${basePath}manifest.json`;
    }

    return `${basePath}/manifest.json`;
  }

  /**
   * Resolve module path
   */
  private resolveModulePath(basePath: string, mainFile: string): string {
    // Remove manifest.json if present
    const base = basePath.replace(/\/manifest\.json$/, '');

    // Handle absolute and relative paths
    if (mainFile.startsWith('/')) {
      return mainFile;
    }

    return `${base}/${mainFile}`;
  }

  /**
   * Find module path for a plugin ID
   */
  private findModulePath(pluginId: PluginId): string | undefined {
    for (const [path, module] of this.loadedModules.entries()) {
      if (module?.manifest?.id === pluginId) {
        return path;
      }
    }
    return undefined;
  }

  /**
   * Clear module from cache
   */
  private clearModuleCache(pluginId: PluginId): void {
    const modulePath = this.findModulePath(pluginId);

    if (modulePath) {
      this.loadedModules.delete(modulePath);

      // Also clear from module cache if available
      if (typeof require !== 'undefined' && require.cache) {
        delete require.cache[modulePath];
      }
    }
  }

  /**
   * Get loader statistics
   */
  getStats(): LoaderStats {
    return {
      modulesLoaded: this.loadedModules.size,
      watchersActive: this.watchers.size,
      cacheEnabled: this.options.cache,
      hotReloadEnabled: this.options.hotReload,
    };
  }
}

export interface LoaderStats {
  modulesLoaded: number;
  watchersActive: number;
  cacheEnabled: boolean;
  hotReloadEnabled: boolean;
}

export const createPluginLoader = (options?: LoaderOptions): PluginLoaderImpl => {
  return new PluginLoaderImpl(options);
};
