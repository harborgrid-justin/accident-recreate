import { useState, useCallback } from 'react';
import {
  ImportRequest,
  ImportResponse,
  TransferProgress,
  FieldSchema,
  ValidationResult,
  TransferConfig,
} from '../types';
import { transferService } from '../services/TransferService';

interface UseImportResult {
  importData: (request: ImportRequest) => Promise<ImportResponse>;
  importStream: (
    request: ImportRequest,
    onProgress?: (progress: TransferProgress) => void
  ) => Promise<ImportResponse>;
  detectSchema: (
    file: File,
    config: Partial<TransferConfig>
  ) => Promise<FieldSchema[]>;
  validateData: (request: ImportRequest) => Promise<ValidationResult>;
  progress: TransferProgress | null;
  error: string | null;
  isImporting: boolean;
}

/**
 * Hook for handling data import operations
 */
export function useImport(): UseImportResult {
  const [progress, setProgress] = useState<TransferProgress | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isImporting, setIsImporting] = useState(false);

  const importData = useCallback(async (request: ImportRequest): Promise<ImportResponse> => {
    setIsImporting(true);
    setError(null);
    setProgress(null);

    try {
      // Check file size to determine import strategy
      const fileSizeMB = request.file.size / (1024 * 1024);

      if (fileSizeMB < 10) {
        // For small files, use direct import
        const response = await transferService.import(request);

        if (!response.success) {
          setError(response.error || 'Import failed');
        }

        return response;
      } else {
        // For large files, use streaming import
        const response = await transferService.importStream(
          request,
          setProgress
        );

        if (!response.success) {
          setError(response.error || 'Import failed');
        }

        return response;
      }
    } catch (err) {
      const errorMessage =
        err instanceof Error ? err.message : 'Import failed';
      setError(errorMessage);
      return {
        success: false,
        error: errorMessage,
      };
    } finally {
      setIsImporting(false);
    }
  }, []);

  const importStream = useCallback(
    async (
      request: ImportRequest,
      onProgress?: (progress: TransferProgress) => void
    ): Promise<ImportResponse> => {
      setIsImporting(true);
      setError(null);
      setProgress(null);

      try {
        const progressCallback = (p: TransferProgress) => {
          setProgress(p);
          onProgress?.(p);
        };

        const response = await transferService.importStream(
          request,
          progressCallback
        );

        if (!response.success) {
          setError(response.error || 'Import failed');
        }

        return response;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Import failed';
        setError(errorMessage);
        return {
          success: false,
          error: errorMessage,
        };
      } finally {
        setIsImporting(false);
      }
    },
    []
  );

  const detectSchema = useCallback(
    async (
      file: File,
      config: Partial<TransferConfig>
    ): Promise<FieldSchema[]> => {
      setError(null);

      try {
        const schema = await transferService.detectSchema(file, config);
        return schema;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Schema detection failed';
        setError(errorMessage);
        throw err;
      }
    },
    []
  );

  const validateData = useCallback(
    async (request: ImportRequest): Promise<ValidationResult> => {
      setError(null);

      try {
        const validationResult = await transferService.validate(request);
        return validationResult;
      } catch (err) {
        const errorMessage =
          err instanceof Error ? err.message : 'Validation failed';
        setError(errorMessage);
        throw err;
      }
    },
    []
  );

  return {
    importData,
    importStream,
    detectSchema,
    validateData,
    progress,
    error,
    isImporting,
  };
}
