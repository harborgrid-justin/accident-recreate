/**
 * AccuScene Enterprise v0.2.0 - Annotations Plugin
 * Built-in plugin for scene annotations
 */

import { Plugin, PluginContext, PluginManifest, PluginType } from '../types';

export const annotationsManifest: PluginManifest = {
  id: 'accuscene.builtin.annotations',
  name: 'Annotations',
  version: '1.0.0',
  description: 'Built-in annotation tools for accident scenes',
  author: 'AccuScene',
  license: 'Proprietary',
  type: PluginType.BUILTIN,
  main: 'annotations.ts',
  permissions: ['scene:write' as any, 'ui:modify' as any],
  contributes: {
    tools: [
      {
        id: 'text',
        name: 'Text Annotation',
        icon: 'text',
        category: 'annotations',
        component: null as any,
      },
      {
        id: 'arrow',
        name: 'Arrow Annotation',
        icon: 'arrow',
        category: 'annotations',
        component: null as any,
      },
      {
        id: 'marker',
        name: 'Marker',
        icon: 'marker',
        category: 'annotations',
        component: null as any,
      },
    ],
    panels: [
      {
        id: 'annotations',
        title: 'Annotations',
        icon: 'list',
        location: 'right_sidebar' as any,
        component: null as any,
      },
    ],
  },
};

export class AnnotationsPlugin implements Plugin {
  readonly manifest = annotationsManifest;
  private annotations: Annotation[] = [];

  async activate(context: PluginContext): Promise<void> {
    context.logger.info('Annotations plugin activated');

    // Load saved annotations
    const saved = await context.storage.get<Annotation[]>('annotations');
    if (saved) {
      this.annotations = saved;
    }

    // Register commands
    context.commands.register({
      id: 'annotation.add',
      title: 'Add Annotation',
      category: 'Annotations',
      handler: async (type: string, data: any) => {
        return this.addAnnotation(type, data, context);
      },
    });

    context.commands.register({
      id: 'annotation.remove',
      title: 'Remove Annotation',
      category: 'Annotations',
      handler: async (id: string) => {
        return this.removeAnnotation(id, context);
      },
    });

    context.commands.register({
      id: 'annotation.update',
      title: 'Update Annotation',
      category: 'Annotations',
      handler: async (id: string, data: any) => {
        return this.updateAnnotation(id, data, context);
      },
    });

    // Register panel
    context.ui.registerPanel({
      id: 'annotations',
      title: 'Annotations',
      icon: 'list',
      location: 'right_sidebar' as any,
      component: null as any, // Would be a React component
    });

    // Register context menu
    context.ui.registerContextMenu({
      id: 'annotations',
      context: 'scene' as any,
      items: [
        {
          id: 'add-text',
          type: 'item',
          label: 'Add Text Annotation',
          command: 'annotation.add',
        },
        {
          id: 'add-arrow',
          type: 'item',
          label: 'Add Arrow',
          command: 'annotation.add',
        },
        {
          id: 'add-marker',
          type: 'item',
          label: 'Add Marker',
          command: 'annotation.add',
        },
      ],
    });
  }

  async deactivate(context: PluginContext): Promise<void> {
    // Save annotations
    await context.storage.set('annotations', this.annotations);
    context.logger.info('Annotations plugin deactivated');
  }

  private async addAnnotation(
    type: string,
    data: any,
    context: PluginContext
  ): Promise<string> {
    const annotation: Annotation = {
      id: `annotation-${Date.now()}`,
      type,
      data,
      createdAt: Date.now(),
    };

    this.annotations.push(annotation);
    await context.storage.set('annotations', this.annotations);

    context.events.emit('annotation.added', annotation);

    return annotation.id;
  }

  private async removeAnnotation(id: string, context: PluginContext): Promise<void> {
    const index = this.annotations.findIndex(a => a.id === id);

    if (index > -1) {
      const annotation = this.annotations[index];
      this.annotations.splice(index, 1);
      await context.storage.set('annotations', this.annotations);

      context.events.emit('annotation.removed', annotation);
    }
  }

  private async updateAnnotation(
    id: string,
    data: any,
    context: PluginContext
  ): Promise<void> {
    const annotation = this.annotations.find(a => a.id === id);

    if (annotation) {
      annotation.data = { ...annotation.data, ...data };
      await context.storage.set('annotations', this.annotations);

      context.events.emit('annotation.updated', annotation);
    }
  }

  getAnnotations(): Annotation[] {
    return [...this.annotations];
  }
}

interface Annotation {
  id: string;
  type: string;
  data: any;
  createdAt: number;
}

export const createAnnotationsPlugin = (context: PluginContext): AnnotationsPlugin => {
  return new AnnotationsPlugin();
};
