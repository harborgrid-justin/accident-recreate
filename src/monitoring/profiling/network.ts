/**
 * AccuScene Enterprise v0.2.0
 * Network Profiling
 *
 * Network request and performance monitoring
 */

import { NetworkProfile, NetworkRequest } from '../types';

export class NetworkProfiler {
  private requests: Map<string, NetworkRequest> = new Map();
  private completedRequests: NetworkRequest[] = [];
  private readonly maxHistorySize: number = 1000;
  private isMonitoring: boolean = false;
  private originalFetch?: typeof fetch;
  private originalXHR?: typeof XMLHttpRequest;

  /**
   * Start network monitoring
   */
  public start(): void {
    if (this.isMonitoring) {
      return;
    }

    this.isMonitoring = true;
    this.patchFetch();
    this.patchXHR();
  }

  /**
   * Stop network monitoring
   */
  public stop(): NetworkProfile {
    this.isMonitoring = false;
    this.unpatchFetch();
    this.unpatchXHR();

    return this.getProfile();
  }

  /**
   * Patch fetch API
   */
  private patchFetch(): void {
    if (typeof fetch === 'undefined') {
      return;
    }

    this.originalFetch = fetch;
    const self = this;

    (global as any).fetch = async function (
      input: RequestInfo | URL,
      init?: RequestInit
    ): Promise<Response> {
      const url = typeof input === 'string' ? input : input instanceof URL ? input.href : input.url;
      const method = init?.method || 'GET';
      const requestId = self.generateRequestId();

      const request: NetworkRequest = {
        id: requestId,
        method,
        url,
        status: 0,
        startTime: Date.now(),
        endTime: 0,
        duration: 0,
        size: 0
      };

      self.requests.set(requestId, request);

      try {
        const response = await self.originalFetch!.call(this, input, init);

        request.status = response.status;
        request.endTime = Date.now();
        request.duration = request.endTime - request.startTime;

        // Estimate response size
        const contentLength = response.headers.get('content-length');
        request.size = contentLength ? parseInt(contentLength, 10) : 0;

        // Check if cached
        const cacheControl = response.headers.get('cache-control');
        request.cached = cacheControl?.includes('from-cache') || false;

        self.completeRequest(requestId);

        return response;
      } catch (error) {
        request.status = 0;
        request.endTime = Date.now();
        request.duration = request.endTime - request.startTime;

        self.completeRequest(requestId);

        throw error;
      }
    };
  }

  /**
   * Unpatch fetch API
   */
  private unpatchFetch(): void {
    if (this.originalFetch) {
      (global as any).fetch = this.originalFetch;
      this.originalFetch = undefined;
    }
  }

  /**
   * Patch XMLHttpRequest
   */
  private patchXHR(): void {
    if (typeof XMLHttpRequest === 'undefined') {
      return;
    }

    this.originalXHR = XMLHttpRequest;
    const self = this;

    (global as any).XMLHttpRequest = function () {
      const xhr = new self.originalXHR!();
      const requestId = self.generateRequestId();

      let request: NetworkRequest = {
        id: requestId,
        method: 'GET',
        url: '',
        status: 0,
        startTime: 0,
        endTime: 0,
        duration: 0,
        size: 0
      };

      const originalOpen = xhr.open;
      xhr.open = function (method: string, url: string | URL, ...args: any[]) {
        request.method = method;
        request.url = url.toString();
        return originalOpen.apply(this, [method, url, ...args] as any);
      };

      const originalSend = xhr.send;
      xhr.send = function (...args: any[]) {
        request.startTime = Date.now();
        self.requests.set(requestId, request);
        return originalSend.apply(this, args);
      };

      xhr.addEventListener('load', () => {
        request.status = xhr.status;
        request.endTime = Date.now();
        request.duration = request.endTime - request.startTime;

        const contentLength = xhr.getResponseHeader('content-length');
        request.size = contentLength ? parseInt(contentLength, 10) : 0;

        self.completeRequest(requestId);
      });

      xhr.addEventListener('error', () => {
        request.status = 0;
        request.endTime = Date.now();
        request.duration = request.endTime - request.startTime;

        self.completeRequest(requestId);
      });

      return xhr;
    };
  }

  /**
   * Unpatch XMLHttpRequest
   */
  private unpatchXHR(): void {
    if (this.originalXHR) {
      (global as any).XMLHttpRequest = this.originalXHR;
      this.originalXHR = undefined;
    }
  }

  /**
   * Complete a request
   */
  private completeRequest(requestId: string): void {
    const request = this.requests.get(requestId);

    if (request) {
      this.completedRequests.push(request);
      this.requests.delete(requestId);

      // Maintain history size
      if (this.completedRequests.length > this.maxHistorySize) {
        this.completedRequests.shift();
      }
    }
  }

  /**
   * Generate unique request ID
   */
  private generateRequestId(): string {
    return `req_${Date.now()}_${Math.random().toString(36).substring(7)}`;
  }

  /**
   * Get network profile
   */
  public getProfile(): NetworkProfile {
    const requests = [...this.completedRequests];

    const totalBytes = requests.reduce((sum, req) => sum + req.size, 0);
    const totalLatency = requests.reduce((sum, req) => sum + req.duration, 0);

    return {
      timestamp: Date.now(),
      requests,
      totalRequests: requests.length,
      totalBytes,
      avgLatency: requests.length > 0 ? totalLatency / requests.length : 0
    };
  }

  /**
   * Get requests by time range
   */
  public getRequestsByTime(start: number, end: number): NetworkRequest[] {
    return this.completedRequests.filter(
      req => req.startTime >= start && req.startTime <= end
    );
  }

  /**
   * Get slow requests
   */
  public getSlowRequests(threshold: number = 1000): NetworkRequest[] {
    return this.completedRequests
      .filter(req => req.duration > threshold)
      .sort((a, b) => b.duration - a.duration);
  }

  /**
   * Get failed requests
   */
  public getFailedRequests(): NetworkRequest[] {
    return this.completedRequests.filter(
      req => req.status === 0 || req.status >= 400
    );
  }

  /**
   * Get requests by URL pattern
   */
  public getRequestsByURL(pattern: string | RegExp): NetworkRequest[] {
    const regex = typeof pattern === 'string' ? new RegExp(pattern) : pattern;

    return this.completedRequests.filter(req => regex.test(req.url));
  }

  /**
   * Get bandwidth usage
   */
  public getBandwidthUsage(windowMs: number = 60000): {
    bytesPerSecond: number;
    requestsPerSecond: number;
  } {
    const now = Date.now();
    const windowStart = now - windowMs;

    const windowRequests = this.completedRequests.filter(
      req => req.endTime >= windowStart
    );

    const totalBytes = windowRequests.reduce((sum, req) => sum + req.size, 0);
    const windowSeconds = windowMs / 1000;

    return {
      bytesPerSecond: totalBytes / windowSeconds,
      requestsPerSecond: windowRequests.length / windowSeconds
    };
  }

  /**
   * Get request statistics
   */
  public getStatistics(): {
    total: number;
    success: number;
    failed: number;
    avgDuration: number;
    avgSize: number;
    p95Duration: number;
    p99Duration: number;
  } {
    const requests = this.completedRequests;

    if (requests.length === 0) {
      return {
        total: 0,
        success: 0,
        failed: 0,
        avgDuration: 0,
        avgSize: 0,
        p95Duration: 0,
        p99Duration: 0
      };
    }

    const success = requests.filter(req => req.status >= 200 && req.status < 400).length;
    const failed = requests.length - success;

    const durations = requests.map(req => req.duration).sort((a, b) => a - b);
    const sizes = requests.map(req => req.size);

    const avgDuration = durations.reduce((a, b) => a + b, 0) / durations.length;
    const avgSize = sizes.reduce((a, b) => a + b, 0) / sizes.length;

    const p95Index = Math.floor(durations.length * 0.95);
    const p99Index = Math.floor(durations.length * 0.99);

    return {
      total: requests.length,
      success,
      failed,
      avgDuration,
      avgSize,
      p95Duration: durations[p95Index] || 0,
      p99Duration: durations[p99Index] || 0
    };
  }

  /**
   * Clear request history
   */
  public clear(): void {
    this.requests.clear();
    this.completedRequests = [];
  }
}
