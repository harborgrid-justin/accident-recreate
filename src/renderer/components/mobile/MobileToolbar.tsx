/**
 * AccuScene Enterprise v0.3.0
 * Mobile Toolbar Component
 *
 * Compact floating toolbar for mobile editing and actions
 */

import React, { useState, useEffect, useRef, CSSProperties } from 'react';
import { HapticFeedback } from './HapticFeedback';

export interface ToolbarAction {
  id: string;
  icon: React.ReactNode;
  label: string;
  onClick: () => void;
  disabled?: boolean;
  active?: boolean;
  badge?: number | string;
}

export interface MobileToolbarProps {
  actions: ToolbarAction[];
  position?: 'top' | 'bottom' | 'floating';
  floatingPosition?: { x: number; y: number };
  orientation?: 'horizontal' | 'vertical';
  backgroundColor?: string;
  activeColor?: string;
  isDraggable?: boolean;
  className?: string;
}

/**
 * Compact toolbar for mobile actions
 * Supports floating, draggable, and docked modes
 *
 * @example
 * ```tsx
 * <MobileToolbar
 *   actions={[
 *     { id: 'draw', icon: <PenIcon />, label: 'Draw', onClick: handleDraw },
 *     { id: 'erase', icon: <EraserIcon />, label: 'Erase', onClick: handleErase },
 *   ]}
 *   position="floating"
 *   isDraggable
 * />
 * ```
 */
export const MobileToolbar: React.FC<MobileToolbarProps> = ({
  actions,
  position = 'bottom',
  floatingPosition = { x: 20, y: 100 },
  orientation = 'horizontal',
  backgroundColor = '#ffffff',
  activeColor = '#007AFF',
  isDraggable = false,
  className = '',
}) => {
  const [currentPosition, setCurrentPosition] = useState(floatingPosition);
  const [isDragging, setIsDragging] = useState(false);
  const [dragStart, setDragStart] = useState({ x: 0, y: 0 });
  const toolbarRef = useRef<HTMLDivElement>(null);

  const handleTouchStart = (e: React.TouchEvent) => {
    if (!isDraggable || position !== 'floating') return;

    const touch = e.touches[0];
    setIsDragging(true);
    setDragStart({
      x: touch.clientX - currentPosition.x,
      y: touch.clientY - currentPosition.y,
    });
    HapticFeedback.light();
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDragging) return;

    const touch = e.touches[0];
    const newX = touch.clientX - dragStart.x;
    const newY = touch.clientY - dragStart.y;

    // Constrain within viewport
    const toolbar = toolbarRef.current;
    if (toolbar) {
      const rect = toolbar.getBoundingClientRect();
      const maxX = window.innerWidth - rect.width;
      const maxY = window.innerHeight - rect.height;

      setCurrentPosition({
        x: Math.max(0, Math.min(newX, maxX)),
        y: Math.max(0, Math.min(newY, maxY)),
      });
    }
  };

  const handleTouchEnd = () => {
    if (isDragging) {
      setIsDragging(false);
      HapticFeedback.light();
    }
  };

  const handleActionClick = (action: ToolbarAction) => {
    if (action.disabled) return;

    HapticFeedback.selection();
    action.onClick();
  };

  const getToolbarStyles = (): CSSProperties => {
    const baseStyles: CSSProperties = {
      display: 'flex',
      flexDirection: orientation === 'horizontal' ? 'row' : 'column',
      alignItems: 'center',
      gap: '0.5rem',
      backgroundColor,
      borderRadius: '12px',
      padding: orientation === 'horizontal' ? '0.5rem 0.75rem' : '0.75rem 0.5rem',
      boxShadow: '0 4px 12px rgba(0, 0, 0, 0.15)',
      zIndex: 200,
      userSelect: 'none',
      WebkitUserSelect: 'none',
      touchAction: isDraggable && position === 'floating' ? 'none' : 'auto',
    };

    if (position === 'floating') {
      return {
        ...baseStyles,
        position: 'fixed',
        left: `${currentPosition.x}px`,
        top: `${currentPosition.y}px`,
        cursor: isDraggable ? (isDragging ? 'grabbing' : 'grab') : 'default',
        transition: isDragging ? 'none' : 'all 0.2s ease',
      };
    }

    if (position === 'top') {
      return {
        ...baseStyles,
        position: 'sticky',
        top: '1rem',
        left: '50%',
        transform: 'translateX(-50%)',
        margin: '1rem auto',
        width: 'fit-content',
      };
    }

    // bottom
    return {
      ...baseStyles,
      position: 'sticky',
      bottom: '1rem',
      left: '50%',
      transform: 'translateX(-50%)',
      margin: '1rem auto',
      width: 'fit-content',
    };
  };

  const actionButtonStyles = (action: ToolbarAction): CSSProperties => ({
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '0.5rem',
    minWidth: '44px',
    minHeight: '44px',
    borderRadius: '8px',
    border: 'none',
    background: action.active ? `${activeColor}15` : 'transparent',
    color: action.active ? activeColor : '#000000',
    cursor: action.disabled ? 'not-allowed' : 'pointer',
    opacity: action.disabled ? 0.4 : 1,
    transition: 'all 0.2s ease',
    WebkitTapHighlightColor: 'transparent',
    position: 'relative',
  });

  const iconStyles: CSSProperties = {
    fontSize: '20px',
    lineHeight: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  };

  const badgeStyles: CSSProperties = {
    position: 'absolute',
    top: '0.25rem',
    right: '0.25rem',
    backgroundColor: '#FF3B30',
    color: '#ffffff',
    borderRadius: '10px',
    padding: '0.125rem 0.375rem',
    fontSize: '10px',
    fontWeight: 600,
    minWidth: '16px',
    height: '16px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
  };

  return (
    <div
      ref={toolbarRef}
      className={`mobile-toolbar mobile-toolbar--${position} mobile-toolbar--${orientation} ${className}`}
      style={getToolbarStyles()}
      onTouchStart={handleTouchStart}
      onTouchMove={handleTouchMove}
      onTouchEnd={handleTouchEnd}
      role="toolbar"
      aria-label="Mobile toolbar"
      data-testid="mobile-toolbar"
    >
      {actions.map((action) => (
        <button
          key={action.id}
          type="button"
          className={`mobile-toolbar__action ${
            action.active ? 'mobile-toolbar__action--active' : ''
          }`}
          style={actionButtonStyles(action)}
          onClick={() => handleActionClick(action)}
          aria-label={action.label}
          aria-pressed={action.active}
          aria-disabled={action.disabled}
          disabled={action.disabled}
        >
          {action.badge && (
            <span className="mobile-toolbar__badge" style={badgeStyles}>
              {typeof action.badge === 'number' && action.badge > 99
                ? '99+'
                : action.badge}
            </span>
          )}

          <span className="mobile-toolbar__icon" style={iconStyles}>
            {action.icon}
          </span>
        </button>
      ))}

      <style>{`
        .mobile-toolbar__action:hover:not(:disabled) {
          background-color: rgba(0, 0, 0, 0.05);
        }

        .mobile-toolbar__action:active:not(:disabled) {
          transform: scale(0.95);
        }

        .mobile-toolbar__action:focus-visible {
          outline: 2px solid ${activeColor};
          outline-offset: 2px;
        }

        .mobile-toolbar__action--active {
          border: 1px solid ${activeColor};
        }

        /* Ripple effect */
        .mobile-toolbar__action::before {
          content: '';
          position: absolute;
          top: 50%;
          left: 50%;
          width: 0;
          height: 0;
          border-radius: 50%;
          background-color: currentColor;
          opacity: 0;
          transform: translate(-50%, -50%);
          transition: width 0.3s ease, height 0.3s ease, opacity 0.3s ease;
        }

        .mobile-toolbar__action:active:not(:disabled)::before {
          width: 100%;
          height: 100%;
          opacity: 0.1;
        }

        /* Drag handle indicator for floating toolbar */
        .mobile-toolbar--floating::after {
          content: ${isDraggable ? "''" : 'none'};
          position: absolute;
          top: ${orientation === 'horizontal' ? '4px' : '50%'};
          left: ${orientation === 'horizontal' ? '50%' : '4px'};
          transform: ${
            orientation === 'horizontal' ? 'translateX(-50%)' : 'translateY(-50%)'
          };
          width: ${orientation === 'horizontal' ? '24px' : '4px'};
          height: ${orientation === 'horizontal' ? '4px' : '24px'};
          background-color: rgba(0, 0, 0, 0.2);
          border-radius: '2px';
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-toolbar {
            background-color: #2c2c2e;
            color: #ffffff;
          }

          .mobile-toolbar__action {
            color: #ffffff;
          }

          .mobile-toolbar__action:hover:not(:disabled) {
            background-color: rgba(255, 255, 255, 0.1);
          }

          .mobile-toolbar--floating::after {
            background-color: rgba(255, 255, 255, 0.3);
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-toolbar,
          .mobile-toolbar__action,
          .mobile-toolbar__action::before {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default MobileToolbar;
