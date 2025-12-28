/**
 * SimulationRecorder - Simulation State Recording and Playback
 * Records complete simulation states for analysis and playback
 */

import { Vector2D } from './Vector2D';

export interface VehicleState {
  id: string;
  position: Vector2D;
  velocity: Vector2D;
  rotation: number;
  angularVelocity: number;
  acceleration: Vector2D;
  isSkidding: boolean;
  isBraking: boolean;
  steeringAngle: number;
}

export interface CollisionEvent {
  time: number;
  vehicle1Id: string;
  vehicle2Id: string;
  contactPoint: Vector2D;
  normal: Vector2D;
  impactSpeed: number;
  penetrationDepth: number;
  energyLoss: number;
}

export interface SimulationFrame {
  time: number;
  deltaTime: number;
  vehicles: Map<string, VehicleState>;
  collisions: CollisionEvent[];
  metadata?: Record<string, any>;
}

export interface SimulationMetadata {
  id: string;
  description: string;
  startTime: number;
  endTime: number;
  frameCount: number;
  frameRate: number;
  vehicles: string[];
  totalCollisions: number;
}

export interface PlaybackOptions {
  startTime?: number;
  endTime?: number;
  speed: number; // Playback speed multiplier (1.0 = normal, 0.5 = half speed, 2.0 = double speed)
  loop?: boolean;
  onFrame?: (frame: SimulationFrame) => void;
  onCollision?: (collision: CollisionEvent) => void;
}

export interface ExportOptions {
  format: 'json' | 'csv' | 'timeline';
  includeMetadata?: boolean;
  includeCollisions?: boolean;
  decimals?: number;
}

export class SimulationRecorder {
  private frames: SimulationFrame[] = [];
  private metadata: SimulationMetadata;
  private isRecording: boolean = false;
  private startTimestamp: number = 0;

  constructor(
    id: string,
    description: string = '',
    frameRate: number = 60
  ) {
    this.metadata = {
      id,
      description,
      startTime: 0,
      endTime: 0,
      frameCount: 0,
      frameRate,
      vehicles: [],
      totalCollisions: 0
    };
  }

  /**
   * Start recording
   */
  startRecording(): void {
    this.isRecording = true;
    this.frames = [];
    this.startTimestamp = Date.now();
    this.metadata.startTime = 0;
    this.metadata.frameCount = 0;
    this.metadata.totalCollisions = 0;
  }

  /**
   * Stop recording
   */
  stopRecording(): void {
    this.isRecording = false;
    if (this.frames.length > 0) {
      this.metadata.endTime = this.frames[this.frames.length - 1].time;
    }
  }

  /**
   * Record a simulation frame
   */
  recordFrame(
    time: number,
    deltaTime: number,
    vehicles: Map<string, VehicleState>,
    collisions: CollisionEvent[] = [],
    metadata?: Record<string, any>
  ): void {
    if (!this.isRecording) {
      return;
    }

    // Clone vehicles map to prevent external modifications
    const vehiclesCopy = new Map<string, VehicleState>();
    vehicles.forEach((state, id) => {
      vehiclesCopy.set(id, this.cloneVehicleState(state));
    });

    const frame: SimulationFrame = {
      time,
      deltaTime,
      vehicles: vehiclesCopy,
      collisions: collisions.map(c => ({ ...c })),
      metadata
    };

    this.frames.push(frame);
    this.metadata.frameCount++;
    this.metadata.totalCollisions += collisions.length;

    // Update vehicle list
    vehicles.forEach((_, id) => {
      if (!this.metadata.vehicles.includes(id)) {
        this.metadata.vehicles.push(id);
      }
    });
  }

  /**
   * Clone vehicle state
   */
  private cloneVehicleState(state: VehicleState): VehicleState {
    return {
      id: state.id,
      position: state.position.clone(),
      velocity: state.velocity.clone(),
      rotation: state.rotation,
      angularVelocity: state.angularVelocity,
      acceleration: state.acceleration.clone(),
      isSkidding: state.isSkidding,
      isBraking: state.isBraking,
      steeringAngle: state.steeringAngle
    };
  }

  /**
   * Get frame by index
   */
  getFrame(index: number): SimulationFrame | null {
    if (index < 0 || index >= this.frames.length) {
      return null;
    }
    return this.frames[index];
  }

  /**
   * Get frame at specific time (interpolated if necessary)
   */
  getFrameAtTime(time: number): SimulationFrame | null {
    if (this.frames.length === 0) {
      return null;
    }

    // Find frames before and after requested time
    let beforeFrame: SimulationFrame | null = null;
    let afterFrame: SimulationFrame | null = null;

    for (const frame of this.frames) {
      if (frame.time <= time) {
        beforeFrame = frame;
      } else {
        afterFrame = frame;
        break;
      }
    }

    if (!beforeFrame) {
      return this.frames[0];
    }

    if (!afterFrame || beforeFrame.time === time) {
      return beforeFrame;
    }

    // Interpolate between frames
    return this.interpolateFrames(beforeFrame, afterFrame, time);
  }

  /**
   * Interpolate between two frames
   */
  private interpolateFrames(
    frame1: SimulationFrame,
    frame2: SimulationFrame,
    time: number
  ): SimulationFrame {
    const t = (time - frame1.time) / (frame2.time - frame1.time);
    const clampedT = Math.max(0, Math.min(1, t));

    const vehicles = new Map<string, VehicleState>();

    frame1.vehicles.forEach((state1, id) => {
      const state2 = frame2.vehicles.get(id);
      if (state2) {
        vehicles.set(id, {
          id,
          position: Vector2D.lerp(state1.position, state2.position, clampedT),
          velocity: Vector2D.lerp(state1.velocity, state2.velocity, clampedT),
          rotation: this.lerpAngle(state1.rotation, state2.rotation, clampedT),
          angularVelocity: this.lerp(state1.angularVelocity, state2.angularVelocity, clampedT),
          acceleration: Vector2D.lerp(state1.acceleration, state2.acceleration, clampedT),
          isSkidding: clampedT < 0.5 ? state1.isSkidding : state2.isSkidding,
          isBraking: clampedT < 0.5 ? state1.isBraking : state2.isBraking,
          steeringAngle: this.lerpAngle(state1.steeringAngle, state2.steeringAngle, clampedT)
        });
      }
    });

    return {
      time,
      deltaTime: frame1.deltaTime,
      vehicles,
      collisions: time - frame1.time < time - frame2.time ? frame1.collisions : frame2.collisions
    };
  }

  /**
   * Linear interpolation for scalars
   */
  private lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t;
  }

  /**
   * Interpolate angles (handles wraparound)
   */
  private lerpAngle(a: number, b: number, t: number): number {
    // Normalize angles to [0, 2π]
    a = ((a % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);
    b = ((b % (2 * Math.PI)) + 2 * Math.PI) % (2 * Math.PI);

    // Find shortest path
    let diff = b - a;
    if (diff > Math.PI) {
      diff -= 2 * Math.PI;
    } else if (diff < -Math.PI) {
      diff += 2 * Math.PI;
    }

    return a + diff * t;
  }

  /**
   * Get all frames
   */
  getAllFrames(): SimulationFrame[] {
    return [...this.frames];
  }

  /**
   * Get frames in time range
   */
  getFramesInRange(startTime: number, endTime: number): SimulationFrame[] {
    return this.frames.filter(f => f.time >= startTime && f.time <= endTime);
  }

  /**
   * Get all collision events
   */
  getAllCollisions(): CollisionEvent[] {
    const allCollisions: CollisionEvent[] = [];
    for (const frame of this.frames) {
      allCollisions.push(...frame.collisions);
    }
    return allCollisions;
  }

  /**
   * Get metadata
   */
  getMetadata(): SimulationMetadata {
    return { ...this.metadata };
  }

  /**
   * Clear all recorded frames
   */
  clear(): void {
    this.frames = [];
    this.metadata.frameCount = 0;
    this.metadata.totalCollisions = 0;
    this.metadata.startTime = 0;
    this.metadata.endTime = 0;
  }

  /**
   * Export simulation data
   */
  export(options: ExportOptions): string {
    switch (options.format) {
      case 'json':
        return this.exportJSON(options);
      case 'csv':
        return this.exportCSV(options);
      case 'timeline':
        return this.exportTimeline(options);
      default:
        throw new Error(`Unknown export format: ${options.format}`);
    }
  }

  /**
   * Export as JSON
   */
  private exportJSON(options: ExportOptions): string {
    const decimals = options.decimals ?? 6;

    const data: any = {
      frames: this.frames.map(frame => ({
        time: this.round(frame.time, decimals),
        deltaTime: this.round(frame.deltaTime, decimals),
        vehicles: Array.from(frame.vehicles.entries()).map(([id, state]) => ({
          id,
          position: {
            x: this.round(state.position.x, decimals),
            y: this.round(state.position.y, decimals)
          },
          velocity: {
            x: this.round(state.velocity.x, decimals),
            y: this.round(state.velocity.y, decimals)
          },
          rotation: this.round(state.rotation, decimals),
          angularVelocity: this.round(state.angularVelocity, decimals),
          isSkidding: state.isSkidding,
          isBraking: state.isBraking
        })),
        collisions: options.includeCollisions ? frame.collisions : undefined
      }))
    };

    if (options.includeMetadata) {
      data.metadata = this.metadata;
    }

    return JSON.stringify(data, null, 2);
  }

  /**
   * Export as CSV
   */
  private exportCSV(options: ExportOptions): string {
    const decimals = options.decimals ?? 3;
    const lines: string[] = [];

    // Header
    lines.push('Time,VehicleID,PosX,PosY,VelX,VelY,Rotation,AngVel,Skidding,Braking');

    // Data rows
    for (const frame of this.frames) {
      frame.vehicles.forEach((state, id) => {
        const row = [
          this.round(frame.time, decimals),
          id,
          this.round(state.position.x, decimals),
          this.round(state.position.y, decimals),
          this.round(state.velocity.x, decimals),
          this.round(state.velocity.y, decimals),
          this.round(state.rotation, decimals),
          this.round(state.angularVelocity, decimals),
          state.isSkidding ? 1 : 0,
          state.isBraking ? 1 : 0
        ].join(',');
        lines.push(row);
      });
    }

    return lines.join('\n');
  }

  /**
   * Export as timeline (human-readable event list)
   */
  private exportTimeline(options: ExportOptions): string {
    const lines: string[] = [];
    const decimals = options.decimals ?? 2;

    lines.push('=== SIMULATION TIMELINE ===');
    lines.push('');

    if (options.includeMetadata) {
      lines.push(`ID: ${this.metadata.id}`);
      lines.push(`Description: ${this.metadata.description}`);
      lines.push(`Duration: ${this.round(this.metadata.endTime, decimals)}s`);
      lines.push(`Frames: ${this.metadata.frameCount}`);
      lines.push(`Vehicles: ${this.metadata.vehicles.join(', ')}`);
      lines.push(`Total Collisions: ${this.metadata.totalCollisions}`);
      lines.push('');
    }

    lines.push('=== EVENTS ===');

    // Add collision events
    if (options.includeCollisions) {
      const collisions = this.getAllCollisions();
      collisions.forEach((collision, index) => {
        lines.push(
          `[${this.round(collision.time, decimals)}s] COLLISION #${index + 1}: ` +
          `${collision.vehicle1Id} ↔ ${collision.vehicle2Id} ` +
          `@ ${this.round(collision.impactSpeed, decimals)} m/s`
        );
      });
    }

    return lines.join('\n');
  }

  /**
   * Round number to specified decimals
   */
  private round(value: number, decimals: number): number {
    const factor = Math.pow(10, decimals);
    return Math.round(value * factor) / factor;
  }

  /**
   * Generate timeline events from frames
   */
  generateTimeline(): Array<{ time: number; event: string; data: any }> {
    const timeline: Array<{ time: number; event: string; data: any }> = [];

    // Add start event
    if (this.frames.length > 0) {
      timeline.push({
        time: this.frames[0].time,
        event: 'simulation_start',
        data: { vehicles: Array.from(this.frames[0].vehicles.keys()) }
      });
    }

    // Add collision events
    for (const frame of this.frames) {
      for (const collision of frame.collisions) {
        timeline.push({
          time: frame.time,
          event: 'collision',
          data: collision
        });
      }
    }

    // Add end event
    if (this.frames.length > 0) {
      const lastFrame = this.frames[this.frames.length - 1];
      timeline.push({
        time: lastFrame.time,
        event: 'simulation_end',
        data: { finalStates: Array.from(lastFrame.vehicles.entries()) }
      });
    }

    return timeline.sort((a, b) => a.time - b.time);
  }

  /**
   * Get statistics from simulation
   */
  getStatistics(): {
    duration: number;
    averageFrameRate: number;
    totalDistance: Map<string, number>;
    maxSpeed: Map<string, number>;
    averageSpeed: Map<string, number>;
  } {
    const totalDistance = new Map<string, number>();
    const maxSpeed = new Map<string, number>();
    const speedSum = new Map<string, number>();
    const speedCount = new Map<string, number>();

    for (const frame of this.frames) {
      frame.vehicles.forEach((state, id) => {
        // Update distance (approximate)
        const dist = totalDistance.get(id) || 0;
        const speed = state.velocity.magnitude();
        totalDistance.set(id, dist + speed * frame.deltaTime);

        // Update max speed
        const currentMax = maxSpeed.get(id) || 0;
        maxSpeed.set(id, Math.max(currentMax, speed));

        // Update average speed
        const sum = speedSum.get(id) || 0;
        const count = speedCount.get(id) || 0;
        speedSum.set(id, sum + speed);
        speedCount.set(id, count + 1);
      });
    }

    const averageSpeed = new Map<string, number>();
    speedSum.forEach((sum, id) => {
      const count = speedCount.get(id) || 1;
      averageSpeed.set(id, sum / count);
    });

    const duration = this.metadata.endTime - this.metadata.startTime;
    const averageFrameRate = duration > 0 ? this.metadata.frameCount / duration : 0;

    return {
      duration,
      averageFrameRate,
      totalDistance,
      maxSpeed,
      averageSpeed
    };
  }
}
