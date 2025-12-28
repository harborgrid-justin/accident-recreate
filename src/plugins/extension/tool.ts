/**
 * AccuScene Enterprise v0.2.0 - Tool Extensions
 * Extension point for custom tools
 */

import {
  ToolContribution,
  PluginId,
  Disposable,
} from '../types';

export class ToolExtensionRegistry {
  private tools = new Map<string, ToolContribution>();
  private categoryMap = new Map<string, Set<string>>();
  private activeTool: string | null = null;

  /**
   * Register a tool contribution
   */
  register(pluginId: PluginId, contribution: ToolContribution): Disposable {
    const id = `${pluginId}.${contribution.id}`;

    if (this.tools.has(id)) {
      throw new Error(`Tool ${id} is already registered`);
    }

    this.tools.set(id, contribution);

    // Track by category
    if (!this.categoryMap.has(contribution.category)) {
      this.categoryMap.set(contribution.category, new Set());
    }
    this.categoryMap.get(contribution.category)!.add(id);

    return {
      dispose: () => {
        this.tools.delete(id);
        this.categoryMap.get(contribution.category)?.delete(id);

        // Clear active tool if it's being removed
        if (this.activeTool === id) {
          this.activeTool = null;
        }
      },
    };
  }

  /**
   * Get tools by category
   */
  getByCategory(category: string): ToolContribution[] {
    const ids = this.categoryMap.get(category) || new Set();

    return Array.from(ids)
      .map(id => this.tools.get(id))
      .filter((t): t is ToolContribution => t !== undefined);
  }

  /**
   * Get all tools
   */
  getAll(): ToolContribution[] {
    return Array.from(this.tools.values());
  }

  /**
   * Get a specific tool
   */
  get(id: string): ToolContribution | undefined {
    return this.tools.get(id);
  }

  /**
   * Get all categories
   */
  getCategories(): string[] {
    return Array.from(this.categoryMap.keys());
  }

  /**
   * Set the active tool
   */
  setActive(toolId: string | null): void {
    if (toolId !== null && !this.tools.has(toolId)) {
      throw new Error(`Tool ${toolId} not found`);
    }

    this.activeTool = toolId;
  }

  /**
   * Get the active tool
   */
  getActive(): ToolContribution | undefined {
    return this.activeTool ? this.tools.get(this.activeTool) : undefined;
  }

  /**
   * Get the active tool ID
   */
  getActiveId(): string | null {
    return this.activeTool;
  }

  /**
   * Check if a tool is active
   */
  isActive(toolId: string): boolean {
    return this.activeTool === toolId;
  }

  /**
   * Deactivate all tools
   */
  deactivateAll(): void {
    this.activeTool = null;
  }
}

export const createToolRegistry = (): ToolExtensionRegistry => {
  return new ToolExtensionRegistry();
};
