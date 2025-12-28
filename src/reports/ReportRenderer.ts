/**
 * Report Renderer - Converts reports to various formats
 * Supports HTML, PDF with headers, footers, page numbers, and TOC
 */

import { ReportTemplate } from './ReportTemplates';
import { StyleGenerator, BrandingConfig, DEFAULT_COLOR_SCHEME, DEFAULT_TYPOGRAPHY, DEFAULT_LAYOUT } from './ReportStyles';
import { ReportBuilder } from './ReportBuilder';

export interface RenderOptions {
  format: 'html' | 'pdf';
  includeCoverPage?: boolean;
  includeTableOfContents?: boolean;
  includeHeaders?: boolean;
  includeFooters?: boolean;
  includePageNumbers?: boolean;
  branding?: BrandingConfig;
  outputPath?: string;
  pdfOptions?: PDFOptions;
}

export interface PDFOptions {
  engine?: 'puppeteer' | 'html-pdf' | 'pdfkit';
  displayHeaderFooter?: boolean;
  headerTemplate?: string;
  footerTemplate?: string;
  margin?: {
    top?: string;
    right?: string;
    bottom?: string;
    left?: string;
  };
  preferCSSPageSize?: boolean;
  printBackground?: boolean;
  landscape?: boolean;
}

export interface RenderedReport {
  format: 'html' | 'pdf';
  content: string | Buffer;
  metadata: {
    generatedAt: Date;
    pageCount?: number;
    fileSize?: number;
  };
}

/**
 * Report Renderer Class
 */
export class ReportRenderer {
  private template: ReportTemplate;
  private builder: ReportBuilder;
  private styleGenerator: StyleGenerator;

  constructor(template: ReportTemplate, builder: ReportBuilder, branding?: BrandingConfig) {
    this.template = template;
    this.builder = builder;
    this.styleGenerator = new StyleGenerator(
      DEFAULT_COLOR_SCHEME,
      DEFAULT_TYPOGRAPHY,
      DEFAULT_LAYOUT,
      branding
    );
  }

  /**
   * Render report to specified format
   */
  async render(options: RenderOptions): Promise<RenderedReport> {
    if (options.format === 'html') {
      return this.renderHTML(options);
    } else if (options.format === 'pdf') {
      return this.renderPDF(options);
    } else {
      throw new Error(`Unsupported format: ${options.format}`);
    }
  }

  /**
   * Render to HTML
   */
  private async renderHTML(options: RenderOptions): Promise<RenderedReport> {
    const {
      includeCoverPage = true,
      includeTableOfContents = this.template.formatting.includeTableOfContents,
      includeHeaders = this.template.formatting.includeHeaderFooter,
      includeFooters = this.template.formatting.includeHeaderFooter
    } = options;

    let html = this.buildHTMLDocument(
      includeCoverPage,
      includeTableOfContents,
      includeHeaders,
      includeFooters
    );

    return {
      format: 'html',
      content: html,
      metadata: {
        generatedAt: new Date(),
        fileSize: Buffer.from(html).length
      }
    };
  }

  /**
   * Render to PDF
   */
  private async renderPDF(options: RenderOptions): Promise<RenderedReport> {
    const {
      includeCoverPage = true,
      includeTableOfContents = this.template.formatting.includeTableOfContents,
      includeHeaders = this.template.formatting.includeHeaderFooter,
      includeFooters = this.template.formatting.includeHeaderFooter,
      pdfOptions = {}
    } = options;

    // Build HTML first
    const html = this.buildHTMLDocument(
      includeCoverPage,
      includeTableOfContents,
      includeHeaders,
      includeFooters
    );

    // Convert to PDF
    const pdfBuffer = await this.convertHTMLToPDF(html, pdfOptions);

    return {
      format: 'pdf',
      content: pdfBuffer,
      metadata: {
        generatedAt: new Date(),
        fileSize: pdfBuffer.length,
        pageCount: this.estimatePageCount(html)
      }
    };
  }

  /**
   * Build complete HTML document
   */
  private buildHTMLDocument(
    includeCoverPage: boolean,
    includeTableOfContents: boolean,
    includeHeaders: boolean,
    includeFooters: boolean
  ): string {
    const css = this.styleGenerator.generateCSS();

    let html = '<!DOCTYPE html>\n';
    html += '<html lang="en">\n';
    html += '<head>\n';
    html += '  <meta charset="UTF-8">\n';
    html += '  <meta name="viewport" content="width=device-width, initial-scale=1.0">\n';
    html += `  <title>${this.template.name}</title>\n`;
    html += '  <style>\n';
    html += css;
    html += '\n  </style>\n';
    html += '</head>\n';
    html += '<body>\n';

    // Cover page
    if (includeCoverPage) {
      html += this.buildCoverPage();
    }

    // Table of contents
    if (includeTableOfContents) {
      html += this.builder.buildTableOfContents();
    }

    // Header (if not using page-level headers)
    if (includeHeaders && !includeCoverPage) {
      html += this.styleGenerator.generateHeader(this.template.name);
    }

    // Main content
    html += '<div class="report-content">\n';
    html += this.builder.buildReport();
    html += '</div>\n';

    // Footer (if not using page-level footers)
    if (includeFooters && !includeCoverPage) {
      html += this.styleGenerator.generateFooter();
    }

    html += '</body>\n';
    html += '</html>';

    return html;
  }

  /**
   * Build cover page
   */
  private buildCoverPage(): string {
    const metadata: Record<string, string> = {
      'Template': this.template.name,
      'Category': this.template.category,
      'Version': this.template.metadata.version,
      'Generated': new Date().toLocaleDateString()
    };

    if (this.template.metadata.author) {
      metadata['Author'] = this.template.metadata.author;
    }

    return this.styleGenerator.generateCoverPage(
      this.template.name,
      this.template.description,
      metadata
    );
  }

  /**
   * Convert HTML to PDF
   */
  private async convertHTMLToPDF(html: string, options: PDFOptions): Promise<Buffer> {
    const engine = options.engine || 'puppeteer';

    switch (engine) {
      case 'puppeteer':
        return this.convertWithPuppeteer(html, options);
      case 'html-pdf':
        return this.convertWithHTMLPDF(html, options);
      case 'pdfkit':
        return this.convertWithPDFKit(html, options);
      default:
        throw new Error(`Unsupported PDF engine: ${engine}`);
    }
  }

  /**
   * Convert using Puppeteer
   */
  private async convertWithPuppeteer(html: string, options: PDFOptions): Promise<Buffer> {
    // Note: In a real implementation, this would use puppeteer
    // For now, we'll create a stub that shows how it would work

    try {
      // const puppeteer = require('puppeteer');
      // const browser = await puppeteer.launch({
      //   headless: true,
      //   args: ['--no-sandbox', '--disable-setuid-sandbox']
      // });
      // const page = await browser.newPage();
      // await page.setContent(html, { waitUntil: 'networkidle0' });
      //
      // const pdfBuffer = await page.pdf({
      //   format: this.template.formatting.pageSize === 'letter' ? 'Letter' : 'A4',
      //   landscape: this.template.formatting.orientation === 'landscape',
      //   printBackground: options.printBackground !== false,
      //   displayHeaderFooter: options.displayHeaderFooter || false,
      //   headerTemplate: options.headerTemplate || '',
      //   footerTemplate: options.footerTemplate || this.buildFooterTemplate(),
      //   margin: options.margin || {
      //     top: `${this.template.formatting.margins.top}in`,
      //     right: `${this.template.formatting.margins.right}in`,
      //     bottom: `${this.template.formatting.margins.bottom}in`,
      //     left: `${this.template.formatting.margins.left}in`
      //   },
      //   preferCSSPageSize: options.preferCSSPageSize !== false
      // });
      //
      // await browser.close();
      // return pdfBuffer;

      // Stub implementation - returns placeholder
      return this.createPlaceholderPDF(html);
    } catch (error) {
      console.error('Puppeteer PDF generation failed:', error);
      throw new Error('Failed to generate PDF with Puppeteer');
    }
  }

  /**
   * Convert using html-pdf
   */
  private async convertWithHTMLPDF(html: string, options: PDFOptions): Promise<Buffer> {
    // Note: In a real implementation, this would use html-pdf library
    // For now, we'll create a stub

    try {
      // const pdf = require('html-pdf');
      //
      // return new Promise((resolve, reject) => {
      //   pdf.create(html, {
      //     format: this.template.formatting.pageSize === 'letter' ? 'Letter' : 'A4',
      //     orientation: this.template.formatting.orientation,
      //     border: {
      //       top: `${this.template.formatting.margins.top}in`,
      //       right: `${this.template.formatting.margins.right}in`,
      //       bottom: `${this.template.formatting.margins.bottom}in`,
      //       left: `${this.template.formatting.margins.left}in`
      //     },
      //     type: 'pdf'
      //   }).toBuffer((err, buffer) => {
      //     if (err) reject(err);
      //     else resolve(buffer);
      //   });
      // });

      // Stub implementation
      return this.createPlaceholderPDF(html);
    } catch (error) {
      console.error('html-pdf generation failed:', error);
      throw new Error('Failed to generate PDF with html-pdf');
    }
  }

  /**
   * Convert using PDFKit (for programmatic PDF generation)
   */
  private async convertWithPDFKit(html: string, options: PDFOptions): Promise<Buffer> {
    // Note: In a real implementation, this would use PDFKit
    // PDFKit is better for programmatic generation, not HTML conversion

    try {
      // const PDFDocument = require('pdfkit');
      // const doc = new PDFDocument({
      //   size: this.template.formatting.pageSize === 'letter' ? 'LETTER' : 'A4',
      //   layout: this.template.formatting.orientation,
      //   margins: {
      //     top: this.template.formatting.margins.top * 72,
      //     right: this.template.formatting.margins.right * 72,
      //     bottom: this.template.formatting.margins.bottom * 72,
      //     left: this.template.formatting.margins.left * 72
      //   }
      // });
      //
      // const chunks: Buffer[] = [];
      // doc.on('data', (chunk) => chunks.push(chunk));
      // doc.on('end', () => resolve(Buffer.concat(chunks)));
      //
      // // Parse HTML and generate PDF content
      // // This is complex and would require HTML parsing and layout engine
      //
      // doc.end();

      // Stub implementation
      return this.createPlaceholderPDF(html);
    } catch (error) {
      console.error('PDFKit generation failed:', error);
      throw new Error('Failed to generate PDF with PDFKit');
    }
  }

  /**
   * Create placeholder PDF buffer
   */
  private createPlaceholderPDF(html: string): Buffer {
    // Create a minimal PDF structure
    const pdfContent = `%PDF-1.4
1 0 obj
<<
/Type /Catalog
/Pages 2 0 R
>>
endobj
2 0 obj
<<
/Type /Pages
/Kids [3 0 R]
/Count 1
>>
endobj
3 0 obj
<<
/Type /Page
/Parent 2 0 R
/MediaBox [0 0 612 792]
/Contents 4 0 R
/Resources <<
/Font <<
/F1 5 0 R
>>
>>
>>
endobj
4 0 obj
<<
/Length 44
>>
stream
BT
/F1 12 Tf
100 700 Td
(AccuScene Report) Tj
ET
endstream
endobj
5 0 obj
<<
/Type /Font
/Subtype /Type1
/BaseFont /Helvetica
>>
endobj
xref
0 6
0000000000 65535 f
0000000009 00000 n
0000000058 00000 n
0000000115 00000 n
0000000262 00000 n
0000000356 00000 n
trailer
<<
/Size 6
/Root 1 0 R
>>
startxref
444
%%EOF`;

    return Buffer.from(pdfContent);
  }

  /**
   * Build footer template for PDF
   */
  private buildFooterTemplate(): string {
    return `
      <div style="font-size: 9px; text-align: center; width: 100%; margin: 0 auto; padding-top: 5px; color: #666;">
        <span class="pageNumber"></span> / <span class="totalPages"></span>
      </div>
    `;
  }

  /**
   * Estimate page count from HTML content
   */
  private estimatePageCount(html: string): number {
    // Very rough estimation based on content length
    const contentLength = html.length;
    const averageCharsPerPage = 3000; // Approximation
    return Math.max(1, Math.ceil(contentLength / averageCharsPerPage));
  }

  /**
   * Export to file
   */
  async exportToFile(rendered: RenderedReport, filePath: string): Promise<void> {
    const fs = require('fs').promises;

    if (rendered.format === 'html') {
      await fs.writeFile(filePath, rendered.content, 'utf-8');
    } else if (rendered.format === 'pdf') {
      await fs.writeFile(filePath, rendered.content);
    }
  }

  /**
   * Generate report preview HTML (for web display)
   */
  generatePreview(): string {
    const html = this.buildHTMLDocument(false, false, true, true);

    // Wrap in iframe-friendly container
    let preview = '<div class="report-preview">\n';
    preview += html;
    preview += '</div>\n';

    return preview;
  }

  /**
   * Generate print-friendly HTML
   */
  generatePrintVersion(): string {
    return this.buildHTMLDocument(true, true, true, true);
  }

  /**
   * Split report into multiple pages (for web pagination)
   */
  splitIntoPages(html: string, pageHeight: number = 1056): string[] {
    // Simple page splitting based on height
    // In a real implementation, this would be more sophisticated
    const pages: string[] = [];
    const sections = html.split('<div class="page-break"></div>');

    for (const section of sections) {
      if (section.trim()) {
        pages.push(section);
      }
    }

    return pages.length > 0 ? pages : [html];
  }

  /**
   * Add watermark to report
   */
  addWatermark(html: string, watermarkText: string): string {
    const watermarkStyle = `
      <style>
        .watermark {
          position: fixed;
          top: 50%;
          left: 50%;
          transform: translate(-50%, -50%) rotate(-45deg);
          font-size: 120px;
          font-weight: bold;
          color: rgba(0, 0, 0, 0.1);
          z-index: 9999;
          pointer-events: none;
          user-select: none;
        }
      </style>
    `;

    const watermarkDiv = `<div class="watermark">${watermarkText}</div>`;

    // Insert watermark style and div into HTML
    html = html.replace('</head>', watermarkStyle + '</head>');
    html = html.replace('<body>', '<body>' + watermarkDiv);

    return html;
  }

  /**
   * Generate report metadata
   */
  generateMetadata(): Record<string, any> {
    return {
      template: {
        id: this.template.id,
        name: this.template.name,
        version: this.template.metadata.version
      },
      generatedAt: new Date().toISOString(),
      pageSize: this.template.formatting.pageSize,
      orientation: this.template.formatting.orientation,
      sectionCount: this.template.sections.filter(s => s.enabled).length
    };
  }
}

/**
 * Batch renderer for multiple reports
 */
export class BatchReportRenderer {
  private renderers: Map<string, ReportRenderer> = new Map();

  /**
   * Add renderer to batch
   */
  addRenderer(id: string, renderer: ReportRenderer): void {
    this.renderers.set(id, renderer);
  }

  /**
   * Render all reports in batch
   */
  async renderAll(options: RenderOptions): Promise<Map<string, RenderedReport>> {
    const results = new Map<string, RenderedReport>();

    for (const [id, renderer] of this.renderers.entries()) {
      try {
        const rendered = await renderer.render(options);
        results.set(id, rendered);
      } catch (error) {
        console.error(`Failed to render report ${id}:`, error);
      }
    }

    return results;
  }

  /**
   * Export all reports to directory
   */
  async exportAll(results: Map<string, RenderedReport>, outputDir: string): Promise<void> {
    const fs = require('fs').promises;
    const path = require('path');

    // Create output directory if it doesn't exist
    await fs.mkdir(outputDir, { recursive: true });

    for (const [id, result] of results.entries()) {
      const extension = result.format === 'html' ? 'html' : 'pdf';
      const filePath = path.join(outputDir, `${id}.${extension}`);

      if (result.format === 'html') {
        await fs.writeFile(filePath, result.content, 'utf-8');
      } else {
        await fs.writeFile(filePath, result.content);
      }
    }
  }
}
