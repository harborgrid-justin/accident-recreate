/**
 * AccuScene Enterprise v0.3.0
 * Gesture Handler Component
 *
 * Multi-touch gesture recognition wrapper component
 */

import React, { ReactNode } from 'react';
import { GestureHandlers } from './types';
import { useGesture } from './hooks/useGesture';

export interface GestureHandlerProps extends GestureHandlers {
  children: ReactNode;
  className?: string;
  style?: React.CSSProperties;
  tapTimeout?: number;
  doubleTapTimeout?: number;
  longPressTimeout?: number;
  swipeThreshold?: number;
  preventDefaultEvents?: boolean;
}

/**
 * Wrapper component for gesture recognition
 * Provides comprehensive touch gesture detection for child elements
 *
 * @example
 * ```tsx
 * <GestureHandler
 *   onTap={(e, point) => console.log('Tapped at', point)}
 *   onSwipe={(e, direction) => console.log('Swiped', direction)}
 *   onPinch={(e, scale) => setZoom(scale)}
 * >
 *   <div>Touch-enabled content</div>
 * </GestureHandler>
 * ```
 */
export const GestureHandler: React.FC<GestureHandlerProps> = ({
  children,
  className = '',
  style = {},
  onTap,
  onDoubleTap,
  onLongPress,
  onSwipe,
  onPinch,
  onRotate,
  onPan,
  onDrag,
  onGestureStart,
  onGestureEnd,
  tapTimeout,
  doubleTapTimeout,
  longPressTimeout,
  swipeThreshold,
  preventDefaultEvents = true,
}) => {
  const gestureRef = useGesture(
    {
      onTap,
      onDoubleTap,
      onLongPress,
      onSwipe,
      onPinch,
      onRotate,
      onPan,
      onDrag,
      onGestureStart,
      onGestureEnd,
    },
    {
      tapTimeout,
      doubleTapTimeout,
      longPressTimeout,
      swipeThreshold,
      preventDefaultEvents,
    }
  );

  const containerStyle: React.CSSProperties = {
    touchAction: 'none',
    userSelect: 'none',
    WebkitUserSelect: 'none',
    WebkitTouchCallout: 'none',
    ...style,
  };

  return (
    <div
      ref={gestureRef as any}
      className={`gesture-handler ${className}`}
      style={containerStyle}
      data-testid="gesture-handler"
    >
      {children}

      <style>{`
        .gesture-handler {
          position: relative;
        }

        .gesture-handler * {
          -webkit-user-select: none;
          -webkit-touch-callout: none;
        }

        /* Prevent pull-to-refresh on mobile browsers */
        .gesture-handler {
          overscroll-behavior: contain;
        }
      `}</style>
    </div>
  );
};

export default GestureHandler;
