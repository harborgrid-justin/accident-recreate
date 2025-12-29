/**
 * AccuScene Enterprise v0.3.0 - CAD Context Menu Component
 */

import React, { useState, useEffect, useRef } from 'react';
import { ContextMenuItem, Point2D } from './types';

interface CADContextMenuProps {
  items: ContextMenuItem[];
  position: Point2D | null;
  onClose: () => void;
  className?: string;
}

export const CADContextMenu: React.FC<CADContextMenuProps> = ({
  items,
  position,
  onClose,
  className = '',
}) => {
  const [activeSubmenu, setActiveSubmenu] = useState<string | null>(null);
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };

    const handleEscape = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onClose();
      }
    };

    if (position) {
      document.addEventListener('mousedown', handleClickOutside);
      document.addEventListener('keydown', handleEscape);
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside);
      document.removeEventListener('keydown', handleEscape);
    };
  }, [position, onClose]);

  if (!position) return null;

  const handleItemClick = (item: ContextMenuItem) => {
    if (item.disabled) return;

    if (item.submenu) {
      setActiveSubmenu(activeSubmenu === item.id ? null : item.id);
    } else {
      item.action?.();
      onClose();
    }
  };

  const renderMenuItems = (
    menuItems: ContextMenuItem[],
    isSubmenu = false
  ): JSX.Element => {
    return (
      <div
        className={`
          min-w-[200px]
          bg-white dark:bg-gray-900
          border border-gray-200 dark:border-gray-700
          rounded-lg shadow-2xl
          py-1
          ${isSubmenu ? 'absolute left-full top-0 ml-1' : ''}
        `}
      >
        {menuItems.map((item, index) => {
          if (item.separator) {
            return (
              <div
                key={`separator-${index}`}
                className="my-1 border-t border-gray-200 dark:border-gray-700"
              />
            );
          }

          const hasSubmenu = item.submenu && item.submenu.length > 0;
          const isActive = activeSubmenu === item.id;

          return (
            <div key={item.id} className="relative group">
              <button
                onClick={() => handleItemClick(item)}
                onMouseEnter={() => hasSubmenu && setActiveSubmenu(item.id)}
                disabled={item.disabled}
                className={`
                  w-full px-3 py-2 flex items-center justify-between
                  text-sm text-left
                  transition-colors
                  ${
                    item.disabled
                      ? 'text-gray-400 dark:text-gray-600 cursor-not-allowed'
                      : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-800'
                  }
                `}
              >
                <div className="flex items-center gap-3">
                  {/* Icon */}
                  {item.icon && (
                    <span className="w-4 text-center">{item.icon}</span>
                  )}

                  {/* Label */}
                  <span>{item.label}</span>
                </div>

                {/* Shortcut or submenu indicator */}
                <div className="flex items-center gap-2 ml-4">
                  {item.shortcut && !hasSubmenu && (
                    <kbd
                      className="
                        px-1.5 py-0.5 text-xs font-mono
                        bg-gray-200 dark:bg-gray-700
                        text-gray-600 dark:text-gray-400
                        rounded border border-gray-300 dark:border-gray-600
                      "
                    >
                      {item.shortcut}
                    </kbd>
                  )}
                  {hasSubmenu && (
                    <span className="text-gray-400 dark:text-gray-500">▶</span>
                  )}
                </div>
              </button>

              {/* Submenu */}
              {hasSubmenu && isActive && (
                <div className="absolute left-full top-0 ml-1">
                  {renderMenuItems(item.submenu!, true)}
                </div>
              )}
            </div>
          );
        })}
      </div>
    );
  };

  return (
    <div
      ref={menuRef}
      className={`fixed z-50 ${className}`}
      style={{
        left: `${position.x}px`,
        top: `${position.y}px`,
      }}
    >
      {renderMenuItems(items)}
    </div>
  );
};
