/**
 * AccuScene Enterprise v0.3.0 - Dashboard Serializer
 * Save and load dashboard layouts with validation
 */

import { DashboardLayout } from './types';

const STORAGE_KEY_PREFIX = 'accuscene-dashboard-';
const STORAGE_KEY_LIST = 'accuscene-dashboard-list';

export class DashboardSerializer {
  /**
   * Serialize dashboard layout to JSON string
   */
  static serialize(layout: DashboardLayout): string {
    return JSON.stringify(layout, null, 2);
  }

  /**
   * Deserialize JSON string to dashboard layout
   */
  static deserialize(json: string): DashboardLayout {
    try {
      const layout = JSON.parse(json);
      this.validateLayout(layout);
      return layout;
    } catch (error) {
      throw new Error(`Failed to deserialize dashboard layout: ${error}`);
    }
  }

  /**
   * Validate dashboard layout structure
   */
  private static validateLayout(layout: any): void {
    if (!layout || typeof layout !== 'object') {
      throw new Error('Invalid layout: must be an object');
    }

    if (!layout.id || typeof layout.id !== 'string') {
      throw new Error('Invalid layout: missing or invalid id');
    }

    if (!layout.name || typeof layout.name !== 'string') {
      throw new Error('Invalid layout: missing or invalid name');
    }

    if (!Array.isArray(layout.widgets)) {
      throw new Error('Invalid layout: widgets must be an array');
    }

    if (!layout.gridSettings || typeof layout.gridSettings !== 'object') {
      throw new Error('Invalid layout: missing or invalid gridSettings');
    }

    // Validate widgets
    layout.widgets.forEach((widget: any, index: number) => {
      if (!widget.id || typeof widget.id !== 'string') {
        throw new Error(`Invalid widget at index ${index}: missing or invalid id`);
      }

      if (!widget.type || typeof widget.type !== 'string') {
        throw new Error(`Invalid widget at index ${index}: missing or invalid type`);
      }

      if (!widget.position || typeof widget.position !== 'object') {
        throw new Error(`Invalid widget at index ${index}: missing or invalid position`);
      }

      if (
        typeof widget.position.x !== 'number' ||
        typeof widget.position.y !== 'number' ||
        typeof widget.position.w !== 'number' ||
        typeof widget.position.h !== 'number'
      ) {
        throw new Error(`Invalid widget at index ${index}: invalid position coordinates`);
      }
    });
  }

  /**
   * Save layout to localStorage
   */
  static saveLayout(layout: DashboardLayout): boolean {
    try {
      const json = this.serialize(layout);
      const key = `${STORAGE_KEY_PREFIX}${layout.id}`;

      localStorage.setItem(key, json);

      // Update layout list
      this.addToLayoutList(layout.id, layout.name);

      return true;
    } catch (error) {
      console.error('Failed to save dashboard layout:', error);
      return false;
    }
  }

  /**
   * Load layout from localStorage
   */
  static loadLayout(id: string): DashboardLayout | null {
    try {
      const key = `${STORAGE_KEY_PREFIX}${id}`;
      const json = localStorage.getItem(key);

      if (!json) {
        return null;
      }

      return this.deserialize(json);
    } catch (error) {
      console.error('Failed to load dashboard layout:', error);
      return null;
    }
  }

  /**
   * Delete layout from localStorage
   */
  static deleteLayout(id: string): boolean {
    try {
      const key = `${STORAGE_KEY_PREFIX}${id}`;
      localStorage.removeItem(key);

      // Update layout list
      this.removeFromLayoutList(id);

      return true;
    } catch (error) {
      console.error('Failed to delete dashboard layout:', error);
      return false;
    }
  }

  /**
   * Get all saved layout IDs and names
   */
  static getLayoutList(): Array<{ id: string; name: string }> {
    try {
      const json = localStorage.getItem(STORAGE_KEY_LIST);
      if (!json) {
        return [];
      }

      return JSON.parse(json);
    } catch (error) {
      console.error('Failed to get layout list:', error);
      return [];
    }
  }

  /**
   * Add layout to list
   */
  private static addToLayoutList(id: string, name: string): void {
    const list = this.getLayoutList();

    // Check if already exists
    const existingIndex = list.findIndex((item) => item.id === id);

    if (existingIndex >= 0) {
      // Update name
      list[existingIndex].name = name;
    } else {
      // Add new entry
      list.push({ id, name });
    }

    localStorage.setItem(STORAGE_KEY_LIST, JSON.stringify(list));
  }

  /**
   * Remove layout from list
   */
  private static removeFromLayoutList(id: string): void {
    const list = this.getLayoutList();
    const filtered = list.filter((item) => item.id !== id);
    localStorage.setItem(STORAGE_KEY_LIST, JSON.stringify(filtered));
  }

  /**
   * Export layout to file
   */
  static exportToFile(layout: DashboardLayout): void {
    const json = this.serialize(layout);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `dashboard-${layout.id}-${Date.now()}.json`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  /**
   * Import layout from file
   */
  static async importFromFile(file: File): Promise<DashboardLayout> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = (event) => {
        try {
          const json = event.target?.result as string;
          const layout = this.deserialize(json);
          resolve(layout);
        } catch (error) {
          reject(error);
        }
      };

      reader.onerror = () => {
        reject(new Error('Failed to read file'));
      };

      reader.readAsText(file);
    });
  }

  /**
   * Clone a layout with new ID
   */
  static cloneLayout(
    layout: DashboardLayout,
    newName?: string
  ): DashboardLayout {
    const cloned: DashboardLayout = {
      ...layout,
      id: `${layout.id}-copy-${Date.now()}`,
      name: newName || `${layout.name} (Copy)`,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      widgets: layout.widgets.map((widget) => ({
        ...widget,
        id: `${widget.id}-copy-${Math.random().toString(36).substr(2, 9)}`,
      })),
    };

    return cloned;
  }

  /**
   * Merge two layouts
   */
  static mergeLayouts(
    layout1: DashboardLayout,
    layout2: DashboardLayout
  ): DashboardLayout {
    return {
      ...layout1,
      id: `merged-${Date.now()}`,
      name: `${layout1.name} + ${layout2.name}`,
      widgets: [...layout1.widgets, ...layout2.widgets],
      updatedAt: Date.now(),
    };
  }

  /**
   * Get layout size (estimated storage size in bytes)
   */
  static getLayoutSize(layout: DashboardLayout): number {
    const json = this.serialize(layout);
    return new Blob([json]).size;
  }

  /**
   * Compress layout (remove unnecessary data)
   */
  static compressLayout(layout: DashboardLayout): DashboardLayout {
    return {
      id: layout.id,
      name: layout.name,
      description: layout.description,
      widgets: layout.widgets.map((widget) => ({
        id: widget.id,
        type: widget.type,
        title: widget.title,
        position: widget.position,
        settings: widget.settings,
        isVisible: widget.isVisible,
      })),
      gridSettings: layout.gridSettings,
      createdAt: layout.createdAt,
      updatedAt: layout.updatedAt,
    };
  }

  /**
   * Get storage usage statistics
   */
  static getStorageStats(): {
    totalLayouts: number;
    totalSize: number;
    layouts: Array<{ id: string; name: string; size: number }>;
  } {
    const list = this.getLayoutList();
    const layouts = list.map((item) => {
      const layout = this.loadLayout(item.id);
      return {
        id: item.id,
        name: item.name,
        size: layout ? this.getLayoutSize(layout) : 0,
      };
    });

    const totalSize = layouts.reduce((sum, layout) => sum + layout.size, 0);

    return {
      totalLayouts: list.length,
      totalSize,
      layouts,
    };
  }

  /**
   * Clear all saved layouts
   */
  static clearAllLayouts(): void {
    const list = this.getLayoutList();

    list.forEach((item) => {
      this.deleteLayout(item.id);
    });

    localStorage.removeItem(STORAGE_KEY_LIST);
  }
}
