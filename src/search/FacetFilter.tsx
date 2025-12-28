/**
 * Faceted Filter Component
 */

import React, { useState } from 'react';
import type { FacetValue } from './types';

export interface FacetFilterProps {
  field: string;
  values: FacetValue[];
  selectedValues: string[];
  onChange: (values: string[]) => void;
  maxVisible?: number;
  searchable?: boolean;
  className?: string;
}

const FacetFilter: React.FC<FacetFilterProps> = ({
  field,
  values,
  selectedValues,
  onChange,
  maxVisible = 5,
  searchable = true,
  className = '',
}) => {
  const [searchTerm, setSearchTerm] = useState('');
  const [showAll, setShowAll] = useState(false);

  const handleToggle = (value: string) => {
    if (selectedValues.includes(value)) {
      onChange(selectedValues.filter((v) => v !== value));
    } else {
      onChange([...selectedValues, value]);
    }
  };

  const handleSelectAll = () => {
    if (selectedValues.length === filteredValues.length) {
      onChange([]);
    } else {
      onChange(filteredValues.map((v) => v.value));
    }
  };

  // Filter values based on search term
  const filteredValues = values.filter((v) =>
    v.value.toLowerCase().includes(searchTerm.toLowerCase())
  );

  // Determine which values to show
  const visibleValues = showAll
    ? filteredValues
    : filteredValues.slice(0, maxVisible);

  const hasMore = filteredValues.length > maxVisible;

  return (
    <div className={`facet-filter ${className}`}>
      {searchable && values.length > 5 && (
        <div className="facet-search">
          <input
            type="text"
            value={searchTerm}
            onChange={(e) => setSearchTerm(e.target.value)}
            placeholder={`Search ${field}...`}
            className="search-input"
          />
        </div>
      )}

      {filteredValues.length > 1 && (
        <div className="facet-actions">
          <button onClick={handleSelectAll} className="select-all-btn">
            {selectedValues.length === filteredValues.length
              ? 'Deselect all'
              : 'Select all'}
          </button>
        </div>
      )}

      <div className="facet-values">
        {visibleValues.length === 0 ? (
          <div className="no-results">No options found</div>
        ) : (
          visibleValues.map((facetValue) => (
            <label
              key={facetValue.value}
              className="facet-value"
            >
              <input
                type="checkbox"
                checked={selectedValues.includes(facetValue.value)}
                onChange={() => handleToggle(facetValue.value)}
                className="facet-checkbox"
              />
              <span className="facet-label">{facetValue.value}</span>
              <span className="facet-count">{facetValue.count}</span>
            </label>
          ))
        )}
      </div>

      {hasMore && !showAll && (
        <button
          onClick={() => setShowAll(true)}
          className="show-more"
        >
          Show {filteredValues.length - maxVisible} more
        </button>
      )}

      {hasMore && showAll && (
        <button
          onClick={() => setShowAll(false)}
          className="show-less"
        >
          Show less
        </button>
      )}

      <style jsx>{`
        .facet-filter {
          width: 100%;
        }

        .facet-search {
          margin-bottom: 12px;
        }

        .search-input {
          width: 100%;
          padding: 8px 12px;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          font-size: 14px;
          outline: none;
          transition: border-color 0.2s;
        }

        .search-input:focus {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .facet-actions {
          margin-bottom: 8px;
        }

        .select-all-btn {
          padding: 4px 8px;
          border: none;
          background: transparent;
          color: #3b82f6;
          font-size: 12px;
          font-weight: 500;
          cursor: pointer;
          transition: opacity 0.2s;
        }

        .select-all-btn:hover {
          opacity: 0.8;
        }

        .facet-values {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .facet-value {
          display: flex;
          align-items: center;
          gap: 8px;
          padding: 6px 0;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .facet-value:hover {
          background: #f9fafb;
        }

        .facet-checkbox {
          width: 16px;
          height: 16px;
          cursor: pointer;
        }

        .facet-label {
          flex: 1;
          font-size: 14px;
          color: #374151;
        }

        .facet-count {
          padding: 2px 8px;
          background: #f3f4f6;
          border-radius: 10px;
          font-size: 12px;
          color: #6b7280;
          font-weight: 500;
        }

        .no-results {
          padding: 12px;
          text-align: center;
          color: #9ca3af;
          font-size: 14px;
        }

        .show-more,
        .show-less {
          width: 100%;
          margin-top: 8px;
          padding: 8px;
          border: none;
          background: #f9fafb;
          color: #3b82f6;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .show-more:hover,
        .show-less:hover {
          background: #f3f4f6;
        }
      `}</style>
    </div>
  );
};

export default FacetFilter;
