/**
 * AccuScene Enterprise v0.2.0 - Plugin Storage
 * Persistent storage API for plugins
 */

import { PluginStorage, PluginId } from '../types';

class PluginStorageImpl implements PluginStorage {
  private storageKey: string;

  constructor(private pluginId: PluginId) {
    this.storageKey = `accuscene.plugin.${pluginId}`;
  }

  async get<T = any>(key: string): Promise<T | undefined> {
    try {
      const fullKey = this.getFullKey(key);
      const data = localStorage.getItem(fullKey);

      if (data === null) {
        return undefined;
      }

      return JSON.parse(data) as T;
    } catch (error) {
      console.error(`Error reading storage for plugin ${this.pluginId}:`, error);
      return undefined;
    }
  }

  async set<T = any>(key: string, value: T): Promise<void> {
    try {
      const fullKey = this.getFullKey(key);
      const data = JSON.stringify(value);
      localStorage.setItem(fullKey, data);
    } catch (error) {
      throw new Error(`Error writing storage for plugin ${this.pluginId}: ${error}`);
    }
  }

  async delete(key: string): Promise<void> {
    try {
      const fullKey = this.getFullKey(key);
      localStorage.removeItem(fullKey);
    } catch (error) {
      throw new Error(`Error deleting storage for plugin ${this.pluginId}: ${error}`);
    }
  }

  async clear(): Promise<void> {
    try {
      const keys = await this.keys();
      for (const key of keys) {
        await this.delete(key);
      }
    } catch (error) {
      throw new Error(`Error clearing storage for plugin ${this.pluginId}: ${error}`);
    }
  }

  async keys(): Promise<string[]> {
    try {
      const prefix = this.storageKey + '.';
      const keys: string[] = [];

      for (let i = 0; i < localStorage.length; i++) {
        const key = localStorage.key(i);
        if (key && key.startsWith(prefix)) {
          keys.push(key.substring(prefix.length));
        }
      }

      return keys;
    } catch (error) {
      throw new Error(`Error listing storage keys for plugin ${this.pluginId}: ${error}`);
    }
  }

  private getFullKey(key: string): string {
    return `${this.storageKey}.${key}`;
  }
}

export const createPluginStorage = (pluginId: PluginId): PluginStorage => {
  return new PluginStorageImpl(pluginId);
};
