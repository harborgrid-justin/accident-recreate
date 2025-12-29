/**
 * AccuScene Enterprise v0.3.0
 * Mobile Property Sheet Component
 *
 * Bottom sheet property editor optimized for mobile
 */

import React, { useState, useEffect, useRef, CSSProperties, ReactNode } from 'react';
import { PropertySection, PropertyField } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface MobilePropertySheetProps {
  isOpen: boolean;
  onClose: () => void;
  sections: PropertySection[];
  title?: string;
  height?: number | 'auto' | 'full';
  draggable?: boolean;
  className?: string;
}

/**
 * Bottom sheet for property editing on mobile
 * Supports draggable dismiss and section collapsing
 *
 * @example
 * ```tsx
 * <MobilePropertySheet
 *   isOpen={sheetOpen}
 *   onClose={() => setSheetOpen(false)}
 *   title="Edit Properties"
 *   sections={propertySections}
 *   draggable
 * />
 * ```
 */
export const MobilePropertySheet: React.FC<MobilePropertySheetProps> = ({
  isOpen,
  onClose,
  sections,
  title = 'Properties',
  height = 'auto',
  draggable = true,
  className = '',
}) => {
  const [translateY, setTranslateY] = useState(0);
  const [isDragging, setIsDragging] = useState(false);
  const [collapsedSections, setCollapsedSections] = useState<Set<string>>(new Set());

  const sheetRef = useRef<HTMLDivElement>(null);
  const startYRef = useRef<number>(0);
  const sheetHeightRef = useRef<number>(0);

  useEffect(() => {
    if (isOpen) {
      document.body.style.overflow = 'hidden';
      HapticFeedback.light();
    } else {
      document.body.style.overflow = '';
      setTranslateY(0);
    }

    return () => {
      document.body.style.overflow = '';
    };
  }, [isOpen]);

  const handleTouchStart = (e: React.TouchEvent) => {
    if (!draggable) return;

    const touch = e.touches[0];
    startYRef.current = touch.clientY;
    setIsDragging(true);

    if (sheetRef.current) {
      sheetHeightRef.current = sheetRef.current.getBoundingClientRect().height;
    }
  };

  const handleTouchMove = (e: React.TouchEvent) => {
    if (!isDragging || !draggable) return;

    const touch = e.touches[0];
    const delta = touch.clientY - startYRef.current;

    // Only allow downward drag
    if (delta > 0) {
      setTranslateY(delta);
    }
  };

  const handleTouchEnd = () => {
    if (!isDragging) return;

    setIsDragging(false);

    // Close if dragged more than 30%
    const threshold = sheetHeightRef.current * 0.3;
    if (translateY > threshold) {
      HapticFeedback.light();
      onClose();
    } else {
      setTranslateY(0);
    }
  };

  const handleOverlayClick = (e: React.MouseEvent) => {
    if (e.target === e.currentTarget) {
      HapticFeedback.light();
      onClose();
    }
  };

  const toggleSection = (sectionId: string) => {
    setCollapsedSections((prev) => {
      const next = new Set(prev);
      if (next.has(sectionId)) {
        next.delete(sectionId);
      } else {
        next.add(sectionId);
      }
      return next;
    });
    HapticFeedback.selection();
  };

  const renderField = (field: PropertyField) => {
    const fieldContainerStyles: CSSProperties = {
      marginBottom: '1rem',
    };

    const labelStyles: CSSProperties = {
      display: 'block',
      fontSize: '0.875rem',
      fontWeight: 500,
      marginBottom: '0.5rem',
      color: '#000000',
    };

    const inputBaseStyles: CSSProperties = {
      width: '100%',
      padding: '0.75rem',
      fontSize: '1rem',
      border: '1px solid #e0e0e0',
      borderRadius: '8px',
      backgroundColor: '#ffffff',
      outline: 'none',
      transition: 'border-color 0.2s ease',
      minHeight: '44px',
    };

    switch (field.type) {
      case 'text':
      case 'number':
        return (
          <div key={field.id} style={fieldContainerStyles}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}
              {field.required && <span style={{ color: '#FF3B30' }}> *</span>}
            </label>
            <input
              id={field.id}
              type={field.type}
              value={field.value || ''}
              onChange={(e) => field.onChange(e.target.value)}
              placeholder={field.placeholder}
              disabled={field.disabled}
              required={field.required}
              min={field.min}
              max={field.max}
              step={field.step}
              style={inputBaseStyles}
            />
          </div>
        );

      case 'select':
        return (
          <div key={field.id} style={fieldContainerStyles}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}
              {field.required && <span style={{ color: '#FF3B30' }}> *</span>}
            </label>
            <select
              id={field.id}
              value={field.value || ''}
              onChange={(e) => field.onChange(e.target.value)}
              disabled={field.disabled}
              required={field.required}
              style={{ ...inputBaseStyles, cursor: 'pointer' }}
            >
              <option value="">{field.placeholder || 'Select...'}</option>
              {field.options?.map((option) => (
                <option key={option.value} value={option.value}>
                  {option.label}
                </option>
              ))}
            </select>
          </div>
        );

      case 'toggle':
        return (
          <div key={field.id} style={{ ...fieldContainerStyles, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}
            </label>
            <input
              id={field.id}
              type="checkbox"
              checked={!!field.value}
              onChange={(e) => {
                field.onChange(e.target.checked);
                HapticFeedback.toggle(e.target.checked);
              }}
              disabled={field.disabled}
              style={{ width: '44px', height: '24px', cursor: 'pointer' }}
            />
          </div>
        );

      case 'slider':
        return (
          <div key={field.id} style={fieldContainerStyles}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}: {field.value}
            </label>
            <input
              id={field.id}
              type="range"
              value={field.value || field.min || 0}
              onChange={(e) => {
                field.onChange(Number(e.target.value));
                HapticFeedback.sliderChange();
              }}
              min={field.min}
              max={field.max}
              step={field.step}
              disabled={field.disabled}
              style={{ width: '100%', minHeight: '44px' }}
            />
          </div>
        );

      case 'color':
        return (
          <div key={field.id} style={fieldContainerStyles}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}
            </label>
            <input
              id={field.id}
              type="color"
              value={field.value || '#000000'}
              onChange={(e) => field.onChange(e.target.value)}
              disabled={field.disabled}
              style={{ ...inputBaseStyles, height: '44px', cursor: 'pointer' }}
            />
          </div>
        );

      case 'date':
        return (
          <div key={field.id} style={fieldContainerStyles}>
            <label htmlFor={field.id} style={labelStyles}>
              {field.label}
            </label>
            <input
              id={field.id}
              type="date"
              value={field.value || ''}
              onChange={(e) => field.onChange(e.target.value)}
              disabled={field.disabled}
              style={inputBaseStyles}
            />
          </div>
        );

      default:
        return null;
    }
  };

  const getSheetHeight = (): string => {
    if (height === 'full') return '90vh';
    if (height === 'auto') return 'auto';
    return `${height}px`;
  };

  const overlayStyles: CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(0, 0, 0, 0.5)',
    zIndex: 1000,
    opacity: isOpen ? 1 : 0,
    visibility: isOpen ? 'visible' : 'hidden',
    transition: 'opacity 0.3s ease, visibility 0.3s ease',
  };

  const sheetStyles: CSSProperties = {
    position: 'fixed',
    left: 0,
    right: 0,
    bottom: 0,
    maxHeight: getSheetHeight(),
    backgroundColor: '#ffffff',
    borderTopLeftRadius: '20px',
    borderTopRightRadius: '20px',
    boxShadow: '0 -2px 16px rgba(0, 0, 0, 0.2)',
    zIndex: 1001,
    transform: isOpen
      ? `translateY(${translateY}px)`
      : 'translateY(100%)',
    transition: isDragging ? 'none' : 'transform 0.3s cubic-bezier(0.4, 0, 0.2, 1)',
    display: 'flex',
    flexDirection: 'column',
    overflowY: 'auto',
  };

  const handleStyles: CSSProperties = {
    width: '40px',
    height: '4px',
    backgroundColor: '#d0d0d0',
    borderRadius: '2px',
    margin: '0.75rem auto',
    cursor: draggable ? 'grab' : 'default',
  };

  const headerStyles: CSSProperties = {
    padding: '1rem 1.5rem',
    borderBottom: '1px solid #e0e0e0',
    fontSize: '1.25rem',
    fontWeight: 600,
  };

  const contentStyles: CSSProperties = {
    padding: '1.5rem',
    overflowY: 'auto',
    flex: 1,
  };

  const sectionHeaderStyles = (isCollapsed: boolean): CSSProperties => ({
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
    padding: '0.75rem 0',
    fontSize: '1rem',
    fontWeight: 600,
    cursor: 'pointer',
    userSelect: 'none',
    minHeight: '44px',
  });

  if (!isOpen) return null;

  return (
    <>
      <div
        className="mobile-property-sheet__overlay"
        style={overlayStyles}
        onClick={handleOverlayClick}
        aria-hidden="true"
      />
      <div
        ref={sheetRef}
        className={`mobile-property-sheet ${className}`}
        style={sheetStyles}
        role="dialog"
        aria-label={title}
        data-testid="mobile-property-sheet"
      >
        <div
          className="mobile-property-sheet__handle"
          style={handleStyles}
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
          aria-hidden="true"
        />

        <div className="mobile-property-sheet__header" style={headerStyles}>
          {title}
        </div>

        <div className="mobile-property-sheet__content" style={contentStyles}>
          {sections.map((section) => {
            const isCollapsed = collapsedSections.has(section.id);

            return (
              <div key={section.id} className="mobile-property-sheet__section">
                <div
                  className="mobile-property-sheet__section-header"
                  style={sectionHeaderStyles(isCollapsed)}
                  onClick={() => toggleSection(section.id)}
                >
                  <span>{section.title}</span>
                  <span style={{ transform: isCollapsed ? 'rotate(0deg)' : 'rotate(180deg)', transition: 'transform 0.2s' }}>
                    ▼
                  </span>
                </div>

                {!isCollapsed && (
                  <div className="mobile-property-sheet__section-content">
                    {section.fields.map(renderField)}
                  </div>
                )}
              </div>
            );
          })}
        </div>

        <style>{`
          .mobile-property-sheet::-webkit-scrollbar {
            display: none;
          }

          .mobile-property-sheet {
            -ms-overflow-style: none;
            scrollbar-width: none;
          }

          .mobile-property-sheet input:focus,
          .mobile-property-sheet select:focus,
          .mobile-property-sheet textarea:focus {
            border-color: #007AFF;
          }

          /* Dark mode support */
          @media (prefers-color-scheme: dark) {
            .mobile-property-sheet {
              background-color: #1c1c1e;
              color: #ffffff;
            }

            .mobile-property-sheet__header {
              border-color: #38383a;
            }

            .mobile-property-sheet input,
            .mobile-property-sheet select {
              background-color: #2c2c2e;
              border-color: #48484a;
              color: #ffffff;
            }
          }

          /* Reduce motion */
          @media (prefers-reduced-motion: reduce) {
            .mobile-property-sheet,
            .mobile-property-sheet__overlay {
              transition: none !important;
            }
          }
        `}</style>
      </div>
    </>
  );
};

export default MobilePropertySheet;
