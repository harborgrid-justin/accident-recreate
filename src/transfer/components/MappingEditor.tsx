import React, { useState } from 'react';
import { FieldMapping, Transform } from '../types';

interface MappingEditorProps {
  mappings: FieldMapping[];
  sourceFields: string[];
  targetFields: string[];
  onChange: (mappings: FieldMapping[]) => void;
  onTransformAdd?: (index: number, transform: Transform) => void;
  onTransformRemove?: (index: number) => void;
  onRemoveMapping?: (index: number) => void;
}

export const MappingEditor: React.FC<MappingEditorProps> = ({
  mappings,
  sourceFields,
  targetFields,
  onChange,
  onTransformAdd,
  onTransformRemove,
  onRemoveMapping,
}) => {
  const [expandedIndex, setExpandedIndex] = useState<number | null>(null);

  const handleSourceChange = (index: number, value: string) => {
    const newMappings = [...mappings];
    newMappings[index] = { ...newMappings[index], source: value };
    onChange(newMappings);
  };

  const handleTargetChange = (index: number, value: string) => {
    const newMappings = [...mappings];
    newMappings[index] = { ...newMappings[index], target: value };
    onChange(newMappings);
  };

  const handleRequiredChange = (index: number, required: boolean) => {
    const newMappings = [...mappings];
    newMappings[index] = { ...newMappings[index], required };
    onChange(newMappings);
  };

  const handleDefaultChange = (index: number, value: string) => {
    const newMappings = [...mappings];
    newMappings[index] = {
      ...newMappings[index],
      default: value || undefined,
    };
    onChange(newMappings);
  };

  const toggleExpand = (index: number) => {
    setExpandedIndex(expandedIndex === index ? null : index);
  };

  return (
    <div className="mapping-editor space-y-2">
      {/* Header */}
      <div className="grid grid-cols-12 gap-4 px-4 py-2 bg-gray-100 rounded-t-lg text-sm font-medium text-gray-700">
        <div className="col-span-4">Source Field</div>
        <div className="col-span-1 text-center">→</div>
        <div className="col-span-4">Target Field</div>
        <div className="col-span-2">Options</div>
        <div className="col-span-1"></div>
      </div>

      {/* Mapping Rows */}
      <div className="space-y-2">
        {mappings.map((mapping, index) => (
          <div
            key={index}
            className="border border-gray-300 rounded-lg bg-white overflow-hidden"
          >
            {/* Main Row */}
            <div className="grid grid-cols-12 gap-4 p-4 items-center">
              {/* Source Field */}
              <div className="col-span-4">
                <select
                  value={mapping.source}
                  onChange={(e) => handleSourceChange(index, e.target.value)}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select source field...</option>
                  {sourceFields.map((field) => (
                    <option key={field} value={field}>
                      {field}
                    </option>
                  ))}
                </select>
              </div>

              {/* Arrow */}
              <div className="col-span-1 text-center text-gray-400">
                <svg
                  className="w-5 h-5 mx-auto"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M14 5l7 7m0 0l-7 7m7-7H3"
                  />
                </svg>
              </div>

              {/* Target Field */}
              <div className="col-span-4">
                <select
                  value={mapping.target}
                  onChange={(e) => handleTargetChange(index, e.target.value)}
                  className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="">Select target field...</option>
                  {targetFields.map((field) => (
                    <option key={field} value={field}>
                      {field}
                    </option>
                  ))}
                </select>
              </div>

              {/* Options */}
              <div className="col-span-2 flex items-center space-x-2">
                <label className="flex items-center text-sm">
                  <input
                    type="checkbox"
                    checked={mapping.required}
                    onChange={(e) =>
                      handleRequiredChange(index, e.target.checked)
                    }
                    className="mr-1"
                  />
                  Required
                </label>
              </div>

              {/* Actions */}
              <div className="col-span-1 flex items-center justify-end space-x-1">
                <button
                  type="button"
                  onClick={() => toggleExpand(index)}
                  className="p-1 text-gray-500 hover:text-gray-700"
                  title="Advanced options"
                >
                  <svg
                    className={`w-5 h-5 transform ${
                      expandedIndex === index ? 'rotate-180' : ''
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
                {onRemoveMapping && (
                  <button
                    type="button"
                    onClick={() => onRemoveMapping(index)}
                    className="p-1 text-red-500 hover:text-red-700"
                    title="Remove mapping"
                  >
                    <svg
                      className="w-5 h-5"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                        clipRule="evenodd"
                      />
                    </svg>
                  </button>
                )}
              </div>
            </div>

            {/* Expanded Options */}
            {expandedIndex === index && (
              <div className="px-4 pb-4 pt-2 bg-gray-50 border-t border-gray-200">
                <div className="grid grid-cols-2 gap-4">
                  {/* Default Value */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Default Value
                    </label>
                    <input
                      type="text"
                      value={(mapping.default as string) || ''}
                      onChange={(e) => handleDefaultChange(index, e.target.value)}
                      placeholder="Optional default value"
                      className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                  </div>

                  {/* Transform */}
                  <div>
                    <label className="block text-sm font-medium text-gray-700 mb-1">
                      Transform
                    </label>
                    <div className="flex items-center space-x-2">
                      {mapping.transform ? (
                        <>
                          <div className="flex-1 px-3 py-2 text-sm bg-blue-50 border border-blue-200 rounded-md text-blue-900">
                            {getTransformLabel(mapping.transform)}
                          </div>
                          {onTransformRemove && (
                            <button
                              type="button"
                              onClick={() => onTransformRemove(index)}
                              className="px-3 py-2 text-sm text-red-600 hover:text-red-800"
                            >
                              Remove
                            </button>
                          )}
                        </>
                      ) : (
                        <button
                          type="button"
                          onClick={() => {
                            if (onTransformAdd) {
                              // Default transform
                              onTransformAdd(index, { type: 'trim' });
                            }
                          }}
                          className="px-3 py-2 text-sm text-blue-600 border border-blue-300 rounded-md hover:bg-blue-50"
                        >
                          Add Transform
                        </button>
                      )}
                    </div>
                  </div>
                </div>

                {/* Transform Options */}
                {mapping.transform && (
                  <div className="mt-4 p-3 bg-white border border-gray-200 rounded-md">
                    <TransformEditor
                      transform={mapping.transform}
                      onChange={(t) => onTransformAdd?.(index, t)}
                    />
                  </div>
                )}
              </div>
            )}
          </div>
        ))}
      </div>

      {mappings.length === 0 && (
        <div className="text-center py-12 text-gray-500">
          No field mappings configured
        </div>
      )}
    </div>
  );
};

interface TransformEditorProps {
  transform: Transform;
  onChange: (transform: Transform) => void;
}

const TransformEditor: React.FC<TransformEditorProps> = ({
  transform,
  onChange,
}) => {
  return (
    <div className="space-y-3">
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-1">
          Transform Type
        </label>
        <select
          value={transform.type}
          onChange={(e) => {
            const newType = e.target.value;
            onChange({ type: newType } as Transform);
          }}
          className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md"
        >
          <option value="trim">Trim Whitespace</option>
          <option value="to_upper_case">To Uppercase</option>
          <option value="to_lower_case">To Lowercase</option>
          <option value="parse_number">Parse Number</option>
          <option value="replace">Replace Text</option>
          <option value="prefix">Add Prefix</option>
          <option value="suffix">Add Suffix</option>
        </select>
      </div>

      {/* Type-specific options */}
      {transform.type === 'replace' && 'from' in transform && (
        <>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Find
            </label>
            <input
              type="text"
              value={transform.from}
              onChange={(e) =>
                onChange({ ...transform, from: e.target.value })
              }
              className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md"
            />
          </div>
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Replace With
            </label>
            <input
              type="text"
              value={transform.to}
              onChange={(e) =>
                onChange({ ...transform, to: e.target.value })
              }
              className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md"
            />
          </div>
        </>
      )}

      {transform.type === 'prefix' && 'prefix' in transform && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Prefix
          </label>
          <input
            type="text"
            value={transform.prefix}
            onChange={(e) =>
              onChange({ ...transform, prefix: e.target.value })
            }
            className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md"
          />
        </div>
      )}

      {transform.type === 'suffix' && 'suffix' in transform && (
        <div>
          <label className="block text-sm font-medium text-gray-700 mb-1">
            Suffix
          </label>
          <input
            type="text"
            value={transform.suffix}
            onChange={(e) =>
              onChange({ ...transform, suffix: e.target.value })
            }
            className="w-full px-3 py-2 text-sm border border-gray-300 rounded-md"
          />
        </div>
      )}
    </div>
  );
};

function getTransformLabel(transform: Transform): string {
  switch (transform.type) {
    case 'trim':
      return 'Trim Whitespace';
    case 'to_upper_case':
      return 'To Uppercase';
    case 'to_lower_case':
      return 'To Lowercase';
    case 'parse_number':
      return 'Parse Number';
    case 'replace':
      return `Replace "${('from' in transform && transform.from) || ''}"`;
    case 'prefix':
      return `Prefix: ${('prefix' in transform && transform.prefix) || ''}`;
    case 'suffix':
      return `Suffix: ${('suffix' in transform && transform.suffix) || ''}`;
    default:
      return 'Custom Transform';
  }
}
