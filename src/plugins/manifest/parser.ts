/**
 * AccuScene Enterprise v0.2.0 - Manifest Parser
 * Parser for plugin manifest files
 */

import { PluginManifest, PluginType } from '../types';

export class ManifestParser {
  /**
   * Parse a manifest from JSON string
   */
  parse(json: string): PluginManifest {
    try {
      const data = JSON.parse(json);
      return this.parseObject(data);
    } catch (error) {
      throw new Error(`Failed to parse manifest: ${error}`);
    }
  }

  /**
   * Parse a manifest from an object
   */
  parseObject(data: any): PluginManifest {
    if (!data || typeof data !== 'object') {
      throw new Error('Manifest must be an object');
    }

    // Parse basic fields
    const manifest: PluginManifest = {
      id: this.parseString(data.id, 'id'),
      name: this.parseString(data.name, 'name'),
      version: this.parseString(data.version, 'version'),
      description: this.parseString(data.description, 'description'),
      author: this.parseString(data.author, 'author'),
      license: this.parseString(data.license, 'license'),
      main: this.parseString(data.main, 'main'),
      type: this.parsePluginType(data.type),
    };

    // Parse optional fields
    if (data.homepage) {
      manifest.homepage = this.parseString(data.homepage, 'homepage');
    }

    if (data.repository) {
      manifest.repository = this.parseString(data.repository, 'repository');
    }

    if (data.keywords) {
      manifest.keywords = this.parseStringArray(data.keywords, 'keywords');
    }

    if (data.category) {
      manifest.category = this.parseString(data.category, 'category');
    }

    if (data.icon) {
      manifest.icon = this.parseString(data.icon, 'icon');
    }

    if (data.dependencies) {
      manifest.dependencies = this.parseDependencies(data.dependencies);
    }

    if (data.peerDependencies) {
      manifest.peerDependencies = this.parseDependencies(data.peerDependencies);
    }

    if (data.permissions) {
      manifest.permissions = this.parseStringArray(data.permissions, 'permissions') as any[];
    }

    if (data.capabilities) {
      manifest.capabilities = this.parseStringArray(data.capabilities, 'capabilities') as any[];
    }

    if (data.exports) {
      manifest.exports = this.parseObject(data.exports);
    }

    if (data.contributes) {
      manifest.contributes = data.contributes;
    }

    if (data.engines) {
      manifest.engines = data.engines;
    }

    return manifest;
  }

  /**
   * Serialize a manifest to JSON
   */
  stringify(manifest: PluginManifest, pretty = true): string {
    return JSON.stringify(manifest, null, pretty ? 2 : 0);
  }

  private parseString(value: any, field: string): string {
    if (typeof value !== 'string') {
      throw new Error(`Field '${field}' must be a string`);
    }
    return value;
  }

  private parseStringArray(value: any, field: string): string[] {
    if (!Array.isArray(value)) {
      throw new Error(`Field '${field}' must be an array`);
    }

    return value.map((item, index) => {
      if (typeof item !== 'string') {
        throw new Error(`Field '${field}[${index}]' must be a string`);
      }
      return item;
    });
  }

  private parsePluginType(value: any): PluginType {
    const validTypes = Object.values(PluginType);

    if (!value) {
      return PluginType.EXTENSION; // Default
    }

    if (!validTypes.includes(value)) {
      throw new Error(`Invalid plugin type: ${value}`);
    }

    return value as PluginType;
  }

  private parseDependencies(value: any): any[] {
    if (!Array.isArray(value)) {
      throw new Error('Dependencies must be an array');
    }

    return value.map((dep, index) => {
      if (!dep || typeof dep !== 'object') {
        throw new Error(`Dependency ${index} must be an object`);
      }

      if (!dep.id || typeof dep.id !== 'string') {
        throw new Error(`Dependency ${index} must have an 'id' string field`);
      }

      if (!dep.version || typeof dep.version !== 'string') {
        throw new Error(`Dependency ${index} must have a 'version' string field`);
      }

      return {
        id: dep.id,
        version: dep.version,
        optional: dep.optional === true,
      };
    });
  }
}

export const createManifestParser = (): ManifestParser => {
  return new ManifestParser();
};
