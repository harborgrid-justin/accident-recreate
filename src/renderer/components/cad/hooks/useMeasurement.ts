/**
 * AccuScene Enterprise v0.3.0 - Measurement Calculation Hook
 */

import { useState, useCallback } from 'react';
import { Point2D, Measurement } from '../types';

export interface UseMeasurementReturn {
  measurements: Measurement[];
  addMeasurement: (measurement: Omit<Measurement, 'id'>) => void;
  removeMeasurement: (id: string) => void;
  clearMeasurements: () => void;
  calculateDistance: (p1: Point2D, p2: Point2D) => number;
  calculateAngle: (p1: Point2D, p2: Point2D, p3: Point2D) => number;
  calculateArea: (points: Point2D[]) => number;
  formatValue: (value: number, unit: string, decimals?: number) => string;
}

export function useMeasurement(): UseMeasurementReturn {
  const [measurements, setMeasurements] = useState<Measurement[]>([]);

  const addMeasurement = useCallback((measurement: Omit<Measurement, 'id'>) => {
    const id = `measurement-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
    setMeasurements((prev) => [...prev, { ...measurement, id }]);
  }, []);

  const removeMeasurement = useCallback((id: string) => {
    setMeasurements((prev) => prev.filter((m) => m.id !== id));
  }, []);

  const clearMeasurements = useCallback(() => {
    setMeasurements([]);
  }, []);

  const calculateDistance = useCallback((p1: Point2D, p2: Point2D): number => {
    return Math.sqrt(Math.pow(p2.x - p1.x, 2) + Math.pow(p2.y - p1.y, 2));
  }, []);

  const calculateAngle = useCallback(
    (p1: Point2D, p2: Point2D, p3: Point2D): number => {
      // Calculate angle at p2 between p1-p2 and p2-p3
      const v1 = { x: p1.x - p2.x, y: p1.y - p2.y };
      const v2 = { x: p3.x - p2.x, y: p3.y - p2.y };

      const dot = v1.x * v2.x + v1.y * v2.y;
      const det = v1.x * v2.y - v1.y * v2.x;
      const angle = Math.atan2(det, dot);

      // Convert to degrees
      let degrees = (angle * 180) / Math.PI;

      // Normalize to 0-360
      if (degrees < 0) {
        degrees += 360;
      }

      return degrees;
    },
    []
  );

  const calculateArea = useCallback((points: Point2D[]): number => {
    if (points.length < 3) return 0;

    // Shoelace formula for polygon area
    let area = 0;
    for (let i = 0; i < points.length; i++) {
      const j = (i + 1) % points.length;
      area += points[i].x * points[j].y;
      area -= points[j].x * points[i].y;
    }
    return Math.abs(area / 2);
  }, []);

  const formatValue = useCallback(
    (value: number, unit: string, decimals: number = 2): string => {
      return `${value.toFixed(decimals)} ${unit}`;
    },
    []
  );

  return {
    measurements,
    addMeasurement,
    removeMeasurement,
    clearMeasurements,
    calculateDistance,
    calculateAngle,
    calculateArea,
    formatValue,
  };
}
