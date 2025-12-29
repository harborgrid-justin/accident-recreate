import React, { useState } from 'react';
import { ValidationResult, ValidationError, ValidationWarning } from '../types';

interface ValidationResultsProps {
  state: any;
  mode: 'import' | 'export';
}

export const ValidationResults: React.FC<ValidationResultsProps> = ({ state, mode }) => {
  const [showErrors, setShowErrors] = useState(true);
  const [showWarnings, setShowWarnings] = useState(true);

  const validationResult: ValidationResult | undefined = state.validationResult;

  if (!validationResult) {
    return (
      <div className="text-center py-12">
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
              d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <p className="text-gray-600">Validation not yet performed</p>
        <p className="text-sm text-gray-500 mt-2">
          Validation will be performed before {mode}
        </p>
      </div>
    );
  }

  const { valid, errors, warnings, totalRecords, validRecords, invalidRecords } =
    validationResult;

  return (
    <div className="validation-results">
      {/* Overall Status */}
      <div
        className={`mb-6 p-6 rounded-lg ${
          valid ? 'bg-green-50 border border-green-200' : 'bg-red-50 border border-red-200'
        }`}
      >
        <div className="flex items-center">
          <div className="flex-shrink-0">
            {valid ? (
              <svg
                className="w-12 h-12 text-green-600"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
                  clipRule="evenodd"
                />
              </svg>
            ) : (
              <svg
                className="w-12 h-12 text-red-600"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clipRule="evenodd"
                />
              </svg>
            )}
          </div>
          <div className="ml-4">
            <h3
              className={`text-xl font-semibold ${
                valid ? 'text-green-900' : 'text-red-900'
              }`}
            >
              {valid ? 'Validation Passed' : 'Validation Failed'}
            </h3>
            <p className={`mt-1 ${valid ? 'text-green-700' : 'text-red-700'}`}>
              {valid
                ? 'All records passed validation checks'
                : `${errors.length} validation error(s) found`}
            </p>
          </div>
        </div>
      </div>

      {/* Statistics */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 mb-6">
        <div className="p-4 bg-blue-50 rounded-lg">
          <p className="text-sm text-blue-700">Total Records</p>
          <p className="text-2xl font-bold text-blue-900">
            {totalRecords.toLocaleString()}
          </p>
        </div>
        <div className="p-4 bg-green-50 rounded-lg">
          <p className="text-sm text-green-700">Valid Records</p>
          <p className="text-2xl font-bold text-green-900">
            {validRecords.toLocaleString()}
          </p>
          <p className="text-xs text-green-600 mt-1">
            {((validRecords / totalRecords) * 100).toFixed(1)}%
          </p>
        </div>
        <div className="p-4 bg-red-50 rounded-lg">
          <p className="text-sm text-red-700">Invalid Records</p>
          <p className="text-2xl font-bold text-red-900">
            {invalidRecords.toLocaleString()}
          </p>
          <p className="text-xs text-red-600 mt-1">
            {((invalidRecords / totalRecords) * 100).toFixed(1)}%
          </p>
        </div>
      </div>

      {/* Errors Section */}
      {errors.length > 0 && (
        <div className="mb-6">
          <button
            onClick={() => setShowErrors(!showErrors)}
            className="flex items-center justify-between w-full p-4 bg-red-50 border border-red-200 rounded-lg hover:bg-red-100"
          >
            <div className="flex items-center">
              <svg
                className="w-5 h-5 text-red-600 mr-2"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clipRule="evenodd"
                />
              </svg>
              <span className="font-medium text-red-900">
                {errors.length} Error{errors.length !== 1 ? 's' : ''}
              </span>
            </div>
            <svg
              className={`w-5 h-5 text-red-600 transform ${
                showErrors ? 'rotate-180' : ''
              }`}
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
          {showErrors && (
            <div className="mt-2 border border-red-200 rounded-lg overflow-hidden">
              <div className="max-h-64 overflow-y-auto">
                {errors.map((error, idx) => (
                  <ErrorItem key={idx} error={error} />
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Warnings Section */}
      {warnings.length > 0 && (
        <div className="mb-6">
          <button
            onClick={() => setShowWarnings(!showWarnings)}
            className="flex items-center justify-between w-full p-4 bg-yellow-50 border border-yellow-200 rounded-lg hover:bg-yellow-100"
          >
            <div className="flex items-center">
              <svg
                className="w-5 h-5 text-yellow-600 mr-2"
                fill="currentColor"
                viewBox="0 0 20 20"
              >
                <path
                  fillRule="evenodd"
                  d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z"
                  clipRule="evenodd"
                />
              </svg>
              <span className="font-medium text-yellow-900">
                {warnings.length} Warning{warnings.length !== 1 ? 's' : ''}
              </span>
            </div>
            <svg
              className={`w-5 h-5 text-yellow-600 transform ${
                showWarnings ? 'rotate-180' : ''
              }`}
              fill="currentColor"
              viewBox="0 0 20 20"
            >
              <path
                fillRule="evenodd"
                d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
                clipRule="evenodd"
              />
            </svg>
          </button>
          {showWarnings && (
            <div className="mt-2 border border-yellow-200 rounded-lg overflow-hidden">
              <div className="max-h-64 overflow-y-auto">
                {warnings.map((warning, idx) => (
                  <WarningItem key={idx} warning={warning} />
                ))}
              </div>
            </div>
          )}
        </div>
      )}

      {/* Success Message */}
      {valid && errors.length === 0 && warnings.length === 0 && (
        <div className="p-6 bg-green-50 border border-green-200 rounded-lg text-center">
          <p className="text-green-800">
            All records are valid and ready for {mode}!
          </p>
        </div>
      )}
    </div>
  );
};

const ErrorItem: React.FC<{ error: ValidationError }> = ({ error }) => (
  <div className="p-3 border-b border-red-100 bg-white hover:bg-red-50">
    <div className="flex items-start">
      <div className="flex-1">
        <p className="text-sm font-medium text-red-900">{error.message}</p>
        <div className="mt-1 flex items-center space-x-4 text-xs text-red-700">
          {error.recordIndex !== undefined && (
            <span>Record #{error.recordIndex + 1}</span>
          )}
          {error.field && <span>Field: {error.field}</span>}
          {error.code && <span className="font-mono">{error.code}</span>}
        </div>
      </div>
    </div>
  </div>
);

const WarningItem: React.FC<{ warning: ValidationWarning }> = ({ warning }) => (
  <div className="p-3 border-b border-yellow-100 bg-white hover:bg-yellow-50">
    <div className="flex items-start">
      <div className="flex-1">
        <p className="text-sm font-medium text-yellow-900">{warning.message}</p>
        <div className="mt-1 flex items-center space-x-4 text-xs text-yellow-700">
          {warning.recordIndex !== undefined && (
            <span>Record #{warning.recordIndex + 1}</span>
          )}
          {warning.field && <span>Field: {warning.field}</span>}
        </div>
      </div>
    </div>
  </div>
);
