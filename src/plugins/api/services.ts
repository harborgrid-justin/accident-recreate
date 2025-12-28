/**
 * AccuScene Enterprise v0.2.0 - Plugin Services
 * Service registry for plugins
 */

import { PluginServices, PluginId } from '../types';

class PluginServicesImpl implements PluginServices {
  private services = new Map<string, any>();

  constructor(private pluginId: PluginId) {}

  get<T = any>(serviceId: string): T | undefined {
    return this.services.get(serviceId);
  }

  register<T = any>(serviceId: string, service: T): void {
    if (this.services.has(serviceId)) {
      console.warn(`Service ${serviceId} is already registered for plugin ${this.pluginId}`);
    }

    this.services.set(serviceId, service);
  }

  unregister(serviceId: string): void {
    this.services.delete(serviceId);
  }

  has(serviceId: string): boolean {
    return this.services.has(serviceId);
  }

  /**
   * Get all registered service IDs
   */
  getServiceIds(): string[] {
    return Array.from(this.services.keys());
  }

  /**
   * Clear all services
   */
  clear(): void {
    this.services.clear();
  }
}

export const createPluginServices = (pluginId: PluginId): PluginServices => {
  return new PluginServicesImpl(pluginId);
};
