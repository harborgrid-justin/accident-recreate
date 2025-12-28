import React from 'react';
import { FormatOption } from '../types';

interface FormatCardProps {
  option: FormatOption;
  selected: boolean;
  onClick: () => void;
  disabled?: boolean;
}

export const FormatCard: React.FC<FormatCardProps> = ({
  option,
  selected,
  onClick,
  disabled = false,
}) => {
  return (
    <button
      type="button"
      onClick={onClick}
      disabled={disabled}
      className={`
        relative p-6 rounded-lg border-2 text-left transition-all
        ${
          selected
            ? 'border-blue-600 bg-blue-50 shadow-md'
            : 'border-gray-300 bg-white hover:border-gray-400 hover:shadow-sm'
        }
        ${disabled ? 'opacity-50 cursor-not-allowed' : 'cursor-pointer'}
      `}
    >
      {/* Selection Indicator */}
      {selected && (
        <div className="absolute top-4 right-4">
          <div className="w-6 h-6 bg-blue-600 rounded-full flex items-center justify-center">
            <svg
              className="w-4 h-4 text-white"
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                clipRule="evenodd"
              />
            </svg>
          </div>
        </div>
      )}

      {/* Icon */}
      <div className="mb-4">
        <div
          className={`
          inline-flex items-center justify-center w-12 h-12 rounded-lg text-2xl
          ${
            selected
              ? 'bg-blue-100 text-blue-700'
              : 'bg-gray-100 text-gray-600'
          }
        `}
        >
          {option.icon}
        </div>
      </div>

      {/* Content */}
      <div className="mb-3">
        <h4
          className={`text-lg font-semibold ${
            selected ? 'text-blue-900' : 'text-gray-900'
          }`}
        >
          {option.name}
        </h4>
        <p className="text-sm text-gray-600 mt-1">{option.description}</p>
      </div>

      {/* Features */}
      <div className="flex flex-wrap gap-2">
        {option.supportsImport && (
          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-green-100 text-green-800">
            Import
          </span>
        )}
        {option.supportsExport && (
          <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
            Export
          </span>
        )}
        <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
          .{option.fileExtension}
        </span>
      </div>

      {/* Additional Info */}
      {option.format === 'archive' && (
        <div className="mt-3 pt-3 border-t border-gray-200">
          <p className="text-xs text-gray-500">
            Includes CSV, JSON, XML, and metadata
          </p>
        </div>
      )}

      {option.format === 'pdf' && (
        <div className="mt-3 pt-3 border-t border-gray-200">
          <p className="text-xs text-gray-500">Export only - Print-ready format</p>
        </div>
      )}

      {option.format === 'excel' && (
        <div className="mt-3 pt-3 border-t border-gray-200">
          <p className="text-xs text-gray-500">
            Supports multiple sheets and formatting
          </p>
        </div>
      )}
    </button>
  );
};
