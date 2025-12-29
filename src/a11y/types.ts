/**
 * TypeScript type definitions for accessibility system
 */

export type WcagLevel = 'A' | 'AA' | 'AAA';

export type ColorScheme = 'light' | 'dark' | 'high-contrast' | 'auto';

export type TextSize = 'small' | 'medium' | 'large' | 'extra-large';

export type MotionPreference = 'no-preference' | 'reduce' | 'none';

export type TextDirection = 'ltr' | 'rtl' | 'auto';

export interface A11yConfig {
  wcagLevel: WcagLevel;
  screenReaderEnabled: boolean;
  keyboardNavEnabled: boolean;
  focusIndicatorsEnabled: boolean;
  colorScheme: ColorScheme;
  motionPreference: MotionPreference;
  textSize: TextSize;
  reduceTransparency: boolean;
  captionsEnabled: boolean;
  audioDescriptionsEnabled: boolean;
  timeoutDuration: number;
  language: string;
  textDirection: TextDirection;
}

export interface A11ySettings extends Partial<A11yConfig> {
  customAriaLabels?: Record<string, string>;
}

export interface ContrastResult {
  foreground: string;
  background: string;
  ratio: number;
  meetsAA: boolean;
  meetsAALarge: boolean;
  meetsAAA: boolean;
  meetsAAALarge: boolean;
}

export type AriaRole =
  // Document structure
  | 'article'
  | 'application'
  | 'banner'
  | 'complementary'
  | 'contentinfo'
  | 'definition'
  | 'directory'
  | 'document'
  | 'feed'
  | 'figure'
  | 'group'
  | 'heading'
  | 'img'
  | 'list'
  | 'listitem'
  | 'main'
  | 'navigation'
  | 'none'
  | 'note'
  | 'presentation'
  | 'region'
  | 'search'
  | 'separator'
  | 'table'
  | 'term'
  | 'toolbar'
  // Widget roles
  | 'alert'
  | 'alertdialog'
  | 'button'
  | 'checkbox'
  | 'combobox'
  | 'dialog'
  | 'gridcell'
  | 'link'
  | 'log'
  | 'marquee'
  | 'menu'
  | 'menubar'
  | 'menuitem'
  | 'menuitemcheckbox'
  | 'menuitemradio'
  | 'option'
  | 'progressbar'
  | 'radio'
  | 'radiogroup'
  | 'scrollbar'
  | 'searchbox'
  | 'slider'
  | 'spinbutton'
  | 'status'
  | 'switch'
  | 'tab'
  | 'tablist'
  | 'tabpanel'
  | 'textbox'
  | 'timer'
  | 'tooltip'
  | 'tree'
  | 'treegrid'
  | 'treeitem';

export interface AriaAttributes {
  // Widget attributes
  'aria-checked'?: boolean | 'mixed';
  'aria-disabled'?: boolean;
  'aria-expanded'?: boolean;
  'aria-hidden'?: boolean;
  'aria-invalid'?: boolean | 'grammar' | 'spelling';
  'aria-pressed'?: boolean | 'mixed';
  'aria-selected'?: boolean;

  // Live region attributes
  'aria-atomic'?: boolean;
  'aria-busy'?: boolean;
  'aria-live'?: 'off' | 'polite' | 'assertive';
  'aria-relevant'?: string;

  // Drag and drop
  'aria-grabbed'?: boolean;
  'aria-dropeffect'?: string;

  // Relationship attributes
  'aria-activedescendant'?: string;
  'aria-controls'?: string;
  'aria-describedby'?: string;
  'aria-details'?: string;
  'aria-errormessage'?: string;
  'aria-flowto'?: string;
  'aria-labelledby'?: string;
  'aria-owns'?: string;

  // Value attributes
  'aria-valuenow'?: number;
  'aria-valuemin'?: number;
  'aria-valuemax'?: number;
  'aria-valuetext'?: string;

  // Other
  'aria-label'?: string;
  'aria-level'?: number;
  'aria-modal'?: boolean;
  'aria-multiselectable'?: boolean;
  'aria-orientation'?: 'horizontal' | 'vertical';
  'aria-placeholder'?: string;
  'aria-readonly'?: boolean;
  'aria-required'?: boolean;

  // Allow any other ARIA attribute
  [key: `aria-${string}`]: any;
}

export interface KeyboardShortcut {
  key: string;
  ctrl?: boolean;
  alt?: boolean;
  shift?: boolean;
  meta?: boolean;
  description: string;
  action: () => void;
}

export interface FocusableElement {
  id: string;
  element: HTMLElement;
  tabIndex: number;
}

export interface FocusManagerState {
  focusedId: string | null;
  focusableElements: FocusableElement[];
  history: string[];
  trapActive: boolean;
}

export interface LiveRegionOptions {
  priority: 'polite' | 'assertive';
  atomic?: boolean;
  relevant?: 'additions' | 'removals' | 'text' | 'all';
}

export interface SkipLink {
  id: string;
  label: string;
  target: string;
}
