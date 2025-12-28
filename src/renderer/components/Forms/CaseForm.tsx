/**
 * CaseForm Component - Form for creating/editing cases
 */

import React, { useState, FormEvent, useEffect } from 'react';
import { CaseData } from '../../services/api';
import Input from '../Common/Input';
import Select from '../Common/Select';
import Button from '../Common/Button';

export interface CaseFormProps {
  initialData?: CaseData;
  onSubmit: (data: CaseData) => Promise<void>;
  onCancel: () => void;
  isLoading?: boolean;
}

export const CaseForm: React.FC<CaseFormProps> = ({
  initialData,
  onSubmit,
  onCancel,
  isLoading = false,
}) => {
  const [formData, setFormData] = useState<CaseData>({
    title: '',
    description: '',
    status: 'draft',
    priority: 'medium',
    clientName: '',
    clientPhone: '',
    clientEmail: '',
    assignedTo: '',
    notes: '',
    ...initialData,
  });

  const [errors, setErrors] = useState<Record<string, string>>({});

  useEffect(() => {
    if (initialData) {
      setFormData({ ...formData, ...initialData });
    }
  }, [initialData]);

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.title?.trim()) {
      newErrors.title = 'Title is required';
    }

    if (formData.clientEmail && !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(formData.clientEmail)) {
      newErrors.clientEmail = 'Invalid email format';
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

  const handleChange = (field: keyof CaseData, value: any) => {
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
    <form className="case-form" onSubmit={handleSubmit}>
      <div className="case-form-section">
        <h3>Case Information</h3>
        <div className="case-form-row">
          <Input
            label="Title"
            value={formData.title}
            onChange={(e) => handleChange('title', e.target.value)}
            error={errors.title}
            placeholder="Enter case title"
            fullWidth
            required
          />
        </div>

        <div className="case-form-row">
          <textarea
            className="case-form-textarea"
            placeholder="Enter case description"
            value={formData.description || ''}
            onChange={(e) => handleChange('description', e.target.value)}
            rows={4}
          />
        </div>

        <div className="case-form-row case-form-row-split">
          <Select
            label="Status"
            value={formData.status}
            onChange={(e) => handleChange('status', e.target.value)}
            options={[
              { value: 'draft', label: 'Draft' },
              { value: 'active', label: 'Active' },
              { value: 'under_review', label: 'Under Review' },
              { value: 'pending_approval', label: 'Pending Approval' },
              { value: 'completed', label: 'Completed' },
              { value: 'archived', label: 'Archived' },
              { value: 'closed', label: 'Closed' },
            ]}
            fullWidth
          />

          <Select
            label="Priority"
            value={formData.priority}
            onChange={(e) => handleChange('priority', e.target.value)}
            options={[
              { value: 'low', label: 'Low' },
              { value: 'medium', label: 'Medium' },
              { value: 'high', label: 'High' },
              { value: 'critical', label: 'Critical' },
            ]}
            fullWidth
          />
        </div>
      </div>

      <div className="case-form-section">
        <h3>Client Information</h3>
        <div className="case-form-row">
          <Input
            label="Client Name"
            value={formData.clientName || ''}
            onChange={(e) => handleChange('clientName', e.target.value)}
            placeholder="Enter client name"
            fullWidth
          />
        </div>

        <div className="case-form-row case-form-row-split">
          <Input
            label="Client Phone"
            type="tel"
            value={formData.clientPhone || ''}
            onChange={(e) => handleChange('clientPhone', e.target.value)}
            placeholder="Enter phone number"
            fullWidth
          />

          <Input
            label="Client Email"
            type="email"
            value={formData.clientEmail || ''}
            onChange={(e) => handleChange('clientEmail', e.target.value)}
            error={errors.clientEmail}
            placeholder="Enter email address"
            fullWidth
          />
        </div>
      </div>

      <div className="case-form-section">
        <h3>Assignment</h3>
        <div className="case-form-row">
          <Input
            label="Assigned To"
            value={formData.assignedTo || ''}
            onChange={(e) => handleChange('assignedTo', e.target.value)}
            placeholder="Enter investigator name"
            fullWidth
          />
        </div>

        <div className="case-form-row">
          <textarea
            className="case-form-textarea"
            placeholder="Additional notes..."
            value={formData.notes || ''}
            onChange={(e) => handleChange('notes', e.target.value)}
            rows={3}
          />
        </div>
      </div>

      <div className="case-form-actions">
        <Button type="button" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
        <Button type="submit" variant="primary" loading={isLoading}>
          {initialData ? 'Update Case' : 'Create Case'}
        </Button>
      </div>
    </form>
  );
};

export default CaseForm;
