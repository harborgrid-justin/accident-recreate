/**
 * useNetworkStatus Hook
 * React hook for network connectivity monitoring
 */

import { useState, useEffect, useCallback } from 'react';
import { NetworkState, NetworkQuality } from '../types';
import { NetworkDetector } from '../NetworkDetector';

export interface UseNetworkStatusOptions {
  apiEndpoint?: string;
  checkIntervalMs?: number;
  autoStart?: boolean;
}

export interface UseNetworkStatusReturn {
  // State
  isOnline: boolean;
  networkState: NetworkState;
  quality: NetworkQuality | null;

  // Actions
  checkNow: () => Promise<void>;
  waitForOnline: (timeoutMs?: number) => Promise<boolean>;

  // Detector access
  detector: NetworkDetector;
}

export function useNetworkStatus(
  options: UseNetworkStatusOptions = {}
): UseNetworkStatusReturn {
  const {
    apiEndpoint = 'https://api.accuscene.com',
    checkIntervalMs = 5000,
    autoStart = true,
  } = options;

  const [detector] = useState(
    () => new NetworkDetector(apiEndpoint, checkIntervalMs)
  );
  const [networkState, setNetworkState] = useState<NetworkState>(
    NetworkState.Unknown
  );
  const [quality, setQuality] = useState<NetworkQuality | null>(null);

  // Setup detector
  useEffect(() => {
    // Listen to state changes
    const unsubscribe = detector.addListener((state) => {
      setNetworkState(state);
      setQuality(detector.getQuality());
    });

    // Start monitoring
    if (autoStart) {
      detector.start();
      setNetworkState(detector.getState());
      setQuality(detector.getQuality());
    }

    return () => {
      unsubscribe();
      detector.stop();
    };
  }, [detector, autoStart]);

  // Check connectivity now
  const checkNow = useCallback(async () => {
    await detector.forceCheck();
    setNetworkState(detector.getState());
    setQuality(detector.getQuality());
  }, [detector]);

  // Wait for online
  const waitForOnline = useCallback(
    async (timeoutMs: number = 30000) => {
      const result = await detector.waitForOnline(timeoutMs);
      setNetworkState(detector.getState());
      setQuality(detector.getQuality());
      return result;
    },
    [detector]
  );

  return {
    isOnline: detector.isOnline(),
    networkState,
    quality,
    checkNow,
    waitForOnline,
    detector,
  };
}
