/**
 * Network Detector
 * Monitors network connectivity and quality
 */

import { NetworkState, NetworkQuality } from './types';

export class NetworkDetector {
  private state: NetworkState = NetworkState.Unknown;
  private quality: NetworkQuality | null = null;
  private listeners: Set<(state: NetworkState) => void> = new Set();
  private checkInterval: NodeJS.Timeout | null = null;
  private apiEndpoint: string;
  private checkIntervalMs: number;

  constructor(apiEndpoint: string, checkIntervalMs: number = 5000) {
    this.apiEndpoint = apiEndpoint;
    this.checkIntervalMs = checkIntervalMs;

    // Listen to browser online/offline events
    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.handleOnline);
      window.addEventListener('offline', this.handleOffline);

      // Initial check
      this.state = navigator.onLine ? NetworkState.Online : NetworkState.Offline;
    }
  }

  /**
   * Start monitoring network state
   */
  start(): void {
    this.checkConnectivity();

    this.checkInterval = setInterval(() => {
      this.checkConnectivity();
    }, this.checkIntervalMs);
  }

  /**
   * Stop monitoring
   */
  stop(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }

    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.handleOnline);
      window.removeEventListener('offline', this.handleOffline);
    }
  }

  /**
   * Handle browser online event
   */
  private handleOnline = () => {
    this.updateState(NetworkState.Online);
    this.checkConnectivity();
  };

  /**
   * Handle browser offline event
   */
  private handleOffline = () => {
    this.updateState(NetworkState.Offline);
  };

  /**
   * Check network connectivity and quality
   */
  private async checkConnectivity(): Promise<void> {
    if (!navigator.onLine) {
      this.updateState(NetworkState.Offline);
      return;
    }

    try {
      const start = performance.now();

      const controller = new AbortController();
      const timeout = setTimeout(() => controller.abort(), 10000);

      const response = await fetch(`${this.apiEndpoint}/health`, {
        method: 'HEAD',
        cache: 'no-cache',
        signal: controller.signal,
      });

      clearTimeout(timeout);

      const latency = performance.now() - start;

      if (response.ok) {
        this.quality = {
          latencyMs: latency,
          bandwidthBps: 0, // Could estimate from response
          packetLoss: 0,
          score: this.calculateQualityScore(latency),
        };

        const state =
          this.quality.score >= 0.5
            ? NetworkState.Online
            : NetworkState.Degraded;

        this.updateState(state);
      } else {
        this.updateState(NetworkState.Degraded);
      }
    } catch (error) {
      this.updateState(NetworkState.Offline);
      this.quality = null;
    }
  }

  /**
   * Calculate network quality score
   */
  private calculateQualityScore(latencyMs: number): number {
    if (latencyMs < 50) return 1.0;
    if (latencyMs < 150) return 0.8;
    if (latencyMs < 300) return 0.5;
    if (latencyMs < 1000) return 0.2;
    return 0.0;
  }

  /**
   * Update network state and notify listeners
   */
  private updateState(newState: NetworkState): void {
    if (this.state !== newState) {
      this.state = newState;
      this.notifyListeners();
    }
  }

  /**
   * Notify all listeners of state change
   */
  private notifyListeners(): void {
    this.listeners.forEach((listener) => listener(this.state));
  }

  /**
   * Add state change listener
   */
  addListener(listener: (state: NetworkState) => void): () => void {
    this.listeners.add(listener);

    // Return unsubscribe function
    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Get current network state
   */
  getState(): NetworkState {
    return this.state;
  }

  /**
   * Get network quality
   */
  getQuality(): NetworkQuality | null {
    return this.quality;
  }

  /**
   * Check if currently online
   */
  isOnline(): boolean {
    return this.state === NetworkState.Online || this.state === NetworkState.Degraded;
  }

  /**
   * Wait for network to become online
   */
  async waitForOnline(timeoutMs: number = 30000): Promise<boolean> {
    if (this.isOnline()) {
      return true;
    }

    return new Promise((resolve) => {
      const timeout = setTimeout(() => {
        unsubscribe();
        resolve(false);
      }, timeoutMs);

      const unsubscribe = this.addListener((state) => {
        if (state === NetworkState.Online || state === NetworkState.Degraded) {
          clearTimeout(timeout);
          unsubscribe();
          resolve(true);
        }
      });

      // Check immediately
      this.checkConnectivity();
    });
  }

  /**
   * Force connectivity check
   */
  async forceCheck(): Promise<void> {
    await this.checkConnectivity();
  }
}
