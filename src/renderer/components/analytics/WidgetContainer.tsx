/**
 * AccuScene Enterprise v0.3.0 - Widget Container
 * Generic widget wrapper with resize, drag, and remove capabilities
 */

import React, { CSSProperties, useMemo } from 'react';
import { WidgetRegistry } from './WidgetRegistry';
import {
  WidgetConfig,
  AnalyticsData,
  DashboardTheme,
} from './types';

interface WidgetContainerProps {
  config: WidgetConfig;
  data: AnalyticsData | null;
  isEditing?: boolean;
  onUpdate?: (config: WidgetConfig) => void;
  onRemove?: (id: string) => void;
  onDragStart?: (event: React.MouseEvent) => void;
  onResizeStart?: (event: React.MouseEvent) => void;
  style?: CSSProperties;
  theme: DashboardTheme;
}

const WidgetContainer: React.FC<WidgetContainerProps> = ({
  config,
  data,
  isEditing = false,
  onUpdate,
  onRemove,
  onDragStart,
  onResizeStart,
  style,
  theme,
}) => {
  // Get widget component from registry
  const widgetDef = useMemo(
    () => WidgetRegistry.getWidget(config.type),
    [config.type]
  );

  if (!widgetDef) {
    return (
      <div
        style={{
          ...style,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          backgroundColor: theme.surface,
          border: `1px solid ${theme.border}`,
          borderRadius: '8px',
          color: theme.error,
        }}
      >
        <div style={{ textAlign: 'center' }}>
          <div style={{ fontSize: '2rem', marginBottom: '0.5rem' }}>⚠️</div>
          <div>Widget type &quot;{config.type}&quot; not found</div>
        </div>
      </div>
    );
  }

  const WidgetComponent = widgetDef.component;

  return (
    <div
      className={`widget-container ${isEditing ? 'editing' : ''}`}
      style={{
        ...style,
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: theme.surface,
        border: `1px solid ${theme.border}`,
        borderRadius: '8px',
        overflow: 'hidden',
        boxShadow: '0 4px 6px rgba(0, 0, 0, 0.1)',
        transition: 'box-shadow 0.2s',
      }}
      onMouseEnter={(e) => {
        if (isEditing) {
          e.currentTarget.style.boxShadow = `0 0 0 2px ${theme.primary}`;
        }
      }}
      onMouseLeave={(e) => {
        if (isEditing) {
          e.currentTarget.style.boxShadow = '0 4px 6px rgba(0, 0, 0, 0.1)';
        }
      }}
    >
      {/* Widget Header */}
      <div
        className="widget-header"
        style={{
          padding: '0.75rem 1rem',
          backgroundColor: theme.background,
          borderBottom: `1px solid ${theme.border}`,
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          cursor: isEditing ? 'move' : 'default',
          userSelect: 'none',
        }}
        onMouseDown={isEditing ? onDragStart : undefined}
      >
        <div style={{ flex: 1 }}>
          <h3
            style={{
              margin: 0,
              fontSize: '0.875rem',
              fontWeight: 600,
              color: theme.text,
            }}
          >
            {config.title}
          </h3>
          {config.description && (
            <p
              style={{
                margin: '0.25rem 0 0 0',
                fontSize: '0.75rem',
                color: theme.textSecondary,
              }}
            >
              {config.description}
            </p>
          )}
        </div>

        <div
          style={{
            display: 'flex',
            gap: '0.5rem',
            alignItems: 'center',
          }}
        >
          {/* Settings Button */}
          {isEditing && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                // TODO: Open settings modal
                alert('Widget settings coming soon!');
              }}
              style={{
                padding: '0.25rem 0.5rem',
                backgroundColor: 'transparent',
                color: theme.textSecondary,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = theme.border;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              ⚙️
            </button>
          )}

          {/* Visibility Toggle */}
          {isEditing && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                onUpdate?.({ ...config, isVisible: !config.isVisible });
              }}
              style={{
                padding: '0.25rem 0.5rem',
                backgroundColor: 'transparent',
                color: theme.textSecondary,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = theme.border;
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              {config.isVisible ? '👁️' : '👁️‍🗨️'}
            </button>
          )}

          {/* Remove Button */}
          {isEditing && onRemove && (
            <button
              onClick={(e) => {
                e.stopPropagation();
                if (confirm(`Remove widget "${config.title}"?`)) {
                  onRemove(config.id);
                }
              }}
              style={{
                padding: '0.25rem 0.5rem',
                backgroundColor: 'transparent',
                color: theme.error,
                border: 'none',
                borderRadius: '4px',
                cursor: 'pointer',
                fontSize: '0.875rem',
              }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = theme.error + '20';
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent';
              }}
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {/* Widget Content */}
      <div
        className="widget-content"
        style={{
          flex: 1,
          padding: '1rem',
          overflow: 'auto',
          position: 'relative',
        }}
      >
        <WidgetComponent
          config={config}
          data={data}
          onUpdate={onUpdate}
          onRemove={onRemove}
          isEditing={isEditing}
        />
      </div>

      {/* Resize Handle */}
      {isEditing && onResizeStart && (
        <div
          className="resize-handle"
          style={{
            position: 'absolute',
            bottom: 0,
            right: 0,
            width: '20px',
            height: '20px',
            cursor: 'nwse-resize',
            background: `linear-gradient(135deg, transparent 50%, ${theme.primary} 50%)`,
            borderRadius: '0 0 8px 0',
          }}
          onMouseDown={onResizeStart}
        />
      )}

      {/* Loading Overlay */}
      {!data && (
        <div
          style={{
            position: 'absolute',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            backgroundColor: theme.surface + 'CC',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            backdropFilter: 'blur(4px)',
          }}
        >
          <div style={{ textAlign: 'center' }}>
            <div
              style={{
                width: '40px',
                height: '40px',
                border: `4px solid ${theme.border}`,
                borderTop: `4px solid ${theme.primary}`,
                borderRadius: '50%',
                animation: 'spin 1s linear infinite',
                margin: '0 auto 0.5rem',
              }}
            />
            <div style={{ color: theme.textSecondary, fontSize: '0.875rem' }}>
              Loading data...
            </div>
          </div>
        </div>
      )}

      {/* Add keyframes for loading spinner */}
      <style>
        {`
          @keyframes spin {
            0% { transform: rotate(0deg); }
            100% { transform: rotate(360deg); }
          }
        `}
      </style>
    </div>
  );
};

export default WidgetContainer;
