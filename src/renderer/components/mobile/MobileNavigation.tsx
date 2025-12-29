/**
 * AccuScene Enterprise v0.3.0
 * Mobile Navigation Component
 *
 * Bottom tab navigation optimized for mobile devices
 */

import React, { useState, CSSProperties } from 'react';
import { NavigationTab } from './types';
import { HapticFeedback } from './HapticFeedback';
import { useBreakpoint } from './hooks/useBreakpoint';

export interface MobileNavigationProps {
  tabs: NavigationTab[];
  activeTab?: string;
  onTabChange?: (tabId: string) => void;
  backgroundColor?: string;
  activeColor?: string;
  inactiveColor?: string;
  showLabels?: boolean;
  position?: 'bottom' | 'top';
  className?: string;
}

/**
 * Bottom tab navigation component for mobile
 * Follows iOS and Android navigation patterns
 *
 * @example
 * ```tsx
 * <MobileNavigation
 *   tabs={[
 *     { id: 'home', label: 'Home', icon: <HomeIcon /> },
 *     { id: 'search', label: 'Search', icon: <SearchIcon /> },
 *   ]}
 *   activeTab="home"
 *   onTabChange={(id) => navigate(id)}
 * />
 * ```
 */
export const MobileNavigation: React.FC<MobileNavigationProps> = ({
  tabs,
  activeTab,
  onTabChange,
  backgroundColor = '#ffffff',
  activeColor = '#007AFF',
  inactiveColor = '#8E8E93',
  showLabels = true,
  position = 'bottom',
  className = '',
}) => {
  const [currentTab, setCurrentTab] = useState(activeTab || tabs[0]?.id);
  const { isMobile } = useBreakpoint();

  const handleTabClick = (tab: NavigationTab) => {
    if (tab.disabled) return;

    HapticFeedback.selection();
    setCurrentTab(tab.id);
    onTabChange?.(tab.id);
  };

  const visibleTabs = tabs.filter((tab) => tab.visible !== false);

  const navStyles: CSSProperties = {
    display: 'flex',
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'space-around',
    width: '100%',
    backgroundColor,
    borderTop: position === 'bottom' ? '1px solid #e0e0e0' : 'none',
    borderBottom: position === 'top' ? '1px solid #e0e0e0' : 'none',
    padding: '0.25rem 0',
    minHeight: showLabels ? '56px' : '48px',
    position: 'relative',
    zIndex: 100,
    boxShadow:
      position === 'bottom'
        ? '0 -2px 8px rgba(0, 0, 0, 0.1)'
        : '0 2px 8px rgba(0, 0, 0, 0.1)',
  };

  const tabItemStyles = (isActive: boolean, disabled: boolean): CSSProperties => ({
    flex: 1,
    display: 'flex',
    flexDirection: 'column',
    alignItems: 'center',
    justifyContent: 'center',
    gap: '0.25rem',
    padding: '0.5rem',
    cursor: disabled ? 'not-allowed' : 'pointer',
    opacity: disabled ? 0.4 : 1,
    color: isActive ? activeColor : inactiveColor,
    transition: 'color 0.2s ease, transform 0.1s ease',
    userSelect: 'none',
    WebkitTapHighlightColor: 'transparent',
    minWidth: '44px',
    minHeight: '44px',
    position: 'relative',
    textDecoration: 'none',
  });

  const iconStyles: CSSProperties = {
    fontSize: isMobile ? '24px' : '22px',
    lineHeight: 1,
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
  };

  const labelStyles: CSSProperties = {
    fontSize: isMobile ? '11px' : '12px',
    fontWeight: 500,
    lineHeight: 1.2,
    textAlign: 'center',
    maxWidth: '100%',
    overflow: 'hidden',
    textOverflow: 'ellipsis',
    whiteSpace: 'nowrap',
  };

  const badgeStyles: CSSProperties = {
    position: 'absolute',
    top: '0.25rem',
    right: '50%',
    transform: 'translateX(50%)',
    backgroundColor: '#FF3B30',
    color: '#ffffff',
    borderRadius: '10px',
    padding: '0.125rem 0.375rem',
    fontSize: '10px',
    fontWeight: 600,
    minWidth: '18px',
    height: '18px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    lineHeight: 1,
  };

  return (
    <nav
      className={`mobile-navigation mobile-navigation--${position} ${className}`}
      style={navStyles}
      role="navigation"
      aria-label="Primary navigation"
      data-testid="mobile-navigation"
    >
      {visibleTabs.map((tab) => {
        const isActive = currentTab === tab.id;
        const TabWrapper = tab.path ? 'a' : 'button';

        return (
          <TabWrapper
            key={tab.id}
            style={tabItemStyles(isActive, !!tab.disabled)}
            onClick={() => handleTabClick(tab)}
            className={`mobile-navigation__tab ${
              isActive ? 'mobile-navigation__tab--active' : ''
            }`}
            role="tab"
            aria-selected={isActive}
            aria-label={tab.label}
            aria-disabled={tab.disabled}
            href={tab.path}
            {...(TabWrapper === 'button' && { type: 'button' })}
          >
            {tab.badge && (
              <span
                className="mobile-navigation__badge"
                style={badgeStyles}
                aria-label={`${tab.badge} notifications`}
              >
                {typeof tab.badge === 'number' && tab.badge > 99 ? '99+' : tab.badge}
              </span>
            )}

            <span className="mobile-navigation__icon" style={iconStyles}>
              {tab.icon}
            </span>

            {showLabels && (
              <span className="mobile-navigation__label" style={labelStyles}>
                {tab.label}
              </span>
            )}

            {isActive && (
              <span
                className="mobile-navigation__active-indicator"
                style={{
                  position: 'absolute',
                  bottom: 0,
                  left: '50%',
                  transform: 'translateX(-50%)',
                  width: '32px',
                  height: '2px',
                  backgroundColor: activeColor,
                  borderRadius: '2px 2px 0 0',
                }}
                aria-hidden="true"
              />
            )}
          </TabWrapper>
        );
      })}

      <style>{`
        .mobile-navigation__tab {
          border: none;
          background: none;
          outline: none;
        }

        .mobile-navigation__tab:active {
          transform: scale(0.95);
        }

        .mobile-navigation__tab:focus-visible {
          outline: 2px solid ${activeColor};
          outline-offset: 2px;
          border-radius: 4px;
        }

        /* Ripple effect on tap */
        .mobile-navigation__tab::before {
          content: '';
          position: absolute;
          top: 50%;
          left: 50%;
          width: 0;
          height: 0;
          border-radius: 50%;
          background-color: currentColor;
          opacity: 0;
          transform: translate(-50%, -50%);
          transition: width 0.3s ease, height 0.3s ease, opacity 0.3s ease;
        }

        .mobile-navigation__tab:active::before {
          width: 100%;
          height: 100%;
          opacity: 0.1;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .mobile-navigation {
            background-color: #1c1c1e;
            border-color: #38383a;
          }
        }

        /* Accessibility: Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .mobile-navigation__tab,
          .mobile-navigation__tab::before {
            transition: none !important;
          }
        }
      `}</style>
    </nav>
  );
};

export default MobileNavigation;
