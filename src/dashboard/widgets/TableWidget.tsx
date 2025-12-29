/**
 * Table Widget Component
 *
 * Displays data in tabular format with sorting, filtering, and pagination
 */

import React, { useMemo, useState, useCallback } from 'react';
import { useWidget } from '../hooks/useDashboard';
import {
  ColumnDef,
  ColumnAlign,
  TableRow,
  SortConfig,
  PaginationConfig,
  TableConfig as TableConfigType,
} from '../types';

/**
 * Table widget props
 */
interface TableWidgetProps {
  widgetId: string;
}

/**
 * Table header cell component
 */
interface TableHeaderCellProps {
  column: ColumnDef;
  sortConfig?: SortConfig;
  onSort?: (columnId: string) => void;
}

const TableHeaderCell: React.FC<TableHeaderCellProps> = ({ column, sortConfig, onSort }) => {
  const isSorted = sortConfig?.columnId === column.id;

  const headerStyle: React.CSSProperties = {
    padding: '12px 16px',
    textAlign: column.align,
    fontWeight: 600,
    fontSize: 12,
    textTransform: 'uppercase',
    backgroundColor: '#f5f5f5',
    borderBottom: '2px solid #e0e0e0',
    cursor: column.sortable && onSort ? 'pointer' : 'default',
    userSelect: 'none',
    whiteSpace: 'nowrap',
    position: column.frozen ? 'sticky' : 'relative',
    left: column.frozen ? 0 : undefined,
    zIndex: column.frozen ? 2 : 1,
  };

  const handleClick = () => {
    if (column.sortable && onSort) {
      onSort(column.id);
    }
  };

  return (
    <th style={headerStyle} onClick={handleClick}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: column.align === 'right' ? 'flex-end' : column.align === 'center' ? 'center' : 'flex-start' }}>
        <span>{column.label}</span>
        {isSorted && (
          <span style={{ marginLeft: 4, fontSize: 10 }}>
            {sortConfig?.direction === 'asc' ? '▲' : '▼'}
          </span>
        )}
      </div>
    </th>
  );
};

/**
 * Table cell component
 */
interface TableCellProps {
  column: ColumnDef;
  value: any;
  row: TableRow;
}

const TableCell: React.FC<TableCellProps> = ({ column, value, row }) => {
  const cellStyle: React.CSSProperties = {
    padding: '12px 16px',
    textAlign: column.align,
    borderBottom: '1px solid #e0e0e0',
    fontSize: 14,
    position: column.frozen ? 'sticky' : 'relative',
    left: column.frozen ? 0 : undefined,
    backgroundColor: column.frozen ? '#ffffff' : undefined,
    zIndex: column.frozen ? 1 : 0,
  };

  const formattedValue = useMemo(() => {
    if (value === null || value === undefined) return '-';

    switch (column.dataType) {
      case 'number':
        return typeof value === 'number' ? value.toLocaleString() : value;
      case 'currency':
        return typeof value === 'number' ? `$${value.toFixed(2)}` : value;
      case 'percentage':
        return typeof value === 'number' ? `${value.toFixed(1)}%` : value;
      case 'date':
      case 'datetime':
        try {
          return new Date(value).toLocaleDateString();
        } catch {
          return value;
        }
      case 'boolean':
        return value ? '✓' : '✗';
      default:
        return String(value);
    }
  }, [value, column.dataType]);

  return <td style={cellStyle}>{formattedValue}</td>;
};

/**
 * Pagination component
 */
interface PaginationProps {
  pagination: PaginationConfig;
  onPageChange: (page: number) => void;
}

const Pagination: React.FC<PaginationProps> = ({ pagination, onPageChange }) => {
  const { page, totalPages, totalRows, pageSize } = pagination;

  const containerStyle: React.CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '12px 16px',
    borderTop: '1px solid #e0e0e0',
    fontSize: 12,
  };

  const buttonStyle: React.CSSProperties = {
    padding: '6px 12px',
    border: '1px solid #e0e0e0',
    backgroundColor: '#ffffff',
    borderRadius: 4,
    cursor: 'pointer',
    fontSize: 12,
    margin: '0 4px',
  };

  const disabledButtonStyle: React.CSSProperties = {
    ...buttonStyle,
    opacity: 0.5,
    cursor: 'not-allowed',
  };

  const startRow = page * pageSize + 1;
  const endRow = Math.min((page + 1) * pageSize, totalRows);

  return (
    <div style={containerStyle}>
      <div>
        Showing {startRow}-{endRow} of {totalRows} rows
      </div>

      <div style={{ display: 'flex', alignItems: 'center' }}>
        <button
          style={page === 0 ? disabledButtonStyle : buttonStyle}
          onClick={() => onPageChange(0)}
          disabled={page === 0}
        >
          ««
        </button>
        <button
          style={page === 0 ? disabledButtonStyle : buttonStyle}
          onClick={() => onPageChange(page - 1)}
          disabled={page === 0}
        >
          ‹
        </button>

        <span style={{ margin: '0 12px' }}>
          Page {page + 1} of {totalPages}
        </span>

        <button
          style={page >= totalPages - 1 ? disabledButtonStyle : buttonStyle}
          onClick={() => onPageChange(page + 1)}
          disabled={page >= totalPages - 1}
        >
          ›
        </button>
        <button
          style={page >= totalPages - 1 ? disabledButtonStyle : buttonStyle}
          onClick={() => onPageChange(totalPages - 1)}
          disabled={page >= totalPages - 1}
        >
          »»
        </button>
      </div>
    </div>
  );
};

/**
 * Table Widget Component
 */
const TableWidget: React.FC<TableWidgetProps> = ({ widgetId }) => {
  const { widget, isLoading, error } = useWidget(widgetId);
  const [currentPage, setCurrentPage] = useState(0);
  const [sortConfig, setSortConfig] = useState<SortConfig | undefined>();

  const tableData = useMemo(() => {
    if (!widget?.data?.data) return null;
    return widget.data.data as {
      columns: ColumnDef[];
      rows: TableRow[];
      pagination?: PaginationConfig;
      config: Partial<TableConfigType>;
    };
  }, [widget]);

  const visibleColumns = useMemo(() => {
    return tableData?.columns.filter((col) => col.visible) || [];
  }, [tableData]);

  const handleSort = useCallback((columnId: string) => {
    setSortConfig((prev) => {
      if (prev?.columnId === columnId) {
        return {
          columnId,
          direction: prev.direction === 'asc' ? 'desc' : 'asc',
        };
      }
      return { columnId, direction: 'asc' };
    });
  }, []);

  const sortedRows = useMemo(() => {
    if (!tableData?.rows || !sortConfig) return tableData?.rows || [];

    const column = tableData.columns.find((col) => col.id === sortConfig.columnId);
    if (!column) return tableData.rows;

    return [...tableData.rows].sort((a, b) => {
      const aValue = a.data[column.field];
      const bValue = b.data[column.field];

      let comparison = 0;
      if (aValue < bValue) comparison = -1;
      if (aValue > bValue) comparison = 1;

      return sortConfig.direction === 'asc' ? comparison : -comparison;
    });
  }, [tableData, sortConfig]);

  const containerStyle: React.CSSProperties = {
    width: '100%',
    height: '100%',
    backgroundColor: widget?.config.display.backgroundColor || '#ffffff',
    borderRadius: 8,
    border: widget?.config.display.showBorder ? '1px solid #e0e0e0' : 'none',
    boxSizing: 'border-box',
    overflow: 'hidden',
    display: 'flex',
    flexDirection: 'column',
  };

  const headerStyle: React.CSSProperties = {
    padding: widget?.config.display.padding || 16,
    paddingBottom: widget?.config.display.showTitle ? 12 : 0,
    borderBottom: widget?.config.display.showTitle ? '1px solid #e0e0e0' : 'none',
  };

  const titleStyle: React.CSSProperties = {
    fontSize: 18,
    fontWeight: 600,
    margin: 0,
    color: widget?.config.display.textColor || 'inherit',
  };

  const tableWrapperStyle: React.CSSProperties = {
    flex: 1,
    overflow: 'auto',
    position: 'relative',
  };

  const tableStyle: React.CSSProperties = {
    width: '100%',
    borderCollapse: 'collapse',
    fontSize: 14,
  };

  if (isLoading) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p>Loading table...</p>
      </div>
    );
  }

  if (error) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p style={{ color: '#f44336' }}>Error: {error}</p>
      </div>
    );
  }

  if (!tableData || !tableData.rows || tableData.rows.length === 0) {
    return (
      <div style={{ ...containerStyle, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
        <p style={{ opacity: 0.5 }}>No table data available</p>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      {widget?.config.display.showTitle && (
        <div style={headerStyle}>
          <h3 style={titleStyle}>{widget.config.metadata.title}</h3>
          {widget.config.metadata.description && (
            <p style={{ margin: '4px 0 0', fontSize: 12, opacity: 0.6 }}>
              {widget.config.metadata.description}
            </p>
          )}
        </div>
      )}

      <div style={tableWrapperStyle}>
        <table style={tableStyle}>
          <thead>
            <tr>
              {visibleColumns.map((column) => (
                <TableHeaderCell
                  key={column.id}
                  column={column}
                  sortConfig={sortConfig}
                  onSort={tableData.config.sortingEnabled !== false ? handleSort : undefined}
                />
              ))}
            </tr>
          </thead>
          <tbody>
            {sortedRows.map((row, rowIndex) => (
              <tr
                key={row.id}
                style={{
                  backgroundColor: tableData.config.striped && rowIndex % 2 === 1 ? '#fafafa' : '#ffffff',
                }}
              >
                {visibleColumns.map((column) => (
                  <TableCell
                    key={column.id}
                    column={column}
                    value={row.data[column.field]}
                    row={row}
                  />
                ))}
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {tableData.config.paginationEnabled !== false && tableData.pagination && (
        <Pagination
          pagination={tableData.pagination}
          onPageChange={setCurrentPage}
        />
      )}
    </div>
  );
};

export default TableWidget;
