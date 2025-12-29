/**
 * AccuScene Enterprise Gesture System
 * Version: 0.2.5
 *
 * Comprehensive mobile gesture recognition and handling
 */

// Types
export * from './types';

// Utilities
export * from './utils/touchMath';

// Recognizers
export * from './recognizers/GestureRecognizer';
export * from './recognizers/MultiTouchRecognizer';

// Hooks
export { useGesture } from './hooks/useGesture';
export { usePinchZoom } from './hooks/usePinchZoom';
export { useSwipe } from './hooks/useSwipe';
export { usePan } from './hooks/usePan';
export { useLongPress } from './hooks/useLongPress';

// Components
export { GestureHandler } from './GestureHandler';
export { TouchableArea } from './components/TouchableArea';

// Default export
export { GestureHandler as default } from './GestureHandler';
