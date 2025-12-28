/**
 * AccuScene Enterprise Gesture System - TypeScript Type Definitions
 * Version: 0.2.5
 */

export interface TouchPoint {
  id: number;
  x: number;
  y: number;
  timestamp: number;
  pressure: number;
  radiusX: number;
  radiusY: number;
}

export enum SwipeDirection {
  Up = 'UP',
  Down = 'DOWN',
  Left = 'LEFT',
  Right = 'RIGHT',
  UpLeft = 'UP_LEFT',
  UpRight = 'UP_RIGHT',
  DownLeft = 'DOWN_LEFT',
  DownRight = 'DOWN_RIGHT',
}

export enum GesturePhase {
  Began = 'BEGAN',
  Changed = 'CHANGED',
  Ended = 'ENDED',
  Cancelled = 'CANCELLED',
  Failed = 'FAILED',
}

export enum GesturePriority {
  Low = 0,
  Normal = 1,
  High = 2,
  Critical = 3,
}

// Gesture Event Types
export interface TapEvent {
  type: 'tap';
  point: TouchPoint;
  count: number;
  timestamp: number;
}

export interface DoubleTapEvent {
  type: 'doubleTap';
  point: TouchPoint;
  timestamp: number;
}

export interface TripleTapEvent {
  type: 'tripleTap';
  point: TouchPoint;
  timestamp: number;
}

export interface SwipeEvent {
  type: 'swipe';
  start: TouchPoint;
  end: TouchPoint;
  direction: SwipeDirection;
  velocity: number;
  distance: number;
  durationMs: number;
}

export interface PinchStartEvent {
  type: 'pinchStart';
  center: TouchPoint;
  initialDistance: number;
  touch1: TouchPoint;
  touch2: TouchPoint;
}

export interface PinchMoveEvent {
  type: 'pinchMove';
  center: TouchPoint;
  scale: number;
  distance: number;
  velocity: number;
  touch1: TouchPoint;
  touch2: TouchPoint;
}

export interface PinchEndEvent {
  type: 'pinchEnd';
  center: TouchPoint;
  finalScale: number;
  totalScaleChange: number;
}

export interface RotateStartEvent {
  type: 'rotateStart';
  center: TouchPoint;
  initialAngle: number;
  touch1: TouchPoint;
  touch2: TouchPoint;
}

export interface RotateMoveEvent {
  type: 'rotateMove';
  center: TouchPoint;
  angle: number;
  deltaAngle: number;
  angularVelocity: number;
  touch1: TouchPoint;
  touch2: TouchPoint;
}

export interface RotateEndEvent {
  type: 'rotateEnd';
  center: TouchPoint;
  finalAngle: number;
  totalRotation: number;
}

export interface PanStartEvent {
  type: 'panStart';
  point: TouchPoint;
}

export interface PanMoveEvent {
  type: 'panMove';
  point: TouchPoint;
  deltaX: number;
  deltaY: number;
  velocityX: number;
  velocityY: number;
  totalDeltaX: number;
  totalDeltaY: number;
}

export interface PanEndEvent {
  type: 'panEnd';
  point: TouchPoint;
  totalDeltaX: number;
  totalDeltaY: number;
  finalVelocityX: number;
  finalVelocityY: number;
}

export interface LongPressStartEvent {
  type: 'longPressStart';
  point: TouchPoint;
}

export interface LongPressEvent {
  type: 'longPress';
  point: TouchPoint;
  durationMs: number;
}

export interface LongPressEndEvent {
  type: 'longPressEnd';
  point: TouchPoint;
  totalDurationMs: number;
}

export interface CustomGestureEvent {
  type: 'customGesture';
  name: string;
  points: TouchPoint[];
  confidence: number;
  metadata: Record<string, any>;
}

export type GestureEvent =
  | TapEvent
  | DoubleTapEvent
  | TripleTapEvent
  | SwipeEvent
  | PinchStartEvent
  | PinchMoveEvent
  | PinchEndEvent
  | RotateStartEvent
  | RotateMoveEvent
  | RotateEndEvent
  | PanStartEvent
  | PanMoveEvent
  | PanEndEvent
  | LongPressStartEvent
  | LongPressEvent
  | LongPressEndEvent
  | CustomGestureEvent;

// Configuration Types
export interface TapConfig {
  maxDurationMs: number;
  maxMovement: number;
  multiTapDelayMs: number;
  multiTapDistance: number;
  priority: GesturePriority;
  enableDoubleTap: boolean;
  enableTripleTap: boolean;
}

export interface SwipeConfig {
  minDistance: number;
  minVelocity: number;
  maxDurationMs: number;
  directionTolerance: number;
  priority: GesturePriority;
  enableDiagonal: boolean;
}

export interface PinchConfig {
  minScaleDelta: number;
  maxScale: number;
  minScale: number;
  sensitivity: number;
  priority: GesturePriority;
  allowSimultaneousRotation: boolean;
}

export interface RotateConfig {
  minAngleDelta: number;
  sensitivity: number;
  priority: GesturePriority;
  allowSimultaneousPinch: boolean;
}

export interface PanConfig {
  minDistance: number;
  maxTouches: number;
  minTouches: number;
  priority: GesturePriority;
  enableHorizontal: boolean;
  enableVertical: boolean;
  enableMomentum: boolean;
  momentumDecay: number;
}

export interface LongPressConfig {
  minDurationMs: number;
  maxMovement: number;
  priority: GesturePriority;
  requiredTouches: number;
}

export interface GeneralConfig {
  enableConflictResolution: boolean;
  maxSimultaneousGestures: number;
  sampleRateHz: number;
  enablePrediction: boolean;
  predictionLookaheadMs: number;
  enableHapticFeedback: boolean;
}

export interface GestureConfig {
  tap: TapConfig;
  swipe: SwipeConfig;
  pinch: PinchConfig;
  rotate: RotateConfig;
  pan: PanConfig;
  longPress: LongPressConfig;
  general: GeneralConfig;
}

// Gesture Handler Props
export interface GestureHandlerProps {
  children: React.ReactNode;
  config?: Partial<GestureConfig>;
  onGesture?: (event: GestureEvent) => void;
  onTap?: (event: TapEvent) => void;
  onDoubleTap?: (event: DoubleTapEvent) => void;
  onTripleTap?: (event: TripleTapEvent) => void;
  onSwipe?: (event: SwipeEvent) => void;
  onPinch?: (event: PinchMoveEvent) => void;
  onPinchStart?: (event: PinchStartEvent) => void;
  onPinchEnd?: (event: PinchEndEvent) => void;
  onRotate?: (event: RotateMoveEvent) => void;
  onRotateStart?: (event: RotateStartEvent) => void;
  onRotateEnd?: (event: RotateEndEvent) => void;
  onPan?: (event: PanMoveEvent) => void;
  onPanStart?: (event: PanStartEvent) => void;
  onPanEnd?: (event: PanEndEvent) => void;
  onLongPress?: (event: LongPressEvent) => void;
  onLongPressStart?: (event: LongPressStartEvent) => void;
  onLongPressEnd?: (event: LongPressEndEvent) => void;
  onCustomGesture?: (event: CustomGestureEvent) => void;
  className?: string;
  style?: React.CSSProperties;
  disabled?: boolean;
}

// Hook Return Types
export interface UseGestureResult {
  handlers: {
    onTouchStart: (e: React.TouchEvent) => void;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: (e: React.TouchEvent) => void;
    onTouchCancel: (e: React.TouchEvent) => void;
  };
  state: {
    isActive: boolean;
    activeGestures: string[];
    touchCount: number;
  };
}

export interface UsePinchZoomResult {
  scale: number;
  handlers: {
    onTouchStart: (e: React.TouchEvent) => void;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: (e: React.TouchEvent) => void;
  };
}

export interface UseSwipeResult {
  handlers: {
    onTouchStart: (e: React.TouchEvent) => void;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: (e: React.TouchEvent) => void;
  };
  state: {
    isSwiping: boolean;
    direction: SwipeDirection | null;
  };
}

export interface UsePanResult {
  position: { x: number; y: number };
  handlers: {
    onTouchStart: (e: React.TouchEvent) => void;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: (e: React.TouchEvent) => void;
  };
}

export interface UseLongPressResult {
  handlers: {
    onTouchStart: (e: React.TouchEvent) => void;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: (e: React.TouchEvent) => void;
  };
  state: {
    isPressed: boolean;
    duration: number;
  };
}

// Utility Types
export interface Point2D {
  x: number;
  y: number;
}

export interface VelocityVector {
  x: number;
  y: number;
  magnitude: number;
}

export interface BoundingBox {
  minX: number;
  minY: number;
  maxX: number;
  maxY: number;
  width: number;
  height: number;
}

// Default Configuration
export const DEFAULT_GESTURE_CONFIG: GestureConfig = {
  tap: {
    maxDurationMs: 300,
    maxMovement: 10,
    multiTapDelayMs: 300,
    multiTapDistance: 50,
    priority: GesturePriority.High,
    enableDoubleTap: true,
    enableTripleTap: true,
  },
  swipe: {
    minDistance: 50,
    minVelocity: 100,
    maxDurationMs: 1000,
    directionTolerance: 30,
    priority: GesturePriority.Normal,
    enableDiagonal: true,
  },
  pinch: {
    minScaleDelta: 0.01,
    maxScale: 10,
    minScale: 0.1,
    sensitivity: 1,
    priority: GesturePriority.High,
    allowSimultaneousRotation: true,
  },
  rotate: {
    minAngleDelta: 5,
    sensitivity: 1,
    priority: GesturePriority.Normal,
    allowSimultaneousPinch: true,
  },
  pan: {
    minDistance: 10,
    maxTouches: 1,
    minTouches: 1,
    priority: GesturePriority.Normal,
    enableHorizontal: true,
    enableVertical: true,
    enableMomentum: true,
    momentumDecay: 0.95,
  },
  longPress: {
    minDurationMs: 500,
    maxMovement: 10,
    priority: GesturePriority.Normal,
    requiredTouches: 1,
  },
  general: {
    enableConflictResolution: true,
    maxSimultaneousGestures: 4,
    sampleRateHz: 60,
    enablePrediction: true,
    predictionLookaheadMs: 16,
    enableHapticFeedback: true,
  },
};
