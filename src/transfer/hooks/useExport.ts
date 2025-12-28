import { useState, useCallback } from 'react';
import {
  ExportRequest,
  ExportResponse,
  TransferProgress,
} from '../types';
import { transferService } from '../services/TransferService';

interface UseExportResult {
  exportData: (request: ExportRequest) => Promise<ExportResponse>;
  exportStream: (
    request: ExportRequest,
    onProgress?: (progress: TransferProgress) => void
  ) => Promise<ExportResponse>;
  downloadFile: (blob: Blob, filename: string) => void;
  progress: TransferProgress | null;
  error: string | null;
  isExporting: boolean;
}

/**
 * Hook for handling data export operations
 */
export function useExport(): UseExportResult {
  const [progress, setProgress] = useState<TransferProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isExporting, setIsExporting] = useState(false);

  const exportData = useCallback(async (request: ExportRequest): Promise<ExportResponse> => {
    setIsExporting(true);
    setError(null);
    setProgress(null);

    try {
      // For small datasets, use direct export
      if (request.data.length < 1000) {
        const response = await transferService.export(request);

        if (!response.success) {
          setError(response.error || 'Export failed');
        }

        return response;
      } else {
        // For large datasets, use streaming export
        const response = await transferService.exportStream(
          request,
          setProgress
        );

        if (!response.success) {
          setError(response.error || 'Export failed');
        }

        return response;
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Export failed';
      setError(errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    } finally {
      setIsExporting(false);
    }
  }, []);

  const exportStream = useCallback(
    async (
      request: ExportRequest,
      onProgress?: (progress: TransferProgress) => void
    ): Promise<ExportResponse> => {
      setIsExporting(true);
      setError(null);
      setProgress(null);

      try {
        const progressCallback = (p: TransferProgress) => {
          setProgress(p);
          onProgress?.(p);
        };

        const response = await transferService.exportStream(
          request,
          progressCallback
        );

        if (!response.success) {
          setError(response.error || 'Export failed');
        }

        return response;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Export failed';
        setError(errorMessage);
        return {
          success: false,
          error: errorMessage,
        };
      } finally {
        setIsExporting(false);
      }
    },
    []
  );

  const downloadFile = useCallback((blob: Blob, filename: string) => {
    transferService.downloadFile(blob, filename);
  }, []);

  return {
    exportData,
    exportStream,
    downloadFile,
    progress,
    error,
    isExporting,
  };
}
