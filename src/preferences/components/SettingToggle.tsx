/**
 * Setting Toggle Component
 * A reusable toggle switch for boolean preferences
 */

import React from 'react';

interface SettingToggleProps {
  label: string;
  description?: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
}

const SettingToggle: React.FC<SettingToggleProps> = ({
  label,
  description,
  checked,
  onChange,
  disabled = false,
}) => {
  const handleToggle = () => {
    if (!disabled) {
      onChange(!checked);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if ((e.key === 'Enter' || e.key === ' ') && !disabled) {
      e.preventDefault();
      onChange(!checked);
    }
  };

  return (
    <div className={`setting-toggle ${disabled ? 'disabled' : ''}`}>
      <div className="toggle-info">
        <label className="toggle-label" htmlFor={`toggle-${label}`}>
          {label}
        </label>
        {description && <div className="toggle-description">{description}</div>}
      </div>

      <div
        className={`toggle-switch ${checked ? 'checked' : ''} ${disabled ? 'disabled' : ''}`}
        role="switch"
        aria-checked={checked}
        aria-label={label}
        tabIndex={disabled ? -1 : 0}
        onClick={handleToggle}
        onKeyPress={handleKeyPress}
      >
        <input
          type="checkbox"
          id={`toggle-${label}`}
          checked={checked}
          onChange={(e) => onChange(e.target.checked)}
          disabled={disabled}
          className="toggle-input"
        />
        <div className="toggle-slider">
          <div className="toggle-thumb" />
        </div>
      </div>

      <style>{`
        .setting-toggle {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 1rem 0;
          gap: 1rem;
        }

        .setting-toggle.disabled {
          opacity: 0.5;
        }

        .toggle-info {
          flex: 1;
        }

        .toggle-label {
          display: block;
          font-weight: 500;
          margin-bottom: 0.25rem;
          cursor: pointer;
        }

        .toggle-description {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .toggle-switch {
          position: relative;
          width: 48px;
          height: 24px;
          cursor: pointer;
          flex-shrink: 0;
        }

        .toggle-switch.disabled {
          cursor: not-allowed;
        }

        .toggle-switch:focus {
          outline: 2px solid var(--primary-color, #0066cc);
          outline-offset: 2px;
          border-radius: 12px;
        }

        .toggle-input {
          position: absolute;
          opacity: 0;
          width: 0;
          height: 0;
        }

        .toggle-slider {
          position: absolute;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: var(--toggle-bg, #ccc);
          border-radius: 12px;
          transition: background 0.2s;
        }

        .toggle-switch.checked .toggle-slider {
          background: var(--primary-color, #0066cc);
        }

        .toggle-thumb {
          position: absolute;
          top: 2px;
          left: 2px;
          width: 20px;
          height: 20px;
          background: white;
          border-radius: 50%;
          transition: transform 0.2s;
          box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
        }

        .toggle-switch.checked .toggle-thumb {
          transform: translateX(24px);
        }

        .toggle-switch:not(.disabled):hover .toggle-slider {
          opacity: 0.9;
        }

        .dark .toggle-description {
          color: #999;
        }

        .dark .toggle-slider {
          background: #555;
        }

        .dark .toggle-switch.checked .toggle-slider {
          background: var(--primary-color, #0066cc);
        }

        .high-contrast .toggle-slider {
          border: 2px solid currentColor;
        }

        .high-contrast .toggle-thumb {
          border: 2px solid currentColor;
        }
      `}</style>
    </div>
  );
};

export default SettingToggle;
