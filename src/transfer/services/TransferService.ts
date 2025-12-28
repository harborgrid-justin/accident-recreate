/**
 * Transfer Service
 * Handles communication with the Rust backend for import/export operations
 */

import {
  ExportRequest,
  ExportResponse,
  ImportRequest,
  ImportResponse,
  TransferConfig,
  FieldSchema,
  ValidationResult,
  TransferProgress,
} from '../types';

export class TransferService {
  private baseUrl: string;
  private progressCallbacks: Map<string, (progress: TransferProgress) => void>;

  constructor(baseUrl: string = '/api/transfer') {
    this.baseUrl = baseUrl;
    this.progressCallbacks = new Map();
  }

  /**
   * Export data
   */
  async export(request: ExportRequest): Promise<ExportResponse> {
    try {
      const response = await fetch(`${this.baseUrl}/export`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Export failed');
      }

      // Get the blob data
      const blob = await response.blob();

      // Get metadata from headers
      const metadataHeader = response.headers.get('X-Transfer-Metadata');
      const metadata = metadataHeader ? JSON.parse(metadataHeader) : undefined;

      return {
        success: true,
        data: blob,
        metadata,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Export with streaming and progress tracking
   */
  async exportStream(
    request: ExportRequest,
    onProgress?: (progress: TransferProgress) => void
  ): Promise<ExportResponse> {
    try {
      // Start the export
      const startResponse = await fetch(`${this.baseUrl}/export/stream`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(request),
      });

      if (!startResponse.ok) {
        throw new Error('Failed to start export');
      }

      const { taskId } = await startResponse.json();

      // Poll for progress
      if (onProgress) {
        this.pollProgress(taskId, onProgress);
      }

      // Wait for completion
      const result = await this.waitForCompletion(taskId);

      if (result.status === 'completed') {
        // Download the result
        const downloadResponse = await fetch(
          `${this.baseUrl}/download/${taskId}`
        );
        const blob = await downloadResponse.blob();

        return {
          success: true,
          data: blob,
          metadata: result.metadata,
          progress: result.progress,
        };
      } else {
        throw new Error(result.error || 'Export failed');
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Import data
   */
  async import(request: ImportRequest): Promise<ImportResponse> {
    try {
      const formData = new FormData();
      formData.append('file', request.file);
      if (request.format) {
        formData.append('format', request.format);
      }
      formData.append('config', JSON.stringify(request.config));
      formData.append('validate', String(request.validate));
      if (request.mappingProfile) {
        formData.append('mapping', JSON.stringify(request.mappingProfile));
      }

      const response = await fetch(`${this.baseUrl}/import`, {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        const error = await response.text();
        throw new Error(error || 'Import failed');
      }

      const result = await response.json();

      return {
        success: true,
        data: result.data,
        metadata: result.metadata,
        schema: result.schema,
        validationResult: result.validationResult,
      };
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Import with streaming and progress tracking
   */
  async importStream(
    request: ImportRequest,
    onProgress?: (progress: TransferProgress) => void
  ): Promise<ImportResponse> {
    try {
      const formData = new FormData();
      formData.append('file', request.file);
      if (request.format) {
        formData.append('format', request.format);
      }
      formData.append('config', JSON.stringify(request.config));
      formData.append('validate', String(request.validate));
      if (request.mappingProfile) {
        formData.append('mapping', JSON.stringify(request.mappingProfile));
      }

      // Start the import
      const startResponse = await fetch(`${this.baseUrl}/import/stream`, {
        method: 'POST',
        body: formData,
      });

      if (!startResponse.ok) {
        throw new Error('Failed to start import');
      }

      const { taskId } = await startResponse.json();

      // Poll for progress
      if (onProgress) {
        this.pollProgress(taskId, onProgress);
      }

      // Wait for completion
      const result = await this.waitForCompletion(taskId);

      if (result.status === 'completed') {
        return {
          success: true,
          data: result.data,
          metadata: result.metadata,
          schema: result.schema,
          validationResult: result.validationResult,
          progress: result.progress,
        };
      } else {
        throw new Error(result.error || 'Import failed');
      }
    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error',
      };
    }
  }

  /**
   * Detect schema from file
   */
  async detectSchema(
    file: File,
    config: Partial<TransferConfig>
  ): Promise<FieldSchema[]> {
    try {
      const formData = new FormData();
      formData.append('file', file);
      formData.append('config', JSON.stringify(config));

      const response = await fetch(`${this.baseUrl}/schema/detect`, {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Schema detection failed');
      }

      const { schema } = await response.json();
      return schema;
    } catch (error) {
      throw new Error(
        error instanceof Error ? error.message : 'Schema detection failed'
      );
    }
  }

  /**
   * Validate data
   */
  async validate(request: ImportRequest): Promise<ValidationResult> {
    try {
      const formData = new FormData();
      formData.append('file', request.file);
      formData.append('config', JSON.stringify(request.config));

      const response = await fetch(`${this.baseUrl}/validate`, {
        method: 'POST',
        body: formData,
      });

      if (!response.ok) {
        throw new Error('Validation failed');
      }

      const result = await response.json();
      return result.validationResult;
    } catch (error) {
      throw new Error(
        error instanceof Error ? error.message : 'Validation failed'
      );
    }
  }

  /**
   * Get progress for a task
   */
  async getProgress(taskId: string): Promise<TransferProgress> {
    const response = await fetch(`${this.baseUrl}/progress/${taskId}`);
    if (!response.ok) {
      throw new Error('Failed to get progress');
    }
    return response.json();
  }

  /**
   * Cancel a task
   */
  async cancel(taskId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/cancel/${taskId}`, {
      method: 'POST',
    });
    if (!response.ok) {
      throw new Error('Failed to cancel task');
    }
  }

  /**
   * Poll for progress updates
   */
  private pollProgress(
    taskId: string,
    callback: (progress: TransferProgress) => void,
    interval: number = 500
  ): void {
    this.progressCallbacks.set(taskId, callback);

    const poll = async () => {
      try {
        const progress = await this.getProgress(taskId);
        callback(progress);

        // Continue polling if task is still running
        if (progress.status === 'running' || progress.status === 'pending') {
          setTimeout(poll, interval);
        } else {
          this.progressCallbacks.delete(taskId);
        }
      } catch (error) {
        console.error('Progress polling error:', error);
        this.progressCallbacks.delete(taskId);
      }
    };

    poll();
  }

  /**
   * Wait for task completion
   */
  private async waitForCompletion(taskId: string): Promise<any> {
    return new Promise((resolve, reject) => {
      const checkStatus = async () => {
        try {
          const response = await fetch(`${this.baseUrl}/status/${taskId}`);
          if (!response.ok) {
            throw new Error('Failed to get status');
          }

          const result = await response.json();

          if (result.status === 'completed' || result.status === 'failed') {
            resolve(result);
          } else {
            setTimeout(checkStatus, 1000);
          }
        } catch (error) {
          reject(error);
        }
      };

      checkStatus();
    });
  }

  /**
   * Download exported file
   */
  async downloadFile(blob: Blob, filename: string): void {
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  /**
   * Get supported formats
   */
  async getSupportedFormats(): Promise<string[]> {
    try {
      const response = await fetch(`${this.baseUrl}/formats`);
      if (!response.ok) {
        throw new Error('Failed to get formats');
      }
      const { formats } = await response.json();
      return formats;
    } catch (error) {
      // Return default formats
      return ['csv', 'excel', 'json', 'xml', 'pdf', 'archive'];
    }
  }
}

// Singleton instance
export const transferService = new TransferService();
