/**
 * AccuScene Enterprise v0.3.0
 * Pull to Refresh Component
 *
 * Pull-down to refresh gesture for scrollable content
 */

import React, { useState, useRef, useCallback, CSSProperties, ReactNode } from 'react';
import { PullToRefreshState } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface PullToRefreshProps {
  children: ReactNode;
  onRefresh: () => Promise<void>;
  threshold?: number;
  resistance?: number;
  refreshingContent?: ReactNode;
  pullingContent?: ReactNode;
  className?: string;
}

/**
 * Pull-to-refresh container for scrollable content
 * Triggers refresh action when pulled down beyond threshold
 *
 * @example
 * ```tsx
 * <PullToRefresh
 *   onRefresh={async () => {
 *     await fetchNewData();
 *   }}
 *   threshold={80}
 * >
 *   <ScrollableContent />
 * </PullToRefresh>
 * ```
 */
export const PullToRefresh: React.FC<PullToRefreshProps> = ({
  children,
  onRefresh,
  threshold = 80,
  resistance = 2.5,
  refreshingContent,
  pullingContent,
  className = '',
}) => {
  const [state, setState] = useState<PullToRefreshState>({
    pulling: false,
    pullDistance: 0,
    refreshing: false,
    threshold,
  });

  const startYRef = useRef<number>(0);
  const currentYRef = useRef<number>(0);
  const scrollElementRef = useRef<HTMLDivElement>(null);
  const hasTriggeredHapticRef = useRef(false);

  const handleTouchStart = useCallback((e: React.TouchEvent) => {
    const scrollElement = scrollElementRef.current;
    if (!scrollElement) return;

    // Only allow pull-to-refresh when scrolled to top
    if (scrollElement.scrollTop === 0) {
      const touch = e.touches[0];
      startYRef.current = touch.clientY;
      currentYRef.current = touch.clientY;
    }
  }, []);

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      const scrollElement = scrollElementRef.current;
      if (!scrollElement || state.refreshing) return;

      const touch = e.touches[0];
      currentYRef.current = touch.clientY;
      const delta = currentYRef.current - startYRef.current;

      // Only pull down when at top of scroll
      if (delta > 0 && scrollElement.scrollTop === 0) {
        e.preventDefault();

        // Apply resistance
        const pullDistance = Math.min(delta / resistance, threshold * 1.5);

        setState((prev) => ({
          ...prev,
          pulling: true,
          pullDistance,
        }));

        // Haptic feedback when crossing threshold
        if (pullDistance >= threshold && !hasTriggeredHapticRef.current) {
          HapticFeedback.medium();
          hasTriggeredHapticRef.current = true;
        } else if (pullDistance < threshold && hasTriggeredHapticRef.current) {
          hasTriggeredHapticRef.current = false;
        }
      }
    },
    [state.refreshing, resistance, threshold]
  );

  const handleTouchEnd = useCallback(async () => {
    hasTriggeredHapticRef.current = false;

    if (state.pullDistance >= threshold && !state.refreshing) {
      // Trigger refresh
      setState((prev) => ({
        ...prev,
        refreshing: true,
        pulling: false,
        pullDistance: threshold,
      }));

      HapticFeedback.success();

      try {
        await onRefresh();
      } catch (error) {
        console.error('Refresh failed:', error);
        HapticFeedback.error();
      } finally {
        // Delay hiding to show completion
        setTimeout(() => {
          setState({
            pulling: false,
            pullDistance: 0,
            refreshing: false,
            threshold,
          });
        }, 300);
      }
    } else {
      // Snap back
      setState({
        pulling: false,
        pullDistance: 0,
        refreshing: false,
        threshold,
      });
    }
  }, [state.pullDistance, state.refreshing, threshold, onRefresh]);

  const containerStyles: CSSProperties = {
    position: 'relative',
    height: '100%',
    overflow: 'hidden',
  };

  const scrollContainerStyles: CSSProperties = {
    height: '100%',
    overflowY: 'auto',
    overflowX: 'hidden',
    WebkitOverflowScrolling: 'touch',
    transform: `translateY(${state.pulling || state.refreshing ? state.pullDistance : 0}px)`,
    transition:
      state.pulling || state.refreshing
        ? 'none'
        : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
  };

  const indicatorStyles: CSSProperties = {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    height: `${threshold}px`,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    transform: `translateY(-${threshold}px) translateY(${
      state.pulling || state.refreshing ? state.pullDistance : 0
    }px)`,
    transition:
      state.pulling || state.refreshing
        ? 'none'
        : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    opacity: state.pullDistance / threshold,
    pointerEvents: 'none',
  };

  const spinnerStyles: CSSProperties = {
    width: '24px',
    height: '24px',
    border: '3px solid #e0e0e0',
    borderTopColor: '#007AFF',
    borderRadius: '50%',
    animation: state.refreshing ? 'spin 0.8s linear infinite' : 'none',
  };

  const renderIndicator = () => {
    if (state.refreshing && refreshingContent) {
      return refreshingContent;
    }

    if (state.pulling && pullingContent) {
      return pullingContent;
    }

    if (state.refreshing) {
      return <div style={spinnerStyles} />;
    }

    return (
      <div
        style={{
          fontSize: '14px',
          color: '#8E8E93',
          textAlign: 'center',
        }}
      >
        {state.pullDistance >= threshold ? 'Release to refresh' : 'Pull to refresh'}
      </div>
    );
  };

  return (
    <div
      className={`pull-to-refresh ${className}`}
      style={containerStyles}
      data-testid="pull-to-refresh"
    >
      {/* Refresh indicator */}
      <div className="pull-to-refresh__indicator" style={indicatorStyles}>
        {renderIndicator()}
      </div>

      {/* Scrollable content */}
      <div
        ref={scrollElementRef}
        className="pull-to-refresh__scroll"
        style={scrollContainerStyles}
        onTouchStart={handleTouchStart}
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onTouchCancel={handleTouchEnd}
      >
        {children}
      </div>

      <style>{`
        @keyframes spin {
          from {
            transform: rotate(0deg);
          }
          to {
            transform: rotate(360deg);
          }
        }

        .pull-to-refresh__scroll::-webkit-scrollbar {
          display: none;
        }

        .pull-to-refresh__scroll {
          -ms-overflow-style: none;
          scrollbar-width: none;
        }

        /* Prevent overscroll bounce on iOS during pull */
        .pull-to-refresh {
          overscroll-behavior-y: contain;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .pull-to-refresh__indicator {
            color: #ffffff;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .pull-to-refresh__scroll,
          .pull-to-refresh__indicator {
            transition: none !important;
          }

          @keyframes spin {
            from {
              transform: rotate(0deg);
            }
            to {
              transform: rotate(0deg);
            }
          }
        }
      `}</style>
    </div>
  );
};

export default PullToRefresh;
