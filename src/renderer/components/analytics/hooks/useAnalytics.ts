/**
 * AccuScene Enterprise v0.3.0 - Analytics Hook
 * Custom hook for managing analytics data with real-time updates
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import { AnalyticsData, AnalyticsState } from '../types';

/**
 * Hook for managing analytics data
 */
export function useAnalytics(caseId?: string): AnalyticsState {
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<Error | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  /**
   * Fetch analytics data
   */
  const fetchData = useCallback(async () => {
    // Cancel previous request if still pending
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    abortControllerRef.current = new AbortController();
    setLoading(true);
    setError(null);

    try {
      // In production, this would fetch from an API
      // For now, generate mock data
      await new Promise((resolve) => setTimeout(resolve, 500)); // Simulate network delay

      const mockData = generateMockData(caseId);
      setData(mockData);
    } catch (err) {
      if (err instanceof Error && err.name !== 'AbortError') {
        setError(err);
      }
    } finally {
      setLoading(false);
    }
  }, [caseId]);

  /**
   * Refresh data
   */
  const refresh = useCallback(async () => {
    await fetchData();
  }, [fetchData]);

  /**
   * Update data (partial update)
   */
  const updateData = useCallback((updates: Partial<AnalyticsData>) => {
    setData((prev) => {
      if (!prev) return prev;
      return {
        ...prev,
        ...updates,
        timestamp: Date.now(),
      };
    });
  }, []);

  // Initial data fetch
  useEffect(() => {
    fetchData();

    return () => {
      // Cleanup: abort any pending requests
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
    };
  }, [fetchData]);

  // Set up real-time updates (optional)
  useEffect(() => {
    if (!caseId) return;

    // In production, this would set up WebSocket or SSE connection
    // For now, use polling as a simple example
    const interval = setInterval(() => {
      // Only refresh if not already loading
      if (!loading) {
        fetchData();
      }
    }, 30000); // Refresh every 30 seconds

    return () => {
      clearInterval(interval);
    };
  }, [caseId, loading, fetchData]);

  return {
    data,
    loading,
    error,
    refresh,
    updateData,
  };
}

/**
 * Generate mock analytics data for development/testing
 */
function generateMockData(caseId?: string): AnalyticsData {
  // Generate mock vehicles
  const vehicles = [
    {
      id: 'vehicle-1',
      name: 'Vehicle A',
      mass: 1500,
      dimensions: { length: 4.5, width: 1.8, height: 1.5 },
      damageProfile: generateMockDamage(8),
      trajectory: generateMockTrajectory(50),
      speed: generateMockSpeed(50),
      forces: generateMockForces(10),
    },
    {
      id: 'vehicle-2',
      name: 'Vehicle B',
      mass: 2000,
      dimensions: { length: 5.0, width: 2.0, height: 1.7 },
      damageProfile: generateMockDamage(6),
      trajectory: generateMockTrajectory(50),
      speed: generateMockSpeed(50),
      forces: generateMockForces(8),
    },
  ];

  // Generate mock impacts
  const impacts = [
    {
      id: 'impact-1',
      timestamp: 2.35,
      location: { x: 5.2, y: 0.8, z: 0.0 },
      force: { x: -25000, y: -8000, z: 0 },
      energy: 125000,
      duration: 0.15,
      vehicles: ['vehicle-1', 'vehicle-2'],
      severity: 0.85,
      type: 'primary' as const,
    },
    {
      id: 'impact-2',
      timestamp: 2.5,
      location: { x: 5.8, y: 1.2, z: 0.0 },
      force: { x: -15000, y: -5000, z: 0 },
      energy: 45000,
      duration: 0.08,
      vehicles: ['vehicle-1', 'vehicle-2'],
      severity: 0.62,
      type: 'secondary' as const,
    },
  ];

  // Generate mock force vectors
  const forceVectors = [
    {
      id: 'force-1',
      origin: { x: 5.2, y: 0.8, z: 0.5 },
      direction: { x: -0.95, y: -0.31, z: 0 },
      magnitude: 45000,
      timestamp: 2.35,
      type: 'impact' as const,
    },
    {
      id: 'force-2',
      origin: { x: 5.2, y: 0.8, z: 0.0 },
      direction: { x: 0, y: 0, z: -1 },
      magnitude: 12000,
      timestamp: 2.35,
      type: 'normal' as const,
    },
    {
      id: 'force-3',
      origin: { x: 5.2, y: 0.8, z: 0.0 },
      direction: { x: 0.31, y: 0.95, z: 0 },
      magnitude: 8000,
      timestamp: 2.36,
      type: 'friction' as const,
    },
  ];

  // Generate mock energy transfers
  const energyTransfers = [
    {
      source: 'Vehicle A - Kinetic',
      target: 'Impact - Deformation',
      amount: 75000,
      type: 'kinetic' as const,
      efficiency: 0.6,
    },
    {
      source: 'Vehicle A - Kinetic',
      target: 'Heat Dissipation',
      amount: 30000,
      type: 'heat' as const,
      efficiency: 0.24,
    },
    {
      source: 'Vehicle B - Kinetic',
      target: 'Impact - Deformation',
      amount: 50000,
      type: 'kinetic' as const,
      efficiency: 0.4,
    },
    {
      source: 'Vehicle B - Kinetic',
      target: 'Heat Dissipation',
      amount: 20000,
      type: 'heat' as const,
      efficiency: 0.16,
    },
  ];

  // Generate mock timeline
  const timeline = [
    {
      id: 'event-1',
      timestamp: 0.0,
      title: 'Initial State',
      description: 'Vehicles approaching intersection',
      type: 'system' as const,
    },
    {
      id: 'event-2',
      timestamp: 1.5,
      title: 'Vehicle A Braking',
      description: 'Driver applies brakes',
      type: 'vehicle-action' as const,
      severity: 'medium' as const,
    },
    {
      id: 'event-3',
      timestamp: 2.35,
      title: 'Primary Impact',
      description: 'Initial collision between vehicles',
      type: 'impact' as const,
      severity: 'critical' as const,
    },
    {
      id: 'event-4',
      timestamp: 2.5,
      title: 'Secondary Impact',
      description: 'Additional contact during separation',
      type: 'impact' as const,
      severity: 'high' as const,
    },
    {
      id: 'event-5',
      timestamp: 3.2,
      title: 'Rest Position',
      description: 'Vehicles come to rest',
      type: 'system' as const,
      severity: 'low' as const,
    },
  ];

  return {
    vehicles,
    impacts,
    energyTransfers,
    forceVectors,
    timeline,
    metadata: {
      caseId: caseId || 'mock-case',
      generatedAt: Date.now(),
      version: '1.0',
    },
    timestamp: Date.now(),
  };
}

// Helper functions for generating mock data
function generateMockDamage(count: number) {
  return Array.from({ length: count }, (_, i) => ({
    id: `damage-${i}`,
    location: {
      x: Math.random() * 4 - 2,
      y: Math.random() * 2 - 1,
      z: Math.random() * 1,
    },
    severity: Math.random() * 0.8 + 0.2,
    type: ['crush', 'scratch', 'dent', 'break', 'deformation'][
      Math.floor(Math.random() * 5)
    ] as any,
    area: Math.random() * 0.5 + 0.1,
    depth: Math.random() * 15 + 2,
  }));
}

function generateMockTrajectory(count: number) {
  return Array.from({ length: count }, (_, i) => {
    const t = i * 0.1;
    return {
      time: t,
      position: {
        x: t * 10,
        y: Math.sin(t) * 2,
        z: 0,
      },
      velocity: {
        x: 10,
        y: Math.cos(t) * 2,
        z: 0,
      },
      heading: t * 0.1,
      roll: 0,
      pitch: 0,
      yaw: t * 0.1,
    };
  });
}

function generateMockSpeed(count: number) {
  return Array.from({ length: count }, (_, i) => ({
    timestamp: i * 0.1,
    value: 60 - i * 0.8 + Math.random() * 2,
  }));
}

function generateMockForces(count: number) {
  return Array.from({ length: count }, (_, i) => ({
    x: Math.random() * 1000 - 500,
    y: Math.random() * 1000 - 500,
    z: Math.random() * 500,
  }));
}
