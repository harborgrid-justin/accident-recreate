/**
 * AccuScene Enterprise v0.2.0 - Lifecycle Events
 * Event system for plugin lifecycle
 */

import { PluginId, PluginEvent, EventHandler, Disposable } from '../types';

export class PluginLifecycleEventEmitter {
  private listeners = new Map<string, Set<EventHandler>>();
  private eventHistory: PluginEvent[] = [];
  private maxHistorySize = 1000;

  /**
   * Subscribe to an event
   */
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

  /**
   * Subscribe to an event once
   */
  once<T = any>(event: string, handler: EventHandler<T>): Disposable {
    const wrappedHandler = async (data: T) => {
      this.off(event, wrappedHandler as EventHandler);
      await handler(data);
    };

    return this.on(event, wrappedHandler as EventHandler);
  }

  /**
   * Unsubscribe from an event
   */
  off<T = any>(event: string, handler: EventHandler<T>): void {
    this.listeners.get(event)?.delete(handler as EventHandler);
  }

  /**
   * Emit an event
   */
  emit<T = any>(event: string, data: T, pluginId?: PluginId): void {
    const pluginEvent: PluginEvent<T> = {
      type: event,
      pluginId: pluginId || 'system',
      timestamp: Date.now(),
      data,
    };

    // Add to history
    this.addToHistory(pluginEvent);

    // Notify listeners
    const handlers = this.listeners.get(event);
    if (handlers) {
      handlers.forEach(handler => {
        try {
          const result = handler(data);
          if (result instanceof Promise) {
            result.catch(error => {
              console.error(`Error in event handler for ${event}:`, error);
            });
          }
        } catch (error) {
          console.error(`Error in event handler for ${event}:`, error);
        }
      });
    }
  }

  /**
   * Wait for an event to be emitted
   */
  async waitFor<T = any>(event: string, timeout?: number): Promise<T> {
    return new Promise((resolve, reject) => {
      let timer: NodeJS.Timeout | undefined;

      const disposable = this.once(event, (data: T) => {
        if (timer) clearTimeout(timer);
        resolve(data);
      });

      if (timeout) {
        timer = setTimeout(() => {
          disposable.dispose();
          reject(new Error(`Timeout waiting for event: ${event}`));
        }, timeout);
      }
    });
  }

  /**
   * Get event history
   */
  getHistory(filter?: { event?: string; pluginId?: PluginId; since?: number }): PluginEvent[] {
    let history = this.eventHistory;

    if (filter) {
      if (filter.event) {
        history = history.filter(e => e.type === filter.event);
      }
      if (filter.pluginId) {
        history = history.filter(e => e.pluginId === filter.pluginId);
      }
      if (filter.since) {
        history = history.filter(e => e.timestamp >= filter.since!);
      }
    }

    return history;
  }

  /**
   * Clear event history
   */
  clearHistory(): void {
    this.eventHistory = [];
  }

  /**
   * Get all event types that have listeners
   */
  getEventTypes(): string[] {
    return Array.from(this.listeners.keys());
  }

  /**
   * Get listener count for an event
   */
  getListenerCount(event: string): number {
    return this.listeners.get(event)?.size || 0;
  }

  /**
   * Remove all listeners
   */
  removeAllListeners(event?: string): void {
    if (event) {
      this.listeners.delete(event);
    } else {
      this.listeners.clear();
    }
  }

  private addToHistory(event: PluginEvent): void {
    this.eventHistory.push(event);

    // Trim history if it exceeds max size
    if (this.eventHistory.length > this.maxHistorySize) {
      this.eventHistory = this.eventHistory.slice(-this.maxHistorySize);
    }
  }
}

export const createLifecycleEventEmitter = (): PluginLifecycleEventEmitter => {
  return new PluginLifecycleEventEmitter();
};
