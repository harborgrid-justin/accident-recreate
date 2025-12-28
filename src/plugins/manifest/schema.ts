/**
 * AccuScene Enterprise v0.2.0 - Manifest Schema
 * JSON schema for plugin manifests
 */

export const MANIFEST_SCHEMA_VERSION = '1.0.0';

export const PluginManifestSchema = {
  $schema: 'http://json-schema.org/draft-07/schema#',
  type: 'object',
  required: ['id', 'name', 'version', 'description', 'author', 'license', 'main'],
  properties: {
    id: {
      type: 'string',
      pattern: '^[a-z0-9-]+(\\.[a-z0-9-]+)*$',
      description: 'Unique identifier for the plugin',
    },
    name: {
      type: 'string',
      minLength: 1,
      maxLength: 100,
      description: 'Human-readable name of the plugin',
    },
    version: {
      type: 'string',
      pattern: '^\\d+\\.\\d+\\.\\d+',
      description: 'Semantic version of the plugin',
    },
    description: {
      type: 'string',
      minLength: 1,
      maxLength: 500,
      description: 'Brief description of the plugin',
    },
    author: {
      type: 'string',
      minLength: 1,
      description: 'Plugin author name or organization',
    },
    license: {
      type: 'string',
      description: 'License identifier (SPDX)',
    },
    homepage: {
      type: 'string',
      format: 'uri',
      description: 'Homepage URL',
    },
    repository: {
      type: 'string',
      format: 'uri',
      description: 'Repository URL',
    },
    keywords: {
      type: 'array',
      items: {
        type: 'string',
      },
      maxItems: 20,
      description: 'Search keywords',
    },
    type: {
      type: 'string',
      enum: ['core', 'builtin', 'extension', 'third_party'],
      description: 'Plugin type',
    },
    category: {
      type: 'string',
      description: 'Plugin category',
    },
    icon: {
      type: 'string',
      description: 'Icon URL or path',
    },
    main: {
      type: 'string',
      pattern: '\\.(js|ts|mjs)$',
      description: 'Main entry point file',
    },
    dependencies: {
      type: 'array',
      items: {
        type: 'object',
        required: ['id', 'version'],
        properties: {
          id: {
            type: 'string',
          },
          version: {
            type: 'string',
          },
          optional: {
            type: 'boolean',
          },
        },
      },
    },
    peerDependencies: {
      type: 'array',
      items: {
        type: 'object',
        required: ['id', 'version'],
        properties: {
          id: {
            type: 'string',
          },
          version: {
            type: 'string',
          },
          optional: {
            type: 'boolean',
          },
        },
      },
    },
    permissions: {
      type: 'array',
      items: {
        type: 'string',
        enum: [
          'storage:read',
          'storage:write',
          'file:read',
          'file:write',
          'network',
          'command:execute',
          'command:register',
          'ui:modify',
          'scene:read',
          'scene:write',
          'clipboard',
          'notifications',
        ],
      },
    },
    capabilities: {
      type: 'array',
      items: {
        type: 'string',
        enum: [
          'hot_reload',
          'background_task',
          'worker_thread',
          'native_module',
          'webgl',
          'webgpu',
          'web_worker',
        ],
      },
    },
    exports: {
      type: 'object',
      additionalProperties: {
        type: 'string',
      },
    },
    contributes: {
      type: 'object',
      properties: {
        toolbars: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        panels: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        menus: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        contextMenus: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        commands: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        exporters: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        importers: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
        tools: {
          type: 'array',
          items: {
            type: 'object',
          },
        },
      },
    },
    engines: {
      type: 'object',
      properties: {
        accuscene: {
          type: 'string',
          description: 'AccuScene version constraint',
        },
        node: {
          type: 'string',
          description: 'Node.js version constraint',
        },
      },
    },
  },
};

export const validateManifestSchema = (manifest: any): { valid: boolean; errors: string[] } => {
  const errors: string[] = [];

  // Basic validation (in production, use a JSON schema validator library)
  const required = ['id', 'name', 'version', 'description', 'author', 'license', 'main'];

  for (const field of required) {
    if (!manifest[field]) {
      errors.push(`Missing required field: ${field}`);
    }
  }

  // Validate ID format
  if (manifest.id && !/^[a-z0-9-]+(\.[a-z0-9-]+)*$/.test(manifest.id)) {
    errors.push('Invalid ID format');
  }

  // Validate version format
  if (manifest.version && !/^\d+\.\d+\.\d+/.test(manifest.version)) {
    errors.push('Invalid version format (must be semantic version)');
  }

  // Validate main entry
  if (manifest.main && !/\.(js|ts|mjs)$/.test(manifest.main)) {
    errors.push('Invalid main entry (must be .js, .ts, or .mjs file)');
  }

  return {
    valid: errors.length === 0,
    errors,
  };
};
