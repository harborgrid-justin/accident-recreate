/**
 * AccuScene Enterprise v0.3.0 - Data Table Widget
 * Sortable, filterable data table with export capabilities
 */

import React, { useState, useMemo } from 'react';
import { WidgetProps, AnalyticsData, TableColumn } from '../types';

interface TableData {
  id: string;
  [key: string]: any;
}

const DataTable: React.FC<WidgetProps<AnalyticsData>> = ({ config, data }) => {
  const [sortColumn, setSortColumn] = useState<string>('');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [filterText, setFilterText] = useState('');
  const [currentPage, setCurrentPage] = useState(1);
  const [pageSize, setPageSize] = useState(10);

  // Transform analytics data to table format
  const tableData = useMemo((): TableData[] => {
    if (!data) return [];

    // Example: Transform impacts into table rows
    if (data.impacts && data.impacts.length > 0) {
      return data.impacts.map((impact) => ({
        id: impact.id,
        timestamp: impact.timestamp,
        type: impact.type,
        location: `(${impact.location.x.toFixed(1)}, ${impact.location.y.toFixed(1)}, ${impact.location.z.toFixed(1)})`,
        force: Math.sqrt(
          impact.force.x ** 2 + impact.force.y ** 2 + impact.force.z ** 2
        ),
        energy: impact.energy,
        severity: impact.severity,
        duration: impact.duration,
      }));
    }

    return [];
  }, [data]);

  // Define columns
  const columns = useMemo((): TableColumn<TableData>[] => {
    return [
      {
        id: 'timestamp',
        header: 'Time (s)',
        accessor: 'timestamp',
        sortable: true,
        width: 100,
        format: (value) => value.toFixed(2),
        align: 'right',
      },
      {
        id: 'type',
        header: 'Type',
        accessor: 'type',
        sortable: true,
        width: 120,
        format: (value) => value.charAt(0).toUpperCase() + value.slice(1),
      },
      {
        id: 'location',
        header: 'Location',
        accessor: 'location',
        width: 150,
      },
      {
        id: 'force',
        header: 'Force (N)',
        accessor: 'force',
        sortable: true,
        width: 120,
        format: (value) => value.toFixed(0),
        align: 'right',
      },
      {
        id: 'energy',
        header: 'Energy (J)',
        accessor: 'energy',
        sortable: true,
        width: 120,
        format: (value) => value.toFixed(0),
        align: 'right',
      },
      {
        id: 'severity',
        header: 'Severity',
        accessor: 'severity',
        sortable: true,
        width: 100,
        format: (value, row) => {
          const percent = (value * 100).toFixed(0);
          const color =
            value > 0.7 ? '#ef4444' : value > 0.4 ? '#f59e0b' : '#10b981';
          return (
            <span style={{ color, fontWeight: 600 }}>
              {percent}%
            </span>
          );
        },
        align: 'center',
      },
      {
        id: 'duration',
        header: 'Duration (s)',
        accessor: 'duration',
        sortable: true,
        width: 120,
        format: (value) => value.toFixed(3),
        align: 'right',
      },
    ];
  }, []);

  // Filter data
  const filteredData = useMemo(() => {
    if (!filterText) return tableData;

    return tableData.filter((row) =>
      Object.values(row).some((value) =>
        String(value).toLowerCase().includes(filterText.toLowerCase())
      )
    );
  }, [tableData, filterText]);

  // Sort data
  const sortedData = useMemo(() => {
    if (!sortColumn) return filteredData;

    return [...filteredData].sort((a, b) => {
      const aVal = a[sortColumn];
      const bVal = b[sortColumn];

      if (aVal < bVal) return sortOrder === 'asc' ? -1 : 1;
      if (aVal > bVal) return sortOrder === 'asc' ? 1 : -1;
      return 0;
    });
  }, [filteredData, sortColumn, sortOrder]);

  // Paginate data
  const paginatedData = useMemo(() => {
    const start = (currentPage - 1) * pageSize;
    const end = start + pageSize;
    return sortedData.slice(start, end);
  }, [sortedData, currentPage, pageSize]);

  const totalPages = Math.ceil(sortedData.length / pageSize);

  // Handle sort
  const handleSort = (columnId: string) => {
    if (sortColumn === columnId) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortColumn(columnId);
      setSortOrder('asc');
    }
  };

  if (tableData.length === 0) {
    return (
      <div
        style={{
          width: '100%',
          height: '100%',
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          color: '#94a3b8',
        }}
      >
        No data available
      </div>
    );
  }

  return (
    <div
      style={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        gap: '0.75rem',
      }}
    >
      {/* Controls */}
      <div style={{ display: 'flex', justifyContent: 'space-between', gap: '0.5rem' }}>
        <input
          type="text"
          placeholder="Filter data..."
          value={filterText}
          onChange={(e) => setFilterText(e.target.value)}
          style={{
            flex: 1,
            padding: '0.5rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            fontSize: '0.875rem',
          }}
        />
        <select
          value={pageSize}
          onChange={(e) => {
            setPageSize(parseInt(e.target.value));
            setCurrentPage(1);
          }}
          style={{
            padding: '0.5rem',
            backgroundColor: '#1e293b',
            color: '#f1f5f9',
            border: '1px solid #334155',
            borderRadius: '4px',
            fontSize: '0.875rem',
          }}
        >
          <option value="10">10 rows</option>
          <option value="25">25 rows</option>
          <option value="50">50 rows</option>
          <option value="100">100 rows</option>
        </select>
      </div>

      {/* Table */}
      <div
        style={{
          flex: 1,
          overflow: 'auto',
          backgroundColor: '#1e293b',
          borderRadius: '4px',
          border: '1px solid #334155',
        }}
      >
        <table
          style={{
            width: '100%',
            borderCollapse: 'collapse',
            fontSize: '0.875rem',
          }}
        >
          <thead
            style={{
              position: 'sticky',
              top: 0,
              backgroundColor: '#0f172a',
              zIndex: 1,
            }}
          >
            <tr>
              {columns.map((column) => (
                <th
                  key={column.id}
                  onClick={() => column.sortable && handleSort(column.id)}
                  style={{
                    padding: '0.75rem',
                    textAlign: column.align || 'left',
                    color: '#94a3b8',
                    fontWeight: 600,
                    borderBottom: '2px solid #334155',
                    cursor: column.sortable ? 'pointer' : 'default',
                    userSelect: 'none',
                    whiteSpace: 'nowrap',
                    width: column.width,
                  }}
                >
                  {column.header}
                  {column.sortable && sortColumn === column.id && (
                    <span style={{ marginLeft: '0.5rem' }}>
                      {sortOrder === 'asc' ? '↑' : '↓'}
                    </span>
                  )}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {paginatedData.map((row, rowIndex) => (
              <tr
                key={row.id}
                style={{
                  backgroundColor: rowIndex % 2 === 0 ? '#1e293b' : '#0f172a',
                  transition: 'background-color 0.15s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.backgroundColor = '#334155';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.backgroundColor =
                    rowIndex % 2 === 0 ? '#1e293b' : '#0f172a';
                }}
              >
                {columns.map((column) => {
                  const value = typeof column.accessor === 'function'
                    ? column.accessor(row)
                    : row[column.accessor as string];

                  const formatted = column.format
                    ? column.format(value, row)
                    : value;

                  return (
                    <td
                      key={column.id}
                      style={{
                        padding: '0.75rem',
                        color: '#f1f5f9',
                        borderBottom: '1px solid #334155',
                        textAlign: column.align || 'left',
                      }}
                    >
                      {formatted}
                    </td>
                  );
                })}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {/* Pagination */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          fontSize: '0.875rem',
          color: '#94a3b8',
        }}
      >
        <div>
          Showing {((currentPage - 1) * pageSize) + 1} to{' '}
          {Math.min(currentPage * pageSize, sortedData.length)} of{' '}
          {sortedData.length} entries
          {filterText && ` (filtered from ${tableData.length})`}
        </div>
        <div style={{ display: 'flex', gap: '0.5rem' }}>
          <button
            onClick={() => setCurrentPage(1)}
            disabled={currentPage === 1}
            style={{
              padding: '0.25rem 0.75rem',
              backgroundColor: currentPage === 1 ? '#1e293b' : '#334155',
              color: currentPage === 1 ? '#64748b' : '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: currentPage === 1 ? 'default' : 'pointer',
              fontSize: '0.875rem',
            }}
          >
            First
          </button>
          <button
            onClick={() => setCurrentPage(currentPage - 1)}
            disabled={currentPage === 1}
            style={{
              padding: '0.25rem 0.75rem',
              backgroundColor: currentPage === 1 ? '#1e293b' : '#334155',
              color: currentPage === 1 ? '#64748b' : '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: currentPage === 1 ? 'default' : 'pointer',
              fontSize: '0.875rem',
            }}
          >
            Previous
          </button>
          <span
            style={{
              padding: '0.25rem 0.75rem',
              backgroundColor: '#1e293b',
              border: '1px solid #334155',
              borderRadius: '4px',
            }}
          >
            {currentPage} / {totalPages}
          </span>
          <button
            onClick={() => setCurrentPage(currentPage + 1)}
            disabled={currentPage === totalPages}
            style={{
              padding: '0.25rem 0.75rem',
              backgroundColor: currentPage === totalPages ? '#1e293b' : '#334155',
              color: currentPage === totalPages ? '#64748b' : '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: currentPage === totalPages ? 'default' : 'pointer',
              fontSize: '0.875rem',
            }}
          >
            Next
          </button>
          <button
            onClick={() => setCurrentPage(totalPages)}
            disabled={currentPage === totalPages}
            style={{
              padding: '0.25rem 0.75rem',
              backgroundColor: currentPage === totalPages ? '#1e293b' : '#334155',
              color: currentPage === totalPages ? '#64748b' : '#f1f5f9',
              border: '1px solid #334155',
              borderRadius: '4px',
              cursor: currentPage === totalPages ? 'default' : 'pointer',
              fontSize: '0.875rem',
            }}
          >
            Last
          </button>
        </div>
      </div>
    </div>
  );
};

export default DataTable;
