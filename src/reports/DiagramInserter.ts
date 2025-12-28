/**
 * Diagram Insertion and Management for Reports
 * Handles diagram images, scaling, legends, and multiple views
 */

export interface DiagramMetadata {
  id: string;
  title: string;
  description?: string;
  timestamp: Date;
  scale: number;
  units: 'feet' | 'meters' | 'yards';
  orientation: number;
  viewType: 'overhead' | 'perspective' | '3d' | 'detail';
}

export interface DiagramLegendItem {
  symbol: string;
  color: string;
  label: string;
  description?: string;
}

export interface DiagramImage {
  metadata: DiagramMetadata;
  imageData: string; // Base64 or URL
  format: 'png' | 'jpg' | 'svg';
  dimensions: {
    width: number;
    height: number;
  };
  legend?: DiagramLegendItem[];
}

export interface DiagramInsertOptions {
  maxWidth?: number;
  maxHeight?: number;
  maintainAspectRatio?: boolean;
  showTitle?: boolean;
  showDescription?: boolean;
  showLegend?: boolean;
  showScale?: boolean;
  showMetadata?: boolean;
  alignment?: 'left' | 'center' | 'right';
  pageBreakBefore?: boolean;
  pageBreakAfter?: boolean;
}

/**
 * Diagram Inserter Class
 */
export class DiagramInserter {
  private printDPI: number = 300; // DPI for print output
  private screenDPI: number = 96; // DPI for screen display

  /**
   * Insert a single diagram into report HTML
   */
  insertDiagram(diagram: DiagramImage, options: DiagramInsertOptions = {}): string {
    const {
      maxWidth = 7.5, // inches (letter size with 1" margins)
      maxHeight = 9.5, // inches
      maintainAspectRatio = true,
      showTitle = true,
      showDescription = true,
      showLegend = true,
      showScale = true,
      showMetadata = false,
      alignment = 'center',
      pageBreakBefore = false,
      pageBreakAfter = false
    } = options;

    let html = '';

    // Page break before if requested
    if (pageBreakBefore) {
      html += '<div class="page-break"></div>\n';
    }

    // Container with no-break to avoid splitting across pages
    html += '<div class="diagram-container no-break" style="text-align: ' + alignment + ';">\n';

    // Title
    if (showTitle && diagram.metadata.title) {
      html += `  <h4 class="diagram-title">${diagram.metadata.title}</h4>\n`;
    }

    // Description
    if (showDescription && diagram.metadata.description) {
      html += `  <p class="diagram-description text-secondary">${diagram.metadata.description}</p>\n`;
    }

    // Calculate scaled dimensions
    const scaledDimensions = this.calculatePrintDimensions(
      diagram.dimensions.width,
      diagram.dimensions.height,
      maxWidth,
      maxHeight,
      maintainAspectRatio
    );

    // Image
    html += '  <figure>\n';
    html += `    <img src="${diagram.imageData}" class="diagram-image" `;
    html += `alt="${diagram.metadata.title}" `;
    html += `style="width: ${scaledDimensions.width}in; height: ${scaledDimensions.height}in;" />\n`;

    // Caption with scale information
    if (showScale || showMetadata) {
      html += '    <figcaption>\n';

      if (showScale) {
        html += `      <div>Scale: 1:${diagram.metadata.scale} (${diagram.metadata.units})</div>\n`;
      }

      if (showMetadata) {
        html += `      <div class="text-muted">View: ${diagram.metadata.viewType} | Created: ${diagram.metadata.timestamp.toLocaleDateString()}</div>\n`;
      }

      html += '    </figcaption>\n';
    }

    html += '  </figure>\n';

    // Legend
    if (showLegend && diagram.legend && diagram.legend.length > 0) {
      html += this.generateLegend(diagram.legend);
    }

    html += '</div>\n';

    // Page break after if requested
    if (pageBreakAfter) {
      html += '<div class="page-break"></div>\n';
    }

    return html;
  }

  /**
   * Insert multiple diagrams with comparison layout
   */
  insertComparisonDiagrams(diagrams: DiagramImage[], title?: string, options: DiagramInsertOptions = {}): string {
    let html = '';

    if (title) {
      html += `<h3>${title}</h3>\n`;
    }

    html += '<div class="diagram-comparison no-break">\n';

    for (const diagram of diagrams) {
      const singleOptions = {
        ...options,
        maxWidth: options.maxWidth ? options.maxWidth / 2 : 3.5, // Split width for side-by-side
        pageBreakBefore: false,
        pageBreakAfter: false
      };

      html += '<div style="display: inline-block; width: 49%; vertical-align: top; margin: 0 0.5%;">\n';
      html += this.insertDiagram(diagram, singleOptions);
      html += '</div>\n';
    }

    html += '</div>\n';

    return html;
  }

  /**
   * Insert diagram sequence (before, during, after)
   */
  insertDiagramSequence(diagrams: DiagramImage[], options: DiagramInsertOptions = {}): string {
    let html = '<div class="diagram-sequence">\n';

    const labels = ['Before Impact', 'At Impact', 'After Impact'];

    diagrams.forEach((diagram, index) => {
      if (index < labels.length) {
        html += `<h4 class="text-center">${labels[index]}</h4>\n`;
      }

      html += this.insertDiagram(diagram, {
        ...options,
        showTitle: false,
        pageBreakBefore: index > 0,
        pageBreakAfter: false
      });
    });

    html += '</div>\n';

    return html;
  }

  /**
   * Insert diagram gallery with thumbnails
   */
  insertDiagramGallery(diagrams: DiagramImage[], columns: number = 3): string {
    let html = '<div class="diagram-gallery">\n';
    html += '<h3>Diagram Gallery</h3>\n';
    html += `<div style="display: grid; grid-template-columns: repeat(${columns}, 1fr); gap: 16px;">\n`;

    for (const diagram of diagrams) {
      html += '  <div class="gallery-item no-break">\n';
      html += `    <img src="${diagram.imageData}" alt="${diagram.metadata.title}" style="width: 100%; border: 1px solid #ccc;" />\n`;
      html += `    <div class="text-center text-small mt-1">${diagram.metadata.title}</div>\n`;
      html += '  </div>\n';
    }

    html += '</div>\n';
    html += '</div>\n';

    return html;
  }

  /**
   * Generate legend HTML
   */
  private generateLegend(legendItems: DiagramLegendItem[]): string {
    let html = '  <div class="diagram-legend">\n';
    html += '    <strong>Legend:</strong>\n';
    html += '    <table style="margin-top: 8px; border: none; width: auto;">\n';

    for (const item of legendItems) {
      html += '      <tr style="border: none;">\n';
      html += `        <td style="border: none; padding: 4px 8px;">\n`;
      html += `          <div style="width: 20px; height: 20px; background-color: ${item.color}; border: 1px solid #666; display: inline-block;"></div>\n`;
      if (item.symbol) {
        html += `          <span style="margin-left: 4px; font-weight: bold;">${item.symbol}</span>\n`;
      }
      html += `        </td>\n`;
      html += `        <td style="border: none; padding: 4px 8px;">${item.label}</td>\n`;
      if (item.description) {
        html += `        <td style="border: none; padding: 4px 8px; color: #666; font-style: italic;">${item.description}</td>\n`;
      }
      html += '      </tr>\n';
    }

    html += '    </table>\n';
    html += '  </div>\n';

    return html;
  }

  /**
   * Calculate print dimensions maintaining aspect ratio
   */
  private calculatePrintDimensions(
    pixelWidth: number,
    pixelHeight: number,
    maxWidthInches: number,
    maxHeightInches: number,
    maintainAspectRatio: boolean
  ): { width: number; height: number } {
    // Convert pixels to inches at print DPI
    let widthInches = pixelWidth / this.printDPI;
    let heightInches = pixelHeight / this.printDPI;

    if (!maintainAspectRatio) {
      return {
        width: Math.min(widthInches, maxWidthInches),
        height: Math.min(heightInches, maxHeightInches)
      };
    }

    // Calculate aspect ratio
    const aspectRatio = pixelWidth / pixelHeight;

    // Scale to fit within max dimensions
    if (widthInches > maxWidthInches) {
      widthInches = maxWidthInches;
      heightInches = widthInches / aspectRatio;
    }

    if (heightInches > maxHeightInches) {
      heightInches = maxHeightInches;
      widthInches = heightInches * aspectRatio;
    }

    return {
      width: Math.round(widthInches * 100) / 100,
      height: Math.round(heightInches * 100) / 100
    };
  }

  /**
   * Convert canvas to base64 image data
   */
  canvasToImageData(canvas: HTMLCanvasElement, format: 'png' | 'jpg' = 'png'): string {
    const mimeType = format === 'png' ? 'image/png' : 'image/jpeg';
    const quality = format === 'jpg' ? 0.95 : undefined;
    return canvas.toDataURL(mimeType, quality);
  }

  /**
   * Add scale bar to diagram
   */
  addScaleBar(
    canvas: HTMLCanvasElement,
    scaleInFeet: number,
    pixelsPerFoot: number,
    position: 'bottom-left' | 'bottom-right' | 'top-left' | 'top-right' = 'bottom-right'
  ): void {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const scaleBarLengthPixels = scaleInFeet * pixelsPerFoot;
    const margin = 20;
    const barHeight = 10;
    const barY = position.startsWith('bottom')
      ? canvas.height - margin - barHeight
      : margin + barHeight;
    const barX = position.endsWith('right')
      ? canvas.width - margin - scaleBarLengthPixels
      : margin;

    // Draw scale bar background
    ctx.fillStyle = 'rgba(255, 255, 255, 0.8)';
    ctx.fillRect(barX - 5, barY - 25, scaleBarLengthPixels + 10, 35);

    // Draw scale bar
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(barX, barY);
    ctx.lineTo(barX + scaleBarLengthPixels, barY);
    ctx.stroke();

    // Draw end marks
    ctx.beginPath();
    ctx.moveTo(barX, barY - 5);
    ctx.lineTo(barX, barY + 5);
    ctx.moveTo(barX + scaleBarLengthPixels, barY - 5);
    ctx.lineTo(barX + scaleBarLengthPixels, barY + 5);
    ctx.stroke();

    // Draw label
    ctx.fillStyle = '#000';
    ctx.font = '12px Arial';
    ctx.textAlign = 'center';
    ctx.fillText(`${scaleInFeet} ft`, barX + scaleBarLengthPixels / 2, barY - 10);
  }

  /**
   * Add north arrow to diagram
   */
  addNorthArrow(
    canvas: HTMLCanvasElement,
    orientation: number,
    position: 'top-left' | 'top-right' | 'bottom-left' | 'bottom-right' = 'top-right'
  ): void {
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const margin = 30;
    const arrowSize = 40;
    const centerX = position.endsWith('right')
      ? canvas.width - margin
      : margin;
    const centerY = position.startsWith('top')
      ? margin
      : canvas.height - margin;

    // Save context
    ctx.save();

    // Translate and rotate
    ctx.translate(centerX, centerY);
    ctx.rotate((orientation * Math.PI) / 180);

    // Draw arrow
    ctx.fillStyle = '#000';
    ctx.strokeStyle = '#000';
    ctx.lineWidth = 2;

    ctx.beginPath();
    ctx.moveTo(0, -arrowSize / 2);
    ctx.lineTo(-arrowSize / 4, arrowSize / 2);
    ctx.lineTo(0, arrowSize / 3);
    ctx.lineTo(arrowSize / 4, arrowSize / 2);
    ctx.closePath();
    ctx.fill();
    ctx.stroke();

    // Draw 'N' label
    ctx.fillStyle = '#fff';
    ctx.font = 'bold 16px Arial';
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    ctx.fillText('N', 0, -arrowSize / 6);

    // Restore context
    ctx.restore();
  }

  /**
   * Export diagram metadata to JSON
   */
  exportMetadata(diagrams: DiagramImage[]): string {
    const metadata = diagrams.map(d => ({
      id: d.metadata.id,
      title: d.metadata.title,
      description: d.metadata.description,
      timestamp: d.metadata.timestamp.toISOString(),
      scale: d.metadata.scale,
      units: d.metadata.units,
      viewType: d.metadata.viewType,
      format: d.format,
      dimensions: d.dimensions
    }));

    return JSON.stringify(metadata, null, 2);
  }

  /**
   * Create contact sheet of all diagrams
   */
  createContactSheet(diagrams: DiagramImage[], columns: number = 4): string {
    let html = '<div class="contact-sheet page-break">\n';
    html += '<h2>Diagram Index</h2>\n';
    html += '<table style="width: 100%;">\n';
    html += '<thead>\n';
    html += '<tr>\n';
    html += '<th>Preview</th>\n';
    html += '<th>Title</th>\n';
    html += '<th>Type</th>\n';
    html += '<th>Scale</th>\n';
    html += '<th>Date</th>\n';
    html += '</tr>\n';
    html += '</thead>\n';
    html += '<tbody>\n';

    for (const diagram of diagrams) {
      html += '<tr>\n';
      html += `<td><img src="${diagram.imageData}" style="width: 100px; height: auto; border: 1px solid #ccc;" /></td>\n`;
      html += `<td>${diagram.metadata.title}</td>\n`;
      html += `<td>${diagram.metadata.viewType}</td>\n`;
      html += `<td>1:${diagram.metadata.scale}</td>\n`;
      html += `<td>${diagram.metadata.timestamp.toLocaleDateString()}</td>\n`;
      html += '</tr>\n';
    }

    html += '</tbody>\n';
    html += '</table>\n';
    html += '</div>\n';

    return html;
  }
}

/**
 * Standard diagram legend items
 */
export const STANDARD_LEGEND_ITEMS: Record<string, DiagramLegendItem> = {
  vehicle1: {
    symbol: 'V1',
    color: '#FF0000',
    label: 'Vehicle 1',
    description: 'Primary vehicle'
  },
  vehicle2: {
    symbol: 'V2',
    color: '#0000FF',
    label: 'Vehicle 2',
    description: 'Secondary vehicle'
  },
  pointOfImpact: {
    symbol: 'POI',
    color: '#FFA500',
    label: 'Point of Impact',
    description: 'Collision location'
  },
  skidMarks: {
    symbol: '---',
    color: '#000000',
    label: 'Skid Marks',
    description: 'Tire friction marks'
  },
  debris: {
    symbol: '▪',
    color: '#808080',
    label: 'Debris Field',
    description: 'Scattered material'
  },
  restPosition: {
    symbol: 'R',
    color: '#00FF00',
    label: 'Rest Position',
    description: 'Final vehicle position'
  },
  pedestrian: {
    symbol: 'P',
    color: '#FF00FF',
    label: 'Pedestrian',
    description: 'Pedestrian location'
  },
  witness: {
    symbol: 'W',
    color: '#00FFFF',
    label: 'Witness',
    description: 'Witness position'
  }
};
