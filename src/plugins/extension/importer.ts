/**
 * AccuScene Enterprise v0.2.0 - Importer Extensions
 * Extension point for custom import formats
 */

import {
  ImporterContribution,
  ImportOptions,
  PluginId,
  Disposable,
} from '../types';

export class ImporterExtensionRegistry {
  private importers = new Map<string, ImporterContribution>();
  private extensionMap = new Map<string, Set<string>>();

  /**
   * Register an importer contribution
   */
  register(pluginId: PluginId, contribution: ImporterContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.importers.has(id)) {
      throw new Error(`Importer ${id} is already registered`);
    }

    this.importers.set(id, contribution);

    // Track by file extensions
    for (const ext of contribution.extensions) {
      if (!this.extensionMap.has(ext)) {
        this.extensionMap.set(ext, new Set());
      }
      this.extensionMap.get(ext)!.add(id);
    }

    return {
      dispose: () => {
        this.importers.delete(id);

        // Remove from extension map
        for (const ext of contribution.extensions) {
          this.extensionMap.get(ext)?.delete(id);
        }
      },
    };
  }

  /**
   * Get importers for a specific file extension
   */
  getByExtension(extension: string): ImporterContribution[] {
    const ext = extension.toLowerCase().replace(/^\./, '');
    const ids = this.extensionMap.get(ext) || new Set();

    return Array.from(ids)
      .map(id => this.importers.get(id))
      .filter((i): i is ImporterContribution => i !== undefined);
  }

  /**
   * Get importers for a MIME type
   */
  getByMimeType(mimeType: string): ImporterContribution[] {
    return Array.from(this.importers.values()).filter(i =>
      i.mimeTypes.includes(mimeType)
    );
  }

  /**
   * Get importers for a file
   */
  getByFile(file: File): ImporterContribution[] {
    // Try by MIME type first
    let importers = this.getByMimeType(file.type);

    // If no importers found, try by extension
    if (importers.length === 0) {
      const ext = file.name.split('.').pop() || '';
      importers = this.getByExtension(ext);
    }

    return importers;
  }

  /**
   * Get all importers
   */
  getAll(): ImporterContribution[] {
    return Array.from(this.importers.values());
  }

  /**
   * Get a specific importer
   */
  get(id: string): ImporterContribution | undefined {
    return this.importers.get(id);
  }

  /**
   * Import data using a specific importer
   */
  async import(
    importerId: string,
    file: File,
    options?: ImportOptions
  ): Promise<any> {
    const importer = this.importers.get(importerId);

    if (!importer) {
      throw new Error(`Importer ${importerId} not found`);
    }

    try {
      return await importer.import(file, options);
    } catch (error) {
      throw new Error(`Import failed with ${importerId}: ${error}`);
    }
  }

  /**
   * Import data automatically (detect best importer for file)
   */
  async importAuto(file: File, options?: ImportOptions): Promise<any> {
    const importers = this.getByFile(file);

    if (importers.length === 0) {
      throw new Error(`No importer found for file: ${file.name}`);
    }

    // Use the first matching importer
    const importer = importers[0];

    try {
      return await importer.import(file, options);
    } catch (error) {
      // If first importer fails, try others
      for (let i = 1; i < importers.length; i++) {
        try {
          return await importers[i].import(file, options);
        } catch (e) {
          // Continue to next importer
        }
      }

      throw new Error(`All importers failed for file: ${file.name}`);
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

export const createImporterRegistry = (): ImporterExtensionRegistry => {
  return new ImporterExtensionRegistry();
};
