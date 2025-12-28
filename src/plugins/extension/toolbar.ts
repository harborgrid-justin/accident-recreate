/**
 * AccuScene Enterprise v0.2.0 - Toolbar Extensions
 * Extension point for custom toolbars
 */

import {
  ToolbarContribution,
  ToolbarItem,
  ToolbarLocation,
  PluginId,
  Disposable,
} from '../types';

export class ToolbarExtensionRegistry {
  private toolbars = new Map<string, ToolbarContribution>();
  private locationMap = new Map<ToolbarLocation, Set<string>>();

  /**
   * Register a toolbar contribution
   */
  register(pluginId: PluginId, contribution: ToolbarContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.toolbars.has(id)) {
      throw new Error(`Toolbar ${id} is already registered`);
    }

    this.toolbars.set(id, contribution);

    // Track by location
    if (!this.locationMap.has(contribution.location)) {
      this.locationMap.set(contribution.location, new Set());
    }
    this.locationMap.get(contribution.location)!.add(id);

    return {
      dispose: () => {
        this.toolbars.delete(id);
        this.locationMap.get(contribution.location)?.delete(id);
      },
    };
  }

  /**
   * Get toolbars for a specific location
   */
  getByLocation(location: ToolbarLocation): ToolbarContribution[] {
    const ids = this.locationMap.get(location) || new Set();
    const toolbars = Array.from(ids)
      .map(id => this.toolbars.get(id))
      .filter((t): t is ToolbarContribution => t !== undefined);

    // Sort by priority (higher first)
    return toolbars.sort((a, b) => (b.priority || 0) - (a.priority || 0));
  }

  /**
   * Get all toolbar items for a location (flattened)
   */
  getItemsByLocation(location: ToolbarLocation): ToolbarItem[] {
    const toolbars = this.getByLocation(location);
    return toolbars.flatMap(t => t.items);
  }

  /**
   * Get a specific toolbar
   */
  get(id: string): ToolbarContribution | undefined {
    return this.toolbars.get(id);
  }

  /**
   * Get all toolbars
   */
  getAll(): ToolbarContribution[] {
    return Array.from(this.toolbars.values());
  }
}

export const createToolbarRegistry = (): ToolbarExtensionRegistry => {
  return new ToolbarExtensionRegistry();
};
