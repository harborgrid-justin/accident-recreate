/**
 * AccuScene Enterprise v0.2.0 - Plugin Context
 * Context provided to plugins during activation
 */

import { PluginContext, PluginManifest, PluginId } from '../types';
import { createPluginServices } from './services';
import { createPluginStorage } from './storage';
import { createPluginEventEmitter } from './events';
import { createPluginUI } from './ui';
import { createPluginCommands } from './commands';
import { createPluginLogger } from '../utils/logger';

export const createPluginContext = (
  manifest: PluginManifest,
  manager: any // Avoid circular dependency
): PluginContext => {
  const id = manifest.id;
  const version = manifest.version;

  return {
    id,
    version,
    manifest,
    services: createPluginServices(id),
    storage: createPluginStorage(id),
    events: createPluginEventEmitter(id),
    ui: createPluginUI(id, manager),
    commands: createPluginCommands(id, manager),
    logger: createPluginLogger(id),
  };
};
