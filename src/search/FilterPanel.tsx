/**
 * Filter Panel Component
 */

import React, { useState } from 'react';
import type { SearchFilters, FacetResult } from './types';
import FacetFilter from './FacetFilter';
import DateRangeFilter from './DateRangeFilter';

export interface FilterPanelProps {
  filters: SearchFilters;
  onFiltersChange: (filters: SearchFilters) => void;
  facets?: FacetResult[];
  onClearAll?: () => void;
  className?: string;
}

const FilterPanel: React.FC<FilterPanelProps> = ({
  filters,
  onFiltersChange,
  facets = [],
  onClearAll,
  className = '',
}) => {
  const [expandedSections, setExpandedSections] = useState<Set<string>>(
    new Set(['category', 'status', 'dateRange'])
  );

  const toggleSection = (section: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(section)) {
        next.delete(section);
      } else {
        next.add(section);
      }
      return next;
    });
  };

  const handleCategoryChange = (values: string[]) => {
    onFiltersChange({
      ...filters,
      categories: values.length > 0 ? values : undefined,
    });
  };

  const handleStatusChange = (values: string[]) => {
    onFiltersChange({
      ...filters,
      statuses: values.length > 0 ? values : undefined,
    });
  };

  const handleSeverityChange = (values: string[]) => {
    onFiltersChange({
      ...filters,
      severities: values.length > 0 ? values : undefined,
    });
  };

  const handleTagsChange = (values: string[]) => {
    onFiltersChange({
      ...filters,
      tags: values.length > 0 ? values : undefined,
    });
  };

  const handleDateRangeChange = (range: { start: Date; end: Date } | null) => {
    onFiltersChange({
      ...filters,
      dateRange: range || undefined,
    });
  };

  const activeFilterCount = [
    filters.categories?.length || 0,
    filters.statuses?.length || 0,
    filters.severities?.length || 0,
    filters.tags?.length || 0,
    filters.dateRange ? 1 : 0,
  ].reduce((sum, count) => sum + count, 0);

  const getFacetForField = (field: string) => {
    return facets.find((f) => f.field === field);
  };

  return (
    <div className={`filter-panel ${className}`}>
      <div className="filter-header">
        <h3>Filters</h3>
        {activeFilterCount > 0 && (
          <button onClick={onClearAll} className="clear-all">
            Clear all ({activeFilterCount})
          </button>
        )}
      </div>

      <div className="filter-sections">
        {/* Category Filter */}
        <div className="filter-section">
          <button
            className="section-header"
            onClick={() => toggleSection('category')}
          >
            <span>Category</span>
            {filters.categories && filters.categories.length > 0 && (
              <span className="active-count">{filters.categories.length}</span>
            )}
            <svg
              className={`chevron ${expandedSections.has('category') ? 'expanded' : ''}`}
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" />
            </svg>
          </button>

          {expandedSections.has('category') && (
            <div className="section-content">
              <FacetFilter
                field="category"
                values={getFacetForField('category')?.values || []}
                selectedValues={filters.categories || []}
                onChange={handleCategoryChange}
              />
            </div>
          )}
        </div>

        {/* Status Filter */}
        <div className="filter-section">
          <button
            className="section-header"
            onClick={() => toggleSection('status')}
          >
            <span>Status</span>
            {filters.statuses && filters.statuses.length > 0 && (
              <span className="active-count">{filters.statuses.length}</span>
            )}
            <svg
              className={`chevron ${expandedSections.has('status') ? 'expanded' : ''}`}
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" />
            </svg>
          </button>

          {expandedSections.has('status') && (
            <div className="section-content">
              <FacetFilter
                field="status"
                values={getFacetForField('status')?.values || []}
                selectedValues={filters.statuses || []}
                onChange={handleStatusChange}
              />
            </div>
          )}
        </div>

        {/* Severity Filter */}
        <div className="filter-section">
          <button
            className="section-header"
            onClick={() => toggleSection('severity')}
          >
            <span>Severity</span>
            {filters.severities && filters.severities.length > 0 && (
              <span className="active-count">{filters.severities.length}</span>
            )}
            <svg
              className={`chevron ${expandedSections.has('severity') ? 'expanded' : ''}`}
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" />
            </svg>
          </button>

          {expandedSections.has('severity') && (
            <div className="section-content">
              <FacetFilter
                field="severity"
                values={getFacetForField('severity')?.values || []}
                selectedValues={filters.severities || []}
                onChange={handleSeverityChange}
              />
            </div>
          )}
        </div>

        {/* Date Range Filter */}
        <div className="filter-section">
          <button
            className="section-header"
            onClick={() => toggleSection('dateRange')}
          >
            <span>Date Range</span>
            {filters.dateRange && <span className="active-count">1</span>}
            <svg
              className={`chevron ${expandedSections.has('dateRange') ? 'expanded' : ''}`}
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z" />
            </svg>
          </button>

          {expandedSections.has('dateRange') && (
            <div className="section-content">
              <DateRangeFilter
                value={filters.dateRange}
                onChange={handleDateRangeChange}
              />
            </div>
          )}
        </div>
      </div>

      <style jsx>{`
        .filter-panel {
          width: 100%;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
        }

        .filter-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .filter-header h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .clear-all {
          padding: 6px 12px;
          border: none;
          background: #f3f4f6;
          color: #374151;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .clear-all:hover {
          background: #e5e7eb;
        }

        .filter-sections {
          display: flex;
          flex-direction: column;
        }

        .filter-section {
          border-bottom: 1px solid #e5e7eb;
        }

        .filter-section:last-child {
          border-bottom: none;
        }

        .section-header {
          width: 100%;
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border: none;
          background: transparent;
          cursor: pointer;
          font-size: 14px;
          font-weight: 500;
          color: #374151;
          transition: background-color 0.2s;
        }

        .section-header:hover {
          background: #f9fafb;
        }

        .section-header span:first-child {
          flex: 1;
          text-align: left;
        }

        .active-count {
          display: inline-flex;
          align-items: center;
          justify-content: center;
          min-width: 20px;
          height: 20px;
          padding: 0 6px;
          margin-right: 8px;
          background: #3b82f6;
          color: white;
          border-radius: 10px;
          font-size: 12px;
          font-weight: 600;
        }

        .chevron {
          transition: transform 0.2s;
        }

        .chevron.expanded {
          transform: rotate(180deg);
        }

        .section-content {
          padding: 8px 16px 16px;
        }
      `}</style>
    </div>
  );
};

export default FilterPanel;
