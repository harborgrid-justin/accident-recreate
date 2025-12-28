import React, { useState, useCallback, useEffect } from 'react';
import {
  ImportWizardState,
  WizardStep,
  TransferFormat,
  DEFAULT_CONFIG,
  MappingProfile,
  FieldSchema,
} from './types';
import { FormatSelection } from './steps/FormatSelection';
import { FieldMapping } from './steps/FieldMapping';
import { DataPreview } from './steps/DataPreview';
import { ValidationResults } from './steps/ValidationResults';
import { ProgressStep } from './steps/ProgressStep';
import { useImport } from './hooks/useImport';

interface ImportWizardProps {
  onComplete?: (data: any[], metadata: any) => void;
  onCancel?: () => void;
  className?: string;
}

export const ImportWizard: React.FC<ImportWizardProps> = ({
  onComplete,
  onCancel,
  className = '',
}) => {
  const [state, setState] = useState<ImportWizardState>({
    currentStep: 0,
    config: DEFAULT_CONFIG,
  });

  const { importData, detectSchema, validateData, progress, error, isImporting } = useImport();

  const steps: WizardStep[] = [
    {
      id: 'upload',
      title: 'Upload File',
      description: 'Select a file to import',
      component: FormatSelection,
    },
    {
      id: 'mapping',
      title: 'Field Mapping',
      description: 'Map imported fields to your schema',
      component: FieldMapping,
      optional: true,
    },
    {
      id: 'preview',
      title: 'Preview Data',
      description: 'Review imported data',
      component: DataPreview,
    },
    {
      id: 'validation',
      title: 'Validation',
      description: 'Validate imported data',
      component: ValidationResults,
    },
    {
      id: 'import',
      title: 'Import',
      description: 'Complete the import',
      component: ProgressStep,
    },
  ];

  const currentStepConfig = steps[state.currentStep];

  // Auto-detect schema when file is selected
  useEffect(() => {
    if (state.file && !state.detectedSchema) {
      detectFileSchema();
    }
  }, [state.file]);

  const detectFileSchema = async () => {
    if (!state.file) return;

    try {
      const schema = await detectSchema(state.file, state.config);
      setState((prev) => ({ ...prev, detectedSchema: schema }));
    } catch (err) {
      console.error('Schema detection failed:', err);
    }
  };

  const handleNext = useCallback(async () => {
    const nextStep = state.currentStep + 1;

    // If we're at the validation step, validate the data
    if (state.currentStep === 3 && state.file) {
      try {
        const validationResult = await validateData({
          file: state.file,
          config: state.config,
          validate: true,
        });

        setState((prev) => ({
          ...prev,
          validationResult,
          currentStep: nextStep,
        }));
      } catch (err) {
        console.error('Validation failed:', err);
        return;
      }
    }
    // If we're at the import step, trigger the import
    else if (nextStep === steps.length - 1 && state.file) {
      try {
        const result = await importData({
          file: state.file,
          format: state.format,
          config: state.config,
          mappingProfile: state.mappingProfile,
          validate: !state.config.skipValidation,
        });

        if (result.success && result.data) {
          setState((prev) => ({
            ...prev,
            currentStep: nextStep,
            importedData: result.data,
            progress: result.progress,
          }));

          if (onComplete && result.data) {
            onComplete(result.data, result.metadata);
          }
        }
      } catch (err) {
        console.error('Import failed:', err);
      }
    } else {
      setState((prev) => ({ ...prev, currentStep: nextStep }));
    }
  }, [state, importData, validateData, onComplete, steps.length]);

  const handleBack = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentStep: Math.max(0, prev.currentStep - 1),
    }));
  }, []);

  const handleSkip = useCallback(() => {
    setState((prev) => ({
      ...prev,
      currentStep: prev.currentStep + 1,
    }));
  }, []);

  const handleFileSelect = useCallback((file: File, format?: TransferFormat) => {
    setState((prev) => ({
      ...prev,
      file,
      format: format || detectFormatFromFile(file),
    }));
  }, []);

  const handleMappingChange = useCallback((profile: MappingProfile) => {
    setState((prev) => ({ ...prev, mappingProfile: profile }));
  }, []);

  const handleConfigChange = useCallback((config: Partial<any>) => {
    setState((prev) => ({
      ...prev,
      config: { ...prev.config, ...config },
    }));
  }, []);

  const canProceed = (): boolean => {
    switch (state.currentStep) {
      case 0: // File upload
        return !!state.file;
      case 1: // Field mapping (optional)
        return true;
      case 2: // Preview
        return true;
      case 3: // Validation
        return true;
      default:
        return true;
    }
  };

  const StepComponent = currentStepConfig.component;

  return (
    <div className={`import-wizard ${className}`}>
      {/* Wizard Header */}
      <div className="wizard-header">
        <h2 className="text-2xl font-bold text-gray-900">Import Data</h2>
        <p className="text-sm text-gray-600 mt-1">
          {state.file
            ? `File: ${state.file.name} (${formatFileSize(state.file.size)})`
            : 'No file selected'}
        </p>
      </div>

      {/* Progress Indicator */}
      <div className="wizard-progress mt-6 mb-8">
        <div className="flex items-center justify-between">
          {steps.map((step, index) => (
            <div
              key={step.id}
              className={`flex-1 ${index > 0 ? 'ml-2' : ''}`}
            >
              <div className="flex items-center">
                {index > 0 && (
                  <div
                    className={`flex-1 h-1 ${
                      index <= state.currentStep
                        ? 'bg-blue-600'
                        : 'bg-gray-300'
                    }`}
                  />
                )}
                <div
                  className={`flex items-center justify-center w-10 h-10 rounded-full ${
                    index === state.currentStep
                      ? 'bg-blue-600 text-white'
                      : index < state.currentStep
                      ? 'bg-green-600 text-white'
                      : 'bg-gray-300 text-gray-600'
                  }`}
                >
                  {index < state.currentStep ? (
                    <svg className="w-6 h-6" fill="currentColor" viewBox="0 0 20 20">
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  ) : (
                    <span className="text-sm font-semibold">{index + 1}</span>
                  )}
                </div>
              </div>
              <div className="mt-2 text-center">
                <p className="text-xs font-medium text-gray-900">{step.title}</p>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Current Step */}
      <div className="wizard-step bg-white rounded-lg shadow-sm p-6 min-h-96">
        <div className="step-header mb-6">
          <h3 className="text-xl font-semibold text-gray-900">
            {currentStepConfig.title}
          </h3>
          <p className="text-sm text-gray-600 mt-1">
            {currentStepConfig.description}
          </p>
        </div>

        <div className="step-content">
          <StepComponent
            state={state}
            onFileSelect={handleFileSelect}
            onMappingChange={handleMappingChange}
            onConfigChange={handleConfigChange}
            mode="import"
          />
        </div>
      </div>

      {/* Error Display */}
      {error && (
        <div className="mt-4 p-4 bg-red-50 border border-red-200 rounded-lg">
          <div className="flex">
            <div className="flex-shrink-0">
              <svg
                className="h-5 w-5 text-red-400"
                viewBox="0 0 20 20"
                fill="currentColor"
              >
                <path
                  fillRule="evenodd"
                  d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                  clipRule="evenodd"
                />
              </svg>
            </div>
            <div className="ml-3">
              <h3 className="text-sm font-medium text-red-800">Import Error</h3>
              <p className="text-sm text-red-700 mt-1">{error}</p>
            </div>
          </div>
        </div>
      )}

      {/* Navigation */}
      <div className="wizard-navigation mt-6 flex items-center justify-between">
        <button
          type="button"
          onClick={onCancel}
          className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
        >
          Cancel
        </button>

        <div className="flex space-x-3">
          {state.currentStep > 0 && state.currentStep < steps.length - 1 && (
            <button
              type="button"
              onClick={handleBack}
              disabled={isImporting}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
            >
              Back
            </button>
          )}

          {currentStepConfig.optional && state.currentStep < steps.length - 2 && (
            <button
              type="button"
              onClick={handleSkip}
              disabled={isImporting}
              className="px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
            >
              Skip
            </button>
          )}

          {state.currentStep < steps.length - 1 && (
            <button
              type="button"
              onClick={handleNext}
              disabled={!canProceed() || isImporting}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {state.currentStep === steps.length - 2 ? 'Import' : 'Next'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};

// Helper functions
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
  return Math.round(bytes / Math.pow(k, i) * 100) / 100 + ' ' + sizes[i];
}
