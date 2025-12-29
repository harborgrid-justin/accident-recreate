/**
 * Skip Links Component
 *
 * Provides keyboard navigation shortcuts to skip to main content areas.
 * Implements WCAG 2.1 Success Criterion 2.4.1 (Bypass Blocks)
 */

import React, { useEffect, useRef } from 'react';
import type { SkipLink } from '../types';
import { VisuallyHidden } from './VisuallyHidden';

export interface SkipLinksProps {
  links: SkipLink[];
  className?: string;
  visible?: 'always' | 'focus-only';
}

export const SkipLinks: React.FC<SkipLinksProps> = ({
  links,
  className = '',
  visible = 'focus-only',
}) => {
  const containerRef = useRef<HTMLDivElement>(null);

  const handleSkipClick = (targetId: string) => (e: React.MouseEvent) => {
    e.preventDefault();
    const target = document.getElementById(targetId);
    if (target) {
      target.focus();
      target.scrollIntoView({ behavior: 'smooth', block: 'start' });
    }
  };

  const baseStyles: React.CSSProperties = {
    position: 'fixed',
    top: 0,
    left: 0,
    zIndex: 10000,
    padding: '1rem',
    backgroundColor: 'var(--a11y-skip-links-bg, #000)',
    display: 'flex',
    gap: '1rem',
    flexWrap: 'wrap',
  };

  const linkStyles: React.CSSProperties = {
    color: 'var(--a11y-skip-links-color, #fff)',
    backgroundColor: 'var(--a11y-skip-links-link-bg, #333)',
    padding: '0.5rem 1rem',
    textDecoration: 'none',
    borderRadius: '0.25rem',
    fontSize: '1rem',
    fontWeight: 'bold',
    border: '2px solid transparent',
    outline: 'none',
  };

  const focusStyles: React.CSSProperties = {
    borderColor: 'var(--a11y-focus-color, #4A9EFF)',
    boxShadow: '0 0 0 3px var(--a11y-focus-ring, rgba(74, 158, 255, 0.3))',
  };

  if (visible === 'focus-only') {
    return (
      <div
        ref={containerRef}
        className={`skip-links ${className}`}
        style={{
          ...baseStyles,
          transform: 'translateY(-100%)',
          transition: 'transform 0.2s ease-in-out',
        }}
        onFocus={() => {
          if (containerRef.current) {
            containerRef.current.style.transform = 'translateY(0)';
          }
        }}
        onBlur={(e) => {
          if (containerRef.current && !containerRef.current.contains(e.relatedTarget as Node)) {
            containerRef.current.style.transform = 'translateY(-100%)';
          }
        }}
      >
        {links.map((link) => (
          <a
            key={link.id}
            href={`#${link.target}`}
            onClick={handleSkipClick(link.target)}
            style={linkStyles}
            onFocus={(e) => {
              Object.assign(e.currentTarget.style, focusStyles);
            }}
            onBlur={(e) => {
              e.currentTarget.style.borderColor = 'transparent';
              e.currentTarget.style.boxShadow = 'none';
            }}
          >
            {link.label}
          </a>
        ))}
      </div>
    );
  }

  return (
    <nav
      ref={containerRef}
      className={`skip-links ${className}`}
      style={baseStyles}
      aria-label="Skip navigation links"
    >
      {links.map((link) => (
        <a
          key={link.id}
          href={`#${link.target}`}
          onClick={handleSkipClick(link.target)}
          style={linkStyles}
          onFocus={(e) => {
            Object.assign(e.currentTarget.style, focusStyles);
          }}
          onBlur={(e) => {
            e.currentTarget.style.borderColor = 'transparent';
            e.currentTarget.style.boxShadow = 'none';
          }}
        >
          {link.label}
        </a>
      ))}
    </nav>
  );
};

export default SkipLinks;
