/**
 * Advanced Search Bar Component
 */

import React, { useState, useRef, useEffect } from 'react';
import type { SearchQuery, Suggestion } from './types';

export interface SearchBarProps {
  query: SearchQuery;
  onQueryChange: (query: SearchQuery) => void;
  onSearch: () => void;
  suggestions?: Suggestion[];
  loading?: boolean;
  placeholder?: string;
  enableFuzzy?: boolean;
  enableAdvanced?: boolean;
  className?: string;
}

const SearchBar: React.FC<SearchBarProps> = ({
  query,
  onQueryChange,
  onSearch,
  suggestions = [],
  loading = false,
  placeholder = 'Search...',
  enableFuzzy = true,
  enableAdvanced = true,
  className = '',
}) => {
  const [showSuggestions, setShowSuggestions] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [selectedSuggestionIndex, setSelectedSuggestionIndex] = useState(-1);
  const inputRef = useRef<HTMLInputElement>(null);
  const suggestionsRef = useRef<HTMLDivElement>(null);

  // Handle text input
  const handleTextChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onQueryChange({ ...query, text: e.target.value });
    setShowSuggestions(true);
    setSelectedSuggestionIndex(-1);
  };

  // Handle search submission
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    setShowSuggestions(false);
    onSearch();
  };

  // Handle suggestion selection
  const handleSuggestionClick = (suggestion: Suggestion) => {
    onQueryChange({ ...query, text: suggestion.text });
    setShowSuggestions(false);
    inputRef.current?.focus();
    onSearch();
  };

  // Keyboard navigation
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (!showSuggestions || suggestions.length === 0) return;

    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault();
        setSelectedSuggestionIndex((prev) =>
          prev < suggestions.length - 1 ? prev + 1 : prev
        );
        break;

      case 'ArrowUp':
        e.preventDefault();
        setSelectedSuggestionIndex((prev) => (prev > 0 ? prev - 1 : -1));
        break;

      case 'Enter':
        if (selectedSuggestionIndex >= 0) {
          e.preventDefault();
          handleSuggestionClick(suggestions[selectedSuggestionIndex]);
        }
        break;

      case 'Escape':
        setShowSuggestions(false);
        setSelectedSuggestionIndex(-1);
        break;
    }
  };

  // Close suggestions when clicking outside
  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (
        suggestionsRef.current &&
        !suggestionsRef.current.contains(e.target as Node) &&
        inputRef.current &&
        !inputRef.current.contains(e.target as Node)
      ) {
        setShowSuggestions(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  return (
    <div className={`search-bar ${className}`}>
      <form onSubmit={handleSubmit} className="search-form">
        <div className="search-input-wrapper">
          <input
            ref={inputRef}
            type="text"
            value={query.text}
            onChange={handleTextChange}
            onKeyDown={handleKeyDown}
            onFocus={() => setShowSuggestions(true)}
            placeholder={placeholder}
            className="search-input"
            autoComplete="off"
          />

          {loading && (
            <div className="search-loading">
              <span className="spinner"></span>
            </div>
          )}

          <button type="submit" className="search-button" disabled={loading}>
            <svg
              width="20"
              height="20"
              viewBox="0 0 20 20"
              fill="currentColor"
            >
              <path d="M8 4a4 4 0 100 8 4 4 0 000-8zM2 8a6 6 0 1110.89 3.476l4.817 4.817a1 1 0 01-1.414 1.414l-4.816-4.816A6 6 0 012 8z" />
            </svg>
          </button>

          {enableAdvanced && (
            <button
              type="button"
              onClick={() => setShowAdvanced(!showAdvanced)}
              className="advanced-toggle"
            >
              Advanced
            </button>
          )}
        </div>

        {/* Suggestions dropdown */}
        {showSuggestions && suggestions.length > 0 && (
          <div ref={suggestionsRef} className="suggestions-dropdown">
            {suggestions.map((suggestion, index) => (
              <div
                key={index}
                className={`suggestion-item ${
                  index === selectedSuggestionIndex ? 'selected' : ''
                }`}
                onClick={() => handleSuggestionClick(suggestion)}
                onMouseEnter={() => setSelectedSuggestionIndex(index)}
              >
                <div className="suggestion-text">{suggestion.text}</div>
                {suggestion.type && (
                  <span className={`suggestion-type ${suggestion.type}`}>
                    {suggestion.type}
                  </span>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Advanced search options */}
        {showAdvanced && (
          <div className="advanced-options">
            <div className="option-group">
              <label>
                <input
                  type="checkbox"
                  checked={query.operator === 'AND'}
                  onChange={(e) =>
                    onQueryChange({
                      ...query,
                      operator: e.target.checked ? 'AND' : 'OR',
                    })
                  }
                />
                <span>Match all terms (AND)</span>
              </label>
            </div>

            {enableFuzzy && (
              <div className="option-group">
                <label>
                  <input
                    type="checkbox"
                    checked={query.fuzzy || false}
                    onChange={(e) =>
                      onQueryChange({ ...query, fuzzy: e.target.checked })
                    }
                  />
                  <span>Fuzzy matching</span>
                </label>
              </div>
            )}

            <div className="option-group">
              <label>
                <span>Search in:</span>
                <select
                  multiple
                  value={query.fields || []}
                  onChange={(e) => {
                    const selected = Array.from(e.target.selectedOptions).map(
                      (opt) => opt.value
                    );
                    onQueryChange({ ...query, fields: selected });
                  }}
                  className="field-selector"
                >
                  <option value="title">Title</option>
                  <option value="content">Content</option>
                  <option value="description">Description</option>
                  <option value="location">Location</option>
                </select>
              </label>
            </div>
          </div>
        )}
      </form>

      <style jsx>{`
        .search-bar {
          position: relative;
          width: 100%;
        }

        .search-form {
          width: 100%;
        }

        .search-input-wrapper {
          position: relative;
          display: flex;
          align-items: center;
          background: white;
          border: 2px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
          transition: border-color 0.2s;
        }

        .search-input-wrapper:focus-within {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .search-input {
          flex: 1;
          padding: 12px 16px;
          border: none;
          outline: none;
          font-size: 16px;
        }

        .search-loading {
          padding: 0 12px;
        }

        .spinner {
          display: inline-block;
          width: 20px;
          height: 20px;
          border: 3px solid #e5e7eb;
          border-top-color: #3b82f6;
          border-radius: 50%;
          animation: spin 0.8s linear infinite;
        }

        @keyframes spin {
          to {
            transform: rotate(360deg);
          }
        }

        .search-button,
        .advanced-toggle {
          padding: 12px 16px;
          border: none;
          background: transparent;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .search-button:hover,
        .advanced-toggle:hover {
          background-color: #f3f4f6;
        }

        .search-button:disabled {
          opacity: 0.5;
          cursor: not-allowed;
        }

        .suggestions-dropdown {
          position: absolute;
          top: 100%;
          left: 0;
          right: 0;
          margin-top: 8px;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          box-shadow: 0 10px 25px rgba(0, 0, 0, 0.1);
          max-height: 300px;
          overflow-y: auto;
          z-index: 1000;
        }

        .suggestion-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .suggestion-item:hover,
        .suggestion-item.selected {
          background-color: #f3f4f6;
        }

        .suggestion-text {
          flex: 1;
        }

        .suggestion-type {
          padding: 2px 8px;
          border-radius: 4px;
          font-size: 12px;
          font-weight: 500;
        }

        .suggestion-type.autocomplete {
          background-color: #dbeafe;
          color: #1e40af;
        }

        .suggestion-type.correction {
          background-color: #fef3c7;
          color: #92400e;
        }

        .suggestion-type.popular {
          background-color: #ddd6fe;
          color: #5b21b6;
        }

        .advanced-options {
          margin-top: 12px;
          padding: 16px;
          background: #f9fafb;
          border-radius: 8px;
        }

        .option-group {
          margin-bottom: 12px;
        }

        .option-group:last-child {
          margin-bottom: 0;
        }

        .option-group label {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .option-group input[type='checkbox'] {
          width: 16px;
          height: 16px;
        }

        .field-selector {
          width: 100%;
          padding: 8px;
          border: 1px solid #e5e7eb;
          border-radius: 4px;
          margin-top: 4px;
        }
      `}</style>
    </div>
  );
};

export default SearchBar;
