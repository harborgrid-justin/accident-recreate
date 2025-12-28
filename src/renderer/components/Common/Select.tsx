/**
 * Select Component - Reusable dropdown select with label and error
 */

import React, { SelectHTMLAttributes } from 'react';

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps extends SelectHTMLAttributes<HTMLSelectElement> {
  label?: string;
  error?: string;
  helperText?: string;
  fullWidth?: boolean;
  options: SelectOption[];
  placeholder?: string;
}

export const Select: React.FC<SelectProps> = ({
  label,
  error,
  helperText,
  fullWidth = false,
  options,
  placeholder,
  className = '',
  id,
  ...props
}) => {
  const selectId = id || `select-${Math.random().toString(36).substr(2, 9)}`;
  const widthClass = fullWidth ? 'select-full-width' : '';
  const errorClass = error ? 'select-error' : '';

  return (
    <div className={`select-wrapper ${widthClass} ${className}`}>
      {label && (
        <label htmlFor={selectId} className="select-label">
          {label}
          {props.required && <span className="select-required">*</span>}
        </label>
      )}
      <select id={selectId} className={`select ${errorClass}`} {...props}>
        {placeholder && (
          <option value="" disabled>
            {placeholder}
          </option>
        )}
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
      {error && <span className="select-error-message">{error}</span>}
      {helperText && !error && <span className="select-helper-text">{helperText}</span>}
    </div>
  );
};

export default Select;
