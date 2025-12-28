/**
 * Search Results Display Component
 */

import React from 'react';
import type { SearchResponse, SearchHit } from './types';

export interface SearchResultsProps {
  results: SearchResponse | null;
  loading?: boolean;
  error?: Error | null;
  onResultClick?: (hit: SearchHit) => void;
  renderResult?: (hit: SearchHit) => React.ReactNode;
  emptyMessage?: string;
  className?: string;
}

const SearchResults: React.FC<SearchResultsProps> = ({
  results,
  loading = false,
  error = null,
  onResultClick,
  renderResult,
  emptyMessage = 'No results found',
  className = '',
}) => {
  // Loading state
  if (loading) {
    return (
      <div className={`search-results loading ${className}`}>
        <div className="loading-container">
          <div className="spinner-large"></div>
          <p>Searching...</p>
        </div>

        <style jsx>{`
          .loading-container {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 48px;
            color: #6b7280;
          }

          .spinner-large {
            width: 48px;
            height: 48px;
            border: 4px solid #e5e7eb;
            border-top-color: #3b82f6;
            border-radius: 50%;
            animation: spin 0.8s linear infinite;
            margin-bottom: 16px;
          }

          @keyframes spin {
            to {
              transform: rotate(360deg);
            }
          }
        `}</style>
      </div>
    );
  }

  // Error state
  if (error) {
    return (
      <div className={`search-results error ${className}`}>
        <div className="error-container">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="#ef4444" strokeWidth="2" />
            <path d="M12 8v4M12 16h.01" stroke="#ef4444" strokeWidth="2" strokeLinecap="round" />
          </svg>
          <h3>Search Error</h3>
          <p>{error.message}</p>
        </div>

        <style jsx>{`
          .error-container {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 48px;
            color: #ef4444;
          }

          .error-container h3 {
            margin: 16px 0 8px;
            font-size: 18px;
            font-weight: 600;
          }

          .error-container p {
            color: #6b7280;
          }
        `}</style>
      </div>
    );
  }

  // Empty state
  if (!results || results.total === 0) {
    return (
      <div className={`search-results empty ${className}`}>
        <div className="empty-container">
          <svg width="64" height="64" viewBox="0 0 24 24" fill="none">
            <path
              d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
              stroke="#9ca3af"
              strokeWidth="2"
              strokeLinecap="round"
            />
          </svg>
          <p>{emptyMessage}</p>
        </div>

        <style jsx>{`
          .empty-container {
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            padding: 48px;
            color: #9ca3af;
          }

          .empty-container p {
            margin-top: 16px;
            font-size: 16px;
          }
        `}</style>
      </div>
    );
  }

  return (
    <div className={`search-results ${className}`}>
      <div className="results-header">
        <div className="results-count">
          {results.total.toLocaleString()} result{results.total !== 1 ? 's' : ''}
        </div>
        <div className="results-time">
          {results.tookMs}ms
        </div>
      </div>

      <div className="results-list">
        {results.hits.map((hit, index) => (
          <div
            key={hit.id || index}
            className="result-item"
            onClick={() => onResultClick?.(hit)}
          >
            {renderResult ? (
              renderResult(hit)
            ) : (
              <DefaultResultRenderer hit={hit} />
            )}
          </div>
        ))}
      </div>

      <style jsx>{`
        .search-results {
          width: 100%;
        }

        .results-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 12px 16px;
          border-bottom: 1px solid #e5e7eb;
          font-size: 14px;
          color: #6b7280;
        }

        .results-count {
          font-weight: 500;
        }

        .results-time {
          color: #9ca3af;
        }

        .results-list {
          display: flex;
          flex-direction: column;
          gap: 1px;
          background: #e5e7eb;
        }

        .result-item {
          background: white;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .result-item:hover {
          background: #f9fafb;
        }
      `}</style>
    </div>
  );
};

// Default result renderer
const DefaultResultRenderer: React.FC<{ hit: SearchHit }> = ({ hit }) => {
  const { document, highlights, score } = hit;

  return (
    <div className="default-result">
      <div className="result-header">
        <h3 className="result-title">{document.title || 'Untitled'}</h3>
        <span className="result-score">{score.toFixed(2)}</span>
      </div>

      {(document.description || highlights?.[0]?.snippet) && (
        <div className="result-description">
          {highlights?.[0]?.snippet ? (
            <div
              dangerouslySetInnerHTML={{ __html: highlights[0].snippet }}
            />
          ) : (
            <p>{document.description}</p>
          )}
        </div>
      )}

      <div className="result-meta">
        {document.category && (
          <span className="meta-badge category">{document.category}</span>
        )}
        {document.status && (
          <span className="meta-badge status">{document.status}</span>
        )}
        {document.severity && (
          <span className="meta-badge severity">{document.severity}</span>
        )}
        {document.created_at && (
          <span className="meta-date">
            {new Date(document.created_at).toLocaleDateString()}
          </span>
        )}
      </div>

      <style jsx>{`
        .default-result {
          padding: 16px;
        }

        .result-header {
          display: flex;
          justify-content: space-between;
          align-items: flex-start;
          margin-bottom: 8px;
        }

        .result-title {
          font-size: 18px;
          font-weight: 600;
          color: #111827;
          margin: 0;
        }

        .result-score {
          padding: 4px 8px;
          background: #f3f4f6;
          border-radius: 4px;
          font-size: 12px;
          color: #6b7280;
          font-weight: 500;
        }

        .result-description {
          margin-bottom: 12px;
          color: #4b5563;
          line-height: 1.5;
        }

        .result-description :global(em) {
          font-style: normal;
          font-weight: 600;
          background: #fef3c7;
          padding: 2px 4px;
          border-radius: 2px;
        }

        .result-meta {
          display: flex;
          gap: 8px;
          flex-wrap: wrap;
          align-items: center;
          font-size: 14px;
        }

        .meta-badge {
          padding: 4px 12px;
          border-radius: 12px;
          font-size: 12px;
          font-weight: 500;
        }

        .meta-badge.category {
          background: #dbeafe;
          color: #1e40af;
        }

        .meta-badge.status {
          background: #d1fae5;
          color: #065f46;
        }

        .meta-badge.severity {
          background: #fee2e2;
          color: #991b1b;
        }

        .meta-date {
          color: #9ca3af;
          font-size: 12px;
        }
      `}</style>
    </div>
  );
};

export default SearchResults;
