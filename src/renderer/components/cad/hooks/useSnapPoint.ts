/**
 * AccuScene Enterprise v0.3.0 - Snap Point Calculation Hook
 */

import { useState, useCallback, useMemo } from 'react';
import { Point2D, SnapPoint, SnapType, SnapSettings } from '../types';

export interface UseSnapPointReturn {
  snapSettings: SnapSettings;
  updateSnapSettings: (settings: Partial<SnapSettings>) => void;
  findSnapPoint: (point: Point2D, objects?: any[]) => SnapPoint | null;
  snapToGrid: (point: Point2D) => Point2D;
  toggleSnap: () => void;
  toggleSnapType: (type: SnapType) => void;
}

const defaultSnapSettings: SnapSettings = {
  enabled: true,
  gridSnap: true,
  objectSnap: true,
  snapTypes: [
    SnapType.ENDPOINT,
    SnapType.MIDPOINT,
    SnapType.CENTER,
    SnapType.INTERSECTION,
  ],
  snapDistance: 10,
  gridSize: 20,
};

export function useSnapPoint(
  initialSettings: Partial<SnapSettings> = {}
): UseSnapPointReturn {
  const [snapSettings, setSnapSettings] = useState<SnapSettings>({
    ...defaultSnapSettings,
    ...initialSettings,
  });

  const updateSnapSettings = useCallback((settings: Partial<SnapSettings>) => {
    setSnapSettings((prev) => ({ ...prev, ...settings }));
  }, []);

  const toggleSnap = useCallback(() => {
    setSnapSettings((prev) => ({ ...prev, enabled: !prev.enabled }));
  }, []);

  const toggleSnapType = useCallback((type: SnapType) => {
    setSnapSettings((prev) => {
      const snapTypes = prev.snapTypes.includes(type)
        ? prev.snapTypes.filter((t) => t !== type)
        : [...prev.snapTypes, type];
      return { ...prev, snapTypes };
    });
  }, []);

  const snapToGrid = useCallback(
    (point: Point2D): Point2D => {
      if (!snapSettings.enabled || !snapSettings.gridSnap) {
        return point;
      }

      const gridSize = snapSettings.gridSize;
      return {
        x: Math.round(point.x / gridSize) * gridSize,
        y: Math.round(point.y / gridSize) * gridSize,
      };
    },
    [snapSettings]
  );

  const distance = useCallback((p1: Point2D, p2: Point2D): number => {
    return Math.sqrt(Math.pow(p2.x - p1.x, 2) + Math.pow(p2.y - p1.y, 2));
  }, []);

  const findEndpoints = useCallback(
    (point: Point2D, objects: any[]): SnapPoint | null => {
      if (!snapSettings.snapTypes.includes(SnapType.ENDPOINT)) {
        return null;
      }

      let closestSnap: SnapPoint | null = null;
      let minDistance = snapSettings.snapDistance;

      objects.forEach((obj) => {
        if (obj.points && Array.isArray(obj.points)) {
          obj.points.forEach((p: Point2D) => {
            const dist = distance(point, p);
            if (dist < minDistance) {
              minDistance = dist;
              closestSnap = {
                point: p,
                type: SnapType.ENDPOINT,
                objectId: obj.id,
              };
            }
          });
        }
      });

      return closestSnap;
    },
    [snapSettings, distance]
  );

  const findMidpoints = useCallback(
    (point: Point2D, objects: any[]): SnapPoint | null => {
      if (!snapSettings.snapTypes.includes(SnapType.MIDPOINT)) {
        return null;
      }

      let closestSnap: SnapPoint | null = null;
      let minDistance = snapSettings.snapDistance;

      objects.forEach((obj) => {
        if (obj.points && Array.isArray(obj.points) && obj.points.length >= 2) {
          for (let i = 0; i < obj.points.length - 1; i++) {
            const p1 = obj.points[i];
            const p2 = obj.points[i + 1];
            const midpoint = {
              x: (p1.x + p2.x) / 2,
              y: (p1.y + p2.y) / 2,
            };
            const dist = distance(point, midpoint);
            if (dist < minDistance) {
              minDistance = dist;
              closestSnap = {
                point: midpoint,
                type: SnapType.MIDPOINT,
                objectId: obj.id,
              };
            }
          }
        }
      });

      return closestSnap;
    },
    [snapSettings, distance]
  );

  const findCenters = useCallback(
    (point: Point2D, objects: any[]): SnapPoint | null => {
      if (!snapSettings.snapTypes.includes(SnapType.CENTER)) {
        return null;
      }

      let closestSnap: SnapPoint | null = null;
      let minDistance = snapSettings.snapDistance;

      objects.forEach((obj) => {
        if (obj.center) {
          const dist = distance(point, obj.center);
          if (dist < minDistance) {
            minDistance = dist;
            closestSnap = {
              point: obj.center,
              type: SnapType.CENTER,
              objectId: obj.id,
            };
          }
        } else if (obj.type === 'circle' && obj.x !== undefined && obj.y !== undefined) {
          const center = { x: obj.x, y: obj.y };
          const dist = distance(point, center);
          if (dist < minDistance) {
            minDistance = dist;
            closestSnap = {
              point: center,
              type: SnapType.CENTER,
              objectId: obj.id,
            };
          }
        }
      });

      return closestSnap;
    },
    [snapSettings, distance]
  );

  const findSnapPoint = useCallback(
    (point: Point2D, objects: any[] = []): SnapPoint | null => {
      if (!snapSettings.enabled) {
        return null;
      }

      // First try grid snap
      if (snapSettings.gridSnap) {
        const gridPoint = snapToGrid(point);
        if (distance(point, gridPoint) < snapSettings.snapDistance) {
          return {
            point: gridPoint,
            type: SnapType.GRID,
          };
        }
      }

      // Then try object snaps
      if (snapSettings.objectSnap && objects.length > 0) {
        const snapCandidates: (SnapPoint | null)[] = [
          findEndpoints(point, objects),
          findMidpoints(point, objects),
          findCenters(point, objects),
        ];

        // Return the closest snap point
        let closestSnap: SnapPoint | null = null;
        let minDist = snapSettings.snapDistance;

        snapCandidates.forEach((snap) => {
          if (snap) {
            const dist = distance(point, snap.point);
            if (dist < minDist) {
              minDist = dist;
              closestSnap = snap;
            }
          }
        });

        return closestSnap;
      }

      return null;
    },
    [
      snapSettings,
      snapToGrid,
      distance,
      findEndpoints,
      findMidpoints,
      findCenters,
    ]
  );

  return {
    snapSettings,
    updateSnapSettings,
    findSnapPoint,
    snapToGrid,
    toggleSnap,
    toggleSnapType,
  };
}
