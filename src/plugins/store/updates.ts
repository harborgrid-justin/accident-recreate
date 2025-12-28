/**
 * AccuScene Enterprise v0.2.0 - Plugin Updates
 * Update management for plugins
 */

import { PluginId, PluginUpdateInfo, PluginManifest } from '../types';

export interface UpdateCheckResult {
  hasUpdates: boolean;
  updates: PluginUpdateInfo[];
}

export class PluginUpdateManager {
  private updateCache = new Map<PluginId, PluginUpdateInfo>();
  private lastCheck: number = 0;
  private checkInterval: number = 3600000; // 1 hour

  /**
   * Check for updates for all installed plugins
   */
  async checkForUpdates(
    installedPlugins: PluginManifest[]
  ): Promise<UpdateCheckResult> {
    const updates: PluginUpdateInfo[] = [];

    for (const plugin of installedPlugins) {
      const updateInfo = await this.checkPluginUpdate(plugin);

      if (updateInfo) {
        updates.push(updateInfo);
        this.updateCache.set(plugin.id, updateInfo);
      }
    }

    this.lastCheck = Date.now();

    return {
      hasUpdates: updates.length > 0,
      updates,
    };
  }

  /**
   * Check for update for a specific plugin
   */
  async checkPluginUpdate(plugin: PluginManifest): Promise<PluginUpdateInfo | null> {
    try {
      // In a real implementation, this would call the marketplace API
      const latestVersion = await this.fetchLatestVersion(plugin.id);

      if (!latestVersion) {
        return null;
      }

      // Compare versions
      if (this.compareVersions(latestVersion.version, plugin.version) > 0) {
        return {
          pluginId: plugin.id,
          currentVersion: plugin.version,
          latestVersion: latestVersion.version,
          changelog: latestVersion.changelog,
          breaking: latestVersion.breaking,
        };
      }

      return null;
    } catch (error) {
      console.error(`Failed to check update for ${plugin.id}:`, error);
      return null;
    }
  }

  /**
   * Get cached update info for a plugin
   */
  getUpdateInfo(pluginId: PluginId): PluginUpdateInfo | undefined {
    return this.updateCache.get(pluginId);
  }

  /**
   * Check if updates are available
   */
  hasUpdatesAvailable(): boolean {
    return this.updateCache.size > 0;
  }

  /**
   * Get all available updates
   */
  getAvailableUpdates(): PluginUpdateInfo[] {
    return Array.from(this.updateCache.values());
  }

  /**
   * Clear update cache
   */
  clearCache(): void {
    this.updateCache.clear();
  }

  /**
   * Get time since last update check
   */
  getTimeSinceLastCheck(): number {
    return Date.now() - this.lastCheck;
  }

  /**
   * Check if it's time to check for updates again
   */
  shouldCheckForUpdates(): boolean {
    return this.getTimeSinceLastCheck() >= this.checkInterval;
  }

  /**
   * Set the check interval
   */
  setCheckInterval(intervalMs: number): void {
    this.checkInterval = intervalMs;
  }

  /**
   * Compare two semantic versions
   */
  private compareVersions(v1: string, v2: string): number {
    const parts1 = v1.split('.').map(Number);
    const parts2 = v2.split('.').map(Number);

    for (let i = 0; i < Math.max(parts1.length, parts2.length); i++) {
      const p1 = parts1[i] || 0;
      const p2 = parts2[i] || 0;

      if (p1 > p2) return 1;
      if (p1 < p2) return -1;
    }

    return 0;
  }

  /**
   * Fetch latest version info from marketplace
   */
  private async fetchLatestVersion(
    pluginId: PluginId
  ): Promise<{
    version: string;
    changelog: any;
    breaking: boolean;
  } | null> {
    // In a real implementation, this would fetch from the marketplace API
    // For now, return null (no update available)
    return null;
  }

  /**
   * Download and install an update
   */
  async installUpdate(
    pluginId: PluginId,
    updateInfo: PluginUpdateInfo
  ): Promise<void> {
    try {
      // In a real implementation, this would:
      // 1. Download the new version
      // 2. Backup the current version
      // 3. Install the new version
      // 4. Restart the plugin

      // For now, just clear from cache
      this.updateCache.delete(pluginId);
    } catch (error) {
      throw new Error(`Failed to install update for ${pluginId}: ${error}`);
    }
  }

  /**
   * Enable automatic updates
   */
  enableAutoUpdate(pluginId: PluginId): void {
    // Store preference in localStorage
    localStorage.setItem(`plugin.${pluginId}.autoUpdate`, 'true');
  }

  /**
   * Disable automatic updates
   */
  disableAutoUpdate(pluginId: PluginId): void {
    localStorage.removeItem(`plugin.${pluginId}.autoUpdate`);
  }

  /**
   * Check if automatic updates are enabled
   */
  isAutoUpdateEnabled(pluginId: PluginId): boolean {
    return localStorage.getItem(`plugin.${pluginId}.autoUpdate`) === 'true';
  }
}

export const createUpdateManager = (): PluginUpdateManager => {
  return new PluginUpdateManager();
};
