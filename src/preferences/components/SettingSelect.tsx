/**
 * Setting Select Component
 * A reusable dropdown selector for preferences
 */

import React from 'react';

interface SelectOption {
  value: string | number;
  label: string;
}

interface SettingSelectProps {
  label: string;
  description?: string;
  value: string | number;
  options: SelectOption[];
  onChange: (value: string | number) => void;
  disabled?: boolean;
}

const SettingSelect: React.FC<SettingSelectProps> = ({
  label,
  description,
  value,
  options,
  onChange,
  disabled = false,
}) => {
  const handleChange = (e: React.ChangeEvent<HTMLSelectElement>) => {
    const selectedValue = e.target.value;
    // Try to parse as number if it looks like a number
    const parsedValue = !isNaN(Number(selectedValue)) && selectedValue !== ''
      ? Number(selectedValue)
      : selectedValue;
    onChange(parsedValue);
  };

  return (
    <div className={`setting-select ${disabled ? 'disabled' : ''}`}>
      <div className="select-info">
        <label className="select-label" htmlFor={`select-${label}`}>
          {label}
        </label>
        {description && <div className="select-description">{description}</div>}
      </div>

      <div className="select-wrapper">
        <select
          id={`select-${label}`}
          value={value}
          onChange={handleChange}
          disabled={disabled}
          className="select-element"
        >
          {options.map((option) => (
            <option key={option.value} value={option.value}>
              {option.label}
            </option>
          ))}
        </select>
        <div className="select-arrow">▼</div>
      </div>

      <style>{`
        .setting-select {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem 0;
          gap: 1rem;
        }

        .setting-select.disabled {
          opacity: 0.5;
        }

        .select-info {
          flex: 1;
        }

        .select-label {
          display: block;
          font-weight: 500;
          margin-bottom: 0.25rem;
        }

        .select-description {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .select-wrapper {
          position: relative;
          min-width: 200px;
        }

        .select-element {
          width: 100%;
          padding: 0.5rem 2rem 0.5rem 0.75rem;
          border: 1px solid var(--border-color, #ccc);
          border-radius: 4px;
          background: white;
          font-size: 0.9375rem;
          cursor: pointer;
          appearance: none;
          transition: all 0.2s;
        }

        .select-element:hover:not(:disabled) {
          border-color: var(--primary-color, #0066cc);
        }

        .select-element:focus {
          outline: none;
          border-color: var(--primary-color, #0066cc);
          box-shadow: 0 0 0 3px var(--primary-color-light, rgba(0, 102, 204, 0.1));
        }

        .select-element:disabled {
          cursor: not-allowed;
          background: var(--disabled-bg, #f5f5f5);
        }

        .select-arrow {
          position: absolute;
          right: 0.75rem;
          top: 50%;
          transform: translateY(-50%);
          pointer-events: none;
          font-size: 0.625rem;
          color: var(--text-secondary, #666);
        }

        .dark .select-description {
          color: #999;
        }

        .dark .select-element {
          background: #2a2a2a;
          border-color: #404040;
          color: #e0e0e0;
        }

        .dark .select-element:disabled {
          background: #1a1a1a;
        }

        .dark .select-arrow {
          color: #999;
        }

        .high-contrast .select-element {
          border-width: 2px;
        }

        .high-contrast .select-element:focus {
          box-shadow: 0 0 0 4px currentColor;
        }
      `}</style>
    </div>
  );
};

export default SettingSelect;
