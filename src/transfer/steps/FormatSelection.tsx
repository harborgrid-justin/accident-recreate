import React, { useCallback, useRef } from 'react';
import { TransferFormat, FORMAT_OPTIONS, FormatOption } from '../types';
import { FormatCard } from '../components/FormatCard';

interface FormatSelectionProps {
  state: any;
  onFormatSelect?: (format: TransferFormat) => void;
  onFileSelect?: (file: File, format?: TransferFormat) => void;
  mode: 'import' | 'export';
}

export const FormatSelection: React.FC<FormatSelectionProps> = ({
  state,
  onFormatSelect,
  onFileSelect,
  mode,
}) => {
  const fileInputRef = useRef<HTMLInputElement>(null);
  const [selectedFile, setSelectedFile] = React.useState<File | null>(state.file || null);
  const [dragActive, setDragActive] = React.useState(false);

  const availableFormats = FORMAT_OPTIONS.filter((fmt) =>
    mode === 'import' ? fmt.supportsImport : fmt.supportsExport
  );

  const handleFormatClick = useCallback(
    (format: TransferFormat) => {
      if (mode === 'export') {
        onFormatSelect?.(format);
      } else {
        // For import, trigger file selection
        if (fileInputRef.current) {
          const option = FORMAT_OPTIONS.find((f) => f.format === format);
          if (option) {
            fileInputRef.current.accept = `.${option.fileExtension},${option.mimeType}`;
          }
          fileInputRef.current.click();
        }
      }
    },
    [mode, onFormatSelect]
  );

  const handleFileChange = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (file) {
        setSelectedFile(file);
        const format = detectFormatFromFile(file);
        onFileSelect?.(file, format);
      }
    },
    [onFileSelect]
  );

  const handleDrag = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.type === 'dragenter' || e.type === 'dragover') {
      setDragActive(true);
    } else if (e.type === 'dragleave') {
      setDragActive(false);
    }
  }, []);

  const handleDrop = useCallback(
    (e: React.DragEvent) => {
      e.preventDefault();
      e.stopPropagation();
      setDragActive(false);

      const file = e.dataTransfer.files?.[0];
      if (file) {
        setSelectedFile(file);
        const format = detectFormatFromFile(file);
        onFileSelect?.(file, format);
      }
    },
    [onFileSelect]
  );

  if (mode === 'import') {
    return (
      <div className="format-selection">
        {/* File Upload Area */}
        <div
          className={`border-2 border-dashed rounded-lg p-12 text-center ${
            dragActive
              ? 'border-blue-500 bg-blue-50'
              : 'border-gray-300 bg-gray-50'
          }`}
          onDragEnter={handleDrag}
          onDragLeave={handleDrag}
          onDragOver={handleDrag}
          onDrop={handleDrop}
        >
          {selectedFile ? (
            <div className="space-y-4">
              <div className="mx-auto w-16 h-16 bg-green-100 rounded-full flex items-center justify-center">
                <svg
                  className="w-8 h-8 text-green-600"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M5 13l4 4L19 7"
                  />
                </svg>
              </div>
              <div>
                <p className="text-lg font-medium text-gray-900">
                  {selectedFile.name}
                </p>
                <p className="text-sm text-gray-500">
                  {formatFileSize(selectedFile.size)} - {state.format?.toUpperCase()}
                </p>
              </div>
              <button
                type="button"
                onClick={() => {
                  setSelectedFile(null);
                  onFileSelect?.(null as any);
                }}
                className="text-sm text-blue-600 hover:text-blue-800"
              >
                Choose different file
              </button>
            </div>
          ) : (
            <>
              <div className="mx-auto w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4">
                <svg
                  className="w-8 h-8 text-gray-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                  />
                </svg>
              </div>
              <p className="text-lg font-medium text-gray-900 mb-2">
                Drop your file here, or click to browse
              </p>
              <p className="text-sm text-gray-500 mb-6">
                Supports CSV, Excel, JSON, XML, and ZIP files up to 100MB
              </p>
              <button
                type="button"
                onClick={() => fileInputRef.current?.click()}
                className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700"
              >
                Select File
              </button>
            </>
          )}
        </div>

        <input
          ref={fileInputRef}
          type="file"
          className="hidden"
          onChange={handleFileChange}
          accept=".csv,.xlsx,.xls,.json,.xml,.zip"
        />

        {/* Format Cards */}
        <div className="mt-8">
          <h4 className="text-sm font-medium text-gray-900 mb-4">
            Or select a format to import
          </h4>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {availableFormats.map((option) => (
              <FormatCard
                key={option.format}
                option={option}
                selected={state.format === option.format}
                onClick={() => handleFormatClick(option.format)}
              />
            ))}
          </div>
        </div>
      </div>
    );
  }

  // Export mode
  return (
    <div className="format-selection">
      <div className="mb-6">
        <h4 className="text-sm font-medium text-gray-900 mb-4">
          Choose an export format
        </h4>
        <p className="text-sm text-gray-600">
          Select the format that best suits your needs. Each format has different
          capabilities and use cases.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {availableFormats.map((option) => (
          <FormatCard
            key={option.format}
            option={option}
            selected={state.format === option.format}
            onClick={() => handleFormatClick(option.format)}
          />
        ))}
      </div>

      {state.format && (
        <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
          <h5 className="text-sm font-medium text-blue-900 mb-2">
            Selected Format: {FORMAT_OPTIONS.find((f) => f.format === state.format)?.name}
          </h5>
          <p className="text-sm text-blue-700">
            {FORMAT_OPTIONS.find((f) => f.format === state.format)?.description}
          </p>
        </div>
      )}
    </div>
  );
};

function detectFormatFromFile(file: File): TransferFormat {
  const ext = file.name.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'csv':
      return 'csv';
    case 'xlsx':
    case 'xls':
      return 'excel';
    case 'json':
      return 'json';
    case 'xml':
      return 'xml';
    case 'zip':
      return 'archive';
    default:
      return 'csv';
  }
}

function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 Bytes';
  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i];
}
