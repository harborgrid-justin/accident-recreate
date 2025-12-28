/**
 * ARIA Attribute Helper Utilities
 *
 * Utilities for working with ARIA attributes and roles.
 */

import type { AriaAttributes, AriaRole } from '../types';

/**
 * Build ARIA attributes object
 */
export const buildAriaAttributes = (
  attrs: Partial<AriaAttributes>
): Record<string, string | number | boolean> => {
  const result: Record<string, string | number | boolean> = {};

  Object.entries(attrs).forEach(([key, value]) => {
    if (value !== undefined && value !== null) {
      result[key] = value;
    }
  });

  return result;
};

/**
 * Get required ARIA attributes for a role
 */
export const getRequiredAriaAttributes = (role: AriaRole): string[] => {
  const requirements: Record<string, string[]> = {
    checkbox: ['aria-checked'],
    radio: ['aria-checked'],
    switch: ['aria-checked'],
    combobox: ['aria-expanded', 'aria-controls'],
    heading: ['aria-level'],
    slider: ['aria-valuenow', 'aria-valuemin', 'aria-valuemax'],
    spinbutton: ['aria-valuenow', 'aria-valuemin', 'aria-valuemax'],
    progressbar: ['aria-valuenow'],
    scrollbar: ['aria-valuenow', 'aria-valuemin', 'aria-valuemax'],
  };

  return requirements[role] || [];
};

/**
 * Validate ARIA attributes for a role
 */
export const validateAriaAttributes = (
  role: AriaRole,
  attrs: Partial<AriaAttributes>
): { valid: boolean; missing: string[]; invalid: string[] } => {
  const required = getRequiredAriaAttributes(role);
  const provided = Object.keys(attrs);

  const missing = required.filter((attr) => !provided.includes(attr));
  const invalid: string[] = [];

  // Check for invalid attribute values
  if (attrs['aria-checked'] !== undefined) {
    if (!['true', 'false', 'mixed', true, false].includes(attrs['aria-checked'] as any)) {
      invalid.push('aria-checked');
    }
  }

  if (attrs['aria-pressed'] !== undefined) {
    if (!['true', 'false', 'mixed', true, false].includes(attrs['aria-pressed'] as any)) {
      invalid.push('aria-pressed');
    }
  }

  if (attrs['aria-invalid'] !== undefined) {
    if (!['true', 'false', 'grammar', 'spelling', true, false].includes(attrs['aria-invalid'] as any)) {
      invalid.push('aria-invalid');
    }
  }

  return {
    valid: missing.length === 0 && invalid.length === 0,
    missing,
    invalid,
  };
};

/**
 * Generate ARIA label from element content
 */
export const generateAriaLabel = (element: HTMLElement): string => {
  // Check for existing aria-label
  if (element.hasAttribute('aria-label')) {
    return element.getAttribute('aria-label') || '';
  }

  // Check for aria-labelledby
  if (element.hasAttribute('aria-labelledby')) {
    const ids = element.getAttribute('aria-labelledby')?.split(' ') || [];
    const labels = ids
      .map((id) => document.getElementById(id)?.textContent?.trim())
      .filter(Boolean);
    return labels.join(' ');
  }

  // Check for label element (for form inputs)
  if (element.id) {
    const label = document.querySelector(`label[for="${element.id}"]`);
    if (label?.textContent) {
      return label.textContent.trim();
    }
  }

  // Use element text content
  return element.textContent?.trim() || '';
};

/**
 * Check if element has accessible name
 */
export const hasAccessibleName = (element: HTMLElement): boolean => {
  if (element.hasAttribute('aria-label')) return true;
  if (element.hasAttribute('aria-labelledby')) return true;
  if (element.hasAttribute('title')) return true;

  // Check for associated label
  if (element.id && document.querySelector(`label[for="${element.id}"]`)) {
    return true;
  }

  // Check for text content (for links, buttons)
  if (['A', 'BUTTON'].includes(element.tagName) && element.textContent?.trim()) {
    return true;
  }

  return false;
};

/**
 * Build button ARIA attributes
 */
export const buttonAriaProps = (
  label?: string,
  pressed?: boolean,
  expanded?: boolean,
  controls?: string
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-label': label,
    'aria-pressed': pressed,
    'aria-expanded': expanded,
    'aria-controls': controls,
  }) as AriaAttributes;
};

/**
 * Build checkbox ARIA attributes
 */
export const checkboxAriaProps = (
  label?: string,
  checked?: boolean | 'mixed',
  describedBy?: string
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-label': label,
    'aria-checked': checked,
    'aria-describedby': describedBy,
  }) as AriaAttributes;
};

/**
 * Build combobox ARIA attributes
 */
export const comboboxAriaProps = (
  label?: string,
  expanded?: boolean,
  controls?: string,
  activeDescendant?: string
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-label': label,
    'aria-expanded': expanded,
    'aria-controls': controls,
    'aria-activedescendant': activeDescendant,
    'aria-haspopup': 'listbox',
  }) as AriaAttributes;
};

/**
 * Build dialog ARIA attributes
 */
export const dialogAriaProps = (
  label?: string,
  labelledBy?: string,
  describedBy?: string,
  modal = true
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-label': label,
    'aria-labelledby': labelledBy,
    'aria-describedby': describedBy,
    'aria-modal': modal,
  }) as AriaAttributes;
};

/**
 * Build tab ARIA attributes
 */
export const tabAriaProps = (
  selected: boolean,
  controls?: string,
  labelledBy?: string
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-selected': selected,
    'aria-controls': controls,
    'aria-labelledby': labelledBy,
  }) as AriaAttributes;
};

/**
 * Build live region ARIA attributes
 */
export const liveRegionAriaProps = (
  priority: 'polite' | 'assertive',
  atomic = true,
  relevant?: string
): AriaAttributes => {
  return buildAriaAttributes({
    'aria-live': priority,
    'aria-atomic': atomic,
    'aria-relevant': relevant,
  }) as AriaAttributes;
};

/**
 * Create describedby ID list
 */
export const createDescribedBy = (...ids: (string | undefined)[]): string | undefined => {
  const validIds = ids.filter((id): id is string => Boolean(id));
  return validIds.length > 0 ? validIds.join(' ') : undefined;
};

/**
 * Create labelledby ID list
 */
export const createLabelledBy = (...ids: (string | undefined)[]): string | undefined => {
  const validIds = ids.filter((id): id is string => Boolean(id));
  return validIds.length > 0 ? validIds.join(' ') : undefined;
};

/**
 * Format ARIA value text
 */
export const formatValueText = (value: number, min: number, max: number, unit?: string): string => {
  const percentage = ((value - min) / (max - min)) * 100;
  return unit ? `${value} ${unit}` : `${percentage.toFixed(0)}%`;
};
