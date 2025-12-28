/**
 * AccuScene Enterprise v0.2.0 - Menu Extensions
 * Extension point for menu items
 */

import {
  MenuContribution,
  MenuItem,
  MenuLocation,
  PluginId,
  Disposable,
} from '../types';

export class MenuExtensionRegistry {
  private menus = new Map<string, MenuContribution>();
  private locationMap = new Map<MenuLocation, Set<string>>();

  /**
   * Register a menu contribution
   */
  register(pluginId: PluginId, contribution: MenuContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.menus.has(id)) {
      throw new Error(`Menu ${id} is already registered`);
    }

    this.menus.set(id, contribution);

    // Track by location
    if (!this.locationMap.has(contribution.location)) {
      this.locationMap.set(contribution.location, new Set());
    }
    this.locationMap.get(contribution.location)!.add(id);

    return {
      dispose: () => {
        this.menus.delete(id);
        this.locationMap.get(contribution.location)?.delete(id);
      },
    };
  }

  /**
   * Get menus for a specific location
   */
  getByLocation(location: MenuLocation): MenuContribution[] {
    const ids = this.locationMap.get(location) || new Set();
    const menus = Array.from(ids)
      .map(id => this.menus.get(id))
      .filter((m): m is MenuContribution => m !== undefined);

    // Sort by priority (higher first)
    return menus.sort((a, b) => (b.priority || 0) - (a.priority || 0));
  }

  /**
   * Get all menu items for a location (flattened)
   */
  getItemsByLocation(location: MenuLocation): MenuItem[] {
    const menus = this.getByLocation(location);
    return menus.flatMap(m => m.items);
  }

  /**
   * Get a specific menu
   */
  get(id: string): MenuContribution | undefined {
    return this.menus.get(id);
  }

  /**
   * Get all menus
   */
  getAll(): MenuContribution[] {
    return Array.from(this.menus.values());
  }

  /**
   * Find a menu item by ID recursively
   */
  findMenuItem(itemId: string): MenuItem | undefined {
    for (const menu of this.menus.values()) {
      const found = this.searchItems(menu.items, itemId);
      if (found) return found;
    }
    return undefined;
  }

  private searchItems(items: MenuItem[], itemId: string): MenuItem | undefined {
    for (const item of items) {
      if (item.id === itemId) {
        return item;
      }
      if (item.submenu) {
        const found = this.searchItems(item.submenu, itemId);
        if (found) return found;
      }
    }
    return undefined;
  }
}

export const createMenuExtensionRegistry = (): MenuExtensionRegistry => {
  return new MenuExtensionRegistry();
};
