/**
 * Date Range Filter Component
 */

import React, { useState } from 'react';
import type { DateRange } from './types';

export interface DateRangeFilterProps {
  value?: DateRange;
  onChange: (range: DateRange | null) => void;
  className?: string;
}

type Preset = {
  label: string;
  getValue: () => DateRange;
};

const DateRangeFilter: React.FC<DateRangeFilterProps> = ({
  value,
  onChange,
  className = '',
}) => {
  const [mode, setMode] = useState<'preset' | 'custom'>('preset');

  const presets: Preset[] = [
    {
      label: 'Last 24 hours',
      getValue: () => {
        const end = new Date();
        const start = new Date();
        start.setDate(start.getDate() - 1);
        return { start, end };
      },
    },
    {
      label: 'Last 7 days',
      getValue: () => {
        const end = new Date();
        const start = new Date();
        start.setDate(start.getDate() - 7);
        return { start, end };
      },
    },
    {
      label: 'Last 30 days',
      getValue: () => {
        const end = new Date();
        const start = new Date();
        start.setDate(start.getDate() - 30);
        return { start, end };
      },
    },
    {
      label: 'Last 90 days',
      getValue: () => {
        const end = new Date();
        const start = new Date();
        start.setDate(start.getDate() - 90);
        return { start, end };
      },
    },
    {
      label: 'This year',
      getValue: () => {
        const now = new Date();
        const start = new Date(now.getFullYear(), 0, 1);
        const end = new Date();
        return { start, end };
      },
    },
  ];

  const handlePresetClick = (preset: Preset) => {
    onChange(preset.getValue());
  };

  const handleCustomChange = (field: 'start' | 'end', dateStr: string) => {
    if (!dateStr) {
      onChange(null);
      return;
    }

    const newDate = new Date(dateStr);

    if (field === 'start') {
      onChange({
        start: newDate,
        end: value?.end || new Date(),
      });
    } else {
      onChange({
        start: value?.start || new Date(),
        end: newDate,
      });
    }
  };

  const formatDateForInput = (date?: Date): string => {
    if (!date) return '';
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const day = String(date.getDate()).padStart(2, '0');
    return `${year}-${month}-${day}`;
  };

  const handleClear = () => {
    onChange(null);
  };

  return (
    <div className={`date-range-filter ${className}`}>
      <div className="mode-tabs">
        <button
          className={`mode-tab ${mode === 'preset' ? 'active' : ''}`}
          onClick={() => setMode('preset')}
        >
          Presets
        </button>
        <button
          className={`mode-tab ${mode === 'custom' ? 'active' : ''}`}
          onClick={() => setMode('custom')}
        >
          Custom
        </button>
      </div>

      {mode === 'preset' && (
        <div className="presets">
          {presets.map((preset) => (
            <button
              key={preset.label}
              onClick={() => handlePresetClick(preset)}
              className="preset-button"
            >
              {preset.label}
            </button>
          ))}
        </div>
      )}

      {mode === 'custom' && (
        <div className="custom-range">
          <div className="date-input-group">
            <label>From</label>
            <input
              type="date"
              value={formatDateForInput(value?.start)}
              onChange={(e) => handleCustomChange('start', e.target.value)}
              className="date-input"
            />
          </div>

          <div className="date-input-group">
            <label>To</label>
            <input
              type="date"
              value={formatDateForInput(value?.end)}
              onChange={(e) => handleCustomChange('end', e.target.value)}
              className="date-input"
            />
          </div>
        </div>
      )}

      {value && (
        <div className="current-range">
          <div className="range-display">
            <span className="range-label">Selected:</span>
            <span className="range-value">
              {value.start.toLocaleDateString()} - {value.end.toLocaleDateString()}
            </span>
          </div>
          <button onClick={handleClear} className="clear-button">
            Clear
          </button>
        </div>
      )}

      <style jsx>{`
        .date-range-filter {
          width: 100%;
        }

        .mode-tabs {
          display: flex;
          margin-bottom: 12px;
          border-bottom: 1px solid #e5e7eb;
        }

        .mode-tab {
          flex: 1;
          padding: 8px;
          border: none;
          background: transparent;
          color: #6b7280;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          border-bottom: 2px solid transparent;
          transition: all 0.2s;
        }

        .mode-tab.active {
          color: #3b82f6;
          border-bottom-color: #3b82f6;
        }

        .mode-tab:hover:not(.active) {
          color: #374151;
        }

        .presets {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .preset-button {
          padding: 10px 12px;
          border: 1px solid #e5e7eb;
          background: white;
          color: #374151;
          border-radius: 6px;
          font-size: 14px;
          text-align: left;
          cursor: pointer;
          transition: all 0.2s;
        }

        .preset-button:hover {
          background: #f9fafb;
          border-color: #3b82f6;
        }

        .custom-range {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .date-input-group {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .date-input-group label {
          font-size: 12px;
          font-weight: 500;
          color: #6b7280;
        }

        .date-input {
          padding: 8px 12px;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          font-size: 14px;
          outline: none;
          transition: border-color 0.2s;
        }

        .date-input:focus {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .current-range {
          margin-top: 12px;
          padding: 12px;
          background: #f9fafb;
          border-radius: 6px;
        }

        .range-display {
          display: flex;
          flex-direction: column;
          gap: 4px;
          margin-bottom: 8px;
        }

        .range-label {
          font-size: 12px;
          font-weight: 500;
          color: #6b7280;
        }

        .range-value {
          font-size: 14px;
          color: #111827;
        }

        .clear-button {
          width: 100%;
          padding: 6px;
          border: none;
          background: white;
          color: #ef4444;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .clear-button:hover {
          background: #fee2e2;
        }
      `}</style>
    </div>
  );
};

export default DateRangeFilter;
