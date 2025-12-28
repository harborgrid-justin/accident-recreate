# AccuScene Enterprise v0.2.0 - Plugin Architecture

A comprehensive, extensible plugin architecture for AccuScene Enterprise with hot-reloading, permission-based security, and marketplace integration.

## Overview

The Plugin Architecture enables third-party developers to extend AccuScene Enterprise with custom functionality while maintaining security, stability, and performance. The system provides:

- **Hot-Reloading**: Develop and test plugins without restarting the application
- **Security**: Multi-layer security with permissions, capabilities, and isolation
- **Marketplace**: Discover, install, and update plugins from a central marketplace
- **Type Safety**: Full TypeScript support with comprehensive type definitions
- **Extensibility**: Multiple extension points for UI, commands, tools, and data formats

## Architecture

```
src/plugins/
├── types.ts                  # Core type definitions
├── index.ts                  # Main exports
│
├── core/                     # Core plugin system
│   ├── manager.ts           # Plugin lifecycle orchestration
│   ├── registry.ts          # Plugin instance management
│   ├── loader.ts            # Dynamic module loading with hot-reload
│   ├── validator.ts         # Multi-level validation
│   └── sandbox.ts           # Security isolation
│
├── lifecycle/               # Lifecycle management
│   ├── hooks.ts            # Lifecycle hooks (pre/post load, activate, etc.)
│   ├── state.ts            # State machine for plugin states
│   └── events.ts           # Event system for lifecycle events
│
├── api/                     # Plugin API
│   ├── context.ts          # Plugin context factory
│   ├── services.ts         # Service registry
│   ├── storage.ts          # Persistent storage API
│   ├── events.ts           # Plugin event emitter
│   ├── ui.ts               # UI extension API
│   ├── commands.ts         # Command registration
│   └── menu.ts             # Menu extension registry
│
├── extension/               # Extension points
│   ├── toolbar.ts          # Toolbar extensions
│   ├── panel.ts            # Panel extensions
│   ├── menu.ts             # Menu extensions
│   ├── contextMenu.ts      # Context menu extensions
│   ├── exporter.ts         # Export format extensions
│   ├── importer.ts         # Import format extensions
│   └── tool.ts             # Tool extensions
│
├── manifest/                # Manifest handling
│   ├── schema.ts           # JSON schema for manifests
│   ├── parser.ts           # Manifest parser
│   └── validator.ts        # Semantic validation
│
├── security/                # Security framework
│   ├── permissions.ts      # Permission management
│   ├── capabilities.ts     # Capability detection
│   └── isolation.ts        # Isolation policies
│
├── store/                   # Plugin marketplace
│   ├── marketplace.ts      # Marketplace integration
│   ├── installation.ts     # Install/uninstall with progress
│   └── updates.ts          # Update management
│
├── builtin/                 # Built-in plugins
│   ├── measurements.ts     # Measurement tools
│   ├── annotations.ts      # Annotation tools
│   └── exports.ts          # Export formats
│
├── template/                # Plugin template
│   └── index.ts            # Developer template
│
└── utils/                   # Utilities
    └── logger.ts           # Plugin logger
```

## Quick Start

### Using the Plugin System

```typescript
import { initializePluginSystem } from './plugins';

// Initialize the plugin system
const pluginManager = await initializePluginSystem({
  pluginDirectory: '/plugins',
  autoLoad: true,
  autoActivate: true,
  hotReload: true,
  maxPlugins: 100,
  timeout: 30000,
});

// Install a plugin
await pluginManager.install('/plugins/my-plugin');

// Activate a plugin
await pluginManager.activate('com.example.my-plugin');

// Get active plugins
const activePlugins = pluginManager.getActivePlugins();

// Listen to plugin events
pluginManager.on('plugin:activated', ({ pluginId }) => {
  console.log(`Plugin ${pluginId} activated`);
});
```

### Creating a Plugin

```typescript
import { Plugin, PluginContext, PluginManifest } from './plugins';

// Define the manifest
export const manifest: PluginManifest = {
  id: 'com.example.my-plugin',
  name: 'My Plugin',
  version: '1.0.0',
  description: 'My awesome plugin',
  author: 'Your Name',
  license: 'MIT',
  type: 'extension',
  main: 'index.ts',
  permissions: ['storage:read', 'storage:write', 'ui:modify'],
  capabilities: ['hot_reload'],
};

// Implement the plugin
export class MyPlugin implements Plugin {
  readonly manifest = manifest;

  async activate(context: PluginContext): Promise<void> {
    // Register commands
    context.commands.register({
      id: 'my-command',
      title: 'My Command',
      handler: async () => {
        context.ui.showNotification({
          type: 'success',
          title: 'Success',
          message: 'Command executed!',
        });
      },
    });

    // Add toolbar button
    context.ui.registerToolbar({
      id: 'my-toolbar',
      location: 'top',
      items: [{
        id: 'my-button',
        type: 'button',
        label: 'My Button',
        command: 'my-command',
      }],
    });

    // Save data
    await context.storage.set('my-data', { initialized: true });
  }

  async deactivate(context: PluginContext): Promise<void> {
    // Cleanup
  }
}

// Export factory function
export default function createPlugin(context: PluginContext): MyPlugin {
  return new MyPlugin();
}
```

## Security Model

### Permissions

Plugins must request permissions in their manifest:

- `storage:read` / `storage:write` - Access to persistent storage
- `file:read` / `file:write` - File system access
- `network` - Network requests
- `command:execute` / `command:register` - Command execution/registration
- `ui:modify` - Modify the UI
- `scene:read` / `scene:write` - Access to scene data
- `clipboard` - Clipboard access
- `notifications` - Show notifications

### Capabilities

System capabilities that plugins can use:

- `hot_reload` - Support hot reloading
- `background_task` - Run background tasks
- `worker_thread` - Use worker threads
- `native_module` - Load native modules
- `webgl` / `webgpu` - Use WebGL/WebGPU
- `web_worker` - Use Web Workers

### Isolation Levels

- **None**: No isolation (trusted plugins only)
- **Partial**: Limited isolation with basic restrictions
- **Full**: Complete isolation with strict sandboxing

## Extension Points

### Commands

Register custom commands:

```typescript
context.commands.register({
  id: 'my-command',
  title: 'My Command',
  category: 'My Plugin',
  handler: async (...args) => {
    // Command implementation
  },
});
```

### Toolbars

Add custom toolbar items:

```typescript
context.ui.registerToolbar({
  id: 'my-toolbar',
  location: 'top', // top, left, right, bottom
  items: [{
    id: 'my-button',
    type: 'button',
    label: 'Click Me',
    icon: 'star',
    command: 'my-command',
  }],
});
```

### Panels

Add custom panels:

```typescript
context.ui.registerPanel({
  id: 'my-panel',
  title: 'My Panel',
  location: 'right_sidebar', // left_sidebar, right_sidebar, bottom_panel, floating
  component: MyPanelComponent,
});
```

### Menus

Add menu items:

```typescript
context.ui.registerMenuItem({
  id: 'my-menu',
  location: 'tools', // main, file, edit, view, tools, help
  items: [{
    id: 'my-item',
    type: 'item',
    label: 'My Menu Item',
    command: 'my-command',
  }],
});
```

### Context Menus

Add context menu items:

```typescript
context.ui.registerContextMenu({
  id: 'my-context-menu',
  context: 'scene', // scene, object, timeline, canvas, editor
  items: [{
    id: 'my-item',
    type: 'item',
    label: 'My Context Item',
    command: 'my-command',
  }],
});
```

### Export Formats

Add custom export formats:

```typescript
// In manifest
contributes: {
  exporters: [{
    id: 'my-format',
    name: 'My Format',
    extensions: ['myformat'],
    mimeTypes: ['application/x-myformat'],
    export: async (data, options) => {
      // Export implementation
      return new Blob([data]);
    },
  }],
}
```

### Import Formats

Add custom import formats:

```typescript
// In manifest
contributes: {
  importers: [{
    id: 'my-format',
    name: 'My Format',
    extensions: ['myformat'],
    mimeTypes: ['application/x-myformat'],
    import: async (file, options) => {
      // Import implementation
      return parsedData;
    },
  }],
}
```

### Tools

Add custom scene tools:

```typescript
// In manifest
contributes: {
  tools: [{
    id: 'my-tool',
    name: 'My Tool',
    icon: 'tool',
    category: 'Custom Tools',
    component: MyToolComponent,
  }],
}
```

## Plugin Lifecycle

```
UNLOADED → LOADING → LOADED → INITIALIZING → ACTIVE
    ↑                                           ↓
    └────────────── UNLOADING ← PAUSED ←───────┘
                                  ↕
                               ERROR
```

### Lifecycle Hooks

```typescript
import { LifecycleHookType } from './plugins';

// Register lifecycle hooks
lifecycleManager.registerHook({
  type: LifecycleHookType.PRE_ACTIVATE,
  handler: async (pluginId, context) => {
    console.log(`About to activate ${pluginId}`);
  },
  priority: 10,
});
```

## Storage API

```typescript
// Save data
await context.storage.set('my-key', { foo: 'bar' });

// Load data
const data = await context.storage.get('my-key');

// Delete data
await context.storage.delete('my-key');

// List keys
const keys = await context.storage.keys();

// Clear all
await context.storage.clear();
```

## Event System

```typescript
// Listen to events
context.events.on('my-event', (data) => {
  console.log('Event received:', data);
});

// Emit events
context.events.emit('my-event', { foo: 'bar' });

// Listen once
context.events.once('my-event', (data) => {
  console.log('Event received once:', data);
});
```

## Marketplace Integration

```typescript
import { createMarketplace } from './plugins/store';

// Create marketplace client
const marketplace = createMarketplace({
  apiEndpoint: 'https://marketplace.accuscene.com/api',
  apiKey: 'your-api-key',
});

// Search for plugins
const results = await marketplace.search('measurement', {
  category: 'tools',
  verified: true,
  minRating: 4.0,
});

// Get plugin details
const plugin = await marketplace.getPlugin('com.example.my-plugin');

// Get download URL
const url = await marketplace.getDownloadUrl('com.example.my-plugin');
```

## Built-in Plugins

### Measurements Plugin
Provides measurement tools for distance, angle, and area calculations.

### Annotations Plugin
Provides annotation tools for text, arrows, and markers.

### Exports Plugin
Provides export functionality for PDF, JSON, PNG, and GLB formats.

## Best Practices

1. **Always validate user input** in your plugin
2. **Handle errors gracefully** and show user-friendly messages
3. **Clean up resources** in the deactivate method
4. **Use TypeScript** for type safety
5. **Request minimal permissions** needed for your plugin
6. **Document your plugin** in the manifest and README
7. **Test with hot-reload** during development
8. **Version your plugin** using semantic versioning
9. **Handle state changes** in onStateChange callback
10. **Use the logger** for debugging and error reporting

## Development

See the [template plugin](./template/index.ts) for a complete example.

## License

Proprietary - AccuScene Enterprise v0.2.0

## Support

For plugin development support, visit: https://docs.accuscene.com/plugins
