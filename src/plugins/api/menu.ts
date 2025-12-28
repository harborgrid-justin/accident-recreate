/**
 * AccuScene Enterprise v0.2.0 - Menu Extension API
 * API for extending application menus
 */

import {
  MenuContribution,
  MenuItem,
  MenuLocation,
  PluginId,
  Disposable,
} from '../types';

export class MenuRegistry {
  private menus = new Map<string, MenuContribution[]>();
  private itemIds = new Set<string>();

  /**
   * Register a menu contribution
   */
  registerMenu(pluginId: PluginId, contribution: MenuContribution): Disposable {
    const fullId = `${pluginId}.${contribution.id}`;

    // Validate menu items
    this.validateMenuItems(contribution.items, fullId);

    // Get or create menu list for this location
    const location = contribution.location;
    if (!this.menus.has(location)) {
      this.menus.set(location, []);
    }

    this.menus.get(location)!.push(contribution);

    // Sort by priority (higher priority first)
    this.menus.set(
      location,
      this.menus.get(location)!.sort((a, b) => (b.priority || 0) - (a.priority || 0))
    );

    // Track item IDs
    this.trackMenuItems(contribution.items, fullId);

    return {
      dispose: () => {
        this.unregisterMenu(location, contribution);
        this.untrackMenuItems(contribution.items, fullId);
      },
    };
  }

  /**
   * Get all menu contributions for a location
   */
  getMenus(location: MenuLocation): MenuContribution[] {
    return this.menus.get(location) || [];
  }

  /**
   * Get all menu items for a location (flattened)
   */
  getMenuItems(location: MenuLocation): MenuItem[] {
    const contributions = this.getMenus(location);
    return contributions.flatMap(c => c.items);
  }

  /**
   * Get a menu item by ID
   */
  getMenuItem(itemId: string): MenuItem | undefined {
    for (const contributions of this.menus.values()) {
      for (const contribution of contributions) {
        const item = this.findMenuItem(contribution.items, itemId);
        if (item) return item;
      }
    }
    return undefined;
  }

  /**
   * Check if a menu item exists
   */
  hasMenuItem(itemId: string): boolean {
    return this.itemIds.has(itemId);
  }

  private validateMenuItems(items: MenuItem[], prefix: string): void {
    for (const item of items) {
      const itemId = `${prefix}.${item.id}`;

      if (this.itemIds.has(itemId) && item.type !== 'separator') {
        throw new Error(`Menu item ${itemId} is already registered`);
      }

      if (item.submenu) {
        this.validateMenuItems(item.submenu, prefix);
      }
    }
  }

  private trackMenuItems(items: MenuItem[], prefix: string): void {
    for (const item of items) {
      const itemId = `${prefix}.${item.id}`;
      this.itemIds.add(itemId);

      if (item.submenu) {
        this.trackMenuItems(item.submenu, prefix);
      }
    }
  }

  private untrackMenuItems(items: MenuItem[], prefix: string): void {
    for (const item of items) {
      const itemId = `${prefix}.${item.id}`;
      this.itemIds.delete(itemId);

      if (item.submenu) {
        this.untrackMenuItems(item.submenu, prefix);
      }
    }
  }

  private findMenuItem(items: MenuItem[], itemId: string): MenuItem | undefined {
    for (const item of items) {
      if (item.id === itemId) {
        return item;
      }

      if (item.submenu) {
        const found = this.findMenuItem(item.submenu, itemId);
        if (found) return found;
      }
    }
    return undefined;
  }

  private unregisterMenu(location: MenuLocation, contribution: MenuContribution): void {
    const menus = this.menus.get(location);
    if (menus) {
      const index = menus.indexOf(contribution);
      if (index > -1) {
        menus.splice(index, 1);
      }
    }
  }
}

export const createMenuRegistry = (): MenuRegistry => {
  return new MenuRegistry();
};
