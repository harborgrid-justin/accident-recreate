/**
 * AccuScene Enterprise v0.3.0
 * Floating Action Button Component
 *
 * Material Design-style floating action button
 */

import React, { useState, CSSProperties, ReactNode } from 'react';
import { FABAction, FABPosition } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface FloatingActionButtonProps {
  icon: ReactNode;
  onClick?: () => void;
  actions?: FABAction[];
  position?: FABPosition;
  color?: string;
  size?: 'small' | 'medium' | 'large';
  label?: string;
  showLabel?: boolean;
  disabled?: boolean;
  className?: string;
}

/**
 * Floating action button (FAB)
 * Supports expandable speed dial actions
 *
 * @example
 * ```tsx
 * <FloatingActionButton
 *   icon={<PlusIcon />}
 *   onClick={() => createNew()}
 *   position="bottom-right"
 *   actions={[
 *     { id: 'photo', icon: <CameraIcon />, label: 'Take Photo', onClick: () => {} },
 *     { id: 'note', icon: <NoteIcon />, label: 'Add Note', onClick: () => {} },
 *   ]}
 * />
 * ```
 */
export const FloatingActionButton: React.FC<FloatingActionButtonProps> = ({
  icon,
  onClick,
  actions = [],
  position = 'bottom-right',
  color = '#007AFF',
  size = 'medium',
  label,
  showLabel = false,
  disabled = false,
  className = '',
}) => {
  const [isExpanded, setIsExpanded] = useState(false);

  const getSizeValue = (): number => {
    switch (size) {
      case 'small':
        return 48;
      case 'large':
        return 64;
      case 'medium':
      default:
        return 56;
    }
  };

  const getPositionStyles = (): CSSProperties => {
    const positions: Record<FABPosition, CSSProperties> = {
      'bottom-right': { bottom: '1.5rem', right: '1.5rem' },
      'bottom-left': { bottom: '1.5rem', left: '1.5rem' },
      'bottom-center': { bottom: '1.5rem', left: '50%', transform: 'translateX(-50%)' },
      'top-right': { top: '1.5rem', right: '1.5rem' },
      'top-left': { top: '1.5rem', left: '1.5rem' },
      'top-center': { top: '1.5rem', left: '50%', transform: 'translateX(-50%)' },
    };

    return positions[position];
  };

  const handleMainClick = () => {
    if (disabled) return;

    HapticFeedback.medium();

    if (actions.length > 0) {
      setIsExpanded(!isExpanded);
    } else {
      onClick?.();
    }
  };

  const handleActionClick = (action: FABAction) => {
    if (action.disabled) return;

    HapticFeedback.selection();
    action.onClick();
    setIsExpanded(false);
  };

  const fabSize = getSizeValue();

  const containerStyles: CSSProperties = {
    position: 'fixed',
    ...getPositionStyles(),
    zIndex: 1000,
    display: 'flex',
    flexDirection: position.startsWith('bottom') ? 'column-reverse' : 'column',
    alignItems: position.includes('right')
      ? 'flex-end'
      : position.includes('left')
      ? 'flex-start'
      : 'center',
    gap: '1rem',
  };

  const mainButtonStyles: CSSProperties = {
    width: `${fabSize}px`,
    height: `${fabSize}px`,
    borderRadius: '50%',
    border: 'none',
    backgroundColor: disabled ? '#d0d0d0' : color,
    color: '#ffffff',
    fontSize: size === 'small' ? '1.25rem' : size === 'large' ? '1.75rem' : '1.5rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: disabled ? 'not-allowed' : 'pointer',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.3)',
    transition: 'transform 0.2s ease, box-shadow 0.2s ease',
    position: 'relative',
    transform: isExpanded ? 'rotate(45deg)' : 'rotate(0)',
  };

  const labelStyles: CSSProperties = {
    position: 'absolute',
    [position.includes('right') ? 'right' : 'left']: '100%',
    [position.includes('right') ? 'marginRight' : 'marginLeft']: '1rem',
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    color: '#ffffff',
    padding: '0.5rem 1rem',
    borderRadius: '4px',
    fontSize: '0.875rem',
    whiteSpace: 'nowrap',
    pointerEvents: 'none',
    opacity: showLabel ? 1 : 0,
    transition: 'opacity 0.2s ease',
  };

  const actionButtonStyles = (action: FABAction): CSSProperties => ({
    width: '48px',
    height: '48px',
    borderRadius: '50%',
    border: 'none',
    backgroundColor: action.disabled ? '#d0d0d0' : action.color || '#ffffff',
    color: action.color ? '#ffffff' : color,
    fontSize: '1.25rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: action.disabled ? 'not-allowed' : 'pointer',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.2)',
    transition: 'transform 0.2s ease',
    opacity: isExpanded ? 1 : 0,
    transform: isExpanded ? 'scale(1)' : 'scale(0)',
    pointerEvents: isExpanded ? 'auto' : 'none',
  });

  const actionLabelStyles: CSSProperties = {
    backgroundColor: 'rgba(0, 0, 0, 0.8)',
    color: '#ffffff',
    padding: '0.375rem 0.75rem',
    borderRadius: '4px',
    fontSize: '0.75rem',
    whiteSpace: 'nowrap',
    marginRight: position.includes('right') ? '0.75rem' : undefined,
    marginLeft: position.includes('left') ? '0.75rem' : undefined,
  };

  const actionContainerStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    gap: '0.5rem',
    flexDirection: position.includes('right') ? 'row-reverse' : 'row',
  };

  const overlayStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.3)',
    zIndex: 999,
    opacity: isExpanded ? 1 : 0,
    pointerEvents: isExpanded ? 'auto' : 'none',
    transition: 'opacity 0.2s ease',
  };

  return (
    <>
      {/* Overlay backdrop when expanded */}
      <div
        className="fab__overlay"
        style={overlayStyles}
        onClick={() => setIsExpanded(false)}
        aria-hidden="true"
      />

      <div
        className={`floating-action-button ${className}`}
        style={containerStyles}
        data-testid="floating-action-button"
      >
        {/* Speed dial actions */}
        {actions.map((action, index) => (
          <div
            key={action.id}
            className="fab__action-container"
            style={{
              ...actionContainerStyles,
              transitionDelay: isExpanded ? `${index * 50}ms` : '0ms',
            }}
          >
            <div className="fab__action-label" style={actionLabelStyles}>
              {action.label}
            </div>

            <button
              className="fab__action"
              style={{
                ...actionButtonStyles(action),
                transitionDelay: isExpanded ? `${index * 50}ms` : '0ms',
              }}
              onClick={() => handleActionClick(action)}
              type="button"
              aria-label={action.label}
              disabled={action.disabled}
            >
              {action.icon}
            </button>
          </div>
        ))}

        {/* Main FAB */}
        <button
          className="fab__main"
          style={mainButtonStyles}
          onClick={handleMainClick}
          type="button"
          aria-label={label || 'Floating action button'}
          aria-expanded={isExpanded}
          disabled={disabled}
        >
          {icon}

          {label && showLabel && (
            <div className="fab__label" style={labelStyles}>
              {label}
            </div>
          )}
        </button>
      </div>

      <style>{`
        .fab__main:hover:not(:disabled) {
          transform: ${isExpanded ? 'rotate(45deg) scale(1.05)' : 'scale(1.05)'};
          box-shadow: 0 6px 16px rgba(0, 0, 0, 0.4);
        }

        .fab__main:active:not(:disabled) {
          transform: ${isExpanded ? 'rotate(45deg) scale(0.95)' : 'scale(0.95)'};
        }

        .fab__main:hover .fab__label {
          opacity: 1;
        }

        .fab__action:hover:not(:disabled) {
          transform: scale(1.1);
        }

        .fab__action:active:not(:disabled) {
          transform: scale(0.9);
        }

        .fab__main:focus-visible,
        .fab__action:focus-visible {
          outline: 2px solid #ffffff;
          outline-offset: 2px;
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .fab__main,
          .fab__action,
          .fab__overlay {
            transition: none !important;
          }
        }
      `}</style>
    </>
  );
};

export default FloatingActionButton;
