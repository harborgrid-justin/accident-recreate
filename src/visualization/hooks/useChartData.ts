import { useState, useEffect, useCallback, useMemo } from 'react';
import { ChartData, SeriesData, DataPoint } from '../types';

interface UseChartDataOptions {
  endpoint?: string;
  transform?: (data: any) => ChartData;
  pollInterval?: number;
}

interface UseChartDataResult {
  data: ChartData | null;
  loading: boolean;
  error: Error | null;
  refetch: () => Promise<void>;
  updateSeries: (seriesName: string, newData: DataPoint[]) => void;
}

/**
 * Hook for fetching and managing chart data
 */
export default function useChartData(
  initialData?: ChartData,
  options: UseChartDataOptions = {}
): UseChartDataResult {
  const [data, setData] = useState<ChartData | null>(initialData || null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const { endpoint, transform, pollInterval } = options;

  const fetchData = useCallback(async () => {
    if (!endpoint) return;

    setLoading(true);
    setError(null);

    try {
      const response = await fetch(endpoint);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const json = await response.json();
      const chartData = transform ? transform(json) : json;
      setData(chartData);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [endpoint, transform]);

  const updateSeries = useCallback((seriesName: string, newData: DataPoint[]) => {
    setData((prevData) => {
      if (!prevData) return null;

      return {
        ...prevData,
        series: prevData.series.map((series) =>
          series.name === seriesName
            ? { ...series, data: newData }
            : series
        ),
      };
    });
  }, []);

  useEffect(() => {
    if (endpoint) {
      fetchData();
    }
  }, [endpoint, fetchData]);

  useEffect(() => {
    if (pollInterval && endpoint) {
      const interval = setInterval(fetchData, pollInterval);
      return () => clearInterval(interval);
    }
  }, [pollInterval, endpoint, fetchData]);

  return {
    data,
    loading,
    error,
    refetch: fetchData,
    updateSeries,
  };
}

/**
 * Hook for managing chart zoom state
 */
export function useChartZoom(initialDomain?: {
  x: [number, number];
  y: [number, number];
}) {
  const [xDomain, setXDomain] = useState<[number, number] | null>(
    initialDomain?.x || null
  );
  const [yDomain, setYDomain] = useState<[number, number] | null>(
    initialDomain?.y || null
  );

  const resetZoom = useCallback(() => {
    setXDomain(null);
    setYDomain(null);
  }, []);

  const setZoom = useCallback(
    (x: [number, number] | null, y: [number, number] | null) => {
      setXDomain(x);
      setYDomain(y);
    },
    []
  );

  return {
    xDomain,
    yDomain,
    setXDomain,
    setYDomain,
    setZoom,
    resetZoom,
  };
}

/**
 * Hook for managing chart dimensions based on container size
 */
export function useChartDimensions(
  ref: React.RefObject<HTMLElement>,
  margin = { top: 20, right: 20, bottom: 40, left: 60 }
) {
  const [dimensions, setDimensions] = useState({
    width: 800,
    height: 600,
    boundedWidth: 720,
    boundedHeight: 540,
  });

  useEffect(() => {
    if (!ref.current) return;

    const updateDimensions = () => {
      if (!ref.current) return;

      const { width, height } = ref.current.getBoundingClientRect();
      const boundedWidth = width - margin.left - margin.right;
      const boundedHeight = height - margin.top - margin.bottom;

      setDimensions({
        width,
        height,
        boundedWidth,
        boundedHeight,
      });
    };

    updateDimensions();

    const resizeObserver = new ResizeObserver(updateDimensions);
    resizeObserver.observe(ref.current);

    return () => resizeObserver.disconnect();
  }, [ref, margin]);

  return dimensions;
}

/**
 * Hook for managing chart tooltip state
 */
export function useChartTooltip<T = any>() {
  const [tooltip, setTooltip] = useState<{
    x: number;
    y: number;
    data: T;
  } | null>(null);

  const showTooltip = useCallback((x: number, y: number, data: T) => {
    setTooltip({ x, y, data });
  }, []);

  const hideTooltip = useCallback(() => {
    setTooltip(null);
  }, []);

  return {
    tooltip,
    showTooltip,
    hideTooltip,
    isVisible: tooltip !== null,
  };
}

/**
 * Hook for managing legend visibility
 */
export function useLegendVisibility(series: SeriesData[]) {
  const [visibility, setVisibility] = useState<Record<string, boolean>>(() => {
    const initial: Record<string, boolean> = {};
    series.forEach((s) => {
      initial[s.name] = true;
    });
    return initial;
  });

  const toggleSeries = useCallback((seriesName: string) => {
    setVisibility((prev) => ({
      ...prev,
      [seriesName]: !prev[seriesName],
    }));
  }, []);

  const showAll = useCallback(() => {
    setVisibility((prev) => {
      const updated = { ...prev };
      Object.keys(updated).forEach((key) => {
        updated[key] = true;
      });
      return updated;
    });
  }, []);

  const hideAll = useCallback(() => {
    setVisibility((prev) => {
      const updated = { ...prev };
      Object.keys(updated).forEach((key) => {
        updated[key] = false;
      });
      return updated;
    });
  }, []);

  const visibleSeries = useMemo(() => {
    return series.filter((s) => visibility[s.name] !== false);
  }, [series, visibility]);

  return {
    visibility,
    toggleSeries,
    showAll,
    hideAll,
    visibleSeries,
  };
}
