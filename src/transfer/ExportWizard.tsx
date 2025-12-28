import React, { useState, useCallback } from 'react';
import {
  ExportWizardState,
  WizardStep,
  TransferFormat,
  DataRecord,
  DEFAULT_CONFIG,
  MappingProfile,
} from './types';
import { FormatSelection } from './steps/FormatSelection';
import { FieldMapping } from './steps/FieldMapping';
import { DataPreview } from './steps/DataPreview';
import { ValidationResults } from './steps/ValidationResults';
import { ProgressStep } from './steps/ProgressStep';
import { useExport } from './hooks/useExport';

interface ExportWizardProps {
  data: DataRecord[];
  onComplete?: (blob: Blob, metadata: any) => void;
  onCancel?: () => void;
  className?: string;
}

export const ExportWizard: React.FC<ExportWizardProps> = ({
  data,
  onComplete,
  onCancel,
  className = '',
}) => {
  const [state, setState] = useState<ExportWizardState>({
    currentStep: 0,
    config: DEFAULT_CONFIG,
    data,
  });

  const { exportData, progress, error, isExporting } = useExport();

  const steps: WizardStep[] = [
    {
      id: 'format',
      title: 'Select Format',
      description: 'Choose the export format for your data',
      component: FormatSelection,
    },
    {
      id: 'mapping',
      title: 'Field Mapping',
      description: 'Map and transform your data fields',
      component: FieldMapping,
      optional: true,
    },
    {
      id: 'preview',
      title: 'Preview Data',
      description: 'Review your data before export',
      component: DataPreview,
    },
    {
      id: 'validation',
      title: 'Validation',
      description: 'Validate data quality',
      component: ValidationResults,
      optional: true,
    },
    {
      id: 'export',
      title: 'Export',
      description: 'Export your data',
      component: ProgressStep,
    },
  ];

  const currentStepConfig = steps[state.currentStep];

  const handleNext = useCallback(async () => {
    const nextStep = state.currentStep + 1;

    // If we're at the export step, trigger the export
    if (nextStep === steps.length - 1) {
      try {
        const result = await exportData({
          format: state.format!,
          config: state.config,
          data: state.data,
          mappingProfile: state.mappingProfile,
        });

        if (result.success && result.data) {
          setState((prev) => ({
            ...prev,
            currentStep: nextStep,
            progress: result.progress,
          }));

          if (onComplete && result.data) {
            onComplete(result.data, result.metadata);
          }
        }
      } catch (err) {
        console.error('Export failed:', err);
      }
    } else {
      setState((prev) => ({ ...prev, currentStep: nextStep }));
    }
  }, [state, exportData, onComplete, steps.length]);

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

  const handleFormatSelect = useCallback((format: TransferFormat) => {
    setState((prev) => ({ ...prev, format }));
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
      case 0: // Format selection
        return !!state.format;
      case 1: // Field mapping (optional)
        return true;
      case 2: // Preview
        return true;
      case 3: // Validation (optional)
        return true;
      default:
        return true;
    }
  };

  const StepComponent = currentStepConfig.component;

  return (
    <div className={`export-wizard ${className}`}>
      {/* Wizard Header */}
      <div className="wizard-header">
        <h2 className="text-2xl font-bold text-gray-900">Export Data</h2>
        <p className="text-sm text-gray-600 mt-1">
          {data.length.toLocaleString()} records ready to export
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
            onFormatSelect={handleFormatSelect}
            onMappingChange={handleMappingChange}
            onConfigChange={handleConfigChange}
            mode="export"
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
              <h3 className="text-sm font-medium text-red-800">Export Error</h3>
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
              disabled={isExporting}
              className="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
            >
              Back
            </button>
          )}

          {currentStepConfig.optional && state.currentStep < steps.length - 2 && (
            <button
              type="button"
              onClick={handleSkip}
              disabled={isExporting}
              className="px-4 py-2 text-sm font-medium text-gray-600 bg-white border border-gray-300 rounded-md hover:bg-gray-50 disabled:opacity-50"
            >
              Skip
            </button>
          )}

          {state.currentStep < steps.length - 1 && (
            <button
              type="button"
              onClick={handleNext}
              disabled={!canProceed() || isExporting}
              className="px-4 py-2 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {state.currentStep === steps.length - 2 ? 'Export' : 'Next'}
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
