/**
 * AccuScene Enterprise v0.3.0
 * Mobile Drawer Component
 *
 * Slide-out navigation drawer with touch gestures
 */

import React, { useState, useEffect, useRef, CSSProperties } from 'react';
import { DrawerMenuItem } from './types';
import { HapticFeedback } from './HapticFeedback';
import { useGesture } from './hooks/useGesture';

export interface MobileDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  items: DrawerMenuItem[];
  position?: 'left' | 'right';
  width?: number | string;
  header?: React.ReactNode;
  footer?: React.ReactNode;
  backgroundColor?: string;
  overlayColor?: string;
  className?: string;
}

/**
 * Slide-out drawer menu with gesture support
 * Supports swipe-to-close and touch-optimized interactions
 *
 * @example
 * ```tsx
 * <MobileDrawer
 *   isOpen={drawerOpen}
 *   onClose={() => setDrawerOpen(false)}
 *   items={menuItems}
 *   position="left"
 * />
 * ```
 */
export const MobileDrawer: React.FC<MobileDrawerProps> = ({
  isOpen,
  onClose,
  items,
  position = 'left',
  width = '280px',
  header,
  footer,
  backgroundColor = '#ffffff',
  overlayColor = 'rgba(0, 0, 0, 0.5)',
  className = '',
}) => {
  const [translateX, setTranslateX] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const drawerRef = useRef<HTMLDivElement>(null);

  const drawerWidth = typeof width === 'number' ? width : parseInt(width);

  const gestureRef = useGesture({
    onPan: (event, delta) => {
      if (!isOpen) return;

      setIsDragging(true);

      if (position === 'left') {
        // Allow dragging left to close
        if (delta.x < 0) {
          setTranslateX(Math.max(delta.x, -drawerWidth));
        }
      } else {
        // Allow dragging right to close
        if (delta.x > 0) {
          setTranslateX(Math.min(delta.x, drawerWidth));
        }
      }
    },
    onGestureEnd: () => {
      if (!isDragging) return;

      setIsDragging(false);

      // Close if dragged more than 50%
      const threshold = drawerWidth * 0.5;
      if (Math.abs(translateX) > threshold) {
        HapticFeedback.light();
        onClose();
      }

      setTranslateX(0);
    },
  });

  useEffect(() => {
    if (isOpen) {
      // Prevent body scroll when drawer is open
      document.body.style.overflow = 'hidden';
      HapticFeedback.light();
    } else {
      document.body.style.overflow = '';
      setTranslateX(0);
    }

    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  const handleItemClick = (item: DrawerMenuItem) => {
    if (item.disabled) return;

    HapticFeedback.selection();
    item.action?.();
    onClose();
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      HapticFeedback.light();
      onClose();
    }
  };

  const overlayStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: overlayColor,
    zIndex: 1000,
    opacity: isOpen ? 1 : 0,
    visibility: isOpen ? 'visible' : 'hidden',
    transition: isDragging ? 'none' : 'opacity 0.3s ease, visibility 0.3s ease',
  };

  const drawerStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    [position]: 0,
    bottom: 0,
    width,
    backgroundColor,
    zIndex: 1001,
    display: 'flex',
    flexDirection: 'column',
    boxShadow:
      position === 'left'
        ? '2px 0 8px rgba(0, 0, 0, 0.15)'
        : '-2px 0 8px rgba(0, 0, 0, 0.15)',
    transform: isOpen
      ? `translateX(${translateX}px)`
      : position === 'left'
      ? `translateX(-${drawerWidth}px)`
      : `translateX(${drawerWidth}px)`,
    transition: isDragging ? 'none' : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    overflowY: 'auto',
    WebkitOverflowScrolling: 'touch',
  };

  const headerStyles: CSSProperties = {
    padding: '1rem',
    borderBottom: '1px solid #e0e0e0',
    flexShrink: 0,
  };

  const contentStyles: CSSProperties = {
    flex: 1,
    overflowY: 'auto',
    padding: '0.5rem 0',
  };

  const footerStyles: CSSProperties = {
    padding: '1rem',
    borderTop: '1px solid #e0e0e0',
    flexShrink: 0,
  };

  const menuItemStyles = (disabled: boolean): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    gap: '1rem',
    padding: '0.875rem 1rem',
    cursor: disabled ? 'not-allowed' : 'pointer',
    opacity: disabled ? 0.4 : 1,
    transition: 'background-color 0.2s ease',
    WebkitTapHighlightColor: 'transparent',
    minHeight: '48px',
    textDecoration: 'none',
    color: 'inherit',
    border: 'none',
    width: '100%',
    textAlign: 'left',
    background: 'none',
    fontSize: '1rem',
  });

  const iconStyles: CSSProperties = {
    fontSize: '20px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    flexShrink: 0,
    width: '24px',
    height: '24px',
  };

  const labelStyles: CSSProperties = {
    flex: 1,
    fontSize: '1rem',
    lineHeight: 1.5,
  };

  const badgeStyles: CSSProperties = {
    backgroundColor: '#FF3B30',
    color: '#ffffff',
    borderRadius: '12px',
    padding: '0.125rem 0.5rem',
    fontSize: '12px',
    fontWeight: 600,
    minWidth: '20px',
    textAlign: 'center',
  };

  const dividerStyles: CSSProperties = {
    height: '1px',
    backgroundColor: '#e0e0e0',
    margin: '0.5rem 0',
  };

  const renderMenuItem = (item: DrawerMenuItem) => {
    if (item.divider) {
      return <div key={item.id} style={dividerStyles} />;
    }

    const MenuItemWrapper = item.path ? 'a' : 'button';

    return (
      <MenuItemWrapper
        key={item.id}
        style={menuItemStyles(!!item.disabled)}
        onClick={() => handleItemClick(item)}
        className="mobile-drawer__item"
        aria-disabled={item.disabled}
        href={item.path}
        {...(MenuItemWrapper === 'button' && { type: 'button' })}
      >
        {item.icon && (
          <span className="mobile-drawer__icon" style={iconStyles}>
            {item.icon}
          </span>
        )}
        <span className="mobile-drawer__label" style={labelStyles}>
          {item.label}
        </span>
        {item.badge && (
          <span className="mobile-drawer__badge" style={badgeStyles}>
            {typeof item.badge === 'number' && item.badge > 99 ? '99+' : item.badge}
          </span>
        )}
      </MenuItemWrapper>
    );
  };

  if (!isOpen && !isDragging) return null;

  return (
    <>
      <div
        className="mobile-drawer__overlay"
        style={overlayStyles}
        onClick={handleOverlayClick}
        aria-hidden="true"
      />
      <div
        ref={gestureRef as any}
        className={`mobile-drawer mobile-drawer--${position} ${className}`}
        style={drawerStyles}
        role="navigation"
        aria-label="Drawer menu"
        data-testid="mobile-drawer"
      >
        {header && (
          <div className="mobile-drawer__header" style={headerStyles}>
            {header}
          </div>
        )}

        <div className="mobile-drawer__content" style={contentStyles}>
          {items.map(renderMenuItem)}
        </div>

        {footer && (
          <div className="mobile-drawer__footer" style={footerStyles}>
            {footer}
          </div>
        )}

        <style>{`
          .mobile-drawer__item:hover:not([aria-disabled="true"]) {
            background-color: rgba(0, 0, 0, 0.05);
          }

          .mobile-drawer__item:active:not([aria-disabled="true"]) {
            background-color: rgba(0, 0, 0, 0.1);
          }

          .mobile-drawer__item:focus-visible {
            outline: 2px solid #007AFF;
            outline-offset: -2px;
          }

          /* Hide scrollbar */
          .mobile-drawer::-webkit-scrollbar {
            display: none;
          }

          .mobile-drawer {
            -ms-overflow-style: none;
            scrollbar-width: none;
          }

          /* Dark mode support */
          @media (prefers-color-scheme: dark) {
            .mobile-drawer {
              background-color: #1c1c1e;
              color: #ffffff;
            }

            .mobile-drawer__header,
            .mobile-drawer__footer {
              border-color: #38383a;
            }

            .mobile-drawer__item:hover:not([aria-disabled="true"]) {
              background-color: rgba(255, 255, 255, 0.1);
            }

            .mobile-drawer__item:active:not([aria-disabled="true"]) {
              background-color: rgba(255, 255, 255, 0.15);
            }
          }

          /* Reduce motion */
          @media (prefers-reduced-motion: reduce) {
            .mobile-drawer,
            .mobile-drawer__overlay {
              transition: none !important;
            }
          }
        `}</style>
      </div>
    </>
  );
};

export default MobileDrawer;
