/**
 * Report Builder - Builds report sections with content
 * Assembles complete reports from templates and case data
 */

import { ReportTemplate, ReportSection } from './ReportTemplates';
import { DiagramImage, DiagramInserter } from './DiagramInserter';

export interface CaseData {
  caseId: string;
  caseNumber: string;
  title: string;
  dateOfIncident: Date;
  timeOfIncident: string;
  location: {
    address: string;
    city: string;
    state: string;
    zipCode: string;
    coordinates?: {
      latitude: number;
      longitude: number;
    };
  };
  investigator: {
    name: string;
    title: string;
    credentials: string[];
    agency?: string;
  };
  reportDate: Date;
}

export interface VehicleData {
  vehicleId: string;
  designation: string; // V1, V2, etc.
  year: number;
  make: string;
  model: string;
  color: string;
  vin?: string;
  licensePlate?: string;
  driver: {
    name: string;
    age?: number;
    licenseNumber?: string;
  };
  passengers?: Array<{
    name: string;
    age?: number;
    position: string;
  }>;
  damage: {
    severity: 'minor' | 'moderate' | 'severe' | 'totaled';
    primaryImpact: string;
    description: string;
  };
  speed?: {
    posted: number;
    estimated: number;
    method: string;
  };
}

export interface EnvironmentalData {
  weather: string;
  visibility: string;
  roadCondition: string;
  lighting: string;
  temperature?: number;
  roadwayType: string;
  numberOfLanes: number;
  trafficControl?: string;
}

export interface PhysicsData {
  speedCalculations: Array<{
    method: string;
    description: string;
    inputs: Record<string, number>;
    result: number;
    unit: string;
  }>;
  collisionDynamics: {
    type: string;
    principalDirectionOfForce: string;
    changeInVelocity?: number;
  };
  trajectory: {
    preImpact: string;
    atImpact: string;
    postImpact: string;
  };
  timeDistance?: Array<{
    description: string;
    distance: number;
    time: number;
    speed: number;
  }>;
}

export interface WitnessStatement {
  witnessId: string;
  name: string;
  type: 'driver' | 'passenger' | 'third-party';
  statement: string;
  timestamp: Date;
  credibility?: 'high' | 'medium' | 'low';
}

export interface ReportContent {
  caseData: CaseData;
  vehicles: VehicleData[];
  environment: EnvironmentalData;
  physics?: PhysicsData;
  witnesses?: WitnessStatement[];
  diagrams?: DiagramImage[];
  conclusions?: {
    summary: string;
    sequence: string[];
    causation: string;
    contributingFactors: string[];
  };
  customSections?: Record<string, string>;
}

/**
 * Report Builder Class
 */
export class ReportBuilder {
  private template: ReportTemplate;
  private content: ReportContent;
  private diagramInserter: DiagramInserter;

  constructor(template: ReportTemplate, content: ReportContent) {
    this.template = template;
    this.content = content;
    this.diagramInserter = new DiagramInserter();
  }

  /**
   * Build complete report HTML
   */
  buildReport(): string {
    let html = '';

    // Build enabled sections in order
    const enabledSections = this.template.sections
      .filter(s => s.enabled)
      .sort((a, b) => a.order - b.order);

    for (const section of enabledSections) {
      html += this.buildSection(section);
    }

    return html;
  }

  /**
   * Build a single section
   */
  private buildSection(section: ReportSection): string {
    let html = `<div class="report-section" id="section-${section.id}">\n`;
    html += `<h2>${section.title}</h2>\n`;

    // Build section content based on section ID
    html += this.buildSectionContent(section);

    // Build subsections if any
    if (section.subsections && section.subsections.length > 0) {
      const enabledSubsections = section.subsections
        .filter(s => s.enabled)
        .sort((a, b) => a.order - b.order);

      for (const subsection of enabledSubsections) {
        html += this.buildSubsection(subsection, section.id);
      }
    }

    html += '</div>\n';

    return html;
  }

  /**
   * Build subsection
   */
  private buildSubsection(subsection: ReportSection, parentId: string): string {
    let html = `<div class="report-subsection" id="section-${parentId}-${subsection.id}">\n`;
    html += `<h3>${subsection.title}</h3>\n`;

    html += this.buildSubsectionContent(subsection, parentId);

    html += '</div>\n';

    return html;
  }

  /**
   * Build section content based on section ID
   */
  private buildSectionContent(section: ReportSection): string {
    // Check for custom content first
    if (this.content.customSections && this.content.customSections[section.id]) {
      return `<div class="custom-content">\n${this.content.customSections[section.id]}\n</div>\n`;
    }

    // Build standard sections
    switch (section.id) {
      case 'executive-summary':
        return this.buildExecutiveSummary();
      case 'incident-overview':
        return this.buildIncidentOverview();
      case 'vehicle-information':
        return this.buildVehicleInformation();
      case 'diagrams-photos':
      case 'supporting-diagrams':
      case 'reconstruction-diagrams':
      case 'visual-evidence':
        return this.buildDiagramsSection();
      case 'physics-analysis':
      case 'technical-analysis':
      case 'technical-findings':
        return this.buildPhysicsAnalysis();
      case 'witness-statements':
        return this.buildWitnessStatements();
      case 'conclusions':
      case 'expert-conclusions':
        return this.buildConclusions();
      default:
        return section.content || '<p>Content not available for this section.</p>\n';
    }
  }

  /**
   * Build subsection content
   */
  private buildSubsectionContent(subsection: ReportSection, parentId: string): string {
    const fullId = `${parentId}-${subsection.id}`;

    // Check for custom content
    if (this.content.customSections && this.content.customSections[fullId]) {
      return `<div class="custom-content">\n${this.content.customSections[fullId]}\n</div>\n`;
    }

    // Build standard subsections
    switch (subsection.id) {
      case 'key-findings':
        return this.buildKeyFindings();
      case 'date-time-location':
        return this.buildDateTimeLocation();
      case 'environmental-conditions':
        return this.buildEnvironmentalConditions();
      case 'vehicle-descriptions':
        return this.buildVehicleDescriptions();
      case 'damage-analysis':
        return this.buildDamageAnalysis();
      case 'speed-calculations':
      case 'speed-analysis':
        return this.buildSpeedCalculations();
      case 'collision-dynamics':
        return this.buildCollisionDynamics();
      case 'sequence-of-events':
        return this.buildSequenceOfEvents();
      default:
        return subsection.content || '<p>Content not available for this subsection.</p>\n';
    }
  }

  /**
   * Build Executive Summary
   */
  private buildExecutiveSummary(): string {
    let html = '<div class="executive-summary">\n';
    html += '<p><strong>Case Overview:</strong></p>\n';
    html += `<p>This report presents the findings of an accident reconstruction investigation conducted for case ${this.content.caseData.caseNumber}. `;
    html += `The incident occurred on ${this.content.caseData.dateOfIncident.toLocaleDateString()} at approximately ${this.content.caseData.timeOfIncident} `;
    html += `at ${this.content.caseData.location.address}, ${this.content.caseData.location.city}, ${this.content.caseData.location.state}.</p>\n`;
    html += `<p>This investigation involved ${this.content.vehicles.length} vehicle(s) and was conducted by ${this.content.caseData.investigator.name}, ${this.content.caseData.investigator.title}.</p>\n`;
    html += '</div>\n';
    return html;
  }

  /**
   * Build Key Findings
   */
  private buildKeyFindings(): string {
    if (!this.content.conclusions) {
      return '<p>Key findings are being compiled.</p>\n';
    }

    let html = '<div class="key-finding">\n';
    html += `<p>${this.content.conclusions.summary}</p>\n`;
    html += '</div>\n';

    return html;
  }

  /**
   * Build Incident Overview
   */
  private buildIncidentOverview(): string {
    let html = '<p>The following section provides a comprehensive overview of the incident, including temporal, spatial, and environmental factors.</p>\n';
    return html;
  }

  /**
   * Build Date Time Location
   */
  private buildDateTimeLocation(): string {
    const { dateOfIncident, timeOfIncident, location } = this.content.caseData;

    let html = '<table>\n';
    html += '<tr><th>Date of Incident</th><td>' + dateOfIncident.toLocaleDateString() + '</td></tr>\n';
    html += '<tr><th>Time of Incident</th><td>' + timeOfIncident + '</td></tr>\n';
    html += '<tr><th>Location</th><td>' + location.address + '</td></tr>\n';
    html += '<tr><th>City</th><td>' + location.city + '</td></tr>\n';
    html += '<tr><th>State</th><td>' + location.state + '</td></tr>\n';
    html += '<tr><th>ZIP Code</th><td>' + location.zipCode + '</td></tr>\n';

    if (location.coordinates) {
      html += '<tr><th>Coordinates</th><td>' + location.coordinates.latitude.toFixed(6) + ', ' + location.coordinates.longitude.toFixed(6) + '</td></tr>\n';
    }

    html += '</table>\n';

    return html;
  }

  /**
   * Build Environmental Conditions
   */
  private buildEnvironmentalConditions(): string {
    const env = this.content.environment;

    let html = '<table>\n';
    html += '<tr><th>Weather</th><td>' + env.weather + '</td></tr>\n';
    html += '<tr><th>Visibility</th><td>' + env.visibility + '</td></tr>\n';
    html += '<tr><th>Road Condition</th><td>' + env.roadCondition + '</td></tr>\n';
    html += '<tr><th>Lighting</th><td>' + env.lighting + '</td></tr>\n';

    if (env.temperature !== undefined) {
      html += '<tr><th>Temperature</th><td>' + env.temperature + '°F</td></tr>\n';
    }

    html += '<tr><th>Roadway Type</th><td>' + env.roadwayType + '</td></tr>\n';
    html += '<tr><th>Number of Lanes</th><td>' + env.numberOfLanes + '</td></tr>\n';

    if (env.trafficControl) {
      html += '<tr><th>Traffic Control</th><td>' + env.trafficControl + '</td></tr>\n';
    }

    html += '</table>\n';

    return html;
  }

  /**
   * Build Vehicle Information
   */
  private buildVehicleInformation(): string {
    let html = '<p>The following vehicles were involved in the incident:</p>\n';
    return html;
  }

  /**
   * Build Vehicle Descriptions
   */
  private buildVehicleDescriptions(): string {
    let html = '';

    for (const vehicle of this.content.vehicles) {
      html += `<h4>Vehicle ${vehicle.designation}</h4>\n`;
      html += '<table>\n';
      html += '<tr><th>Year/Make/Model</th><td>' + vehicle.year + ' ' + vehicle.make + ' ' + vehicle.model + '</td></tr>\n';
      html += '<tr><th>Color</th><td>' + vehicle.color + '</td></tr>\n';

      if (vehicle.vin) {
        html += '<tr><th>VIN</th><td>' + vehicle.vin + '</td></tr>\n';
      }

      if (vehicle.licensePlate) {
        html += '<tr><th>License Plate</th><td>' + vehicle.licensePlate + '</td></tr>\n';
      }

      html += '<tr><th>Driver</th><td>' + vehicle.driver.name;
      if (vehicle.driver.age) {
        html += ', Age ' + vehicle.driver.age;
      }
      html += '</td></tr>\n';

      if (vehicle.passengers && vehicle.passengers.length > 0) {
        html += '<tr><th>Passengers</th><td>';
        html += vehicle.passengers.map(p => `${p.name} (${p.position})`).join(', ');
        html += '</td></tr>\n';
      }

      html += '</table>\n';
    }

    return html;
  }

  /**
   * Build Damage Analysis
   */
  private buildDamageAnalysis(): string {
    let html = '';

    for (const vehicle of this.content.vehicles) {
      html += `<h4>Vehicle ${vehicle.designation} Damage</h4>\n`;
      html += '<table>\n';
      html += '<tr><th>Severity</th><td>' + vehicle.damage.severity.toUpperCase() + '</td></tr>\n';
      html += '<tr><th>Primary Impact</th><td>' + vehicle.damage.primaryImpact + '</td></tr>\n';
      html += '<tr><th>Description</th><td>' + vehicle.damage.description + '</td></tr>\n';
      html += '</table>\n';
    }

    return html;
  }

  /**
   * Build Diagrams Section
   */
  private buildDiagramsSection(): string {
    if (!this.content.diagrams || this.content.diagrams.length === 0) {
      return '<p>No diagrams available for this case.</p>\n';
    }

    let html = '<p>The following diagrams illustrate the accident scene and vehicle positions:</p>\n';

    for (const diagram of this.content.diagrams) {
      html += this.diagramInserter.insertDiagram(diagram, {
        showTitle: true,
        showDescription: true,
        showLegend: true,
        showScale: true,
        pageBreakBefore: true
      });
    }

    return html;
  }

  /**
   * Build Physics Analysis
   */
  private buildPhysicsAnalysis(): string {
    if (!this.content.physics) {
      return '<p>Physics analysis is not available for this case.</p>\n';
    }

    let html = '<p>The following technical analysis was conducted to determine vehicle speeds, collision dynamics, and trajectory paths.</p>\n';
    return html;
  }

  /**
   * Build Speed Calculations
   */
  private buildSpeedCalculations(): string {
    if (!this.content.physics || !this.content.physics.speedCalculations) {
      return '<p>Speed calculations not available.</p>\n';
    }

    let html = '';

    for (const calc of this.content.physics.speedCalculations) {
      html += '<div class="calculation-block">\n';
      html += `<div class="calculation-title">${calc.method}</div>\n`;
      html += `<p>${calc.description}</p>\n`;

      html += '<p><strong>Inputs:</strong></p>\n';
      html += '<ul>\n';
      for (const [key, value] of Object.entries(calc.inputs)) {
        html += `<li>${key}: ${value}</li>\n`;
      }
      html += '</ul>\n';

      html += '<div class="calculation-formula">\n';
      html += `<strong>Result:</strong> ${calc.result} ${calc.unit}\n`;
      html += '</div>\n';

      html += '</div>\n';
    }

    return html;
  }

  /**
   * Build Collision Dynamics
   */
  private buildCollisionDynamics(): string {
    if (!this.content.physics || !this.content.physics.collisionDynamics) {
      return '<p>Collision dynamics analysis not available.</p>\n';
    }

    const dynamics = this.content.physics.collisionDynamics;

    let html = '<table>\n';
    html += '<tr><th>Collision Type</th><td>' + dynamics.type + '</td></tr>\n';
    html += '<tr><th>Principal Direction of Force</th><td>' + dynamics.principalDirectionOfForce + '</td></tr>\n';

    if (dynamics.changeInVelocity) {
      html += '<tr><th>Change in Velocity (ΔV)</th><td>' + dynamics.changeInVelocity + ' mph</td></tr>\n';
    }

    html += '</table>\n';

    return html;
  }

  /**
   * Build Witness Statements
   */
  private buildWitnessStatements(): string {
    if (!this.content.witnesses || this.content.witnesses.length === 0) {
      return '<p>No witness statements available.</p>\n';
    }

    let html = '';

    for (const witness of this.content.witnesses) {
      html += '<div class="witness-statement">\n';
      html += `<div class="witness-name">${witness.name} (${witness.type})</div>\n`;
      html += `<div class="text-small text-muted">Statement taken: ${witness.timestamp.toLocaleString()}</div>\n`;

      if (witness.credibility) {
        html += `<div class="text-small text-muted">Credibility: ${witness.credibility}</div>\n`;
      }

      html += '<div class="witness-quote">\n';
      html += `<p>"${witness.statement}"</p>\n`;
      html += '</div>\n';
      html += '</div>\n';
    }

    return html;
  }

  /**
   * Build Conclusions
   */
  private buildConclusions(): string {
    if (!this.content.conclusions) {
      return '<p>Conclusions are being finalized.</p>\n';
    }

    let html = '<div class="executive-summary">\n';
    html += `<p>${this.content.conclusions.summary}</p>\n`;
    html += '</div>\n';

    return html;
  }

  /**
   * Build Sequence of Events
   */
  private buildSequenceOfEvents(): string {
    if (!this.content.conclusions || !this.content.conclusions.sequence) {
      return '<p>Sequence of events is being compiled.</p>\n';
    }

    let html = '<ol>\n';
    for (const event of this.content.conclusions.sequence) {
      html += `<li>${event}</li>\n`;
    }
    html += '</ol>\n';

    return html;
  }

  /**
   * Build table of contents
   */
  buildTableOfContents(): string {
    let html = '<div class="table-of-contents">\n';
    html += '<h2 class="toc-title">Table of Contents</h2>\n';

    const enabledSections = this.template.sections
      .filter(s => s.enabled)
      .sort((a, b) => a.order - b.order);

    let pageNumber = 1;

    for (const section of enabledSections) {
      html += `<div class="toc-item level-1">\n`;
      html += `  <span>${section.title}</span>\n`;
      html += `  <span>${pageNumber}</span>\n`;
      html += '</div>\n';

      pageNumber++;

      if (section.subsections && section.subsections.length > 0) {
        const enabledSubsections = section.subsections
          .filter(s => s.enabled)
          .sort((a, b) => a.order - b.order);

        for (const subsection of enabledSubsections) {
          html += `<div class="toc-item level-2">\n`;
          html += `  <span>${subsection.title}</span>\n`;
          html += `  <span>${pageNumber}</span>\n`;
          html += '</div>\n';
        }
      }
    }

    html += '</div>\n';

    return html;
  }
}
