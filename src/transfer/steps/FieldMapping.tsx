import React, { useState, useEffect, useMemo } from 'react';
import { FieldMapping as FieldMappingType, MappingProfile, Transform } from '../types';
import { MappingEditor } from '../components/MappingEditor';

interface FieldMappingProps {
  state: any;
  onMappingChange?: (profile: MappingProfile) => void;
  mode: 'import' | 'export';
}

export const FieldMapping: React.FC<FieldMappingProps> = ({
  state,
  onMappingChange,
  mode,
}) => {
  const [profile, setProfile] = useState<MappingProfile>(
    state.mappingProfile || {
      name: 'default',
      mappings: [],
      globalTransforms: [],
    }
  );

  const [autoMapped, setAutoMapped] = useState(false);

  // Get source and target fields
  const sourceFields = useMemo(() => {
    if (mode === 'import' && state.detectedSchema) {
      return state.detectedSchema.map((s: any) => s.name);
    } else if (mode === 'export' && state.data && state.data.length > 0) {
      return Object.keys(state.data[0].fields || {});
    }
    return [];
  }, [mode, state.detectedSchema, state.data]);

  const targetFields = useMemo(() => {
    // In a real implementation, this would come from your target schema
    // For now, we'll use the source fields as a placeholder
    return sourceFields;
  }, [sourceFields]);

  // Auto-generate initial mappings
  useEffect(() => {
    if (!autoMapped && sourceFields.length > 0 && profile.mappings.length === 0) {
      const mappings: FieldMappingType[] = sourceFields.map((field) => ({
        source: field,
        target: field,
        required: false,
      }));

      const newProfile = {
        ...profile,
        mappings,
      };

      setProfile(newProfile);
      setAutoMapped(true);
      onMappingChange?.(newProfile);
    }
  }, [sourceFields, autoMapped, profile, onMappingChange]);

  const handleMappingUpdate = (mappings: FieldMappingType[]) => {
    const newProfile = {
      ...profile,
      mappings,
    };
    setProfile(newProfile);
    onMappingChange?.(newProfile);
  };

  const handleTransformAdd = (mappingIndex: number, transform: Transform) => {
    const newMappings = [...profile.mappings];
    newMappings[mappingIndex] = {
      ...newMappings[mappingIndex],
      transform,
    };
    handleMappingUpdate(newMappings);
  };

  const handleTransformRemove = (mappingIndex: number) => {
    const newMappings = [...profile.mappings];
    newMappings[mappingIndex] = {
      ...newMappings[mappingIndex],
      transform: undefined,
    };
    handleMappingUpdate(newMappings);
  };

  const handleAddMapping = () => {
    const newMapping: FieldMappingType = {
      source: '',
      target: '',
      required: false,
    };
    handleMappingUpdate([...profile.mappings, newMapping]);
  };

  const handleRemoveMapping = (index: number) => {
    const newMappings = profile.mappings.filter((_, i) => i !== index);
    handleMappingUpdate(newMappings);
  };

  const handleResetMappings = () => {
    const mappings: FieldMappingType[] = sourceFields.map((field) => ({
      source: field,
      target: field,
      required: false,
    }));
    handleMappingUpdate(mappings);
  };

  if (sourceFields.length === 0) {
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
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            />
          </svg>
        </div>
        <p className="text-gray-600">
          {mode === 'import'
            ? 'No schema detected. Please select a file first.'
            : 'No data available for mapping.'}
        </p>
      </div>
    );
  }

  return (
    <div className="field-mapping">
      {/* Header */}
      <div className="mb-6 flex items-center justify-between">
        <div>
          <p className="text-sm text-gray-600">
            Map {sourceFields.length} source fields to target fields
          </p>
        </div>
        <div className="flex space-x-2">
          <button
            type="button"
            onClick={handleResetMappings}
            className="px-3 py-1 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-md hover:bg-gray-50"
          >
            Reset
          </button>
          <button
            type="button"
            onClick={handleAddMapping}
            className="px-3 py-1 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700"
          >
            Add Mapping
          </button>
        </div>
      </div>

      {/* Schema Info */}
      {mode === 'import' && state.detectedSchema && (
        <div className="mb-6 p-4 bg-gray-50 rounded-lg">
          <h5 className="text-sm font-medium text-gray-900 mb-2">Detected Schema</h5>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-xs">
            {state.detectedSchema.slice(0, 8).map((field: any) => (
              <div key={field.name} className="flex items-center space-x-2">
                <span className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                  {field.dataType}
                </span>
                <span className="text-gray-700 truncate">{field.name}</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Mapping Editor */}
      <MappingEditor
        mappings={profile.mappings}
        sourceFields={sourceFields}
        targetFields={targetFields}
        onChange={handleMappingUpdate}
        onTransformAdd={handleTransformAdd}
        onTransformRemove={handleTransformRemove}
        onRemoveMapping={handleRemoveMapping}
      />

      {/* Summary */}
      <div className="mt-6 p-4 bg-blue-50 border border-blue-200 rounded-lg">
        <h5 className="text-sm font-medium text-blue-900 mb-2">Mapping Summary</h5>
        <div className="grid grid-cols-3 gap-4 text-sm">
          <div>
            <p className="text-blue-700">Total Mappings</p>
            <p className="text-2xl font-bold text-blue-900">{profile.mappings.length}</p>
          </div>
          <div>
            <p className="text-blue-700">With Transforms</p>
            <p className="text-2xl font-bold text-blue-900">
              {profile.mappings.filter((m) => m.transform).length}
            </p>
          </div>
          <div>
            <p className="text-blue-700">Required Fields</p>
            <p className="text-2xl font-bold text-blue-900">
              {profile.mappings.filter((m) => m.required).length}
            </p>
          </div>
        </div>
      </div>
    </div>
  );
};
