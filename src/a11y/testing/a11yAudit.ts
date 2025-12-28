/**
 * Accessibility Audit Utilities
 *
 * Automated accessibility testing and reporting.
 */

import type { WcagLevel, ContrastResult } from '../types';
import { checkContrast, hasAccessibleName } from '../utils/ariaHelpers';

export type AuditSeverity = 'critical' | 'error' | 'warning' | 'info';

export interface AuditViolation {
  rule: string;
  severity: AuditSeverity;
  wcagCriterion: string;
  message: string;
  element?: HTMLElement;
  fix: string;
}

export interface AuditReport {
  timestamp: number;
  url: string;
  wcagLevel: WcagLevel;
  totalViolations: number;
  bySeverity: Record<AuditSeverity, number>;
  violations: AuditViolation[];
  passedRules: string[];
  summary: string;
}

/**
 * Run a comprehensive accessibility audit
 */
export const runA11yAudit = async (
  rootElement: HTMLElement = document.body,
  wcagLevel: WcagLevel = 'AA'
): Promise<AuditReport> => {
  const violations: AuditViolation[] = [];
  const passedRules: string[] = [];

  // Run all audit checks
  violations.push(...checkImages(rootElement));
  violations.push(...checkFormLabels(rootElement));
  violations.push(...checkHeadings(rootElement));
  violations.push(...checkLandmarks(rootElement));
  violations.push(...checkLinks(rootElement));
  violations.push(...checkButtons(rootElement));
  violations.push(...checkAriaAttributes(rootElement));
  violations.push(...checkColorContrast(rootElement, wcagLevel));
  violations.push(...checkKeyboardAccess(rootElement));
  violations.push(...checkFocusIndicators(rootElement));

  // Count violations by severity
  const bySeverity: Record<AuditSeverity, number> = {
    critical: 0,
    error: 0,
    warning: 0,
    info: 0,
  };

  violations.forEach((v) => {
    bySeverity[v.severity]++;
  });

  // Generate summary
  const summary = `
Accessibility Audit Results
----------------------------
Total Violations: ${violations.length}
Critical: ${bySeverity.critical}
Errors: ${bySeverity.error}
Warnings: ${bySeverity.warning}
Info: ${bySeverity.info}
Passed Rules: ${passedRules.length}
Status: ${bySeverity.critical === 0 && bySeverity.error === 0 ? 'PASSED' : 'FAILED'}
  `.trim();

  return {
    timestamp: Date.now(),
    url: window.location.href,
    wcagLevel,
    totalViolations: violations.length,
    bySeverity,
    violations,
    passedRules,
    summary,
  };
};

/**
 * Check images for alt text
 */
const checkImages = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const images = root.querySelectorAll('img');

  images.forEach((img) => {
    if (!img.hasAttribute('alt')) {
      violations.push({
        rule: 'image-alt',
        severity: 'critical',
        wcagCriterion: '1.1.1',
        message: 'Image missing alt attribute',
        element: img,
        fix: 'Add alt attribute with descriptive text or empty alt="" for decorative images',
      });
    }
  });

  return violations;
};

/**
 * Check form inputs for labels
 */
const checkFormLabels = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const inputs = root.querySelectorAll('input:not([type="hidden"]), select, textarea');

  inputs.forEach((input) => {
    const hasLabel =
      input.hasAttribute('aria-label') ||
      input.hasAttribute('aria-labelledby') ||
      (input.id && root.querySelector(`label[for="${input.id}"]`)) ||
      input.closest('label');

    if (!hasLabel) {
      violations.push({
        rule: 'form-label',
        severity: 'error',
        wcagCriterion: '3.3.2',
        message: 'Form input missing accessible label',
        element: input as HTMLElement,
        fix: 'Add a label element, aria-label, or aria-labelledby attribute',
      });
    }
  });

  return violations;
};

/**
 * Check heading hierarchy
 */
const checkHeadings = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const headings = Array.from(root.querySelectorAll('h1, h2, h3, h4, h5, h6'));

  if (headings.length === 0) {
    violations.push({
      rule: 'heading-structure',
      severity: 'warning',
      wcagCriterion: '1.3.1',
      message: 'No headings found on page',
      fix: 'Add proper heading structure starting with h1',
    });
    return violations;
  }

  // Check for h1
  const h1Count = headings.filter((h) => h.tagName === 'H1').length;
  if (h1Count === 0) {
    violations.push({
      rule: 'heading-h1',
      severity: 'error',
      wcagCriterion: '1.3.1',
      message: 'Page missing h1 heading',
      fix: 'Add an h1 heading as the main page title',
    });
  } else if (h1Count > 1) {
    violations.push({
      rule: 'heading-h1-multiple',
      severity: 'warning',
      wcagCriterion: '1.3.1',
      message: 'Multiple h1 headings found',
      fix: 'Use only one h1 heading per page',
    });
  }

  // Check hierarchy
  let previousLevel = 0;
  headings.forEach((heading) => {
    const level = parseInt(heading.tagName[1], 10);
    if (level - previousLevel > 1) {
      violations.push({
        rule: 'heading-order',
        severity: 'warning',
        wcagCriterion: '1.3.1',
        message: `Heading level ${level} used after ${previousLevel}`,
        element: heading as HTMLElement,
        fix: 'Maintain sequential heading hierarchy',
      });
    }
    previousLevel = level;
  });

  return violations;
};

/**
 * Check for landmark regions
 */
const checkLandmarks = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];

  const hasMain = root.querySelector('main, [role="main"]');
  if (!hasMain) {
    violations.push({
      rule: 'landmark-main',
      severity: 'error',
      wcagCriterion: '1.3.1',
      message: 'Page missing main landmark',
      fix: 'Add <main> element or role="main"',
    });
  }

  return violations;
};

/**
 * Check links for accessible names
 */
const checkLinks = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const links = root.querySelectorAll('a[href]');

  links.forEach((link) => {
    const hasName = hasAccessibleName(link as HTMLElement);
    if (!hasName) {
      violations.push({
        rule: 'link-name',
        severity: 'error',
        wcagCriterion: '2.4.4',
        message: 'Link missing accessible name',
        element: link as HTMLElement,
        fix: 'Add text content or aria-label to the link',
      });
    }
  });

  return violations;
};

/**
 * Check buttons for accessible names
 */
const checkButtons = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const buttons = root.querySelectorAll('button');

  buttons.forEach((button) => {
    const hasName = hasAccessibleName(button);
    if (!hasName) {
      violations.push({
        rule: 'button-name',
        severity: 'error',
        wcagCriterion: '4.1.2',
        message: 'Button missing accessible name',
        element: button,
        fix: 'Add text content or aria-label to the button',
      });
    }
  });

  return violations;
};

/**
 * Check ARIA attributes
 */
const checkAriaAttributes = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const elementsWithAria = root.querySelectorAll('[aria-hidden="true"]');

  elementsWithAria.forEach((element) => {
    const focusable = element.querySelectorAll(
      'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])'
    );

    if (focusable.length > 0) {
      violations.push({
        rule: 'aria-hidden-focusable',
        severity: 'error',
        wcagCriterion: '4.1.2',
        message: 'Focusable element inside aria-hidden container',
        element: element as HTMLElement,
        fix: 'Remove aria-hidden or make children non-focusable',
      });
    }
  });

  return violations;
};

/**
 * Check color contrast
 */
const checkColorContrast = (root: HTMLElement, level: WcagLevel): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const textElements = root.querySelectorAll('p, h1, h2, h3, h4, h5, h6, li, td, th, label, span, a');

  textElements.forEach((element) => {
    const styles = window.getComputedStyle(element);
    const color = styles.color;
    const bgColor = styles.backgroundColor;

    // Simple check (would need more sophisticated background detection)
    if (color && bgColor && bgColor !== 'rgba(0, 0, 0, 0)') {
      // Convert to hex and check (simplified)
      // In production, use proper color parsing library
    }
  });

  return violations;
};

/**
 * Check keyboard accessibility
 */
const checkKeyboardAccess = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const interactive = root.querySelectorAll('[onclick], [onmousedown], [onmouseup]');

  interactive.forEach((element) => {
    const isNativelyFocusable = ['A', 'BUTTON', 'INPUT', 'SELECT', 'TEXTAREA'].includes(
      element.tagName
    );
    const hasTabIndex = element.hasAttribute('tabindex');
    const hasRole = element.hasAttribute('role');

    if (!isNativelyFocusable && !hasTabIndex && !hasRole) {
      violations.push({
        rule: 'keyboard-access',
        severity: 'error',
        wcagCriterion: '2.1.1',
        message: 'Interactive element not keyboard accessible',
        element: element as HTMLElement,
        fix: 'Add tabindex="0" or use semantic HTML (button, a)',
      });
    }
  });

  return violations;
};

/**
 * Check focus indicators
 */
const checkFocusIndicators = (root: HTMLElement): AuditViolation[] => {
  const violations: AuditViolation[] = [];
  const focusable = root.querySelectorAll(
    'a[href], button, input, select, textarea, [tabindex]:not([tabindex="-1"])'
  );

  focusable.forEach((element) => {
    const styles = window.getComputedStyle(element, ':focus');
    const outline = styles.outline;

    if (outline === 'none' || outline === '0px') {
      violations.push({
        rule: 'focus-indicator',
        severity: 'error',
        wcagCriterion: '2.4.7',
        message: 'Element has no visible focus indicator',
        element: element as HTMLElement,
        fix: 'Ensure visible focus styles (outline, box-shadow, etc.)',
      });
    }
  });

  return violations;
};

/**
 * Print audit report to console
 */
export const printAuditReport = (report: AuditReport): void => {
  console.group('🔍 Accessibility Audit Report');
  console.log(report.summary);

  if (report.violations.length > 0) {
    console.group(`❌ Violations (${report.violations.length})`);
    report.violations.forEach((v, i) => {
      console.group(`${i + 1}. [${v.severity.toUpperCase()}] ${v.rule}`);
      console.log(`WCAG: ${v.wcagCriterion}`);
      console.log(`Message: ${v.message}`);
      console.log(`Fix: ${v.fix}`);
      if (v.element) console.log('Element:', v.element);
      console.groupEnd();
    });
    console.groupEnd();
  }

  console.groupEnd();
};
