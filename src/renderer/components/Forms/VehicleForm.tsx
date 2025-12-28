/**
 * VehicleForm Component - Form for adding/editing vehicles
 */

import React, { useState, FormEvent } from 'react';
import { VehicleData } from '../../services/api';
import Input from '../Common/Input';
import Select from '../Common/Select';
import Button from '../Common/Button';

export interface VehicleFormProps {
  initialData?: VehicleData;
  accidentId: string;
  vehicleNumber: number;
  onSubmit: (data: VehicleData) => Promise<void>;
  onCancel: () => void;
  isLoading?: boolean;
}

export const VehicleForm: React.FC<VehicleFormProps> = ({
  initialData,
  accidentId,
  vehicleNumber,
  onSubmit,
  onCancel,
  isLoading = false,
}) => {
  const [formData, setFormData] = useState<VehicleData>({
    accidentId,
    vehicleNumber,
    type: 'sedan',
    make: '',
    model: '',
    year: new Date().getFullYear(),
    color: '',
    licensePlate: '',
    driverName: '',
    driverPhone: '',
    speed: 0,
    damageSeverity: 'none',
    ...initialData,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.make?.trim()) {
      newErrors.make = 'Make is required';
    }

    if (!formData.model?.trim()) {
      newErrors.model = 'Model is required';
    }

    if (!formData.driverName?.trim()) {
      newErrors.driverName = 'Driver name is required';
    }

    if (formData.year && (formData.year < 1900 || formData.year > new Date().getFullYear() + 2)) {
      newErrors.year = 'Invalid year';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: FormEvent) => {
    e.preventDefault();

    if (!validate()) {
      return;
    }

    await onSubmit(formData);
  };

  const handleChange = (field: keyof VehicleData, value: any) => {
    setFormData((prev) => ({ ...prev, [field]: value }));
    if (errors[field]) {
      setErrors((prev) => {
        const newErrors = { ...prev };
        delete newErrors[field];
        return newErrors;
      });
    }
  };

  return (
    <form className="vehicle-form" onSubmit={handleSubmit}>
      <div className="vehicle-form-section">
        <h3>Vehicle Information</h3>
        <div className="vehicle-form-row">
          <Select
            label="Vehicle Type"
            value={formData.type}
            onChange={(e) => handleChange('type', e.target.value)}
            options={[
              { value: 'sedan', label: 'Sedan' },
              { value: 'suv', label: 'SUV' },
              { value: 'truck', label: 'Truck' },
              { value: 'van', label: 'Van' },
              { value: 'motorcycle', label: 'Motorcycle' },
              { value: 'bus', label: 'Bus' },
              { value: 'commercial', label: 'Commercial' },
              { value: 'bicycle', label: 'Bicycle' },
              { value: 'pedestrian', label: 'Pedestrian' },
              { value: 'other', label: 'Other' },
            ]}
            fullWidth
          />
        </div>

        <div className="vehicle-form-row vehicle-form-row-split">
          <Input
            label="Make"
            value={formData.make}
            onChange={(e) => handleChange('make', e.target.value)}
            error={errors.make}
            placeholder="e.g., Toyota"
            fullWidth
            required
          />

          <Input
            label="Model"
            value={formData.model}
            onChange={(e) => handleChange('model', e.target.value)}
            error={errors.model}
            placeholder="e.g., Camry"
            fullWidth
            required
          />
        </div>

        <div className="vehicle-form-row vehicle-form-row-triple">
          <Input
            label="Year"
            type="number"
            value={formData.year?.toString() || ''}
            onChange={(e) => handleChange('year', parseInt(e.target.value))}
            error={errors.year}
            placeholder="2024"
            fullWidth
          />

          <Input
            label="Color"
            value={formData.color || ''}
            onChange={(e) => handleChange('color', e.target.value)}
            placeholder="e.g., Red"
            fullWidth
          />

          <Input
            label="License Plate"
            value={formData.licensePlate || ''}
            onChange={(e) => handleChange('licensePlate', e.target.value)}
            placeholder="ABC-1234"
            fullWidth
          />
        </div>
      </div>

      <div className="vehicle-form-section">
        <h3>Driver Information</h3>
        <div className="vehicle-form-row">
          <Input
            label="Driver Name"
            value={formData.driverName}
            onChange={(e) => handleChange('driverName', e.target.value)}
            error={errors.driverName}
            placeholder="Enter driver name"
            fullWidth
            required
          />
        </div>

        <div className="vehicle-form-row">
          <Input
            label="Driver Phone"
            type="tel"
            value={formData.driverPhone || ''}
            onChange={(e) => handleChange('driverPhone', e.target.value)}
            placeholder="Enter phone number"
            fullWidth
          />
        </div>
      </div>

      <div className="vehicle-form-section">
        <h3>Damage Information</h3>
        <div className="vehicle-form-row vehicle-form-row-split">
          <Input
            label="Speed (MPH)"
            type="number"
            value={formData.speed?.toString() || ''}
            onChange={(e) => handleChange('speed', parseInt(e.target.value))}
            placeholder="0"
            fullWidth
          />

          <Select
            label="Damage Severity"
            value={formData.damageSeverity}
            onChange={(e) => handleChange('damageSeverity', e.target.value)}
            options={[
              { value: 'none', label: 'None' },
              { value: 'minor', label: 'Minor' },
              { value: 'moderate', label: 'Moderate' },
              { value: 'severe', label: 'Severe' },
              { value: 'totaled', label: 'Totaled' },
            ]}
            fullWidth
          />
        </div>

        <div className="vehicle-form-row">
          <textarea
            className="vehicle-form-textarea"
            placeholder="Describe vehicle damage..."
            value={formData.damage || ''}
            onChange={(e) => handleChange('damage', e.target.value)}
            rows={3}
          />
        </div>
      </div>

      <div className="vehicle-form-actions">
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" variant="primary" loading={isLoading}>
          {initialData ? 'Update Vehicle' : 'Add Vehicle'}
        </Button>
      </div>
    </form>
  );
};

export default VehicleForm;
