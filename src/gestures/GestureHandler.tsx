/**
 * Main Gesture Handler Component
 * Comprehensive gesture detection and handling
 */

import React, { useCallback, forwardRef } from 'react';
import {
  GestureHandlerProps,
  GestureEvent,
  TapEvent,
  DoubleTapEvent,
  TripleTapEvent,
  SwipeEvent,
  PinchStartEvent,
  PinchMoveEvent,
  PinchEndEvent,
  RotateStartEvent,
  RotateMoveEvent,
  RotateEndEvent,
  PanStartEvent,
  PanMoveEvent,
  PanEndEvent,
  LongPressStartEvent,
  LongPressEvent,
  LongPressEndEvent,
  CustomGestureEvent,
} from './types';
import { useGesture } from './hooks/useGesture';

export const GestureHandler = forwardRef<HTMLDivElement, GestureHandlerProps>(
  (
    {
      children,
      config,
      onGesture,
      onTap,
      onDoubleTap,
      onTripleTap,
      onSwipe,
      onPinch,
      onPinchStart,
      onPinchEnd,
      onRotate,
      onRotateStart,
      onRotateEnd,
      onPan,
      onPanStart,
      onPanEnd,
      onLongPress,
      onLongPressStart,
      onLongPressEnd,
      onCustomGesture,
      className,
      style,
      disabled = false,
    },
    ref
  ) => {
    const handleGesture = useCallback(
      (event: GestureEvent) => {
        // Call general handler
        onGesture?.(event);

        // Call specific handlers based on event type
        switch (event.type) {
          case 'tap':
            onTap?.(event as TapEvent);
            break;
          case 'doubleTap':
            onDoubleTap?.(event as DoubleTapEvent);
            break;
          case 'tripleTap':
            onTripleTap?.(event as TripleTapEvent);
            break;
          case 'swipe':
            onSwipe?.(event as SwipeEvent);
            break;
          case 'pinchStart':
            onPinchStart?.(event as PinchStartEvent);
            break;
          case 'pinchMove':
            onPinch?.(event as PinchMoveEvent);
            break;
          case 'pinchEnd':
            onPinchEnd?.(event as PinchEndEvent);
            break;
          case 'rotateStart':
            onRotateStart?.(event as RotateStartEvent);
            break;
          case 'rotateMove':
            onRotate?.(event as RotateMoveEvent);
            break;
          case 'rotateEnd':
            onRotateEnd?.(event as RotateEndEvent);
            break;
          case 'panStart':
            onPanStart?.(event as PanStartEvent);
            break;
          case 'panMove':
            onPan?.(event as PanMoveEvent);
            break;
          case 'panEnd':
            onPanEnd?.(event as PanEndEvent);
            break;
          case 'longPressStart':
            onLongPressStart?.(event as LongPressStartEvent);
            break;
          case 'longPress':
            onLongPress?.(event as LongPressEvent);
            break;
          case 'longPressEnd':
            onLongPressEnd?.(event as LongPressEndEvent);
            break;
          case 'customGesture':
            onCustomGesture?.(event as CustomGestureEvent);
            break;
        }
      },
      [
        onGesture,
        onTap,
        onDoubleTap,
        onTripleTap,
        onSwipe,
        onPinch,
        onPinchStart,
        onPinchEnd,
        onRotate,
        onRotateStart,
        onRotateEnd,
        onPan,
        onPanStart,
        onPanEnd,
        onLongPress,
        onLongPressStart,
        onLongPressEnd,
        onCustomGesture,
      ]
    );

    const { handlers, state } = useGesture({
      config,
      onGesture: handleGesture,
      enabled: !disabled,
    });

    const defaultStyle: React.CSSProperties = {
      touchAction: 'none',
      userSelect: 'none',
      WebkitUserSelect: 'none',
      MozUserSelect: 'none',
      msUserSelect: 'none',
      WebkitTouchCallout: 'none',
      ...style,
    };

    return (
      <div
        ref={ref}
        className={className}
        style={defaultStyle}
        data-gesture-active={state.isActive}
        data-active-gestures={state.activeGestures.join(',')}
        data-touch-count={state.touchCount}
        {...handlers}
      >
        {children}
      </div>
    );
  }
);

GestureHandler.displayName = 'GestureHandler';

export default GestureHandler;
