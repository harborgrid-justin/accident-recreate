/**
 * AccuScene Enterprise v0.2.0 - Exporter Extensions
 * Extension point for custom export formats
 */

import {
  ExporterContribution,
  ExportOptions,
  PluginId,
  Disposable,
} from '../types';

export class ExporterExtensionRegistry {
  private exporters = new Map<string, ExporterContribution>();
  private extensionMap = new Map<string, Set<string>>();

  /**
   * Register an exporter contribution
   */
  register(pluginId: PluginId, contribution: ExporterContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.exporters.has(id)) {
      throw new Error(`Exporter ${id} is already registered`);
    }

    this.exporters.set(id, contribution);

    // Track by file extensions
    for (const ext of contribution.extensions) {
      if (!this.extensionMap.has(ext)) {
        this.extensionMap.set(ext, new Set());
      }
      this.extensionMap.get(ext)!.add(id);
    }

    return {
      dispose: () => {
        this.exporters.delete(id);

        // Remove from extension map
        for (const ext of contribution.extensions) {
          this.extensionMap.get(ext)?.delete(id);
        }
      },
    };
  }

  /**
   * Get exporters for a specific file extension
   */
  getByExtension(extension: string): ExporterContribution[] {
    const ext = extension.toLowerCase().replace(/^\./, '');
    const ids = this.extensionMap.get(ext) || new Set();

    return Array.from(ids)
      .map(id => this.exporters.get(id))
      .filter((e): e is ExporterContribution => e !== undefined);
  }

  /**
   * Get exporters for a MIME type
   */
  getByMimeType(mimeType: string): ExporterContribution[] {
    return Array.from(this.exporters.values()).filter(e =>
      e.mimeTypes.includes(mimeType)
    );
  }

  /**
   * Get all exporters
   */
  getAll(): ExporterContribution[] {
    return Array.from(this.exporters.values());
  }

  /**
   * Get a specific exporter
   */
  get(id: string): ExporterContribution | undefined {
    return this.exporters.get(id);
  }

  /**
   * Export data using a specific exporter
   */
  async export(
    exporterId: string,
    data: any,
    options?: ExportOptions
  ): Promise<Blob> {
    const exporter = this.exporters.get(exporterId);

    if (!exporter) {
      throw new Error(`Exporter ${exporterId} not found`);
    }

    try {
      return await exporter.export(data, options);
    } catch (error) {
      throw new Error(`Export failed with ${exporterId}: ${error}`);
    }
  }

  /**
   * Get all supported extensions
   */
  getSupportedExtensions(): string[] {
    return Array.from(this.extensionMap.keys());
  }

  /**
   * Check if an extension is supported
   */
  supportsExtension(extension: string): boolean {
    const ext = extension.toLowerCase().replace(/^\./, '');
    return this.extensionMap.has(ext);
  }
}

export const createExporterRegistry = (): ExporterExtensionRegistry => {
  return new ExporterExtensionRegistry();
};
