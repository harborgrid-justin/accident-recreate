/**
 * AccuScene Enterprise v0.2.0 - Context Menu Extensions
 * Extension point for context menus
 */

import {
  ContextMenuContribution,
  ContextMenuContext,
  MenuItem,
  PluginId,
  Disposable,
} from '../types';

export class ContextMenuExtensionRegistry {
  private contextMenus = new Map<string, ContextMenuContribution>();
  private contextMap = new Map<ContextMenuContext, Set<string>>();

  /**
   * Register a context menu contribution
   */
  register(pluginId: PluginId, contribution: ContextMenuContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.contextMenus.has(id)) {
      throw new Error(`Context menu ${id} is already registered`);
    }

    this.contextMenus.set(id, contribution);

    // Track by context
    if (!this.contextMap.has(contribution.context)) {
      this.contextMap.set(contribution.context, new Set());
    }
    this.contextMap.get(contribution.context)!.add(id);

    return {
      dispose: () => {
        this.contextMenus.delete(id);
        this.contextMap.get(contribution.context)?.delete(id);
      },
    };
  }

  /**
   * Get context menus for a specific context
   */
  getByContext(context: ContextMenuContext): ContextMenuContribution[] {
    const ids = this.contextMap.get(context) || new Set();
    const menus = Array.from(ids)
      .map(id => this.contextMenus.get(id))
      .filter((m): m is ContextMenuContribution => m !== undefined);

    // Sort by priority (higher first)
    return menus.sort((a, b) => (b.priority || 0) - (a.priority || 0));
  }

  /**
   * Get all menu items for a context (flattened)
   */
  getItemsByContext(context: ContextMenuContext): MenuItem[] {
    const menus = this.getByContext(context);
    return menus.flatMap(m => m.items);
  }

  /**
   * Get context menu items with when clause evaluation
   */
  getItemsByContextWithConditions(
    context: ContextMenuContext,
    evaluator: (when: string) => boolean
  ): MenuItem[] {
    const contributions = this.getByContext(context);
    const items: MenuItem[] = [];

    for (const contribution of contributions) {
      // Check contribution-level when clause
      if (contribution.when && !evaluator(contribution.when)) {
        continue;
      }

      // Add items that pass their when clauses
      for (const item of contribution.items) {
        if (!item.when || evaluator(item.when)) {
          items.push(item);
        }
      }
    }

    return items;
  }

  /**
   * Get a specific context menu
   */
  get(id: string): ContextMenuContribution | undefined {
    return this.contextMenus.get(id);
  }

  /**
   * Get all context menus
   */
  getAll(): ContextMenuContribution[] {
    return Array.from(this.contextMenus.values());
  }
}

export const createContextMenuRegistry = (): ContextMenuExtensionRegistry => {
  return new ContextMenuExtensionRegistry();
};
