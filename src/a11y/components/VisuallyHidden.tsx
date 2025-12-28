/**
 * Visually Hidden Component
 *
 * Hides content visually but keeps it accessible to screen readers.
 * Useful for providing additional context that's not needed visually.
 */

import React, { ReactNode, CSSProperties } from 'react';

export interface VisuallyHiddenProps {
  children: ReactNode;
  as?: keyof JSX.IntrinsicElements;
  focusable?: boolean;
  className?: string;
  style?: CSSProperties;
}

/**
 * CSS to visually hide an element while keeping it accessible
 */
const visuallyHiddenStyles: CSSProperties = {
  position: 'absolute',
  width: '1px',
  height: '1px',
  padding: 0,
  margin: '-1px',
  overflow: 'hidden',
  clip: 'rect(0, 0, 0, 0)',
  whiteSpace: 'nowrap',
  border: 0,
};

/**
 * Additional styles for focusable visually hidden elements
 */
const focusableStyles: CSSProperties = {
  ...visuallyHiddenStyles,
  transition: 'all 0.2s ease-in-out',
};

export const VisuallyHidden: React.FC<VisuallyHiddenProps> = ({
  children,
  as: Component = 'span',
  focusable = false,
  className = '',
  style = {},
}) => {
  const handleFocus = (e: React.FocusEvent<HTMLElement>) => {
    if (focusable) {
      // Temporarily show the element when focused
      e.currentTarget.style.position = 'static';
      e.currentTarget.style.width = 'auto';
      e.currentTarget.style.height = 'auto';
      e.currentTarget.style.clip = 'auto';
      e.currentTarget.style.whiteSpace = 'normal';
    }
  };

  const handleBlur = (e: React.FocusEvent<HTMLElement>) => {
    if (focusable) {
      // Re-hide the element when focus is lost
      Object.assign(e.currentTarget.style, visuallyHiddenStyles);
    }
  };

  const combinedStyles = {
    ...(focusable ? focusableStyles : visuallyHiddenStyles),
    ...style,
  };

  return React.createElement(
    Component,
    {
      className: `visually-hidden ${className}`,
      style: combinedStyles,
      ...(focusable && {
        tabIndex: 0,
        onFocus: handleFocus,
        onBlur: handleBlur,
      }),
    },
    children
  );
};

/**
 * Utility function to generate visually hidden styles
 */
export const getVisuallyHiddenStyles = (focusable = false): CSSProperties => {
  return focusable ? focusableStyles : visuallyHiddenStyles;
};

export default VisuallyHidden;
