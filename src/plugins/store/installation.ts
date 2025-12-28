/**
 * AccuScene Enterprise v0.2.0 - Plugin Installation
 * Plugin installation and uninstallation
 */

import { PluginId, PluginInstallOptions, PluginManifest } from '../types';

export enum InstallationStatus {
  PENDING = 'pending',
  DOWNLOADING = 'downloading',
  INSTALLING = 'installing',
  COMPLETE = 'complete',
  FAILED = 'failed',
}

export interface InstallationProgress {
  status: InstallationStatus;
  progress: number; // 0-100
  message: string;
  error?: Error;
}

export class PluginInstaller {
  private installations = new Map<PluginId, InstallationProgress>();
  private progressListeners = new Map<PluginId, Set<ProgressListener>>();

  /**
   * Install a plugin from a URL
   */
  async install(
    url: string,
    options: PluginInstallOptions = {}
  ): Promise<{ pluginId: PluginId; manifest: PluginManifest }> {
    const tempId = `installing-${Date.now()}`;

    try {
      // Update status: downloading
      this.updateProgress(tempId, {
        status: InstallationStatus.DOWNLOADING,
        progress: 0,
        message: 'Downloading plugin...',
      });

      // Download the plugin package
      const packageData = await this.downloadPackage(url, tempId);

      // Update status: installing
      this.updateProgress(tempId, {
        status: InstallationStatus.INSTALLING,
        progress: 50,
        message: 'Installing plugin...',
      });

      // Extract and validate manifest
      const manifest = await this.extractManifest(packageData);
      const pluginId = manifest.id;

      // Install files
      await this.installFiles(pluginId, packageData);

      // Update progress with actual plugin ID
      const progress = this.installations.get(tempId);
      if (progress) {
        this.installations.set(pluginId, progress);
        this.installations.delete(tempId);
      }

      // Update status: complete
      this.updateProgress(pluginId, {
        status: InstallationStatus.COMPLETE,
        progress: 100,
        message: 'Installation complete',
      });

      return { pluginId, manifest };
    } catch (error) {
      this.updateProgress(tempId, {
        status: InstallationStatus.FAILED,
        progress: 0,
        message: 'Installation failed',
        error: error as Error,
      });

      throw error;
    }
  }

  /**
   * Install from local file
   */
  async installFromFile(
    file: File,
    options: PluginInstallOptions = {}
  ): Promise<{ pluginId: PluginId; manifest: PluginManifest }> {
    const tempId = `installing-${Date.now()}`;

    try {
      this.updateProgress(tempId, {
        status: InstallationStatus.INSTALLING,
        progress: 0,
        message: 'Reading plugin file...',
      });

      // Read file as ArrayBuffer
      const packageData = await file.arrayBuffer();

      // Extract and validate manifest
      const manifest = await this.extractManifest(packageData);
      const pluginId = manifest.id;

      // Install files
      await this.installFiles(pluginId, packageData);

      // Update progress
      const progress = this.installations.get(tempId);
      if (progress) {
        this.installations.set(pluginId, progress);
        this.installations.delete(tempId);
      }

      this.updateProgress(pluginId, {
        status: InstallationStatus.COMPLETE,
        progress: 100,
        message: 'Installation complete',
      });

      return { pluginId, manifest };
    } catch (error) {
      this.updateProgress(tempId, {
        status: InstallationStatus.FAILED,
        progress: 0,
        message: 'Installation failed',
        error: error as Error,
      });

      throw error;
    }
  }

  /**
   * Uninstall a plugin
   */
  async uninstall(pluginId: PluginId): Promise<void> {
    try {
      // Remove plugin files
      await this.removeFiles(pluginId);

      // Clear installation record
      this.installations.delete(pluginId);
      this.progressListeners.delete(pluginId);
    } catch (error) {
      throw new Error(`Failed to uninstall plugin ${pluginId}: ${error}`);
    }
  }

  /**
   * Get installation progress
   */
  getProgress(pluginId: PluginId): InstallationProgress | undefined {
    return this.installations.get(pluginId);
  }

  /**
   * Listen to installation progress
   */
  onProgress(pluginId: PluginId, listener: ProgressListener): () => void {
    if (!this.progressListeners.has(pluginId)) {
      this.progressListeners.set(pluginId, new Set());
    }

    this.progressListeners.get(pluginId)!.add(listener);

    return () => {
      this.progressListeners.get(pluginId)?.delete(listener);
    };
  }

  private async downloadPackage(url: string, tempId: string): Promise<ArrayBuffer> {
    const response = await fetch(url);

    if (!response.ok) {
      throw new Error(`Download failed: HTTP ${response.status}`);
    }

    const contentLength = parseInt(response.headers.get('content-length') || '0');
    const reader = response.body?.getReader();

    if (!reader) {
      throw new Error('Response body is not readable');
    }

    const chunks: Uint8Array[] = [];
    let receivedLength = 0;

    while (true) {
      const { done, value } = await reader.read();

      if (done) break;

      chunks.push(value);
      receivedLength += value.length;

      // Update download progress
      const progress = contentLength > 0 ? (receivedLength / contentLength) * 50 : 25;

      this.updateProgress(tempId, {
        status: InstallationStatus.DOWNLOADING,
        progress,
        message: `Downloading... ${Math.round(progress * 2)}%`,
      });
    }

    // Combine chunks
    const allChunks = new Uint8Array(receivedLength);
    let position = 0;

    for (const chunk of chunks) {
      allChunks.set(chunk, position);
      position += chunk.length;
    }

    return allChunks.buffer;
  }

  private async extractManifest(packageData: ArrayBuffer): Promise<PluginManifest> {
    // In a real implementation, this would extract from a zip/tar file
    // For now, assume it's a JSON manifest
    const decoder = new TextDecoder();
    const text = decoder.decode(packageData);

    try {
      return JSON.parse(text);
    } catch (error) {
      throw new Error('Invalid plugin package: could not parse manifest');
    }
  }

  private async installFiles(pluginId: PluginId, packageData: ArrayBuffer): Promise<void> {
    // In a real implementation, this would:
    // 1. Extract all files from the package
    // 2. Validate file integrity
    // 3. Copy files to the plugin directory
    // 4. Set proper permissions

    // For now, store in localStorage as a placeholder
    const key = `plugin.package.${pluginId}`;
    const base64 = btoa(String.fromCharCode(...new Uint8Array(packageData)));
    localStorage.setItem(key, base64);
  }

  private async removeFiles(pluginId: PluginId): Promise<void> {
    // Remove from localStorage
    const key = `plugin.package.${pluginId}`;
    localStorage.removeItem(key);
  }

  private updateProgress(pluginId: PluginId, progress: InstallationProgress): void {
    this.installations.set(pluginId, progress);

    // Notify listeners
    const listeners = this.progressListeners.get(pluginId);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(progress);
        } catch (error) {
          console.error('Error in progress listener:', error);
        }
      });
    }
  }
}

export type ProgressListener = (progress: InstallationProgress) => void;

export const createInstaller = (): PluginInstaller => {
  return new PluginInstaller();
};
