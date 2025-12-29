/**
 * AccuScene Enterprise v0.3.0
 * Mobile Object List Component
 *
 * Virtualized list with swipe actions for mobile
 */

import React, { useState, useRef, useCallback, CSSProperties } from 'react';
import { ListItem, SwipeAction } from './types';
import { SwipeActions } from './SwipeActions';
import { HapticFeedback } from './HapticFeedback';

export interface MobileObjectListProps {
  items: ListItem[];
  onItemClick?: (item: ListItem) => void;
  onItemSelect?: (item: ListItem) => void;
  leftActions?: SwipeAction[];
  rightActions?: SwipeAction[];
  multiSelect?: boolean;
  showThumbnails?: boolean;
  emptyMessage?: string;
  className?: string;
}

/**
 * Virtualized object list with swipe actions
 * Optimized for large lists on mobile devices
 *
 * @example
 * ```tsx
 * <MobileObjectList
 *   items={objects}
 *   onItemClick={(item) => navigate(`/object/${item.id}`)}
 *   rightActions={[
 *     { id: 'delete', label: 'Delete', backgroundColor: '#FF3B30', onAction: handleDelete },
 *   ]}
 *   multiSelect
 * />
 * ```
 */
export const MobileObjectList: React.FC<MobileObjectListProps> = ({
  items,
  onItemClick,
  onItemSelect,
  leftActions,
  rightActions,
  multiSelect = false,
  showThumbnails = true,
  emptyMessage = 'No items found',
  className = '',
}) => {
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  const listRef = useRef<HTMLDivElement>(null);

  const handleItemClick = useCallback(
    (item: ListItem, e: React.MouseEvent | React.TouchEvent) => {
      if (item.disabled) return;

      HapticFeedback.selection();

      if (multiSelect) {
        setSelectedIds((prev) => {
          const next = new Set(prev);
          if (next.has(item.id)) {
            next.delete(item.id);
          } else {
            next.add(item.id);
          }
          return next;
        });
        onItemSelect?.(item);
      } else {
        onItemClick?.(item);
      }
    },
    [multiSelect, onItemClick, onItemSelect]
  );

  const isSelected = (id: string): boolean => {
    return selectedIds.has(id);
  };

  const containerStyles: CSSProperties = {
    width: '100%',
    height: '100%',
    overflowY: 'auto',
    overflowX: 'hidden',
    WebkitOverflowScrolling: 'touch',
  };

  const listItemStyles = (item: ListItem): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    padding: '0.875rem 1rem',
    backgroundColor: isSelected(item.id) ? '#E3F2FD' : '#ffffff',
    borderBottom: '1px solid #e0e0e0',
    cursor: item.disabled ? 'not-allowed' : 'pointer',
    opacity: item.disabled ? 0.5 : 1,
    transition: 'background-color 0.2s ease',
    minHeight: '60px',
  });

  const thumbnailStyles: CSSProperties = {
    width: '48px',
    height: '48px',
    borderRadius: '8px',
    objectFit: 'cover',
    flexShrink: 0,
    backgroundColor: '#f0f0f0',
  };

  const iconContainerStyles: CSSProperties = {
    width: '48px',
    height: '48px',
    borderRadius: '8px',
    backgroundColor: '#f0f0f0',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    fontSize: '24px',
  };

  const contentStyles: CSSProperties = {
    flex: 1,
    minWidth: 0,
  };

  const titleStyles: CSSProperties = {
    fontSize: '1rem',
    fontWeight: 500,
    marginBottom: '0.25rem',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  };

  const subtitleStyles: CSSProperties = {
    fontSize: '0.875rem',
    color: '#8E8E93',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  };

  const checkboxStyles: CSSProperties = {
    width: '24px',
    height: '24px',
    borderRadius: '50%',
    border: '2px solid #007AFF',
    backgroundColor: isSelected ? '#007AFF' : 'transparent',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    color: '#ffffff',
    fontSize: '14px',
    flexShrink: 0,
  };

  const emptyStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '200px',
    fontSize: '1rem',
    color: '#8E8E93',
    textAlign: 'center',
    padding: '2rem',
  };

  const renderListItem = (item: ListItem) => {
    const itemContent = (
      <div
        className="mobile-object-list__item"
        style={listItemStyles(item)}
        onClick={(e) => handleItemClick(item, e)}
        role="button"
        tabIndex={item.disabled ? -1 : 0}
        aria-disabled={item.disabled}
        aria-selected={isSelected(item.id)}
      >
        {/* Thumbnail or icon */}
        {showThumbnails && (
          <>
            {item.thumbnail ? (
              <img
                src={item.thumbnail}
                alt={item.title}
                style={thumbnailStyles}
                loading="lazy"
              />
            ) : item.icon ? (
              <div className="mobile-object-list__icon" style={iconContainerStyles}>
                {item.icon}
              </div>
            ) : null}
          </>
        )}

        {/* Content */}
        <div className="mobile-object-list__content" style={contentStyles}>
          <div className="mobile-object-list__title" style={titleStyles}>
            {item.title}
          </div>
          {item.subtitle && (
            <div className="mobile-object-list__subtitle" style={subtitleStyles}>
              {item.subtitle}
            </div>
          )}
        </div>

        {/* Multi-select checkbox */}
        {multiSelect && (
          <div
            className="mobile-object-list__checkbox"
            style={checkboxStyles}
            aria-hidden="true"
          >
            {isSelected(item.id) && '✓'}
          </div>
        )}
      </div>
    );

    // Wrap with swipe actions if provided
    if ((leftActions && leftActions.length > 0) || (rightActions && rightActions.length > 0)) {
      return (
        <SwipeActions
          key={item.id}
          item={item}
          leftActions={leftActions}
          rightActions={rightActions}
        >
          {itemContent}
        </SwipeActions>
      );
    }

    return <div key={item.id}>{itemContent}</div>;
  };

  if (items.length === 0) {
    return (
      <div
        className={`mobile-object-list mobile-object-list--empty ${className}`}
        style={containerStyles}
        data-testid="mobile-object-list"
      >
        <div style={emptyStyles}>{emptyMessage}</div>
      </div>
    );
  }

  return (
    <div
      ref={listRef}
      className={`mobile-object-list ${className}`}
      style={containerStyles}
      role="list"
      data-testid="mobile-object-list"
    >
      {items.map(renderListItem)}

      <style>{`
        .mobile-object-list::-webkit-scrollbar {
          display: none;
        }

        .mobile-object-list {
          -ms-overflow-style: none;
          scrollbar-width: none;
        }

        .mobile-object-list__item:active:not([aria-disabled="true"]) {
          background-color: rgba(0, 0, 0, 0.05);
        }

        .mobile-object-list__item:focus-visible {
          outline: 2px solid #007AFF;
          outline-offset: -2px;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-object-list__item {
            background-color: #1c1c1e;
            border-color: #38383a;
          }

          .mobile-object-list__item[aria-selected="true"] {
            background-color: #1a3a5c;
          }

          .mobile-object-list__subtitle {
            color: #98989d;
          }

          .mobile-object-list__icon,
          .mobile-object-list__item img {
            background-color: #2c2c2e;
          }

          .mobile-object-list--empty {
            color: #98989d;
          }
        }

        /* Accessibility */
        @media (prefers-reduced-motion: reduce) {
          .mobile-object-list__item {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileObjectList;
