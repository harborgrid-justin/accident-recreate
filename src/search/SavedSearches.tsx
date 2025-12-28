/**
 * Saved Searches Management Component
 */

import React, { useState, useEffect } from 'react';
import type { SavedSearch, SearchQuery, SearchFilters } from './types';
import { SearchService } from './services/SearchService';

export interface SavedSearchesProps {
  onApplySearch: (query: SearchQuery, filters?: SearchFilters) => void;
  service?: SearchService;
  className?: string;
}

const SavedSearches: React.FC<SavedSearchesProps> = ({
  onApplySearch,
  service = new SearchService(),
  className = '',
}) => {
  const [searches, setSearches] = useState<SavedSearch[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  const [showSaveDialog, setShowSaveDialog] = useState(false);
  const [newSearchName, setNewSearchName] = useState('');

  // Load saved searches
  useEffect(() => {
    loadSearches();
  }, []);

  const loadSearches = async () => {
    try {
      setLoading(true);
      const data = await service.getSavedSearches();
      setSearches(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to load searches'));
    } finally {
      setLoading(false);
    }
  };

  const handleApply = (search: SavedSearch) => {
    onApplySearch(search.query, search.filters);
  };

  const handleDelete = async (id: string) => {
    if (!confirm('Are you sure you want to delete this saved search?')) {
      return;
    }

    try {
      await service.deleteSavedSearch(id);
      setSearches((prev) => prev.filter((s) => s.id !== id));
    } catch (err) {
      alert('Failed to delete search');
    }
  };

  const handleSave = async () => {
    if (!newSearchName.trim()) {
      alert('Please enter a name for the search');
      return;
    }

    try {
      const saved = await service.saveSearch(
        newSearchName,
        { text: '' }, // Current query would come from parent
        undefined // Current filters would come from parent
      );

      setSearches((prev) => [...prev, saved]);
      setNewSearchName('');
      setShowSaveDialog(false);
    } catch (err) {
      alert('Failed to save search');
    }
  };

  if (loading) {
    return (
      <div className={`saved-searches loading ${className}`}>
        <div className="spinner"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className={`saved-searches error ${className}`}>
        <p>Failed to load saved searches</p>
      </div>
    );
  }

  return (
    <div className={`saved-searches ${className}`}>
      <div className="header">
        <h3>Saved Searches</h3>
        <button
          onClick={() => setShowSaveDialog(true)}
          className="save-new-btn"
        >
          + Save Current
        </button>
      </div>

      {searches.length === 0 ? (
        <div className="empty-state">
          <svg width="48" height="48" viewBox="0 0 24 24" fill="none">
            <path
              d="M5 3v18l7-5 7 5V3H5z"
              stroke="#9ca3af"
              strokeWidth="2"
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
          <p>No saved searches yet</p>
          <button
            onClick={() => setShowSaveDialog(true)}
            className="empty-action"
          >
            Save your first search
          </button>
        </div>
      ) : (
        <div className="searches-list">
          {searches.map((search) => (
            <div key={search.id} className="search-item">
              <div className="search-info">
                <h4 className="search-name">{search.name}</h4>
                <p className="search-query">{search.query.text || 'All results'}</p>
                <span className="search-date">
                  {new Date(search.createdAt).toLocaleDateString()}
                </span>
              </div>

              <div className="search-actions">
                <button
                  onClick={() => handleApply(search)}
                  className="apply-btn"
                  title="Apply search"
                >
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M8 5a3 3 0 11-6 0 3 3 0 016 0zm6 0a3 3 0 11-6 0 3 3 0 016 0zm6 0a3 3 0 11-6 0 3 3 0 016 0z" />
                  </svg>
                </button>

                <button
                  onClick={() => handleDelete(search.id)}
                  className="delete-btn"
                  title="Delete search"
                >
                  <svg width="20" height="20" viewBox="0 0 20 20" fill="currentColor">
                    <path
                      fillRule="evenodd"
                      d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z"
                      clipRule="evenodd"
                    />
                  </svg>
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Save Dialog */}
      {showSaveDialog && (
        <div className="dialog-overlay" onClick={() => setShowSaveDialog(false)}>
          <div className="dialog" onClick={(e) => e.stopPropagation()}>
            <h3>Save Search</h3>
            <input
              type="text"
              value={newSearchName}
              onChange={(e) => setNewSearchName(e.target.value)}
              placeholder="Enter search name..."
              className="name-input"
              autoFocus
            />
            <div className="dialog-actions">
              <button
                onClick={() => setShowSaveDialog(false)}
                className="cancel-btn"
              >
                Cancel
              </button>
              <button onClick={handleSave} className="save-btn">
                Save
              </button>
            </div>
          </div>
        </div>
      )}

      <style jsx>{`
        .saved-searches {
          width: 100%;
          background: white;
          border: 1px solid #e5e7eb;
          border-radius: 8px;
          overflow: hidden;
        }

        .header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px;
          border-bottom: 1px solid #e5e7eb;
        }

        .header h3 {
          margin: 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .save-new-btn {
          padding: 8px 16px;
          border: none;
          background: #3b82f6;
          color: white;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: background-color 0.2s;
        }

        .save-new-btn:hover {
          background: #2563eb;
        }

        .loading,
        .error {
          display: flex;
          justify-content: center;
          align-items: center;
          padding: 48px;
        }

        .spinner {
          width: 32px;
          height: 32px;
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

        .empty-state {
          display: flex;
          flex-direction: column;
          align-items: center;
          justify-content: center;
          padding: 48px 24px;
          color: #9ca3af;
        }

        .empty-state p {
          margin: 16px 0;
          font-size: 14px;
        }

        .empty-action {
          padding: 8px 16px;
          border: 1px solid #e5e7eb;
          background: white;
          color: #374151;
          border-radius: 6px;
          font-size: 14px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .empty-action:hover {
          border-color: #3b82f6;
          color: #3b82f6;
        }

        .searches-list {
          display: flex;
          flex-direction: column;
        }

        .search-item {
          display: flex;
          justify-content: space-between;
          align-items: center;
          padding: 16px;
          border-bottom: 1px solid #e5e7eb;
          transition: background-color 0.2s;
        }

        .search-item:last-child {
          border-bottom: none;
        }

        .search-item:hover {
          background: #f9fafb;
        }

        .search-info {
          flex: 1;
        }

        .search-name {
          margin: 0 0 4px 0;
          font-size: 16px;
          font-weight: 600;
          color: #111827;
        }

        .search-query {
          margin: 0 0 4px 0;
          font-size: 14px;
          color: #6b7280;
        }

        .search-date {
          font-size: 12px;
          color: #9ca3af;
        }

        .search-actions {
          display: flex;
          gap: 8px;
        }

        .apply-btn,
        .delete-btn {
          padding: 8px;
          border: none;
          background: transparent;
          color: #6b7280;
          border-radius: 6px;
          cursor: pointer;
          transition: all 0.2s;
        }

        .apply-btn:hover {
          background: #dbeafe;
          color: #3b82f6;
        }

        .delete-btn:hover {
          background: #fee2e2;
          color: #ef4444;
        }

        .dialog-overlay {
          position: fixed;
          top: 0;
          left: 0;
          right: 0;
          bottom: 0;
          background: rgba(0, 0, 0, 0.5);
          display: flex;
          justify-content: center;
          align-items: center;
          z-index: 1000;
        }

        .dialog {
          background: white;
          border-radius: 12px;
          padding: 24px;
          width: 90%;
          max-width: 400px;
          box-shadow: 0 20px 25px -5px rgba(0, 0, 0, 0.1);
        }

        .dialog h3 {
          margin: 0 0 16px 0;
          font-size: 18px;
          font-weight: 600;
          color: #111827;
        }

        .name-input {
          width: 100%;
          padding: 12px;
          border: 1px solid #e5e7eb;
          border-radius: 6px;
          font-size: 14px;
          outline: none;
          margin-bottom: 16px;
        }

        .name-input:focus {
          border-color: #3b82f6;
          box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.1);
        }

        .dialog-actions {
          display: flex;
          gap: 8px;
          justify-content: flex-end;
        }

        .cancel-btn,
        .save-btn {
          padding: 8px 16px;
          border: none;
          border-radius: 6px;
          font-size: 14px;
          font-weight: 500;
          cursor: pointer;
          transition: all 0.2s;
        }

        .cancel-btn {
          background: #f3f4f6;
          color: #374151;
        }

        .cancel-btn:hover {
          background: #e5e7eb;
        }

        .save-btn {
          background: #3b82f6;
          color: white;
        }

        .save-btn:hover {
          background: #2563eb;
        }
      `}</style>
    </div>
  );
};

export default SavedSearches;
