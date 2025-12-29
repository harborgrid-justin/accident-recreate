/**
 * AccuScene Enterprise v0.3.0 - Export Hook
 * Custom hook for data export functionality with progress tracking
 */

import { useState, useCallback, useRef } from 'react';
import { ExportOptions, ExportState, AnalyticsData } from '../types';
import { DataExporter } from '../DataExporter';

/**
 * Hook for managing data export operations
 */
export function useExport(): ExportState & {
  exportData: (data: AnalyticsData, options: ExportOptions) => Promise<void>;
} {
  const [exporting, setExporting] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<Error | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  /**
   * Export data with progress tracking
   */
  const exportData = useCallback(
    async (data: AnalyticsData, options: ExportOptions) => {
      // Cancel any ongoing export
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      abortControllerRef.current = new AbortController();
      setExporting(true);
      setProgress(0);
      setError(null);

      try {
        // Simulate progress for different export stages
        setProgress(10);

        // Prepare data
        await new Promise((resolve) => setTimeout(resolve, 200));
        setProgress(30);

        // Convert data
        await new Promise((resolve) => setTimeout(resolve, 300));
        setProgress(60);

        // Export
        await DataExporter.export(data, options);
        setProgress(90);

        // Finalize
        await new Promise((resolve) => setTimeout(resolve, 100));
        setProgress(100);

        // Reset after a short delay
        setTimeout(() => {
          setExporting(false);
          setProgress(0);
        }, 500);
      } catch (err) {
        if (err instanceof Error && err.name !== 'AbortError') {
          setError(err);
          setExporting(false);
          setProgress(0);
        }
      }
    },
    []
  );

  /**
   * Cancel ongoing export
   */
  const cancel = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    setExporting(false);
    setProgress(0);
    setError(null);
  }, []);

  return {
    exporting,
    progress,
    error,
    exportData,
    cancel,
  };
}

/**
 * Hook for batch export operations
 */
export function useBatchExport() {
  const [exports, setExports] = useState<
    Map<string, { progress: number; error: Error | null }>
  >(new Map());

  const addExport = useCallback((id: string) => {
    setExports((prev) => {
      const next = new Map(prev);
      next.set(id, { progress: 0, error: null });
      return next;
    });
  }, []);

  const updateProgress = useCallback((id: string, progress: number) => {
    setExports((prev) => {
      const next = new Map(prev);
      const existing = next.get(id);
      if (existing) {
        next.set(id, { ...existing, progress });
      }
      return next;
    });
  }, []);

  const setExportError = useCallback((id: string, error: Error) => {
    setExports((prev) => {
      const next = new Map(prev);
      const existing = next.get(id);
      if (existing) {
        next.set(id, { ...existing, error });
      }
      return next;
    });
  }, []);

  const removeExport = useCallback((id: string) => {
    setExports((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  }, []);

  const clearCompleted = useCallback(() => {
    setExports((prev) => {
      const next = new Map(prev);
      for (const [id, status] of next.entries()) {
        if (status.progress === 100 || status.error) {
          next.delete(id);
        }
      }
      return next;
    });
  }, []);

  return {
    exports: Array.from(exports.entries()).map(([id, status]) => ({
      id,
      ...status,
    })),
    addExport,
    updateProgress,
    setExportError,
    removeExport,
    clearCompleted,
  };
}

/**
 * Hook for export history tracking
 */
export function useExportHistory() {
  const [history, setHistory] = useState<
    Array<{
      id: string;
      timestamp: number;
      format: string;
      filename: string;
      success: boolean;
      error?: string;
    }>
  >([]);

  const addToHistory = useCallback(
    (entry: {
      format: string;
      filename: string;
      success: boolean;
      error?: string;
    }) => {
      setHistory((prev) => [
        {
          id: `export-${Date.now()}`,
          timestamp: Date.now(),
          ...entry,
        },
        ...prev.slice(0, 99), // Keep last 100 entries
      ]);
    },
    []
  );

  const clearHistory = useCallback(() => {
    setHistory([]);
  }, []);

  const removeFromHistory = useCallback((id: string) => {
    setHistory((prev) => prev.filter((entry) => entry.id !== id));
  }, []);

  return {
    history,
    addToHistory,
    clearHistory,
    removeFromHistory,
  };
}

/**
 * Hook for export presets management
 */
export function useExportPresets() {
  const [presets, setPresets] = useState<
    Map<string, { name: string; options: ExportOptions }>
  >(new Map([
    [
      'csv-standard',
      {
        name: 'CSV - Standard Export',
        options: {
          format: 'csv',
          includeMetadata: true,
          compression: false,
        },
      },
    ],
    [
      'json-full',
      {
        name: 'JSON - Full Data',
        options: {
          format: 'json',
          includeMetadata: true,
          includeCharts: true,
          includeTables: true,
        },
      },
    ],
    [
      'pdf-report',
      {
        name: 'PDF - Executive Report',
        options: {
          format: 'pdf',
          includeMetadata: true,
          includeCharts: true,
          includeTables: true,
          pageSize: 'letter',
          orientation: 'portrait',
        },
      },
    ],
  ]));

  const addPreset = useCallback(
    (id: string, name: string, options: ExportOptions) => {
      setPresets((prev) => {
        const next = new Map(prev);
        next.set(id, { name, options });
        return next;
      });
    },
    []
  );

  const removePreset = useCallback((id: string) => {
    setPresets((prev) => {
      const next = new Map(prev);
      next.delete(id);
      return next;
    });
  }, []);

  const getPreset = useCallback(
    (id: string) => {
      return presets.get(id);
    },
    [presets]
  );

  return {
    presets: Array.from(presets.entries()).map(([id, preset]) => ({
      id,
      ...preset,
    })),
    addPreset,
    removePreset,
    getPreset,
  };
}

/**
 * Hook for clipboard export (copy to clipboard)
 */
export function useClipboardExport() {
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  const copyToClipboard = useCallback(async (data: any, format: 'text' | 'json' = 'text') => {
    setError(null);

    try {
      let text: string;

      if (format === 'json') {
        text = JSON.stringify(data, null, 2);
      } else {
        text = typeof data === 'string' ? data : String(data);
      }

      await navigator.clipboard.writeText(text);
      setCopied(true);

      // Reset copied state after 2 seconds
      if (timeoutRef.current) {
        clearTimeout(timeoutRef.current);
      }

      timeoutRef.current = setTimeout(() => {
        setCopied(false);
      }, 2000);
    } catch (err) {
      if (err instanceof Error) {
        setError(err);
      }
      setCopied(false);
    }
  }, []);

  return {
    copied,
    error,
    copyToClipboard,
  };
}
