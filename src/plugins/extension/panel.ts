/**
 * AccuScene Enterprise v0.2.0 - Panel Extensions
 * Extension point for custom panels
 */

import {
  PanelContribution,
  PanelLocation,
  PluginId,
  Disposable,
} from '../types';

export class PanelExtensionRegistry {
  private panels = new Map<string, PanelContribution>();
  private locationMap = new Map<PanelLocation, Set<string>>();
  private activePanel = new Map<PanelLocation, string | null>();

  /**
   * Register a panel contribution
   */
  register(pluginId: PluginId, contribution: PanelContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.panels.has(id)) {
      throw new Error(`Panel ${id} is already registered`);
    }

    this.panels.set(id, contribution);

    // Track by location
    if (!this.locationMap.has(contribution.location)) {
      this.locationMap.set(contribution.location, new Set());
    }
    this.locationMap.get(contribution.location)!.add(id);

    return {
      dispose: () => {
        this.panels.delete(id);
        this.locationMap.get(contribution.location)?.delete(id);

        // Clear active panel if it's being removed
        if (this.activePanel.get(contribution.location) === id) {
          this.activePanel.set(contribution.location, null);
        }
      },
    };
  }

  /**
   * Get panels for a specific location
   */
  getByLocation(location: PanelLocation): PanelContribution[] {
    const ids = this.locationMap.get(location) || new Set();
    const panels = Array.from(ids)
      .map(id => this.panels.get(id))
      .filter((p): p is PanelContribution => p !== undefined);

    // Sort by priority (higher first)
    return panels.sort((a, b) => (b.priority || 0) - (a.priority || 0));
  }

  /**
   * Get a specific panel
   */
  get(id: string): PanelContribution | undefined {
    return this.panels.get(id);
  }

  /**
   * Get all panels
   */
  getAll(): PanelContribution[] {
    return Array.from(this.panels.values());
  }

  /**
   * Set the active panel for a location
   */
  setActive(location: PanelLocation, panelId: string | null): void {
    if (panelId !== null && !this.panels.has(panelId)) {
      throw new Error(`Panel ${panelId} not found`);
    }

    this.activePanel.set(location, panelId);
  }

  /**
   * Get the active panel for a location
   */
  getActive(location: PanelLocation): PanelContribution | undefined {
    const id = this.activePanel.get(location);
    return id ? this.panels.get(id) : undefined;
  }

  /**
   * Check if a panel is active
   */
  isActive(panelId: string): boolean {
    for (const activeId of this.activePanel.values()) {
      if (activeId === panelId) {
        return true;
      }
    }
    return false;
  }
}

export const createPanelRegistry = (): PanelExtensionRegistry => {
  return new PanelExtensionRegistry();
};
