/**
 * AccuScene Enterprise v0.2.0 - Plugin Architecture
 * Main exports for the plugin system
 *
 * A comprehensive, extensible plugin architecture with:
 * - Hot-reloading support
 * - Permission-based security
 * - Dependency resolution
 * - Marketplace integration
 * - Built-in plugins for core functionality
 */

// Core plugin system
export * from './types';
export * from './core';
export * from './lifecycle';
export * from './api';
export * from './extension';
export * from './manifest';
export * from './security';
export * from './store';
export * from './builtin';

// Re-export commonly used types
export type {
  Plugin,
  PluginContext,
  PluginManifest,
  PluginId,
  PluginPermission,
  PluginCapability,
  PluginState,
  PluginType,
  Command,
  Disposable,
} from './types';

// Re-export main classes
export {
  PluginManager,
  createPluginManager,
} from './core/manager';

export {
  PluginRegistryImpl,
  createPluginRegistry,
} from './core/registry';

export {
  PluginLoaderImpl,
  createPluginLoader,
} from './core/loader';

export {
  PluginValidatorImpl,
  createPluginValidator,
} from './core/validator';

export {
  PluginSandbox,
  createPluginSandbox,
} from './core/sandbox';

// Security exports
export {
  PermissionManager,
  createPermissionManager,
} from './security/permissions';

export {
  CapabilityManager,
  createCapabilityManager,
} from './security/capabilities';

export {
  IsolationPolicyManager,
  createIsolationPolicyManager,
  IsolationLevel,
} from './security/isolation';

// Store exports
export {
  PluginMarketplace,
  createMarketplace,
} from './store/marketplace';

export {
  PluginInstaller,
  createInstaller,
  InstallationStatus,
} from './store/installation';

export {
  PluginUpdateManager,
  createUpdateManager,
} from './store/updates';

// Extension registries
export {
  ToolbarExtensionRegistry,
  createToolbarRegistry,
} from './extension/toolbar';

export {
  PanelExtensionRegistry,
  createPanelRegistry,
} from './extension/panel';

export {
  MenuExtensionRegistry,
  createMenuExtensionRegistry,
} from './extension/menu';

export {
  ContextMenuExtensionRegistry,
  createContextMenuRegistry,
} from './extension/contextMenu';

export {
  ExporterExtensionRegistry,
  createExporterRegistry,
} from './extension/exporter';

export {
  ImporterExtensionRegistry,
  createImporterRegistry,
} from './extension/importer';

export {
  ToolExtensionRegistry,
  createToolRegistry,
} from './extension/tool';

// Built-in plugins
export {
  getBuiltinPlugins,
  createMeasurementsPlugin,
  createAnnotationsPlugin,
  createExportsPlugin,
} from './builtin';

/**
 * Initialize the plugin system
 *
 * @example
 * ```typescript
 * import { initializePluginSystem } from './plugins';
 *
 * const pluginManager = await initializePluginSystem({
 *   pluginDirectory: '/plugins',
 *   autoLoad: true,
 *   autoActivate: true,
 *   hotReload: true,
 * });
 *
 * // Install a plugin
 * await pluginManager.install('/plugins/my-plugin');
 *
 * // Activate a plugin
 * await pluginManager.activate('com.example.my-plugin');
 * ```
 */
export async function initializePluginSystem(config: {
  pluginDirectory: string;
  autoLoad?: boolean;
  autoActivate?: boolean;
  hotReload?: boolean;
  maxPlugins?: number;
  timeout?: number;
}) {
  const manager = createPluginManager(config);
  await manager.initialize();
  return manager;
}

/**
 * Version information
 */
export const PLUGIN_SYSTEM_VERSION = '0.2.0';
export const PLUGIN_API_VERSION = '1.0.0';
