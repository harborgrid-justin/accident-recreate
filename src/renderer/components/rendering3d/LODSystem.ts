/**
 * AccuScene Enterprise v0.3.0
 * Level-of-Detail System with Smooth Transitions
 */

import { LODGroup, LODLevel, Camera, Vec3, Sphere } from './types';

export class LODSystem {
  private lodGroups: Map<string, LODGroup> = new Map();
  private transitionDuration = 0.3; // seconds
  private activeTransitions: Map<string, {
    fromLevel: number;
    toLevel: number;
    startTime: number;
    progress: number;
  }> = new Map();

  // Register LOD group
  public registerLODGroup(group: LODGroup): void {
    // Sort levels by distance
    group.levels.sort((a, b) => a.distance - b.distance);
    this.lodGroups.set(group.id, group);
  }

  // Unregister LOD group
  public unregisterLODGroup(id: string): void {
    this.lodGroups.delete(id);
    this.activeTransitions.delete(id);
  }

  // Update LOD levels based on camera
  public update(camera: Camera, deltaTime: number): void {
    this.lodGroups.forEach((group, id) => {
      const distance = this.calculateDistance(camera.position, group.bounds.center);
      const newLevel = this.selectLODLevel(group, distance, camera);

      if (newLevel !== group.currentLevel) {
        this.startTransition(id, group.currentLevel, newLevel);
        group.currentLevel = newLevel;
      }

      // Update transitions
      const transition = this.activeTransitions.get(id);
      if (transition) {
        transition.progress += deltaTime / this.transitionDuration;

        if (transition.progress >= 1.0) {
          this.activeTransitions.delete(id);
        }
      }
    });
  }

  // Calculate distance from camera to object
  private calculateDistance(cameraPos: Vec3, objectPos: Vec3): number {
    const dx = cameraPos.x - objectPos.x;
    const dy = cameraPos.y - objectPos.y;
    const dz = cameraPos.z - objectPos.z;
    return Math.sqrt(dx * dx + dy * dy + dz * dz);
  }

  // Select appropriate LOD level
  private selectLODLevel(group: LODGroup, distance: number, camera: Camera): number {
    // Calculate screen coverage if needed
    const screenCoverage = this.calculateScreenCoverage(group.bounds, camera);

    for (let i = 0; i < group.levels.length; i++) {
      const level = group.levels[i];

      if (level.screenCoverage !== undefined) {
        if (screenCoverage >= level.screenCoverage) {
          return i;
        }
      } else {
        if (distance <= level.distance) {
          return i;
        }
      }
    }

    // Return lowest detail level
    return group.levels.length - 1;
  }

  // Calculate screen coverage percentage
  private calculateScreenCoverage(bounds: Sphere, camera: Camera): number {
    const distance = this.calculateDistance(camera.position, bounds.center);
    const radius = bounds.radius;

    // Simple approximation of screen coverage
    const fovRadians = (camera.fov * Math.PI) / 180;
    const screenHeight = 2 * distance * Math.tan(fovRadians / 2);
    const coverage = (radius * 2) / screenHeight;

    return Math.min(coverage, 1.0);
  }

  // Start LOD transition
  private startTransition(id: string, fromLevel: number, toLevel: number): void {
    this.activeTransitions.set(id, {
      fromLevel,
      toLevel,
      startTime: performance.now(),
      progress: 0,
    });
  }

  // Get current LOD mesh for rendering
  public getLODMesh(id: string): { level: number; blend?: { from: number; to: number; factor: number } } {
    const group = this.lodGroups.get(id);
    if (!group) return { level: 0 };

    const transition = this.activeTransitions.get(id);
    if (transition && transition.progress < 1.0) {
      // Smooth transition between levels
      return {
        level: transition.toLevel,
        blend: {
          from: transition.fromLevel,
          to: transition.toLevel,
          factor: this.easeInOutCubic(transition.progress),
        },
      };
    }

    return { level: group.currentLevel };
  }

  // Easing function for smooth transitions
  private easeInOutCubic(t: number): number {
    return t < 0.5
      ? 4 * t * t * t
      : 1 - Math.pow(-2 * t + 2, 3) / 2;
  }

  // Get all LOD groups
  public getLODGroups(): Map<string, LODGroup> {
    return this.lodGroups;
  }

  // Force specific LOD level
  public forceLODLevel(id: string, level: number): void {
    const group = this.lodGroups.get(id);
    if (!group || level < 0 || level >= group.levels.length) return;

    group.currentLevel = level;
    this.activeTransitions.delete(id);
  }

  // Auto-generate LOD levels from base mesh
  public static generateLODLevels(
    baseMesh: any,
    levelCount: number,
    distances: number[]
  ): LODLevel[] {
    const levels: LODLevel[] = [];

    for (let i = 0; i < levelCount; i++) {
      const simplificationRatio = i / levelCount;

      // In a real implementation, this would use mesh simplification algorithms
      // For now, we'll just reference the same mesh
      levels.push({
        distance: distances[i] || (i + 1) * 10,
        mesh: baseMesh, // Would be simplified version
        screenCoverage: 1.0 - simplificationRatio * 0.8,
      });
    }

    return levels;
  }

  // Calculate optimal LOD distances based on object size
  public static calculateOptimalDistances(
    objectSize: number,
    levelCount: number
  ): number[] {
    const distances: number[] = [];
    const baseDistance = objectSize * 5;

    for (let i = 0; i < levelCount; i++) {
      distances.push(baseDistance * Math.pow(2, i));
    }

    return distances;
  }

  // Get statistics
  public getStats(): {
    totalGroups: number;
    activeTransitions: number;
    levelDistribution: Record<number, number>;
  } {
    const levelDistribution: Record<number, number> = {};

    this.lodGroups.forEach(group => {
      levelDistribution[group.currentLevel] = (levelDistribution[group.currentLevel] || 0) + 1;
    });

    return {
      totalGroups: this.lodGroups.size,
      activeTransitions: this.activeTransitions.size,
      levelDistribution,
    };
  }
}
