/**
 * AccuScene Enterprise v0.3.0
 * Swipe Actions Component
 *
 * Swipe left/right action handler for list items
 */

import React, { useState, useRef, useCallback, CSSProperties, ReactNode } from 'react';
import { SwipeAction, ListItem } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface SwipeActionsProps {
  item: ListItem;
  children: ReactNode;
  leftActions?: SwipeAction[];
  rightActions?: SwipeAction[];
  threshold?: number;
  onSwipeStart?: () => void;
  onSwipeEnd?: () => void;
  className?: string;
}

/**
 * Swipe action wrapper for list items
 * Reveals actions on left or right swipe (like iOS Mail)
 *
 * @example
 * ```tsx
 * <SwipeActions
 *   item={item}
 *   leftActions={[
 *     { id: 'archive', label: 'Archive', backgroundColor: '#007AFF', onAction: handleArchive },
 *   ]}
 *   rightActions={[
 *     { id: 'delete', label: 'Delete', backgroundColor: '#FF3B30', onAction: handleDelete },
 *   ]}
 * >
 *   <ListItemContent />
 * </SwipeActions>
 * ```
 */
export const SwipeActions: React.FC<SwipeActionsProps> = ({
  item,
  children,
  leftActions = [],
  rightActions = [],
  threshold = 80,
  onSwipeStart,
  onSwipeEnd,
  className = '',
}) => {
  const [offsetX, setOffsetX] = useState(0);
  const [isSwiping, setIsSwiping] = useState(false);
  const [triggeredAction, setTriggeredAction] = useState<SwipeAction | null>(null);

  const startXRef = useRef<number>(0);
  const currentXRef = useRef<number>(0);
  const itemRef = useRef<HTMLDivElement>(null);

  const handleTouchStart = useCallback(
    (e: React.TouchEvent) => {
      const touch = e.touches[0];
      startXRef.current = touch.clientX;
      currentXRef.current = touch.clientX;
      setIsSwiping(true);
      onSwipeStart?.();
    },
    [onSwipeStart]
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      if (!isSwiping) return;

      const touch = e.touches[0];
      currentXRef.current = touch.clientX;
      const delta = currentXRef.current - startXRef.current;

      // Constrain swipe
      const maxSwipe = 200;
      const constrainedDelta = Math.max(-maxSwipe, Math.min(maxSwipe, delta));

      setOffsetX(constrainedDelta);

      // Haptic feedback when crossing threshold
      if (Math.abs(constrainedDelta) >= threshold) {
        const actions = constrainedDelta > 0 ? leftActions : rightActions;
        if (actions.length > 0 && !triggeredAction) {
          HapticFeedback.selection();
          setTriggeredAction(actions[0]);
        }
      } else if (triggeredAction) {
        setTriggeredAction(null);
      }
    },
    [isSwiping, threshold, leftActions, rightActions, triggeredAction]
  );

  const handleTouchEnd = useCallback(() => {
    setIsSwiping(false);

    if (triggeredAction) {
      // Execute action
      HapticFeedback.medium();
      triggeredAction.onAction(item);
      setTriggeredAction(null);
    }

    // Snap back or reveal actions
    if (Math.abs(offsetX) >= threshold) {
      // Keep actions revealed
      const revealAmount = offsetX > 0 ? 80 : -80;
      setOffsetX(revealAmount);
    } else {
      // Snap back
      setOffsetX(0);
    }

    onSwipeEnd?.();
  }, [offsetX, threshold, triggeredAction, item, onSwipeEnd]);

  const handleActionClick = (action: SwipeAction) => {
    HapticFeedback.medium();
    action.onAction(item);
    setOffsetX(0);
  };

  const containerStyles: CSSProperties = {
    position: 'relative',
    overflow: 'hidden',
    touchAction: 'pan-y',
    userSelect: 'none',
    WebkitUserSelect: 'none',
  };

  const contentStyles: CSSProperties = {
    transform: `translateX(${offsetX}px)`,
    transition: isSwiping ? 'none' : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    position: 'relative',
    zIndex: 2,
    backgroundColor: '#ffffff',
  };

  const actionsContainerStyles = (side: 'left' | 'right'): CSSProperties => ({
    position: 'absolute',
    top: 0,
    bottom: 0,
    [side]: 0,
    display: 'flex',
    flexDirection: 'row',
    alignItems: 'stretch',
    zIndex: 1,
  });

  const actionButtonStyles = (action: SwipeAction): CSSProperties => ({
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '0 1.25rem',
    backgroundColor: action.backgroundColor || '#007AFF',
    color: action.color || '#ffffff',
    border: 'none',
    cursor: 'pointer',
    minWidth: '80px',
    fontSize: '0.875rem',
    fontWeight: 500,
    gap: '0.25rem',
    transition: 'opacity 0.2s ease',
  });

  const iconStyles: CSSProperties = {
    fontSize: '20px',
    lineHeight: 1,
  };

  return (
    <div
      ref={itemRef}
      className={`swipe-actions ${className}`}
      style={containerStyles}
      data-testid="swipe-actions"
    >
      {/* Left actions */}
      {leftActions.length > 0 && (
        <div className="swipe-actions__left" style={actionsContainerStyles('left')}>
          {leftActions.map((action) => (
            <button
              key={action.id}
              className="swipe-actions__action"
              style={actionButtonStyles(action)}
              onClick={() => handleActionClick(action)}
              type="button"
              aria-label={action.label}
            >
              {action.icon && (
                <span className="swipe-actions__icon" style={iconStyles}>
                  {action.icon}
                </span>
              )}
              <span className="swipe-actions__label">{action.label}</span>
            </button>
          ))}
        </div>
      )}

      {/* Right actions */}
      {rightActions.length > 0 && (
        <div className="swipe-actions__right" style={actionsContainerStyles('right')}>
          {rightActions.map((action) => (
            <button
              key={action.id}
              className="swipe-actions__action"
              style={actionButtonStyles(action)}
              onClick={() => handleActionClick(action)}
              type="button"
              aria-label={action.label}
            >
              {action.icon && (
                <span className="swipe-actions__icon" style={iconStyles}>
                  {action.icon}
                </span>
              )}
              <span className="swipe-actions__label">{action.label}</span>
            </button>
          ))}
        </div>
      )}

      {/* Content */}
      <div
        className="swipe-actions__content"
        style={contentStyles}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onTouchCancel={handleTouchEnd}
      >
        {children}
      </div>

      <style>{`
        .swipe-actions__action:active {
          opacity: 0.8;
        }

        .swipe-actions__action:focus-visible {
          outline: 2px solid #ffffff;
          outline-offset: -4px;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .swipe-actions__content {
            background-color: #1c1c1e;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .swipe-actions__content {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default SwipeActions;
