/**
 * AccuScene Enterprise v0.2.0 - Plugin Events
 * Event system for plugin communication
 */

import { PluginEventEmitter, EventHandler, Disposable, PluginId } from '../types';

class PluginEventEmitterImpl implements PluginEventEmitter {
  private listeners = new Map<string, Set<EventHandler>>();

  constructor(private pluginId: PluginId) {}

  on<T = any>(event: string, handler: EventHandler<T>): Disposable {
    if (!this.listeners.has(event)) {
      this.listeners.set(event, new Set());
    }

    this.listeners.get(event)!.add(handler as EventHandler);

    return {
      dispose: () => {
        this.off(event, handler);
      },
    };
  }

  once<T = any>(event: string, handler: EventHandler<T>): Disposable {
    const wrappedHandler = async (data: T) => {
      this.off(event, wrappedHandler as EventHandler);
      await handler(data);
    };

    return this.on(event, wrappedHandler as EventHandler);
  }

  off<T = any>(event: string, handler: EventHandler<T>): void {
    this.listeners.get(event)?.delete(handler as EventHandler);
  }

  emit<T = any>(event: string, data: T): void {
    const handlers = this.listeners.get(event);

    if (handlers) {
      handlers.forEach(handler => {
        try {
          const result = handler(data);
          if (result instanceof Promise) {
            result.catch(error => {
              console.error(
                `Error in event handler for ${event} in plugin ${this.pluginId}:`,
                error
              );
            });
          }
        } catch (error) {
          console.error(
            `Error in event handler for ${event} in plugin ${this.pluginId}:`,
            error
          );
        }
      });
    }
  }

  /**
   * Get all event names that have listeners
   */
  getEventNames(): string[] {
    return Array.from(this.listeners.keys());
  }

  /**
   * Get listener count for an event
   */
  getListenerCount(event: string): number {
    return this.listeners.get(event)?.size || 0;
  }

  /**
   * Remove all listeners for an event or all events
   */
  removeAllListeners(event?: string): void {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }
}

export const createPluginEventEmitter = (pluginId: PluginId): PluginEventEmitter => {
  return new PluginEventEmitterImpl(pluginId);
};
