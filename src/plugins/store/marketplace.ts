/**
 * AccuScene Enterprise v0.2.0 - Plugin Marketplace
 * Integration with plugin marketplace
 */

import { PluginStoreEntry, PluginManifest, PluginId } from '../types';

export interface MarketplaceConfig {
  apiEndpoint: string;
  apiKey?: string;
  cacheTimeout?: number;
}

export class PluginMarketplace {
  private config: Required<MarketplaceConfig>;
  private cache = new Map<string, { data: any; timestamp: number }>();

  constructor(config: MarketplaceConfig) {
    this.config = {
      apiKey: '',
      cacheTimeout: 3600000, // 1 hour
      ...config,
    };
  }

  /**
   * Search for plugins in the marketplace
   */
  async search(query: string, filters?: SearchFilters): Promise<PluginStoreEntry[]> {
    const cacheKey = `search:${query}:${JSON.stringify(filters)}`;
    const cached = this.getFromCache(cacheKey);

    if (cached) {
      return cached;
    }

    try {
      const params = new URLSearchParams({
        q: query,
        ...this.filtersToParams(filters),
      });

      const response = await this.fetch(`/search?${params}`);
      const results = await response.json();

      this.setCache(cacheKey, results);
      return results;
    } catch (error) {
      throw new Error(`Marketplace search failed: ${error}`);
    }
  }

  /**
   * Get plugin details from marketplace
   */
  async getPlugin(pluginId: PluginId): Promise<PluginStoreEntry> {
    const cacheKey = `plugin:${pluginId}`;
    const cached = this.getFromCache(cacheKey);

    if (cached) {
      return cached;
    }

    try {
      const response = await this.fetch(`/plugins/${pluginId}`);
      const plugin = await response.json();

      this.setCache(cacheKey, plugin);
      return plugin;
    } catch (error) {
      throw new Error(`Failed to get plugin ${pluginId}: ${error}`);
    }
  }

  /**
   * Get featured plugins
   */
  async getFeatured(): Promise<PluginStoreEntry[]> {
    const cacheKey = 'featured';
    const cached = this.getFromCache(cacheKey);

    if (cached) {
      return cached;
    }

    try {
      const response = await this.fetch('/featured');
      const plugins = await response.json();

      this.setCache(cacheKey, plugins);
      return plugins;
    } catch (error) {
      throw new Error(`Failed to get featured plugins: ${error}`);
    }
  }

  /**
   * Get popular plugins
   */
  async getPopular(limit = 10): Promise<PluginStoreEntry[]> {
    const cacheKey = `popular:${limit}`;
    const cached = this.getFromCache(cacheKey);

    if (cached) {
      return cached;
    }

    try {
      const response = await this.fetch(`/popular?limit=${limit}`);
      const plugins = await response.json();

      this.setCache(cacheKey, plugins);
      return plugins;
    } catch (error) {
      throw new Error(`Failed to get popular plugins: ${error}`);
    }
  }

  /**
   * Get recently updated plugins
   */
  async getRecent(limit = 10): Promise<PluginStoreEntry[]> {
    const cacheKey = `recent:${limit}`;
    const cached = this.getFromCache(cacheKey);

    if (cached) {
      return cached;
    }

    try {
      const response = await this.fetch(`/recent?limit=${limit}`);
      const plugins = await response.json();

      this.setCache(cacheKey, plugins);
      return plugins;
    } catch (error) {
      throw new Error(`Failed to get recent plugins: ${error}`);
    }
  }

  /**
   * Get plugin download URL
   */
  async getDownloadUrl(pluginId: PluginId, version?: string): Promise<string> {
    try {
      const versionParam = version ? `?version=${version}` : '';
      const response = await this.fetch(`/plugins/${pluginId}/download${versionParam}`);
      const data = await response.json();

      return data.url;
    } catch (error) {
      throw new Error(`Failed to get download URL for ${pluginId}: ${error}`);
    }
  }

  /**
   * Submit a review for a plugin
   */
  async submitReview(
    pluginId: PluginId,
    review: { rating: number; comment: string }
  ): Promise<void> {
    try {
      await this.fetch(`/plugins/${pluginId}/reviews`, {
        method: 'POST',
        body: JSON.stringify(review),
      });

      // Invalidate plugin cache
      this.cache.delete(`plugin:${pluginId}`);
    } catch (error) {
      throw new Error(`Failed to submit review: ${error}`);
    }
  }

  /**
   * Report a plugin
   */
  async reportPlugin(pluginId: PluginId, reason: string): Promise<void> {
    try {
      await this.fetch(`/plugins/${pluginId}/report`, {
        method: 'POST',
        body: JSON.stringify({ reason }),
      });
    } catch (error) {
      throw new Error(`Failed to report plugin: ${error}`);
    }
  }

  /**
   * Clear the cache
   */
  clearCache(): void {
    this.cache.clear();
  }

  private async fetch(path: string, options: RequestInit = {}): Promise<Response> {
    const url = `${this.config.apiEndpoint}${path}`;
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
      ...options.headers,
    };

    if (this.config.apiKey) {
      headers['Authorization'] = `Bearer ${this.config.apiKey}`;
    }

    const response = await fetch(url, {
      ...options,
      headers,
    });

    if (!response.ok) {
      throw new Error(`HTTP ${response.status}: ${response.statusText}`);
    }

    return response;
  }

  private getFromCache<T>(key: string): T | undefined {
    const cached = this.cache.get(key);

    if (!cached) {
      return undefined;
    }

    const age = Date.now() - cached.timestamp;

    if (age > this.config.cacheTimeout) {
      this.cache.delete(key);
      return undefined;
    }

    return cached.data as T;
  }

  private setCache(key: string, data: any): void {
    this.cache.set(key, {
      data,
      timestamp: Date.now(),
    });
  }

  private filtersToParams(filters?: SearchFilters): Record<string, string> {
    if (!filters) {
      return {};
    }

    const params: Record<string, string> = {};

    if (filters.category) {
      params.category = filters.category;
    }

    if (filters.verified !== undefined) {
      params.verified = filters.verified.toString();
    }

    if (filters.minRating !== undefined) {
      params.minRating = filters.minRating.toString();
    }

    if (filters.sort) {
      params.sort = filters.sort;
    }

    return params;
  }
}

export interface SearchFilters {
  category?: string;
  verified?: boolean;
  minRating?: number;
  sort?: 'downloads' | 'rating' | 'updated' | 'name';
}

export const createMarketplace = (config: MarketplaceConfig): PluginMarketplace => {
  return new PluginMarketplace(config);
};
