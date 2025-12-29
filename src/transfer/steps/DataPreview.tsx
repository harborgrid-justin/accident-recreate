import React, { useState, useMemo } from 'react';
import { DataRecord } from '../types';

interface DataPreviewProps {
  state: any;
  mode: 'import' | 'export';
}

export const DataPreview: React.FC<DataPreviewProps> = ({ state, mode }) => {
  const [currentPage, setCurrentPage] = useState(0);
  const [pageSize, setPageSize] = useState(10);
  const [searchTerm, setSearchTerm] = useState('');

  const data = useMemo(() => {
    if (mode === 'export') {
      return state.data || [];
    } else {
      return state.previewData || state.importedData || [];
    }
  }, [mode, state.data, state.previewData, state.importedData]);

  // Get all field names
  const fieldNames = useMemo(() => {
    if (data.length === 0) return [];
    return Object.keys(data[0].fields || data[0] || {});
  }, [data]);

  // Filter data by search term
  const filteredData = useMemo(() => {
    if (!searchTerm) return data;

    return data.filter((record: DataRecord) => {
      const values = Object.values(record.fields || record);
      return values.some((value) =>
        String(value).toLowerCase().includes(searchTerm.toLowerCase())
      );
    });
  }, [data, searchTerm]);

  // Paginate data
  const paginatedData = useMemo(() => {
    const start = currentPage * pageSize;
    const end = start + pageSize;
    return filteredData.slice(start, end);
  }, [filteredData, currentPage, pageSize]);

  const totalPages = Math.ceil(filteredData.length / pageSize);

  const handlePageChange = (newPage: number) => {
    setCurrentPage(Math.max(0, Math.min(newPage, totalPages - 1)));
  };

  if (data.length === 0) {
    return (
      <div className="text-center py-12">
        <div className="mx-auto w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mb-4">
          <svg
            className="w-8 h-8 text-gray-400"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4"
            />
          </svg>
        </div>
        <p className="text-gray-600">No data available to preview</p>
      </div>
    );
  }

  return (
    <div className="data-preview">
      {/* Header Controls */}
      <div className="mb-4 flex items-center justify-between">
        <div className="flex items-center space-x-4">
          <div className="flex items-center space-x-2">
            <label className="text-sm text-gray-700">Show</label>
            <select
              value={pageSize}
              onChange={(e) => {
                setPageSize(Number(e.target.value));
                setCurrentPage(0);
              }}
              className="px-2 py-1 text-sm border border-gray-300 rounded-md"
            >
              <option value={10}>10</option>
              <option value={25}>25</option>
              <option value={50}>50</option>
              <option value={100}>100</option>
            </select>
            <label className="text-sm text-gray-700">rows</label>
          </div>
          <div className="text-sm text-gray-600">
            Total: {filteredData.length.toLocaleString()} records
          </div>
        </div>

        <div className="flex items-center space-x-2">
          <input
            type="text"
            placeholder="Search..."
            value={searchTerm}
            onChange={(e) => {
              setSearchTerm(e.target.value);
              setCurrentPage(0);
            }}
            className="px-3 py-1 text-sm border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
          />
        </div>
      </div>

      {/* Data Table */}
      <div className="overflow-x-auto border border-gray-200 rounded-lg">
        <table className="min-w-full divide-y divide-gray-200">
          <thead className="bg-gray-50">
            <tr>
              <th className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                #
              </th>
              {fieldNames.map((field) => (
                <th
                  key={field}
                  className="px-3 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider"
                >
                  {field}
                </th>
              ))}
            </tr>
          </thead>
          <tbody className="bg-white divide-y divide-gray-200">
            {paginatedData.map((record: any, idx: number) => (
              <tr key={idx} className="hover:bg-gray-50">
                <td className="px-3 py-2 whitespace-nowrap text-sm text-gray-500">
                  {currentPage * pageSize + idx + 1}
                </td>
                {fieldNames.map((field) => {
                  const value = (record.fields || record)[field];
                  return (
                    <td
                      key={field}
                      className="px-3 py-2 text-sm text-gray-900"
                      title={String(value)}
                    >
                      {formatValue(value)}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      {totalPages > 1 && (
        <div className="mt-4 flex items-center justify-between">
          <div className="text-sm text-gray-600">
            Showing {currentPage * pageSize + 1} to{' '}
            {Math.min((currentPage + 1) * pageSize, filteredData.length)} of{' '}
            {filteredData.length} records
          </div>
          <div className="flex items-center space-x-2">
            <button
              onClick={() => handlePageChange(0)}
              disabled={currentPage === 0}
              className="px-2 py-1 text-sm border border-gray-300 rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
            >
              First
            </button>
            <button
              onClick={() => handlePageChange(currentPage - 1)}
              disabled={currentPage === 0}
              className="px-2 py-1 text-sm border border-gray-300 rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
            >
              Previous
            </button>
            <span className="px-3 py-1 text-sm text-gray-700">
              Page {currentPage + 1} of {totalPages}
            </span>
            <button
              onClick={() => handlePageChange(currentPage + 1)}
              disabled={currentPage >= totalPages - 1}
              className="px-2 py-1 text-sm border border-gray-300 rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
            >
              Next
            </button>
            <button
              onClick={() => handlePageChange(totalPages - 1)}
              disabled={currentPage >= totalPages - 1}
              className="px-2 py-1 text-sm border border-gray-300 rounded-md disabled:opacity-50 disabled:cursor-not-allowed hover:bg-gray-50"
            >
              Last
            </button>
          </div>
        </div>
      )}

      {/* Statistics */}
      <div className="mt-6 grid grid-cols-2 md:grid-cols-4 gap-4">
        <div className="p-4 bg-blue-50 rounded-lg">
          <p className="text-sm text-blue-700">Records</p>
          <p className="text-2xl font-bold text-blue-900">
            {data.length.toLocaleString()}
          </p>
        </div>
        <div className="p-4 bg-green-50 rounded-lg">
          <p className="text-sm text-green-700">Fields</p>
          <p className="text-2xl font-bold text-green-900">{fieldNames.length}</p>
        </div>
        <div className="p-4 bg-purple-50 rounded-lg">
          <p className="text-sm text-purple-700">Format</p>
          <p className="text-2xl font-bold text-purple-900">
            {state.format?.toUpperCase() || 'N/A'}
          </p>
        </div>
        <div className="p-4 bg-orange-50 rounded-lg">
          <p className="text-sm text-orange-700">Est. Size</p>
          <p className="text-2xl font-bold text-orange-900">
            {estimateSize(data)}
          </p>
        </div>
      </div>
    </div>
  );
};

function formatValue(value: any): string {
  if (value === null || value === undefined) {
    return '-';
  }
  if (typeof value === 'object') {
    return JSON.stringify(value).substring(0, 50) + '...';
  }
  const str = String(value);
  return str.length > 50 ? str.substring(0, 47) + '...' : str;
}

function estimateSize(data: any[]): string {
  const jsonSize = JSON.stringify(data).length;
  if (jsonSize < 1024) {
    return jsonSize + ' B';
  } else if (jsonSize < 1024 * 1024) {
    return Math.round(jsonSize / 1024) + ' KB';
  } else {
    return Math.round(jsonSize / (1024 * 1024)) + ' MB';
  }
}
