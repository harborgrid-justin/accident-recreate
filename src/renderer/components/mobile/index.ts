/**
 * AccuScene Enterprise v0.3.0
 * Mobile Components Module Exports
 *
 * Comprehensive mobile-first responsive GUI system
 * Production-ready components for iOS and Android
 */

// ============================================================================
// Types
// ============================================================================
export * from './types';

// ============================================================================
// Hooks
// ============================================================================
export { useBreakpoint } from './hooks/useBreakpoint';
export { useGesture } from './hooks/useGesture';
export { useOrientation } from './hooks/useOrientation';
export { useVibration, useHapticFeedback } from './hooks/useVibration';

// ============================================================================
// Utilities
// ============================================================================
export { HapticFeedback, withHaptic, withHapticClick } from './HapticFeedback';
export { OrientationLock, OrientationLockWrapper } from './OrientationLock';

// ============================================================================
// Core Layout Components
// ============================================================================
export { MobileLayout } from './MobileLayout';
export type { MobileLayoutProps } from './types';

export { MobileNavigation } from './MobileNavigation';
export type { MobileNavigationProps } from './MobileNavigation';

export { MobileDrawer } from './MobileDrawer';
export type { MobileDrawerProps } from './MobileDrawer';

export { MobileToolbar } from './MobileToolbar';
export type { MobileToolbarProps, ToolbarAction } from './MobileToolbar';

// ============================================================================
// Touch Interaction Components
// ============================================================================
export { TouchCanvas } from './TouchCanvas';
export type { TouchCanvasProps } from './TouchCanvas';

export { GestureHandler } from './GestureHandler';
export type { GestureHandlerProps } from './GestureHandler';

export { PinchZoom } from './PinchZoom';
export type { PinchZoomProps } from './PinchZoom';

export { SwipeActions } from './SwipeActions';
export type { SwipeActionsProps } from './SwipeActions';

export { PullToRefresh } from './PullToRefresh';
export type { PullToRefreshProps } from './PullToRefresh';

// ============================================================================
// Mobile-Specific UI Components
// ============================================================================
export { MobilePropertySheet } from './MobilePropertySheet';
export type { MobilePropertySheetProps } from './MobilePropertySheet';

export { MobileTimeline } from './MobileTimeline';

export { MobileObjectList } from './MobileObjectList';
export type { MobileObjectListProps } from './MobileObjectList';

// ============================================================================
// Device Feature Components
// ============================================================================
export { MobileCamera } from './MobileCamera';
export type { MobileCameraProps } from './MobileCamera';

export { MobileScanner } from './MobileScanner';
export type { MobileScannerProps } from './MobileScanner';

export { MobileSignature } from './MobileSignature';
export type { MobileSignatureProps } from './MobileSignature';

export { VoiceInput } from './VoiceInput';
export type { VoiceInputProps } from './VoiceInput';

// ============================================================================
// Status and Utility Components
// ============================================================================
export { OfflineIndicator } from './OfflineIndicator';
export type { OfflineIndicatorProps } from './OfflineIndicator';

export { SyncProgress } from './SyncProgress';
export type { SyncProgressProps } from './SyncProgress';

export { MobileNotifications, useMobileNotifications } from './MobileNotifications';
export type { MobileNotificationsProps } from './MobileNotifications';

export { AdaptiveImage } from './AdaptiveImage';

export { SkeletonLoader, SkeletonLayouts } from './SkeletonLoader';

export { FloatingActionButton } from './FloatingActionButton';
export type { FloatingActionButtonProps } from './FloatingActionButton';

// ============================================================================
// Default Exports (convenience)
// ============================================================================
export default {
  // Layout
  MobileLayout,
  MobileNavigation,
  MobileDrawer,
  MobileToolbar,

  // Touch
  TouchCanvas,
  GestureHandler,
  PinchZoom,
  SwipeActions,
  PullToRefresh,

  // UI Components
  MobilePropertySheet,
  MobileTimeline,
  MobileObjectList,

  // Device Features
  MobileCamera,
  MobileScanner,
  MobileSignature,
  VoiceInput,

  // Status & Utility
  OfflineIndicator,
  SyncProgress,
  MobileNotifications,
  AdaptiveImage,
  SkeletonLoader,
  FloatingActionButton,

  // Utilities
  HapticFeedback,
  OrientationLock,

  // Hooks
  useBreakpoint,
  useGesture,
  useOrientation,
  useVibration,
  useHapticFeedback,
  useMobileNotifications,

  // Layouts
  SkeletonLayouts,
};
