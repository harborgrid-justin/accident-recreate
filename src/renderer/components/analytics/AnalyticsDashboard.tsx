/**
 * AccuScene Enterprise v0.3.0 - Analytics Dashboard
 * Main container component for the analytics dashboard with layout management
 */

import React, { useState, useCallback, useEffect, useMemo } from 'react';
import DashboardGrid from './DashboardGrid';
import { WidgetRegistry } from './WidgetRegistry';
import { DashboardSerializer } from './DashboardSerializer';
import { useAnalytics } from './hooks/useAnalytics';
import {
  DashboardLayout,
  WidgetConfig,
  WidgetType,
  DashboardTheme,
} from './types';

interface AnalyticsDashboardProps {
  caseId?: string;
  initialLayout?: DashboardLayout;
  readonly?: boolean;
  onLayoutChange?: (layout: DashboardLayout) => void;
  theme?: DashboardTheme;
  className?: string;
}

const DEFAULT_THEME: DashboardTheme = {
  primary: '#2563eb',
  secondary: '#64748b',
  background: '#0f172a',
  surface: '#1e293b',
  text: '#f1f5f9',
  textSecondary: '#94a3b8',
  border: '#334155',
  success: '#10b981',
  warning: '#f59e0b',
  error: '#ef4444',
  info: '#3b82f6',
};

const DEFAULT_LAYOUT: DashboardLayout = {
  id: 'default',
  name: 'Default Dashboard',
  description: 'Default analytics dashboard layout',
  widgets: [],
  gridSettings: {
    cols: 12,
    rowHeight: 80,
    margin: [16, 16],
    containerPadding: [16, 16],
    compactType: 'vertical',
    preventCollision: false,
    isDraggable: true,
    isResizable: true,
  },
  createdAt: Date.now(),
  updatedAt: Date.now(),
};

const AnalyticsDashboard: React.FC<AnalyticsDashboardProps> = ({
  caseId,
  initialLayout,
  readonly = false,
  onLayoutChange,
  theme = DEFAULT_THEME,
  className = '',
}) => {
  const [layout, setLayout] = useState<DashboardLayout>(
    initialLayout || DEFAULT_LAYOUT
  );
  const [isEditing, setIsEditing] = useState(false);
  const [selectedWidget, setSelectedWidget] = useState<string | null>(null);
  const [showWidgetPicker, setShowWidgetPicker] = useState(false);

  const { data, loading, error, refresh } = useAnalytics(caseId);

  // Save layout to localStorage on change
  useEffect(() => {
    if (!readonly && layout.id !== 'default') {
      DashboardSerializer.saveLayout(layout);
      onLayoutChange?.(layout);
    }
  }, [layout, readonly, onLayoutChange]);

  // Handle widget updates
  const handleWidgetUpdate = useCallback((config: WidgetConfig) => {
    setLayout((prev) => ({
      ...prev,
      widgets: prev.widgets.map((w) => (w.id === config.id ? config : w)),
      updatedAt: Date.now(),
    }));
  }, []);

  // Handle widget removal
  const handleWidgetRemove = useCallback((id: string) => {
    setLayout((prev) => ({
      ...prev,
      widgets: prev.widgets.filter((w) => w.id !== id),
      updatedAt: Date.now(),
    }));
  }, []);

  // Handle widget addition
  const handleWidgetAdd = useCallback((type: WidgetType) => {
    const widgetDef = WidgetRegistry.getWidget(type);
    if (!widgetDef) return;

    const newWidget: WidgetConfig = {
      id: `widget-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
      type,
      title: widgetDef.name,
      description: widgetDef.description,
      position: {
        x: 0,
        y: Infinity, // Add to bottom
        w: widgetDef.defaultSize.w,
        h: widgetDef.defaultSize.h,
      },
      settings: { ...widgetDef.defaultSettings },
      isVisible: true,
    };

    setLayout((prev) => ({
      ...prev,
      widgets: [...prev.widgets, newWidget],
      updatedAt: Date.now(),
    }));

    setShowWidgetPicker(false);
  }, []);

  // Handle layout reset
  const handleResetLayout = useCallback(() => {
    if (confirm('Are you sure you want to reset the dashboard layout?')) {
      setLayout({ ...DEFAULT_LAYOUT, id: layout.id, name: layout.name });
    }
  }, [layout.id, layout.name]);

  // Handle layout save
  const handleSaveLayout = useCallback(() => {
    const saved = DashboardSerializer.saveLayout(layout);
    if (saved) {
      alert('Dashboard layout saved successfully!');
    }
  }, [layout]);

  // Handle layout export
  const handleExportLayout = useCallback(() => {
    const json = DashboardSerializer.serialize(layout);
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `dashboard-${layout.name}-${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  }, [layout]);

  // Handle layout import
  const handleImportLayout = useCallback(
    (event: React.ChangeEvent<HTMLInputElement>) => {
      const file = event.target.files?.[0];
      if (!file) return;

      const reader = new FileReader();
      reader.onload = (e) => {
        try {
          const imported = DashboardSerializer.deserialize(
            e.target?.result as string
          );
          setLayout(imported);
        } catch (err) {
          alert('Failed to import layout: Invalid file format');
        }
      };
      reader.readAsText(file);
    },
    []
  );

  // Available widgets for the picker
  const availableWidgets = useMemo(() => WidgetRegistry.getAllWidgets(), []);

  // Render error state
  if (error) {
    return (
      <div
        className={`analytics-dashboard-error ${className}`}
        style={{
          padding: '2rem',
          textAlign: 'center',
          color: theme.error,
          backgroundColor: theme.background,
        }}
      >
        <h2>Error loading analytics data</h2>
        <p>{error.message}</p>
        <button
          onClick={refresh}
          style={{
            marginTop: '1rem',
            padding: '0.5rem 1rem',
            backgroundColor: theme.primary,
            color: theme.text,
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          Retry
        </button>
      </div>
    );
  }

  return (
    <div
      className={`analytics-dashboard ${className}`}
      style={{
        width: '100%',
        height: '100%',
        backgroundColor: theme.background,
        color: theme.text,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
      }}
    >
      {/* Toolbar */}
      {!readonly && (
        <div
          className="dashboard-toolbar"
          style={{
            padding: '1rem',
            backgroundColor: theme.surface,
            borderBottom: `1px solid ${theme.border}`,
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            flexShrink: 0,
          }}
        >
          <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
            <h2 style={{ margin: 0, fontSize: '1.25rem', fontWeight: 600 }}>
              {layout.name}
            </h2>
            {loading && (
              <span
                style={{
                  fontSize: '0.875rem',
                  color: theme.textSecondary,
                }}
              >
                Loading...
              </span>
            )}
          </div>

          <div style={{ display: 'flex', gap: '0.5rem' }}>
            <button
              onClick={() => setShowWidgetPicker(!showWidgetPicker)}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.primary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              + Add Widget
            </button>

            <button
              onClick={() => setIsEditing(!isEditing)}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: isEditing ? theme.warning : theme.secondary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              {isEditing ? 'Done' : 'Edit'}
            </button>

            <button
              onClick={handleSaveLayout}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.secondary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              Save
            </button>

            <button
              onClick={handleExportLayout}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.secondary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              Export
            </button>

            <label
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.secondary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              Import
              <input
                type="file"
                accept=".json"
                onChange={handleImportLayout}
                style={{ display: 'none' }}
              />
            </label>

            <button
              onClick={handleResetLayout}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.error,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              Reset
            </button>

            <button
              onClick={refresh}
              style={{
                padding: '0.5rem 1rem',
                backgroundColor: theme.secondary,
                color: theme.text,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
                fontWeight: 500,
              }}
            >
              ↻ Refresh
            </button>
          </div>
        </div>
      )}

      {/* Widget Picker */}
      {showWidgetPicker && (
        <div
          className="widget-picker"
          style={{
            padding: '1rem',
            backgroundColor: theme.surface,
            borderBottom: `1px solid ${theme.border}`,
            maxHeight: '300px',
            overflowY: 'auto',
            flexShrink: 0,
          }}
        >
          <h3 style={{ margin: '0 0 1rem 0', fontSize: '1rem' }}>
            Add Widget
          </h3>
          <div
            style={{
              display: 'grid',
              gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
              gap: '1rem',
            }}
          >
            {availableWidgets.map((widget) => (
              <div
                key={widget.type}
                onClick={() => handleWidgetAdd(widget.type)}
                style={{
                  padding: '1rem',
                  backgroundColor: theme.background,
                  border: `1px solid ${theme.border}`,
                  borderRadius: '4px',
                  cursor: 'pointer',
                  transition: 'all 0.2s',
                }}
                onMouseEnter={(e) => {
                  e.currentTarget.style.borderColor = theme.primary;
                  e.currentTarget.style.transform = 'translateY(-2px)';
                }}
                onMouseLeave={(e) => {
                  e.currentTarget.style.borderColor = theme.border;
                  e.currentTarget.style.transform = 'translateY(0)';
                }}
              >
                <div style={{ fontSize: '1.5rem', marginBottom: '0.5rem' }}>
                  {widget.icon}
                </div>
                <div style={{ fontWeight: 500, marginBottom: '0.25rem' }}>
                  {widget.name}
                </div>
                <div
                  style={{ fontSize: '0.75rem', color: theme.textSecondary }}
                >
                  {widget.description}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Dashboard Grid */}
      <div
        className="dashboard-content"
        style={{
          flex: 1,
          overflow: 'auto',
          padding: `${layout.gridSettings.containerPadding[1]}px ${layout.gridSettings.containerPadding[0]}px`,
        }}
      >
        <DashboardGrid
          layout={layout}
          data={data}
          isEditing={isEditing}
          onWidgetUpdate={handleWidgetUpdate}
          onWidgetRemove={handleWidgetRemove}
          theme={theme}
        />
      </div>
    </div>
  );
};

export default AnalyticsDashboard;
