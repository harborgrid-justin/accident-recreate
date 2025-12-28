/**
 * AccuScene Enterprise v0.2.0 - Plugin UI API
 * UI extension API for plugins
 */

import {
  PluginUI,
  PluginId,
  ToolbarContribution,
  PanelContribution,
  MenuContribution,
  ContextMenuContribution,
  Notification,
  Dialog,
  DialogResult,
  Disposable,
} from '../types';

class PluginUIImpl implements PluginUI {
  private toolbars = new Map<string, ToolbarContribution>();
  private panels = new Map<string, PanelContribution>();
  private menus = new Map<string, MenuContribution>();
  private contextMenus = new Map<string, ContextMenuContribution>();

  constructor(
    private pluginId: PluginId,
    private manager: any
  ) {}

  registerToolbar(contribution: ToolbarContribution): Disposable {
    const id = `${this.pluginId}.${contribution.id}`;
    this.toolbars.set(id, contribution);

    // Notify UI system of new toolbar
    this.notifyUIUpdate('toolbar.register', contribution);

    return {
      dispose: () => {
        this.toolbars.delete(id);
        this.notifyUIUpdate('toolbar.unregister', contribution);
      },
    };
  }

  registerPanel(contribution: PanelContribution): Disposable {
    const id = `${this.pluginId}.${contribution.id}`;
    this.panels.set(id, contribution);

    // Notify UI system of new panel
    this.notifyUIUpdate('panel.register', contribution);

    return {
      dispose: () => {
        this.panels.delete(id);
        this.notifyUIUpdate('panel.unregister', contribution);
      },
    };
  }

  registerMenuItem(contribution: MenuContribution): Disposable {
    const id = `${this.pluginId}.${contribution.id}`;
    this.menus.set(id, contribution);

    // Notify UI system of new menu item
    this.notifyUIUpdate('menu.register', contribution);

    return {
      dispose: () => {
        this.menus.delete(id);
        this.notifyUIUpdate('menu.unregister', contribution);
      },
    };
  }

  registerContextMenu(contribution: ContextMenuContribution): Disposable {
    const id = `${this.pluginId}.${contribution.id}`;
    this.contextMenus.set(id, contribution);

    // Notify UI system of new context menu
    this.notifyUIUpdate('contextMenu.register', contribution);

    return {
      dispose: () => {
        this.contextMenus.delete(id);
        this.notifyUIUpdate('contextMenu.unregister', contribution);
      },
    };
  }

  showNotification(notification: Notification): void {
    // In a real implementation, this would integrate with a notification system
    console.log(`[${this.pluginId}] ${notification.type.toUpperCase()}: ${notification.title}`);
    console.log(notification.message);

    // Emit to global notification system
    this.notifyUIUpdate('notification.show', notification);
  }

  async showDialog(dialog: Dialog): Promise<DialogResult> {
    // In a real implementation, this would show a modal dialog
    console.log(`[${this.pluginId}] Dialog: ${dialog.title}`);
    console.log(dialog.message);

    // Emit to global dialog system
    this.notifyUIUpdate('dialog.show', dialog);

    // Return a mock result
    return {
      confirmed: false,
    };
  }

  /**
   * Get all registered toolbars
   */
  getToolbars(): ToolbarContribution[] {
    return Array.from(this.toolbars.values());
  }

  /**
   * Get all registered panels
   */
  getPanels(): PanelContribution[] {
    return Array.from(this.panels.values());
  }

  /**
   * Get all registered menus
   */
  getMenus(): MenuContribution[] {
    return Array.from(this.menus.values());
  }

  /**
   * Get all registered context menus
   */
  getContextMenus(): ContextMenuContribution[] {
    return Array.from(this.contextMenus.values());
  }

  private notifyUIUpdate(type: string, data: any): void {
    // Emit event to manager for UI system integration
    if (this.manager && this.manager.emit) {
      this.manager.emit(`ui:${type}`, { pluginId: this.pluginId, data });
    }
  }
}

export const createPluginUI = (pluginId: PluginId, manager: any): PluginUI => {
  return new PluginUIImpl(pluginId, manager);
};
