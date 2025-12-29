/**
 * AccuScene Enterprise v0.3.0 - Report Generator
 * Automated report generation with templates and customization
 */

import {
  GeneratedReport,
  ReportTemplate,
  ReportSection,
  AnalyticsData,
} from './types';

export class ReportGenerator {
  /**
   * Generate a report from data using a template
   */
  static generate(
    data: AnalyticsData,
    template: ReportTemplate,
    author?: string
  ): GeneratedReport {
    const sections: ReportSection[] = template.sections.map((section) =>
      this.generateSection(section, data)
    );

    return {
      id: `report-${Date.now()}`,
      template: template.id,
      data,
      generatedAt: Date.now(),
      author,
      sections,
      metadata: {
        ...template.metadata,
        generatedBy: 'AccuScene Enterprise v0.3.0',
      },
    };
  }

  /**
   * Generate a section based on type and data
   */
  private static generateSection(
    template: ReportSection,
    data: AnalyticsData
  ): ReportSection {
    switch (template.type) {
      case 'text':
        return this.generateTextSection(template, data);
      case 'chart':
        return this.generateChartSection(template, data);
      case 'table':
        return this.generateTableSection(template, data);
      case 'summary':
        return this.generateSummarySection(template, data);
      default:
        return template;
    }
  }

  /**
   * Generate text section
   */
  private static generateTextSection(
    template: ReportSection,
    data: AnalyticsData
  ): ReportSection {
    // Replace template variables with actual data
    let content = template.content;

    if (typeof content === 'string') {
      content = content
        .replace('{vehicleCount}', String(data.vehicles?.length || 0))
        .replace('{impactCount}', String(data.impacts?.length || 0))
        .replace('{forceVectorCount}', String(data.forceVectors?.length || 0))
        .replace('{energyTransferCount}', String(data.energyTransfers?.length || 0));
    }

    return {
      ...template,
      content,
    };
  }

  /**
   * Generate chart section
   */
  private static generateChartSection(
    template: ReportSection,
    data: AnalyticsData
  ): ReportSection {
    // Extract chart data based on template configuration
    // In production, this would prepare data for specific chart types

    return {
      ...template,
      content: {
        chartType: template.content?.chartType || 'line',
        data: this.extractChartData(template.content?.dataSource, data),
      },
    };
  }

  /**
   * Generate table section
   */
  private static generateTableSection(
    template: ReportSection,
    data: AnalyticsData
  ): ReportSection {
    const tableData = this.extractTableData(template.content?.dataSource, data);

    return {
      ...template,
      content: {
        headers: template.content?.headers || [],
        rows: tableData,
      },
    };
  }

  /**
   * Generate summary section
   */
  private static generateSummarySection(
    template: ReportSection,
    data: AnalyticsData
  ): ReportSection {
    const summary = {
      overview: this.generateOverview(data),
      keyFindings: this.generateKeyFindings(data),
      recommendations: this.generateRecommendations(data),
    };

    return {
      ...template,
      content: summary,
    };
  }

  /**
   * Generate overview text
   */
  private static generateOverview(data: AnalyticsData): string {
    const vehicleCount = data.vehicles?.length || 0;
    const impactCount = data.impacts?.length || 0;

    return `Analysis of ${vehicleCount} vehicle${vehicleCount !== 1 ? 's' : ''} involved in ${impactCount} impact event${impactCount !== 1 ? 's' : ''}. ` +
      `Total energy dissipated: ${this.calculateTotalEnergy(data).toFixed(0)} Joules.`;
  }

  /**
   * Generate key findings
   */
  private static generateKeyFindings(data: AnalyticsData): string[] {
    const findings: string[] = [];

    // Critical impacts
    if (data.impacts) {
      const criticalImpacts = data.impacts.filter((i) => i.severity > 0.7);
      if (criticalImpacts.length > 0) {
        findings.push(
          `${criticalImpacts.length} critical impact event${criticalImpacts.length !== 1 ? 's' : ''} detected with severity above 70%`
        );
      }
    }

    // Maximum force
    if (data.forceVectors && data.forceVectors.length > 0) {
      const maxForce = Math.max(...data.forceVectors.map((f) => f.magnitude));
      findings.push(`Maximum force recorded: ${maxForce.toFixed(0)} Newtons`);
    }

    // Energy analysis
    const totalEnergy = this.calculateTotalEnergy(data);
    if (totalEnergy > 100000) {
      findings.push('High-energy collision detected requiring detailed structural analysis');
    }

    return findings;
  }

  /**
   * Generate recommendations
   */
  private static generateRecommendations(data: AnalyticsData): string[] {
    const recommendations: string[] = [];

    if (data.impacts) {
      const criticalImpacts = data.impacts.filter((i) => i.severity > 0.7);
      if (criticalImpacts.length > 0) {
        recommendations.push('Conduct detailed structural integrity assessment');
        recommendations.push('Review safety equipment performance');
      }
    }

    if (data.vehicles) {
      data.vehicles.forEach((vehicle) => {
        if (vehicle.damageProfile && vehicle.damageProfile.length > 0) {
          const criticalDamage = vehicle.damageProfile.filter((d) => d.severity > 0.7);
          if (criticalDamage.length > 0) {
            recommendations.push(`${vehicle.name}: Comprehensive damage assessment required`);
          }
        }
      });
    }

    return recommendations;
  }

  /**
   * Extract chart data based on data source
   */
  private static extractChartData(dataSource: string | undefined, data: AnalyticsData): any[] {
    if (!dataSource) return [];

    switch (dataSource) {
      case 'impacts':
        return data.impacts || [];
      case 'forces':
        return data.forceVectors || [];
      case 'energy':
        return data.energyTransfers || [];
      default:
        return [];
    }
  }

  /**
   * Extract table data based on data source
   */
  private static extractTableData(dataSource: string | undefined, data: AnalyticsData): any[][] {
    if (!dataSource) return [];

    switch (dataSource) {
      case 'impacts':
        return (data.impacts || []).map((impact) => [
          impact.timestamp.toFixed(2),
          impact.type,
          `(${impact.location.x.toFixed(1)}, ${impact.location.y.toFixed(1)}, ${impact.location.z.toFixed(1)})`,
          impact.energy.toFixed(0),
          `${(impact.severity * 100).toFixed(0)}%`,
        ]);

      case 'forces':
        return (data.forceVectors || []).map((force) => [
          force.timestamp.toFixed(2),
          force.type,
          force.magnitude.toFixed(0),
          `(${force.direction.x.toFixed(2)}, ${force.direction.y.toFixed(2)}, ${force.direction.z.toFixed(2)})`,
        ]);

      default:
        return [];
    }
  }

  /**
   * Calculate total energy from impacts
   */
  private static calculateTotalEnergy(data: AnalyticsData): number {
    if (!data.impacts) return 0;
    return data.impacts.reduce((sum, impact) => sum + impact.energy, 0);
  }

  /**
   * Get default report template
   */
  static getDefaultTemplate(): ReportTemplate {
    return {
      id: 'default',
      name: 'Standard Analytics Report',
      description: 'Comprehensive analytics report with all metrics',
      sections: [
        {
          id: 'title',
          type: 'text',
          title: 'Executive Summary',
          content: 'Analysis of {vehicleCount} vehicles in {impactCount} impact events.',
          order: 0,
        },
        {
          id: 'summary',
          type: 'summary',
          title: 'Summary',
          content: {},
          order: 1,
        },
        {
          id: 'impacts-table',
          type: 'table',
          title: 'Impact Events',
          content: {
            dataSource: 'impacts',
            headers: ['Time', 'Type', 'Location', 'Energy', 'Severity'],
          },
          order: 2,
        },
        {
          id: 'forces-table',
          type: 'table',
          title: 'Force Vectors',
          content: {
            dataSource: 'forces',
            headers: ['Time', 'Type', 'Magnitude', 'Direction'],
          },
          order: 3,
        },
      ],
      metadata: {
        version: '1.0',
        author: 'AccuScene Enterprise',
      },
    };
  }

  /**
   * Convert report to HTML
   */
  static toHTML(report: GeneratedReport): string {
    const html: string[] = [];

    html.push('<!DOCTYPE html>');
    html.push('<html lang="en">');
    html.push('<head>');
    html.push('<meta charset="UTF-8">');
    html.push('<title>AccuScene Analytics Report</title>');
    html.push('<style>');
    html.push('body { font-family: Arial, sans-serif; margin: 2rem; }');
    html.push('h1 { color: #2563eb; }');
    html.push('h2 { color: #475569; margin-top: 2rem; }');
    html.push('table { border-collapse: collapse; width: 100%; margin: 1rem 0; }');
    html.push('th, td { border: 1px solid #cbd5e1; padding: 0.5rem; text-align: left; }');
    html.push('th { background-color: #f1f5f9; }');
    html.push('.metadata { color: #64748b; font-size: 0.875rem; }');
    html.push('</style>');
    html.push('</head>');
    html.push('<body>');

    html.push('<h1>AccuScene Analytics Report</h1>');
    html.push(`<div class="metadata">Generated: ${new Date(report.generatedAt).toLocaleString()}</div>`);
    if (report.author) {
      html.push(`<div class="metadata">Author: ${report.author}</div>`);
    }

    // Render sections
    report.sections
      .sort((a, b) => a.order - b.order)
      .forEach((section) => {
        html.push(this.sectionToHTML(section));
      });

    html.push('</body>');
    html.push('</html>');

    return html.join('\n');
  }

  /**
   * Convert section to HTML
   */
  private static sectionToHTML(section: ReportSection): string {
    const html: string[] = [];

    if (section.title) {
      html.push(`<h2>${section.title}</h2>`);
    }

    switch (section.type) {
      case 'text':
        html.push(`<p>${section.content}</p>`);
        break;

      case 'table':
        if (section.content?.headers && section.content?.rows) {
          html.push('<table>');
          html.push('<thead><tr>');
          section.content.headers.forEach((header: string) => {
            html.push(`<th>${header}</th>`);
          });
          html.push('</tr></thead>');
          html.push('<tbody>');
          section.content.rows.forEach((row: any[]) => {
            html.push('<tr>');
            row.forEach((cell) => {
              html.push(`<td>${cell}</td>`);
            });
            html.push('</tr>');
          });
          html.push('</tbody>');
          html.push('</table>');
        }
        break;

      case 'summary':
        if (section.content?.overview) {
          html.push(`<p>${section.content.overview}</p>`);
        }
        if (section.content?.keyFindings) {
          html.push('<h3>Key Findings</h3>');
          html.push('<ul>');
          section.content.keyFindings.forEach((finding: string) => {
            html.push(`<li>${finding}</li>`);
          });
          html.push('</ul>');
        }
        if (section.content?.recommendations) {
          html.push('<h3>Recommendations</h3>');
          html.push('<ul>');
          section.content.recommendations.forEach((rec: string) => {
            html.push(`<li>${rec}</li>`);
          });
          html.push('</ul>');
        }
        break;
    }

    return html.join('\n');
  }
}
