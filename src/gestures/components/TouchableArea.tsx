/**
 * Touchable wrapper component
 * Provides a simple way to make any component gesture-enabled
 */

import React, { forwardRef } from 'react';
import { useGesture } from '../hooks/useGesture';
import { GestureEvent, GestureConfig } from '../types';

export interface TouchableAreaProps {
  children: React.ReactNode;
  config?: Partial<GestureConfig>;
  onGesture?: (event: GestureEvent) => void;
  className?: string;
  style?: React.CSSProperties;
  disabled?: boolean;
  testID?: string;
}

export const TouchableArea = forwardRef<HTMLDivElement, TouchableAreaProps>(
  ({ children, config, onGesture, className, style, disabled = false, testID }, ref) => {
    const { handlers, state } = useGesture({
      config,
      onGesture,
      enabled: !disabled,
    });

    const defaultStyle: React.CSSProperties = {
      touchAction: 'none',
      userSelect: 'none',
      WebkitUserSelect: 'none',
      MozUserSelect: 'none',
      msUserSelect: 'none',
      WebkitTouchCallout: 'none',
      position: 'relative',
      ...style,
    };

    return (
      <div
        ref={ref}
        className={className}
        style={defaultStyle}
        data-testid={testID}
        data-gesture-active={state.isActive}
        data-touch-count={state.touchCount}
        {...handlers}
      >
        {children}
      </div>
    );
  }
);

TouchableArea.displayName = 'TouchableArea';

export default TouchableArea;
