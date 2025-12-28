/**
 * AccuScene Enterprise v0.2.0 - Plugin Template
 * Template for creating new plugins
 */

import { Plugin, PluginContext, PluginManifest, PluginType } from '../types';

/**
 * Plugin manifest
 * Update this with your plugin information
 */
export const manifest: PluginManifest = {
  id: 'com.example.my-plugin',
  name: 'My Plugin',
  version: '1.0.0',
  description: 'A brief description of what this plugin does',
  author: 'Your Name',
  license: 'MIT',
  type: PluginType.EXTENSION,
  main: 'index.ts',

  // Optional: Specify dependencies
  dependencies: [
    // { id: 'com.example.required-plugin', version: '^1.0.0' }
  ],

  // Optional: Specify permissions needed
  permissions: [
    // 'storage:read',
    // 'storage:write',
    // 'ui:modify',
  ],

  // Optional: Specify capabilities
  capabilities: [
    // 'hot_reload',
    // 'background_task',
  ],

  // Optional: Specify UI contributions
  contributes: {
    // Commands
    commands: [
      {
        id: 'my-command',
        title: 'My Command',
        category: 'My Plugin',
        handler: null as any, // Will be set in activate()
      },
    ],

    // Toolbar items
    // toolbars: [{
    //   id: 'my-toolbar',
    //   location: 'top',
    //   items: [{
    //     id: 'my-button',
    //     type: 'button',
    //     label: 'My Button',
    //     command: 'my-command',
    //   }],
    // }],

    // Panels
    // panels: [{
    //   id: 'my-panel',
    //   title: 'My Panel',
    //   location: 'right_sidebar',
    //   component: MyPanelComponent,
    // }],

    // Menu items
    // menus: [{
    //   id: 'my-menu',
    //   location: 'tools',
    //   items: [{
    //     id: 'my-item',
    //     type: 'item',
    //     label: 'My Menu Item',
    //     command: 'my-command',
    //   }],
    // }],
  },
};

/**
 * Main plugin class
 */
export class MyPlugin implements Plugin {
  readonly manifest = manifest;
  private context?: PluginContext;

  /**
   * Called when the plugin is activated
   */
  async activate(context: PluginContext): Promise<void> {
    this.context = context;

    // Log activation
    context.logger.info('Plugin activated');

    // Register commands
    context.commands.register({
      id: 'my-command',
      title: 'My Command',
      category: 'My Plugin',
      handler: async (...args: any[]) => {
        return this.handleCommand(...args);
      },
    });

    // Register event listeners
    context.events.on('my-event', (data) => {
      this.handleEvent(data);
    });

    // Load saved data
    const savedData = await context.storage.get('my-data');
    if (savedData) {
      context.logger.info('Loaded saved data', savedData);
    }

    // Show activation notification
    context.ui.showNotification({
      type: 'success',
      title: 'Plugin Activated',
      message: 'My Plugin is now active',
      duration: 3000,
    });
  }

  /**
   * Called when the plugin is deactivated
   */
  async deactivate(context: PluginContext): Promise<void> {
    context.logger.info('Plugin deactivated');

    // Save data before deactivation
    await context.storage.set('my-data', {
      timestamp: Date.now(),
    });

    // Clean up resources
    this.cleanup();
  }

  /**
   * Handle plugin command
   */
  private async handleCommand(...args: any[]): Promise<any> {
    if (!this.context) return;

    this.context.logger.info('Command executed', args);

    // Implement command logic here

    return { success: true };
  }

  /**
   * Handle plugin event
   */
  private handleEvent(data: any): void {
    if (!this.context) return;

    this.context.logger.info('Event received', data);

    // Implement event handling here
  }

  /**
   * Clean up resources
   */
  private cleanup(): void {
    // Clean up any resources, listeners, etc.
  }
}

/**
 * Plugin factory function
 * This is the main export that the plugin system will call
 */
export default function createPlugin(context: PluginContext): MyPlugin {
  return new MyPlugin();
}

// Alternative export formats that are also supported:

// Named export
// export const plugin = new MyPlugin();

// Factory function
// export function createPlugin(context: PluginContext): Plugin {
//   return new MyPlugin();
// }
