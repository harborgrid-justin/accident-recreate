/**
 * AccuScene Enterprise v0.2.0
 * CPU Profiling
 *
 * CPU profiling and performance monitoring
 */

import { CPUProfile, ProfileNode } from '../types';

export class CPUProfiler {
  private isRunning: boolean = false;
  private startTime: number = 0;
  private samples: number = 0;
  private sampleInterval: number = 10; // ms
  private nodes: Map<number, ProfileNode> = new Map();
  private intervalId?: NodeJS.Timeout;

  /**
   * Start CPU profiling
   */
  public start(sampleInterval: number = 10): void {
    if (this.isRunning) {
      throw new Error('CPU profiler is already running');
    }

    this.isRunning = true;
    this.startTime = Date.now();
    this.samples = 0;
    this.nodes.clear();
    this.sampleInterval = sampleInterval;

    // Start sampling
    this.intervalId = setInterval(() => {
      this.takeSample();
    }, sampleInterval);
  }

  /**
   * Stop CPU profiling
   */
  public stop(): CPUProfile {
    if (!this.isRunning) {
      throw new Error('CPU profiler is not running');
    }

    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
    }

    this.isRunning = false;

    const duration = Date.now() - this.startTime;

    return {
      timestamp: this.startTime,
      duration,
      samples: this.samples,
      nodes: Array.from(this.nodes.values())
    };
  }

  /**
   * Take a CPU sample
   */
  private takeSample(): void {
    this.samples++;

    // Get current stack trace
    const stack = this.getCurrentStack();

    // Process stack and update nodes
    for (let i = 0; i < stack.length; i++) {
      const frame = stack[i];
      const nodeId = this.getNodeId(frame);

      let node = this.nodes.get(nodeId);

      if (!node) {
        node = {
          id: nodeId,
          callFrame: frame,
          hitCount: 0,
          children: [],
          positionTicks: []
        };
        this.nodes.set(nodeId, node);
      }

      // Increment hit count for leaf frame
      if (i === 0) {
        node.hitCount++;

        // Track line-level hits
        const lineNumber = frame.lineNumber;
        const tick = node.positionTicks?.find(t => t.line === lineNumber);

        if (tick) {
          tick.ticks++;
        } else {
          node.positionTicks?.push({ line: lineNumber, ticks: 1 });
        }
      }

      // Link parent-child relationship
      if (i < stack.length - 1) {
        const childId = this.getNodeId(stack[i + 1]);
        if (!node.children?.includes(childId)) {
          node.children?.push(childId);
        }
      }
    }
  }

  /**
   * Get current call stack
   */
  private getCurrentStack(): Array<{
    functionName: string;
    scriptId: string;
    url: string;
    lineNumber: number;
    columnNumber: number;
  }> {
    const error = new Error();
    const stack = error.stack || '';
    const frames: any[] = [];

    // Parse stack trace
    const lines = stack.split('\n').slice(2); // Skip Error and getCurrentStack

    for (const line of lines) {
      const match = line.match(/at\s+(?:(.+?)\s+\()?(.+?):(\d+):(\d+)\)?$/);

      if (match) {
        const [, functionName, url, lineNumber, columnNumber] = match;

        frames.push({
          functionName: functionName || '(anonymous)',
          scriptId: url,
          url: url,
          lineNumber: parseInt(lineNumber, 10),
          columnNumber: parseInt(columnNumber, 10)
        });
      }
    }

    return frames;
  }

  /**
   * Generate unique node ID from call frame
   */
  private getNodeId(frame: {
    functionName: string;
    url: string;
    lineNumber: number;
    columnNumber: number;
  }): number {
    const str = `${frame.functionName}:${frame.url}:${frame.lineNumber}:${frame.columnNumber}`;
    let hash = 0;

    for (let i = 0; i < str.length; i++) {
      const char = str.charCodeAt(i);
      hash = ((hash << 5) - hash) + char;
      hash = hash & hash; // Convert to 32bit integer
    }

    return Math.abs(hash);
  }

  /**
   * Get CPU usage percentage
   */
  public static async getCPUUsage(): Promise<number> {
    if (typeof process !== 'undefined' && process.cpuUsage) {
      const startUsage = process.cpuUsage();
      const startTime = Date.now();

      await new Promise(resolve => setTimeout(resolve, 100));

      const endUsage = process.cpuUsage(startUsage);
      const endTime = Date.now();

      const elapsed = (endTime - startTime) * 1000; // Convert to microseconds
      const totalUsage = endUsage.user + endUsage.system;

      return (totalUsage / elapsed) * 100;
    }

    return 0;
  }

  /**
   * Get top hotspots from profile
   */
  public static getHotspots(profile: CPUProfile, limit: number = 10): ProfileNode[] {
    return profile.nodes
      .filter(node => node.hitCount > 0)
      .sort((a, b) => b.hitCount - a.hitCount)
      .slice(0, limit);
  }

  /**
   * Get flame graph data
   */
  public static getFlameGraphData(profile: CPUProfile): any {
    const rootNodes = profile.nodes.filter(node => {
      // Root nodes are those not referenced as children
      const isChild = profile.nodes.some(n =>
        n.children?.includes(node.id)
      );
      return !isChild;
    });

    const buildFlameGraph = (nodeId: number): any => {
      const node = profile.nodes.find(n => n.id === nodeId);

      if (!node) {
        return null;
      }

      return {
        name: node.callFrame.functionName,
        value: node.hitCount,
        children: node.children
          ?.map(childId => buildFlameGraph(childId))
          .filter(Boolean) || []
      };
    };

    return rootNodes.map(node => buildFlameGraph(node.id));
  }
}
