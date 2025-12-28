/**
 * Diagram Exporter
 * AccuScene Enterprise Accident Recreation Platform
 */

import { DiagramState, DiagramElement, ExportOptions, Point } from '../types/diagram';
import { getElementBounds } from './DiagramElements';

export class DiagramExporter {
  /**
   * Export diagram to JSON
   */
  exportToJSON(state: DiagramState, options: ExportOptions = { format: 'json' }): string {
    const exportData = {
      version: '1.0.0',
      timestamp: new Date().toISOString(),
      diagram: options.selectedOnly
        ? {
            ...state,
            elements: state.elements.filter((e) => state.selectedIds.includes(e.id)),
          }
        : state,
      metadata: {
        exportedBy: 'AccuScene Enterprise',
        scale: state.scale,
        gridSize: state.gridSize,
      },
    };

    return JSON.stringify(exportData, null, 2);
  }

  /**
   * Import diagram from JSON
   */
  importFromJSON(json: string): DiagramState | null {
    try {
      const data = JSON.parse(json);
      if (data.diagram) {
        return data.diagram;
      }
      return data;
    } catch (error) {
      console.error('Failed to import JSON:', error);
      return null;
    }
  }

  /**
   * Export diagram to SVG
   */
  exportToSVG(state: DiagramState, options: ExportOptions = { format: 'svg' }): string {
    const elements = options.selectedOnly
      ? state.elements.filter((e) => state.selectedIds.includes(e.id))
      : state.elements;

    if (elements.length === 0) {
      return '<svg></svg>';
    }

    // Calculate bounds
    const bounds = this.calculateBounds(elements);
    const padding = 50;
    const width = bounds.width + padding * 2;
    const height = bounds.height + padding * 2;
    const offsetX = -bounds.x + padding;
    const offsetY = -bounds.y + padding;

    let svg = `<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" width="${width}" height="${height}" viewBox="0 0 ${width} ${height}">
  <defs>
    <style>
      .element { stroke: #000; stroke-width: 2; }
      .label { font-family: Arial, sans-serif; font-size: 12px; fill: #000; }
      .measurement { stroke: #3B82F6; stroke-width: 1; stroke-dasharray: 5,5; }
    </style>
  </defs>
`;

    // Add background
    if (options.includeBackground) {
      svg += `  <rect width="${width}" height="${height}" fill="${state.backgroundColor}"/>\n`;
    }

    // Add grid
    if (options.includeGrid && state.gridVisible) {
      svg += this.generateGridSVG(width, height, state.gridSize * state.scale, offsetX, offsetY);
    }

    // Sort elements by zIndex
    const sortedElements = [...elements].sort((a, b) => a.zIndex - b.zIndex);

    // Add elements
    sortedElements.forEach((element) => {
      if (element.visible) {
        svg += this.elementToSVG(element, offsetX, offsetY, state.scale);
      }
    });

    // Add measurements
    state.measurements.forEach((measurement) => {
      svg += this.measurementToSVG(measurement, offsetX, offsetY);
    });

    svg += '</svg>';
    return svg;
  }

  /**
   * Generate grid SVG
   */
  private generateGridSVG(
    width: number,
    height: number,
    gridSize: number,
    offsetX: number,
    offsetY: number
  ): string {
    let svg = '  <g class="grid" opacity="0.2">\n';

    // Vertical lines
    for (let x = 0; x < width; x += gridSize) {
      svg += `    <line x1="${x}" y1="0" x2="${x}" y2="${height}" stroke="#999" stroke-width="1"/>\n`;
    }

    // Horizontal lines
    for (let y = 0; y < height; y += gridSize) {
      svg += `    <line x1="0" y1="${y}" x2="${width}" y2="${y}" stroke="#999" stroke-width="1"/>\n`;
    }

    svg += '  </g>\n';
    return svg;
  }

  /**
   * Convert element to SVG
   */
  private elementToSVG(element: DiagramElement, offsetX: number, offsetY: number, scale: number): string {
    const { position, rotation } = element.transform;
    const x = position.x + offsetX;
    const y = position.y + offsetY;
    const width = (element.properties.width || 1) * scale;
    const height = (element.properties.height || 1) * scale;
    const color = element.properties.color || element.color || '#000';

    let svg = `  <g transform="translate(${x},${y}) rotate(${rotation})">\n`;

    if (element.properties.shape === 'rectangle' || !element.properties.shape) {
      svg += `    <rect x="${-width / 2}" y="${-height / 2}" width="${width}" height="${height}" fill="${color}" class="element"/>\n`;
    } else if (element.properties.shape === 'circle') {
      const radius = width / 2;
      svg += `    <circle cx="0" cy="0" r="${radius}" fill="${color}" class="element"/>\n`;
    } else if (element.properties.shape === 'polygon' && element.properties.points) {
      const points = element.properties.points
        .map((p: Point) => `${p.x * width},${p.y * height}`)
        .join(' ');
      svg += `    <polygon points="${points}" fill="${color}" class="element"/>\n`;
    }

    // Add label
    if (element.label) {
      svg += `    <text x="0" y="${height / 2 + 15}" text-anchor="middle" class="label">${element.label}</text>\n`;
    }

    svg += '  </g>\n';
    return svg;
  }

  /**
   * Convert measurement to SVG
   */
  private measurementToSVG(measurement: any, offsetX: number, offsetY: number): string {
    let svg = '  <g class="measurement">\n';

    if (measurement.type === 'distance' && measurement.points.length >= 2) {
      const points = measurement.points
        .map((p: Point) => `${p.x + offsetX},${p.y + offsetY}`)
        .join(' ');
      svg += `    <polyline points="${points}" fill="none" stroke="${measurement.color}"/>\n`;

      // Add label at midpoint
      const midpoint = {
        x: (measurement.points[0].x + measurement.points[measurement.points.length - 1].x) / 2 + offsetX,
        y: (measurement.points[0].y + measurement.points[measurement.points.length - 1].y) / 2 + offsetY,
      };
      svg += `    <text x="${midpoint.x}" y="${midpoint.y - 5}" text-anchor="middle" class="label">${measurement.label}</text>\n`;
    }

    svg += '  </g>\n';
    return svg;
  }

  /**
   * Calculate bounds of elements
   */
  private calculateBounds(elements: DiagramElement[]): {
    x: number;
    y: number;
    width: number;
    height: number;
  } {
    if (elements.length === 0) {
      return { x: 0, y: 0, width: 800, height: 600 };
    }

    let minX = Infinity;
    let minY = Infinity;
    let maxX = -Infinity;
    let maxY = -Infinity;

    elements.forEach((element) => {
      const bounds = getElementBounds(element);
      minX = Math.min(minX, bounds.x);
      minY = Math.min(minY, bounds.y);
      maxX = Math.max(maxX, bounds.x + bounds.width);
      maxY = Math.max(maxY, bounds.y + bounds.height);
    });

    return {
      x: minX,
      y: minY,
      width: maxX - minX,
      height: maxY - minY,
    };
  }

  /**
   * Export diagram to PNG (requires canvas element)
   */
  async exportToPNG(
    canvasElement: HTMLCanvasElement,
    options: ExportOptions = { format: 'png', quality: 1.0 }
  ): Promise<Blob | null> {
    return new Promise((resolve) => {
      canvasElement.toBlob(
        (blob) => {
          resolve(blob);
        },
        'image/png',
        options.quality || 1.0
      );
    });
  }

  /**
   * Export diagram to JPG (requires canvas element)
   */
  async exportToJPG(
    canvasElement: HTMLCanvasElement,
    options: ExportOptions = { format: 'jpg', quality: 0.9 }
  ): Promise<Blob | null> {
    return new Promise((resolve) => {
      canvasElement.toBlob(
        (blob) => {
          resolve(blob);
        },
        'image/jpeg',
        options.quality || 0.9
      );
    });
  }

  /**
   * Download file
   */
  downloadFile(content: string | Blob, filename: string): void {
    const blob = typeof content === 'string' ? new Blob([content], { type: 'text/plain' }) : content;
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    URL.revokeObjectURL(url);
  }

  /**
   * Export and download diagram
   */
  async exportAndDownload(
    state: DiagramState,
    canvasElement: HTMLCanvasElement | null,
    options: ExportOptions,
    filename: string
  ): Promise<void> {
    switch (options.format) {
      case 'json': {
        const json = this.exportToJSON(state, options);
        this.downloadFile(json, `${filename}.json`);
        break;
      }
      case 'svg': {
        const svg = this.exportToSVG(state, options);
        this.downloadFile(svg, `${filename}.svg`);
        break;
      }
      case 'png': {
        if (canvasElement) {
          const blob = await this.exportToPNG(canvasElement, options);
          if (blob) {
            this.downloadFile(blob, `${filename}.png`);
          }
        }
        break;
      }
      case 'jpg': {
        if (canvasElement) {
          const blob = await this.exportToJPG(canvasElement, options);
          if (blob) {
            this.downloadFile(blob, `${filename}.jpg`);
          }
        }
        break;
      }
    }
  }

  /**
   * Copy to clipboard
   */
  async copyToClipboard(state: DiagramState, format: 'json' | 'svg' = 'json'): Promise<boolean> {
    try {
      let content: string;

      if (format === 'json') {
        content = this.exportToJSON(state);
      } else {
        content = this.exportToSVG(state);
      }

      await navigator.clipboard.writeText(content);
      return true;
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
      return false;
    }
  }

  /**
   * Generate report data
   */
  generateReportData(state: DiagramState): any {
    return {
      summary: {
        totalElements: state.elements.length,
        vehicles: state.elements.filter((e) => e.type === 'vehicle').length,
        roadElements: state.elements.filter((e) => e.type === 'road').length,
        markers: state.elements.filter((e) => e.type === 'marker').length,
        measurements: state.measurements.length,
      },
      elements: state.elements.map((e) => ({
        id: e.id,
        type: e.type,
        subType: e.subType,
        position: e.transform.position,
        rotation: e.transform.rotation,
        label: e.label,
      })),
      measurements: state.measurements.map((m) => ({
        type: m.type,
        value: m.value,
        unit: m.unit,
        label: m.label,
      })),
      metadata: {
        scale: state.scale,
        gridSize: state.gridSize,
        canvasSize: state.canvasSize,
        timestamp: new Date().toISOString(),
      },
    };
  }
}

/**
 * Create a singleton instance
 */
export const diagramExporter = new DiagramExporter();
