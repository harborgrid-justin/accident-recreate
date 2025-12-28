/**
 * Accessible Icon Component
 *
 * Wraps icon components to provide proper accessibility attributes.
 * Implements WCAG 2.1 Success Criterion 1.1.1 (Non-text Content)
 */

import React, { ReactNode } from 'react';
import { VisuallyHidden } from './VisuallyHidden';

export interface AccessibleIconProps {
  children: ReactNode;
  label?: string;
  decorative?: boolean;
  className?: string;
  size?: number | string;
}

export const AccessibleIcon: React.FC<AccessibleIconProps> = ({
  children,
  label,
  decorative = false,
  className = '',
  size,
}) => {
  const iconStyles: React.CSSProperties = {
    display: 'inline-block',
    verticalAlign: 'middle',
    ...(size && {
      width: typeof size === 'number' ? `${size}px` : size,
      height: typeof size === 'number' ? `${size}px` : size,
    }),
  };

  // Decorative icons (no semantic meaning)
  if (decorative) {
    return (
      <span
        className={`accessible-icon ${className}`}
        aria-hidden="true"
        style={iconStyles}
      >
        {children}
      </span>
    );
  }

  // Semantic icons with label
  if (label) {
    return (
      <span
        className={`accessible-icon ${className}`}
        role="img"
        aria-label={label}
        style={iconStyles}
      >
        {children}
        <VisuallyHidden>{label}</VisuallyHidden>
      </span>
    );
  }

  // Default: treat as decorative if no label provided
  console.warn(
    'AccessibleIcon: Non-decorative icon should have a label. Consider setting decorative={true} if the icon is purely visual.'
  );

  return (
    <span
      className={`accessible-icon ${className}`}
      aria-hidden="true"
      style={iconStyles}
    >
      {children}
    </span>
  );
};

/**
 * Icon Button Component with accessibility
 */
export interface AccessibleIconButtonProps {
  icon: ReactNode;
  label: string;
  onClick?: () => void;
  disabled?: boolean;
  className?: string;
  type?: 'button' | 'submit' | 'reset';
  size?: number | string;
}

export const AccessibleIconButton: React.FC<AccessibleIconButtonProps> = ({
  icon,
  label,
  onClick,
  disabled = false,
  className = '',
  type = 'button',
  size,
}) => {
  const buttonStyles: React.CSSProperties = {
    display: 'inline-flex',
    alignItems: 'center',
    justifyContent: 'center',
    padding: '0.5rem',
    border: 'none',
    background: 'transparent',
    cursor: disabled ? 'not-allowed' : 'pointer',
    borderRadius: '0.25rem',
    transition: 'background-color 0.2s ease-in-out',
    ...(size && {
      width: typeof size === 'number' ? `${size}px` : size,
      height: typeof size === 'number' ? `${size}px` : size,
    }),
  };

  return (
    <button
      type={type}
      onClick={onClick}
      disabled={disabled}
      aria-label={label}
      className={`accessible-icon-button ${className}`}
      style={buttonStyles}
      onFocus={(e) => {
        e.currentTarget.style.outline = '2px solid var(--a11y-focus-color, #4A9EFF)';
        e.currentTarget.style.outlineOffset = '2px';
      }}
      onBlur={(e) => {
        e.currentTarget.style.outline = 'none';
        e.currentTarget.style.outlineOffset = '0';
      }}
      onMouseEnter={(e) => {
        if (!disabled) {
          e.currentTarget.style.backgroundColor = 'var(--a11y-hover-bg, rgba(0, 0, 0, 0.05))';
        }
      }}
      onMouseLeave={(e) => {
        e.currentTarget.style.backgroundColor = 'transparent';
      }}
    >
      <AccessibleIcon decorative label={label}>
        {icon}
      </AccessibleIcon>
      <VisuallyHidden>{label}</VisuallyHidden>
    </button>
  );
};

export default AccessibleIcon;
