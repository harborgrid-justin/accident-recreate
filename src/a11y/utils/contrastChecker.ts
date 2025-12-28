/**
 * Color Contrast Checker Utilities
 *
 * Implements WCAG 2.1 contrast ratio calculations.
 */

import type { ContrastResult, WcagLevel } from '../types';

/**
 * RGB color type
 */
export interface RgbColor {
  r: number;
  g: number;
  b: number;
}

/**
 * Parse hex color to RGB
 */
export const hexToRgb = (hex: string): RgbColor | null => {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : null;
};

/**
 * Convert RGB to hex
 */
export const rgbToHex = (r: number, g: number, b: number): string => {
  return '#' + [r, g, b].map((x) => {
    const hex = x.toString(16);
    return hex.length === 1 ? '0' + hex : hex;
  }).join('');
};

/**
 * Calculate relative luminance of a color
 * Formula from WCAG 2.1
 */
export const getRelativeLuminance = (rgb: RgbColor): number => {
  const { r, g, b } = rgb;

  const rsRGB = r / 255;
  const gsRGB = g / 255;
  const bsRGB = b / 255;

  const rLinear = rsRGB <= 0.03928 ? rsRGB / 12.92 : Math.pow((rsRGB + 0.055) / 1.055, 2.4);
  const gLinear = gsRGB <= 0.03928 ? gsRGB / 12.92 : Math.pow((gsRGB + 0.055) / 1.055, 2.4);
  const bLinear = bsRGB <= 0.03928 ? bsRGB / 12.92 : Math.pow((bsRGB + 0.055) / 1.055, 2.4);

  return 0.2126 * rLinear + 0.7152 * gLinear + 0.0722 * bLinear;
};

/**
 * Calculate contrast ratio between two colors
 * Formula from WCAG 2.1
 */
export const getContrastRatio = (rgb1: RgbColor, rgb2: RgbColor): number => {
  const lum1 = getRelativeLuminance(rgb1);
  const lum2 = getRelativeLuminance(rgb2);

  const lighter = Math.max(lum1, lum2);
  const darker = Math.min(lum1, lum2);

  return (lighter + 0.05) / (darker + 0.05);
};

/**
 * Check if contrast ratio meets WCAG requirements
 */
export const meetsContrastRequirements = (
  ratio: number,
  level: WcagLevel,
  isLargeText: boolean
): boolean => {
  const requirements = {
    A: isLargeText ? 3.0 : 3.0,
    AA: isLargeText ? 3.0 : 4.5,
    AAA: isLargeText ? 4.5 : 7.0,
  };

  return ratio >= requirements[level];
};

/**
 * Check contrast between two hex colors
 */
export const checkContrast = (
  foreground: string,
  background: string
): ContrastResult | null => {
  const fgRgb = hexToRgb(foreground);
  const bgRgb = hexToRgb(background);

  if (!fgRgb || !bgRgb) {
    return null;
  }

  const ratio = getContrastRatio(fgRgb, bgRgb);

  return {
    foreground,
    background,
    ratio,
    meetsAA: ratio >= 4.5,
    meetsAALarge: ratio >= 3.0,
    meetsAAA: ratio >= 7.0,
    meetsAAALarge: ratio >= 4.5,
  };
};

/**
 * Suggest an accessible foreground color for a given background
 */
export const suggestForegroundColor = (
  background: string,
  level: WcagLevel = 'AA',
  isLargeText = false
): string => {
  const bgRgb = hexToRgb(background);
  if (!bgRgb) return '#000000';

  const bgLuminance = getRelativeLuminance(bgRgb);
  const requiredRatio = meetsContrastRequirements(0, level, isLargeText)
    ? (level === 'AAA' ? (isLargeText ? 4.5 : 7.0) : (isLargeText ? 3.0 : 4.5))
    : 4.5;

  // Try black first
  const black: RgbColor = { r: 0, g: 0, b: 0 };
  const blackRatio = getContrastRatio(black, bgRgb);

  if (blackRatio >= requiredRatio) {
    return '#000000';
  }

  // Try white
  const white: RgbColor = { r: 255, g: 255, b: 255 };
  const whiteRatio = getContrastRatio(white, bgRgb);

  if (whiteRatio >= requiredRatio) {
    return '#FFFFFF';
  }

  // Return the one with better contrast
  return bgLuminance > 0.5 ? '#000000' : '#FFFFFF';
};

/**
 * Find accessible color pairs from a palette
 */
export const findAccessiblePairs = (
  colors: string[],
  level: WcagLevel = 'AA',
  isLargeText = false
): Array<{ foreground: string; background: string; ratio: number }> => {
  const pairs: Array<{ foreground: string; background: string; ratio: number }> = [];

  for (let i = 0; i < colors.length; i++) {
    for (let j = 0; j < colors.length; j++) {
      if (i === j) continue;

      const result = checkContrast(colors[i], colors[j]);
      if (result && meetsContrastRequirements(result.ratio, level, isLargeText)) {
        pairs.push({
          foreground: colors[i],
          background: colors[j],
          ratio: result.ratio,
        });
      }
    }
  }

  return pairs.sort((a, b) => b.ratio - a.ratio);
};

/**
 * Lighten a color by a percentage
 */
export const lightenColor = (hex: string, percent: number): string => {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;

  const amount = Math.round(2.55 * percent);
  const r = Math.min(255, rgb.r + amount);
  const g = Math.min(255, rgb.g + amount);
  const b = Math.min(255, rgb.b + amount);

  return rgbToHex(r, g, b);
};

/**
 * Darken a color by a percentage
 */
export const darkenColor = (hex: string, percent: number): string => {
  const rgb = hexToRgb(hex);
  if (!rgb) return hex;

  const amount = Math.round(2.55 * percent);
  const r = Math.max(0, rgb.r - amount);
  const g = Math.max(0, rgb.g - amount);
  const b = Math.max(0, rgb.b - amount);

  return rgbToHex(r, g, b);
};

/**
 * Get WCAG contrast level name
 */
export const getContrastLevel = (ratio: number, isLargeText = false): string => {
  if (isLargeText) {
    if (ratio >= 4.5) return 'AAA';
    if (ratio >= 3.0) return 'AA';
    return 'Fail';
  } else {
    if (ratio >= 7.0) return 'AAA';
    if (ratio >= 4.5) return 'AA';
    if (ratio >= 3.0) return 'A';
    return 'Fail';
  }
};
