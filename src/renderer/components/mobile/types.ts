/**
 * AccuScene Enterprise v0.3.0
 * Mobile Component Types
 *
 * Comprehensive TypeScript type definitions for mobile-first responsive GUI system
 */

import { CSSProperties, ReactNode } from 'react';

// ============================================================================
// Breakpoint and Responsive Types
// ============================================================================

export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';

export interface BreakpointConfig {
  xs: number;  // 0px - Extra small devices (phones)
  sm: number;  // 640px - Small devices (large phones)
  md: number;  // 768px - Medium devices (tablets)
  lg: number;  // 1024px - Large devices (desktops)
  xl: number;  // 1280px - Extra large devices
  '2xl': number; // 1536px - 2X Extra large devices
}

export interface ResponsiveValue<T> {
  xs?: T;
  sm?: T;
  md?: T;
  lg?: T;
  xl?: T;
  '2xl'?: T;
}

// ============================================================================
// Gesture and Touch Types
// ============================================================================

export type GestureType =
  | 'tap'
  | 'doubleTap'
  | 'longPress'
  | 'swipe'
  | 'pinch'
  | 'rotate'
  | 'pan'
  | 'drag';

export type SwipeDirection = 'left' | 'right' | 'up' | 'down';

export interface Point {
  x: number;
  y: number;
}

export interface GestureState {
  type: GestureType;
  startPoint: Point;
  currentPoint: Point;
  deltaX: number;
  deltaY: number;
  distance: number;
  angle: number;
  velocity: number;
  scale: number;
  rotation: number;
  timestamp: number;
}

export interface GestureHandlers {
  onTap?: (event: TouchEvent, point: Point) => void;
  onDoubleTap?: (event: TouchEvent, point: Point) => void;
  onLongPress?: (event: TouchEvent, point: Point) => void;
  onSwipe?: (event: TouchEvent, direction: SwipeDirection, distance: number) => void;
  onPinch?: (event: TouchEvent, scale: number, center: Point) => void;
  onRotate?: (event: TouchEvent, rotation: number, center: Point) => void;
  onPan?: (event: TouchEvent, delta: Point) => void;
  onDrag?: (event: TouchEvent, position: Point) => void;
  onGestureStart?: (event: TouchEvent) => void;
  onGestureEnd?: (event: TouchEvent) => void;
}

export interface TouchState {
  touching: boolean;
  touchCount: number;
  touches: Point[];
  timestamp: number;
}

// ============================================================================
// Navigation Types
// ============================================================================

export interface NavigationTab {
  id: string;
  label: string;
  icon: ReactNode;
  badge?: number | string;
  disabled?: boolean;
  visible?: boolean;
  path?: string;
}

export interface DrawerMenuItem {
  id: string;
  label: string;
  icon?: ReactNode;
  action?: () => void;
  path?: string;
  divider?: boolean;
  children?: DrawerMenuItem[];
  badge?: number | string;
  disabled?: boolean;
}

// ============================================================================
// Layout Types
// ============================================================================

export interface MobileLayoutProps {
  children: ReactNode;
  header?: ReactNode;
  footer?: ReactNode;
  navigation?: ReactNode;
  drawer?: ReactNode;
  backgroundColor?: string;
  fullScreen?: boolean;
  safeArea?: boolean;
  className?: string;
}

export type Orientation = 'portrait' | 'landscape';

export interface OrientationState {
  orientation: Orientation;
  angle: number;
  locked: boolean;
}

// ============================================================================
// Canvas and Drawing Types
// ============================================================================

export interface CanvasState {
  scale: number;
  offsetX: number;
  offsetY: number;
  rotation: number;
  width: number;
  height: number;
}

export interface DrawingTool {
  type: 'pen' | 'eraser' | 'select' | 'shape' | 'text';
  color?: string;
  width?: number;
  opacity?: number;
}

// ============================================================================
// Property Sheet Types
// ============================================================================

export interface PropertyField {
  id: string;
  label: string;
  type: 'text' | 'number' | 'select' | 'toggle' | 'slider' | 'color' | 'date';
  value: any;
  options?: Array<{ label: string; value: any }>;
  min?: number;
  max?: number;
  step?: number;
  placeholder?: string;
  required?: boolean;
  disabled?: boolean;
  onChange: (value: any) => void;
}

export interface PropertySection {
  id: string;
  title: string;
  fields: PropertyField[];
  collapsed?: boolean;
}

// ============================================================================
// Timeline Types
// ============================================================================

export interface TimelineMarker {
  id: string;
  time: number;
  label?: string;
  color?: string;
  type?: 'keyframe' | 'event' | 'marker';
}

export interface TimelineProps {
  duration: number;
  currentTime: number;
  markers?: TimelineMarker[];
  onSeek: (time: number) => void;
  onPlay?: () => void;
  onPause?: () => void;
  playing?: boolean;
  fps?: number;
}

// ============================================================================
// Object List Types
// ============================================================================

export interface ListItem {
  id: string;
  title: string;
  subtitle?: string;
  icon?: ReactNode;
  thumbnail?: string;
  metadata?: Record<string, any>;
  selected?: boolean;
  disabled?: boolean;
}

export interface SwipeAction {
  id: string;
  label: string;
  icon?: ReactNode;
  color?: string;
  backgroundColor?: string;
  onAction: (item: ListItem) => void;
}

// ============================================================================
// Camera and Scanner Types
// ============================================================================

export type CameraFacing = 'user' | 'environment';

export interface CameraConstraints {
  facing: CameraFacing;
  width?: number;
  height?: number;
  aspectRatio?: number;
  frameRate?: number;
}

export interface CapturedMedia {
  id: string;
  type: 'photo' | 'video';
  blob: Blob;
  url: string;
  timestamp: number;
  metadata?: {
    width: number;
    height: number;
    size: number;
    duration?: number;
  };
}

export interface ScanResult {
  type: 'qr' | 'barcode' | 'document';
  data: string;
  format?: string;
  timestamp: number;
  confidence?: number;
}

// ============================================================================
// Voice Input Types
// ============================================================================

export interface VoiceRecognitionConfig {
  language?: string;
  continuous?: boolean;
  interimResults?: boolean;
  maxAlternatives?: number;
}

export interface VoiceTranscript {
  text: string;
  confidence: number;
  isFinal: boolean;
  timestamp: number;
}

// ============================================================================
// Haptic Feedback Types
// ============================================================================

export type HapticPattern =
  | 'light'
  | 'medium'
  | 'heavy'
  | 'success'
  | 'warning'
  | 'error'
  | 'selection';

export interface HapticOptions {
  pattern?: HapticPattern;
  duration?: number;
  intensity?: number;
}

// ============================================================================
// Notification Types
// ============================================================================

export type NotificationVariant = 'info' | 'success' | 'warning' | 'error';

export interface Notification {
  id: string;
  message: string;
  variant?: NotificationVariant;
  duration?: number;
  icon?: ReactNode;
  action?: {
    label: string;
    onClick: () => void;
  };
  dismissible?: boolean;
  timestamp: number;
}

// ============================================================================
// Sync and Offline Types
// ============================================================================

export interface SyncStatus {
  online: boolean;
  syncing: boolean;
  lastSync?: Date;
  pendingChanges: number;
  error?: string;
}

export interface SyncProgress {
  total: number;
  completed: number;
  failed: number;
  inProgress: number;
  percentage: number;
  estimatedTimeRemaining?: number;
}

// ============================================================================
// Floating Action Button Types
// ============================================================================

export interface FABAction {
  id: string;
  label: string;
  icon: ReactNode;
  onClick: () => void;
  color?: string;
  disabled?: boolean;
}

export type FABPosition =
  | 'bottom-right'
  | 'bottom-left'
  | 'bottom-center'
  | 'top-right'
  | 'top-left'
  | 'top-center';

// ============================================================================
// Image and Media Types
// ============================================================================

export interface ImageSource {
  src: string;
  width: number;
  height: number;
  type?: string;
}

export interface ResponsiveImageProps {
  src: string;
  alt: string;
  sources?: ImageSource[];
  loading?: 'lazy' | 'eager';
  objectFit?: CSSProperties['objectFit'];
  placeholder?: string;
  onLoad?: () => void;
  onError?: () => void;
  className?: string;
}

// ============================================================================
// Skeleton Loader Types
// ============================================================================

export type SkeletonVariant = 'text' | 'circular' | 'rectangular' | 'rounded';

export interface SkeletonProps {
  variant?: SkeletonVariant;
  width?: number | string;
  height?: number | string;
  animation?: 'pulse' | 'wave' | 'none';
  count?: number;
  className?: string;
}

// ============================================================================
// Pull to Refresh Types
// ============================================================================

export interface PullToRefreshState {
  pulling: boolean;
  pullDistance: number;
  refreshing: boolean;
  threshold: number;
}

// ============================================================================
// Signature Types
// ============================================================================

export interface SignatureData {
  points: Point[];
  timestamp: number;
  blob?: Blob;
  dataUrl?: string;
}

export interface SignatureOptions {
  penColor?: string;
  penWidth?: number;
  backgroundColor?: string;
  minStrokeWidth?: number;
  maxStrokeWidth?: number;
  velocityFilterWeight?: number;
}

// ============================================================================
// Animation Types
// ============================================================================

export interface AnimationConfig {
  duration: number;
  easing?: 'linear' | 'ease-in' | 'ease-out' | 'ease-in-out' | 'spring';
  delay?: number;
}

export interface SpringConfig {
  tension?: number;
  friction?: number;
  mass?: number;
}

// ============================================================================
// Export Default Breakpoint Configuration
// ============================================================================

export const DEFAULT_BREAKPOINTS: BreakpointConfig = {
  xs: 0,
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
  '2xl': 1536,
};

// Minimum touch target size (44x44px per iOS and Android guidelines)
export const MIN_TOUCH_TARGET = 44;

// Animation frame rate target
export const TARGET_FPS = 60;
export const FRAME_TIME = 1000 / TARGET_FPS;
