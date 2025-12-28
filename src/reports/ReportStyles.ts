/**
 * Report Styling and Formatting System
 * Provides professional typography and branding support
 */

export interface ColorScheme {
  primary: string;
  secondary: string;
  accent: string;
  text: {
    primary: string;
    secondary: string;
    muted: string;
  };
  background: {
    primary: string;
    secondary: string;
    accent: string;
  };
  borders: {
    light: string;
    medium: string;
    dark: string;
  };
}

export interface Typography {
  fontFamily: {
    heading: string;
    body: string;
    monospace: string;
  };
  fontSize: {
    h1: string;
    h2: string;
    h3: string;
    h4: string;
    h5: string;
    h6: string;
    body: string;
    small: string;
    tiny: string;
  };
  fontWeight: {
    light: number;
    normal: number;
    medium: number;
    semibold: number;
    bold: number;
  };
  lineHeight: {
    tight: number;
    normal: number;
    relaxed: number;
  };
}

export interface BrandingConfig {
  companyName: string;
  logoUrl?: string;
  logoWidth?: number;
  logoHeight?: number;
  tagline?: string;
  primaryColor: string;
  secondaryColor: string;
  contactInfo?: {
    address?: string;
    phone?: string;
    email?: string;
    website?: string;
  };
}

export interface LayoutConfig {
  pageSize: 'letter' | 'a4' | 'legal';
  orientation: 'portrait' | 'landscape';
  margins: {
    top: number;
    right: number;
    bottom: number;
    left: number;
  };
  header: {
    height: number;
    showLogo: boolean;
    showTitle: boolean;
    showDate: boolean;
  };
  footer: {
    height: number;
    showPageNumbers: boolean;
    showCompanyInfo: boolean;
    alignment: 'left' | 'center' | 'right';
  };
  spacing: {
    sectionGap: number;
    paragraphGap: number;
    listItemGap: number;
  };
}

/**
 * Default Professional Color Scheme
 */
export const DEFAULT_COLOR_SCHEME: ColorScheme = {
  primary: '#1a365d',
  secondary: '#2c5282',
  accent: '#3182ce',
  text: {
    primary: '#1a202c',
    secondary: '#4a5568',
    muted: '#718096'
  },
  background: {
    primary: '#ffffff',
    secondary: '#f7fafc',
    accent: '#edf2f7'
  },
  borders: {
    light: '#e2e8f0',
    medium: '#cbd5e0',
    dark: '#a0aec0'
  }
};

/**
 * Professional Typography Settings
 */
export const DEFAULT_TYPOGRAPHY: Typography = {
  fontFamily: {
    heading: "'Helvetica Neue', 'Arial', sans-serif",
    body: "'Georgia', 'Times New Roman', serif",
    monospace: "'Courier New', 'Courier', monospace"
  },
  fontSize: {
    h1: '28pt',
    h2: '22pt',
    h3: '18pt',
    h4: '16pt',
    h5: '14pt',
    h6: '12pt',
    body: '11pt',
    small: '9pt',
    tiny: '8pt'
  },
  fontWeight: {
    light: 300,
    normal: 400,
    medium: 500,
    semibold: 600,
    bold: 700
  },
  lineHeight: {
    tight: 1.2,
    normal: 1.5,
    relaxed: 1.8
  }
};

/**
 * Default Layout Configuration
 */
export const DEFAULT_LAYOUT: LayoutConfig = {
  pageSize: 'letter',
  orientation: 'portrait',
  margins: {
    top: 1.0,
    right: 1.0,
    bottom: 1.0,
    left: 1.0
  },
  header: {
    height: 0.75,
    showLogo: true,
    showTitle: true,
    showDate: true
  },
  footer: {
    height: 0.5,
    showPageNumbers: true,
    showCompanyInfo: true,
    alignment: 'center'
  },
  spacing: {
    sectionGap: 24,
    paragraphGap: 12,
    listItemGap: 6
  }
};

/**
 * Style Generator Class
 */
export class StyleGenerator {
  private colorScheme: ColorScheme;
  private typography: Typography;
  private layout: LayoutConfig;
  private branding?: BrandingConfig;

  constructor(
    colorScheme: ColorScheme = DEFAULT_COLOR_SCHEME,
    typography: Typography = DEFAULT_TYPOGRAPHY,
    layout: LayoutConfig = DEFAULT_LAYOUT,
    branding?: BrandingConfig
  ) {
    this.colorScheme = colorScheme;
    this.typography = typography;
    this.layout = layout;
    this.branding = branding;
  }

  /**
   * Generate complete CSS styles for report
   */
  generateCSS(): string {
    return `
/* AccuScene Report Styles */

/* Page Setup */
@page {
  size: ${this.layout.pageSize} ${this.layout.orientation};
  margin: ${this.layout.margins.top}in ${this.layout.margins.right}in ${this.layout.margins.bottom}in ${this.layout.margins.left}in;
}

@media print {
  body {
    -webkit-print-color-adjust: exact;
    print-color-adjust: exact;
  }

  .page-break {
    page-break-before: always;
  }

  .no-break {
    page-break-inside: avoid;
  }
}

/* Base Typography */
body {
  font-family: ${this.typography.fontFamily.body};
  font-size: ${this.typography.fontSize.body};
  line-height: ${this.typography.lineHeight.normal};
  color: ${this.colorScheme.text.primary};
  background-color: ${this.colorScheme.background.primary};
  margin: 0;
  padding: 0;
}

/* Headings */
h1, h2, h3, h4, h5, h6 {
  font-family: ${this.typography.fontFamily.heading};
  font-weight: ${this.typography.fontWeight.bold};
  line-height: ${this.typography.lineHeight.tight};
  margin-top: ${this.layout.spacing.sectionGap}px;
  margin-bottom: ${this.layout.spacing.paragraphGap}px;
  color: ${this.colorScheme.primary};
}

h1 {
  font-size: ${this.typography.fontSize.h1};
  border-bottom: 3px solid ${this.colorScheme.primary};
  padding-bottom: 8px;
  margin-top: 0;
}

h2 {
  font-size: ${this.typography.fontSize.h2};
  border-bottom: 2px solid ${this.colorScheme.borders.medium};
  padding-bottom: 6px;
}

h3 {
  font-size: ${this.typography.fontSize.h3};
  color: ${this.colorScheme.secondary};
}

h4 {
  font-size: ${this.typography.fontSize.h4};
}

h5 {
  font-size: ${this.typography.fontSize.h5};
}

h6 {
  font-size: ${this.typography.fontSize.h6};
}

/* Paragraphs */
p {
  margin: ${this.layout.spacing.paragraphGap}px 0;
  text-align: justify;
  hyphens: auto;
}

/* Lists */
ul, ol {
  margin: ${this.layout.spacing.paragraphGap}px 0;
  padding-left: 30px;
}

li {
  margin-bottom: ${this.layout.spacing.listItemGap}px;
}

/* Tables */
table {
  width: 100%;
  border-collapse: collapse;
  margin: ${this.layout.spacing.paragraphGap}px 0;
  font-size: ${this.typography.fontSize.body};
}

thead {
  background-color: ${this.colorScheme.primary};
  color: ${this.colorScheme.background.primary};
}

th {
  padding: 10px;
  text-align: left;
  font-weight: ${this.typography.fontWeight.semibold};
  border: 1px solid ${this.colorScheme.borders.medium};
}

td {
  padding: 8px 10px;
  border: 1px solid ${this.colorScheme.borders.light};
}

tbody tr:nth-child(even) {
  background-color: ${this.colorScheme.background.secondary};
}

tbody tr:hover {
  background-color: ${this.colorScheme.background.accent};
}

/* Code and Preformatted Text */
code, pre {
  font-family: ${this.typography.fontFamily.monospace};
  background-color: ${this.colorScheme.background.secondary};
  border: 1px solid ${this.colorScheme.borders.light};
  border-radius: 3px;
}

code {
  padding: 2px 6px;
  font-size: ${this.typography.fontSize.small};
}

pre {
  padding: 12px;
  overflow-x: auto;
  white-space: pre-wrap;
  word-wrap: break-word;
}

/* Images and Figures */
figure {
  margin: ${this.layout.spacing.sectionGap}px 0;
  text-align: center;
  page-break-inside: avoid;
}

figcaption {
  font-size: ${this.typography.fontSize.small};
  color: ${this.colorScheme.text.secondary};
  font-style: italic;
  margin-top: 8px;
  text-align: center;
}

img {
  max-width: 100%;
  height: auto;
  display: block;
  margin: 0 auto;
}

/* Special Elements */
.executive-summary {
  background-color: ${this.colorScheme.background.accent};
  border-left: 4px solid ${this.colorScheme.accent};
  padding: 16px;
  margin: ${this.layout.spacing.sectionGap}px 0;
  page-break-inside: avoid;
}

.key-finding {
  background-color: ${this.colorScheme.background.secondary};
  border-left: 3px solid ${this.colorScheme.secondary};
  padding: 12px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
}

.warning {
  background-color: #fff5f5;
  border-left: 3px solid #fc8181;
  padding: 12px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
}

.note {
  background-color: #fffff0;
  border-left: 3px solid #fbd38d;
  padding: 12px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
}

.info {
  background-color: #ebf8ff;
  border-left: 3px solid ${this.colorScheme.accent};
  padding: 12px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
}

/* Header */
.report-header {
  height: ${this.layout.header.height}in;
  border-bottom: 2px solid ${this.colorScheme.primary};
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 0;
  margin-bottom: ${this.layout.spacing.sectionGap}px;
}

.report-header .logo {
  max-height: 50px;
  max-width: 200px;
}

.report-header .title {
  font-size: ${this.typography.fontSize.h3};
  font-weight: ${this.typography.fontWeight.semibold};
  color: ${this.colorScheme.primary};
}

.report-header .date {
  font-size: ${this.typography.fontSize.small};
  color: ${this.colorScheme.text.secondary};
}

/* Footer */
.report-footer {
  height: ${this.layout.footer.height}in;
  border-top: 1px solid ${this.colorScheme.borders.medium};
  display: flex;
  justify-content: ${this.layout.footer.alignment === 'left' ? 'flex-start' : this.layout.footer.alignment === 'right' ? 'flex-end' : 'center'};
  align-items: center;
  padding: 8px 0;
  margin-top: ${this.layout.spacing.sectionGap}px;
  font-size: ${this.typography.fontSize.small};
  color: ${this.colorScheme.text.muted};
}

.page-number {
  font-weight: ${this.typography.fontWeight.medium};
}

/* Table of Contents */
.table-of-contents {
  margin: ${this.layout.spacing.sectionGap * 2}px 0;
  page-break-after: always;
}

.toc-title {
  font-size: ${this.typography.fontSize.h2};
  font-weight: ${this.typography.fontWeight.bold};
  color: ${this.colorScheme.primary};
  margin-bottom: ${this.layout.spacing.sectionGap}px;
  border-bottom: 2px solid ${this.colorScheme.primary};
  padding-bottom: 8px;
}

.toc-item {
  display: flex;
  justify-content: space-between;
  padding: 6px 0;
  border-bottom: 1px dotted ${this.colorScheme.borders.light};
}

.toc-item.level-1 {
  font-weight: ${this.typography.fontWeight.semibold};
  margin-top: ${this.layout.spacing.paragraphGap}px;
}

.toc-item.level-2 {
  padding-left: 20px;
  font-size: ${this.typography.fontSize.small};
}

.toc-item.level-3 {
  padding-left: 40px;
  font-size: ${this.typography.fontSize.small};
  color: ${this.colorScheme.text.secondary};
}

/* Cover Page */
.cover-page {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 100vh;
  text-align: center;
  page-break-after: always;
}

.cover-logo {
  max-width: 300px;
  margin-bottom: 40px;
}

.cover-title {
  font-size: ${this.typography.fontSize.h1};
  font-weight: ${this.typography.fontWeight.bold};
  color: ${this.colorScheme.primary};
  margin-bottom: 20px;
}

.cover-subtitle {
  font-size: ${this.typography.fontSize.h3};
  color: ${this.colorScheme.text.secondary};
  margin-bottom: 40px;
}

.cover-info {
  font-size: ${this.typography.fontSize.body};
  color: ${this.colorScheme.text.secondary};
  line-height: ${this.typography.lineHeight.relaxed};
}

/* Diagrams */
.diagram-container {
  margin: ${this.layout.spacing.sectionGap}px 0;
  page-break-inside: avoid;
  text-align: center;
}

.diagram-image {
  max-width: 100%;
  border: 1px solid ${this.colorScheme.borders.medium};
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
}

.diagram-legend {
  display: inline-block;
  margin-top: 12px;
  padding: 10px;
  background-color: ${this.colorScheme.background.secondary};
  border: 1px solid ${this.colorScheme.borders.light};
  border-radius: 4px;
  text-align: left;
  font-size: ${this.typography.fontSize.small};
}

/* Witness Statements */
.witness-statement {
  background-color: ${this.colorScheme.background.secondary};
  border: 1px solid ${this.colorScheme.borders.medium};
  border-radius: 4px;
  padding: 16px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
  page-break-inside: avoid;
}

.witness-name {
  font-weight: ${this.typography.fontWeight.semibold};
  color: ${this.colorScheme.primary};
  margin-bottom: 8px;
}

.witness-quote {
  font-style: italic;
  color: ${this.colorScheme.text.secondary};
  padding-left: 16px;
  border-left: 3px solid ${this.colorScheme.borders.medium};
}

/* Calculations */
.calculation-block {
  background-color: ${this.colorScheme.background.secondary};
  padding: 16px;
  margin: ${this.layout.spacing.paragraphGap}px 0;
  border-radius: 4px;
  page-break-inside: avoid;
}

.calculation-title {
  font-weight: ${this.typography.fontWeight.semibold};
  color: ${this.colorScheme.primary};
  margin-bottom: 8px;
}

.calculation-formula {
  font-family: ${this.typography.fontFamily.monospace};
  background-color: ${this.colorScheme.background.primary};
  padding: 8px;
  border: 1px solid ${this.colorScheme.borders.light};
  border-radius: 3px;
  margin: 8px 0;
}

/* Utilities */
.text-center { text-align: center; }
.text-left { text-align: left; }
.text-right { text-align: right; }
.text-justify { text-align: justify; }

.font-bold { font-weight: ${this.typography.fontWeight.bold}; }
.font-semibold { font-weight: ${this.typography.fontWeight.semibold}; }
.font-normal { font-weight: ${this.typography.fontWeight.normal}; }

.text-primary { color: ${this.colorScheme.primary}; }
.text-secondary { color: ${this.colorScheme.text.secondary}; }
.text-muted { color: ${this.colorScheme.text.muted}; }

.mt-1 { margin-top: 8px; }
.mt-2 { margin-top: 16px; }
.mt-3 { margin-top: 24px; }
.mb-1 { margin-bottom: 8px; }
.mb-2 { margin-bottom: 16px; }
.mb-3 { margin-bottom: 24px; }

.hidden-print {
  display: none;
}

@media screen {
  .hidden-print {
    display: block;
  }
}
    `.trim();
  }

  /**
   * Generate header HTML
   */
  generateHeader(title: string, date?: Date): string {
    const dateStr = date ? date.toLocaleDateString() : new Date().toLocaleDateString();

    let html = '<div class="report-header">';

    if (this.layout.header.showLogo && this.branding?.logoUrl) {
      html += `<img src="${this.branding.logoUrl}" class="logo" alt="${this.branding.companyName || 'Logo'}" />`;
    } else if (this.branding?.companyName) {
      html += `<div class="company-name font-bold text-primary">${this.branding.companyName}</div>`;
    }

    if (this.layout.header.showTitle) {
      html += `<div class="title">${title}</div>`;
    }

    if (this.layout.header.showDate) {
      html += `<div class="date">${dateStr}</div>`;
    }

    html += '</div>';
    return html;
  }

  /**
   * Generate footer HTML
   */
  generateFooter(pageNumber?: number, totalPages?: number): string {
    let html = '<div class="report-footer">';

    const parts: string[] = [];

    if (this.layout.footer.showCompanyInfo && this.branding?.companyName) {
      parts.push(`<span>${this.branding.companyName}</span>`);
    }

    if (this.layout.footer.showPageNumbers && pageNumber) {
      const pageText = totalPages
        ? `Page ${pageNumber} of ${totalPages}`
        : `Page ${pageNumber}`;
      parts.push(`<span class="page-number">${pageText}</span>`);
    }

    html += parts.join(' | ');
    html += '</div>';

    return html;
  }

  /**
   * Generate cover page HTML
   */
  generateCoverPage(title: string, subtitle?: string, metadata?: Record<string, string>): string {
    let html = '<div class="cover-page">';

    if (this.branding?.logoUrl) {
      html += `<img src="${this.branding.logoUrl}" class="cover-logo" alt="${this.branding.companyName || 'Logo'}" />`;
    }

    html += `<h1 class="cover-title">${title}</h1>`;

    if (subtitle) {
      html += `<div class="cover-subtitle">${subtitle}</div>`;
    }

    if (metadata) {
      html += '<div class="cover-info">';
      for (const [key, value] of Object.entries(metadata)) {
        html += `<div><strong>${key}:</strong> ${value}</div>`;
      }
      html += '</div>';
    }

    if (this.branding?.contactInfo) {
      html += '<div class="cover-info mt-3">';
      const { address, phone, email, website } = this.branding.contactInfo;
      if (address) html += `<div>${address}</div>`;
      if (phone) html += `<div>Phone: ${phone}</div>`;
      if (email) html += `<div>Email: ${email}</div>`;
      if (website) html += `<div>Web: ${website}</div>`;
      html += '</div>';
    }

    html += '</div>';
    return html;
  }
}

/**
 * Pre-configured style themes
 */
export const STYLE_THEMES = {
  professional: new StyleGenerator(DEFAULT_COLOR_SCHEME, DEFAULT_TYPOGRAPHY, DEFAULT_LAYOUT),

  modern: new StyleGenerator(
    {
      ...DEFAULT_COLOR_SCHEME,
      primary: '#2d3748',
      secondary: '#4a5568',
      accent: '#4299e1'
    },
    {
      ...DEFAULT_TYPOGRAPHY,
      fontFamily: {
        heading: "'Inter', 'Helvetica Neue', sans-serif",
        body: "'Inter', 'Arial', sans-serif",
        monospace: "'Fira Code', 'Courier New', monospace"
      }
    },
    DEFAULT_LAYOUT
  ),

  classic: new StyleGenerator(
    {
      ...DEFAULT_COLOR_SCHEME,
      primary: '#2c3e50',
      secondary: '#34495e',
      accent: '#3498db'
    },
    {
      ...DEFAULT_TYPOGRAPHY,
      fontFamily: {
        heading: "'Garamond', 'Times New Roman', serif",
        body: "'Garamond', 'Georgia', serif",
        monospace: "'Courier New', monospace"
      }
    },
    DEFAULT_LAYOUT
  )
};
