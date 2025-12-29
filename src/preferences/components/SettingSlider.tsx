/**
 * Setting Slider Component
 * A reusable slider for numeric preferences
 */

import React, { useState } from 'react';

interface SettingSliderProps {
  label: string;
  description?: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (value: number) => void;
  disabled?: boolean;
  showValue?: boolean;
}

const SettingSlider: React.FC<SettingSliderProps> = ({
  label,
  description,
  value,
  min,
  max,
  step = 1,
  unit = '',
  onChange,
  disabled = false,
  showValue = true,
}) => {
  const [isDragging, setIsDragging] = useState(false);

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onChange(Number(e.target.value));
  };

  const handleMouseDown = () => setIsDragging(true);
  const handleMouseUp = () => setIsDragging(false);

  const percentage = ((value - min) / (max - min)) * 100;

  return (
    <div className={`setting-slider ${disabled ? 'disabled' : ''}`}>
      <div className="slider-header">
        <div className="slider-info">
          <label className="slider-label" htmlFor={`slider-${label}`}>
            {label}
          </label>
          {description && <div className="slider-description">{description}</div>}
        </div>
        {showValue && (
          <div className="slider-value">
            {value}{unit}
          </div>
        )}
      </div>

      <div className="slider-wrapper">
        <div className="slider-track">
          <div
            className="slider-fill"
            style={{ width: `${percentage}%` }}
          />
        </div>
        <input
          type="range"
          id={`slider-${label}`}
          min={min}
          max={max}
          step={step}
          value={value}
          onChange={handleChange}
          onMouseDown={handleMouseDown}
          onMouseUp={handleMouseUp}
          onTouchStart={handleMouseDown}
          onTouchEnd={handleMouseUp}
          disabled={disabled}
          className={`slider-input ${isDragging ? 'dragging' : ''}`}
          aria-label={label}
          aria-valuemin={min}
          aria-valuemax={max}
          aria-valuenow={value}
          aria-valuetext={`${value}${unit}`}
        />
        <div className="slider-marks">
          <span className="slider-mark">{min}{unit}</span>
          <span className="slider-mark">{max}{unit}</span>
        </div>
      </div>

      <style>{`
        .setting-slider {
          padding: 1rem 0;
        }

        .setting-slider.disabled {
          opacity: 0.5;
        }

        .slider-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 1rem;
        }

        .slider-info {
          flex: 1;
        }

        .slider-label {
          display: block;
          font-weight: 500;
          margin-bottom: 0.25rem;
        }

        .slider-description {
          font-size: 0.875rem;
          color: var(--text-secondary, #666);
        }

        .slider-value {
          font-weight: 600;
          font-size: 1rem;
          color: var(--primary-color, #0066cc);
          min-width: 60px;
          text-align: right;
        }

        .slider-wrapper {
          position: relative;
          padding: 0.5rem 0;
        }

        .slider-track {
          position: absolute;
          width: 100%;
          height: 6px;
          background: var(--slider-track-bg, #e0e0e0);
          border-radius: 3px;
          top: 50%;
          transform: translateY(-50%);
          pointer-events: none;
        }

        .slider-fill {
          height: 100%;
          background: var(--primary-color, #0066cc);
          border-radius: 3px;
          transition: width 0.1s;
        }

        .slider-input {
          position: relative;
          width: 100%;
          height: 24px;
          -webkit-appearance: none;
          appearance: none;
          background: transparent;
          cursor: pointer;
          z-index: 1;
        }

        .slider-input:disabled {
          cursor: not-allowed;
        }

        /* Webkit browsers (Chrome, Safari) */
        .slider-input::-webkit-slider-thumb {
          -webkit-appearance: none;
          appearance: none;
          width: 20px;
          height: 20px;
          background: white;
          border: 2px solid var(--primary-color, #0066cc);
          border-radius: 50%;
          cursor: pointer;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
          transition: all 0.2s;
        }

        .slider-input::-webkit-slider-thumb:hover {
          transform: scale(1.1);
          box-shadow: 0 3px 6px rgba(0, 0, 0, 0.3);
        }

        .slider-input.dragging::-webkit-slider-thumb,
        .slider-input:active::-webkit-slider-thumb {
          transform: scale(1.2);
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        }

        /* Firefox */
        .slider-input::-moz-range-thumb {
          width: 20px;
          height: 20px;
          background: white;
          border: 2px solid var(--primary-color, #0066cc);
          border-radius: 50%;
          cursor: pointer;
          box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
          transition: all 0.2s;
        }

        .slider-input::-moz-range-thumb:hover {
          transform: scale(1.1);
          box-shadow: 0 3px 6px rgba(0, 0, 0, 0.3);
        }

        .slider-input.dragging::-moz-range-thumb,
        .slider-input:active::-moz-range-thumb {
          transform: scale(1.2);
          box-shadow: 0 4px 8px rgba(0, 0, 0, 0.3);
        }

        .slider-input:focus {
          outline: none;
        }

        .slider-input:focus::-webkit-slider-thumb {
          box-shadow: 0 0 0 4px var(--primary-color-light, rgba(0, 102, 204, 0.2));
        }

        .slider-input:focus::-moz-range-thumb {
          box-shadow: 0 0 0 4px var(--primary-color-light, rgba(0, 102, 204, 0.2));
        }

        .slider-marks {
          display: flex;
          justify-content: space-between;
          margin-top: 0.5rem;
        }

        .slider-mark {
          font-size: 0.75rem;
          color: var(--text-secondary, #666);
        }

        .dark .slider-description,
        .dark .slider-mark {
          color: #999;
        }

        .dark .slider-track {
          background: #404040;
        }

        .dark .slider-input::-webkit-slider-thumb {
          background: #2a2a2a;
        }

        .dark .slider-input::-moz-range-thumb {
          background: #2a2a2a;
        }

        .high-contrast .slider-track {
          border: 2px solid currentColor;
        }

        .high-contrast .slider-input::-webkit-slider-thumb,
        .high-contrast .slider-input::-moz-range-thumb {
          border-width: 3px;
        }
      `}</style>
    </div>
  );
};

export default SettingSlider;
