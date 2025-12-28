/**
 * AccuScene Enterprise v0.2.0 - Plugin Logger
 * Logging utility for plugins
 */

import { PluginLogger, PluginId } from '../types';

export class PluginLoggerImpl implements PluginLogger {
  constructor(private pluginId: PluginId) {}

  debug(message: string, ...args: any[]): void {
    if (process.env.NODE_ENV === 'development') {
      console.debug(`[${this.pluginId}] ${message}`, ...args);
    }
  }

  info(message: string, ...args: any[]): void {
    console.info(`[${this.pluginId}] ${message}`, ...args);
  }

  warn(message: string, ...args: any[]): void {
    console.warn(`[${this.pluginId}] ${message}`, ...args);
  }

  error(message: string, error?: Error, ...args: any[]): void {
    console.error(`[${this.pluginId}] ${message}`, error, ...args);
  }
}

export const createPluginLogger = (pluginId: PluginId): PluginLogger => {
  return new PluginLoggerImpl(pluginId);
};
