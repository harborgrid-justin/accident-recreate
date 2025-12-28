/**
 * Report Template Definitions for AccuScene Enterprise
 * Provides predefined templates for various report types
 */

export interface ReportSection {
  id: string;
  title: string;
  enabled: boolean;
  order: number;
  content?: string;
  subsections?: ReportSection[];
}

export interface ReportTemplate {
  id: string;
  name: string;
  description: string;
  category: 'investigation' | 'insurance' | 'legal' | 'law-enforcement' | 'custom';
  sections: ReportSection[];
  metadata: {
    author?: string;
    version: string;
    createdAt: Date;
    updatedAt: Date;
  };
  formatting: {
    pageSize: 'letter' | 'a4' | 'legal';
    orientation: 'portrait' | 'landscape';
    margins: {
      top: number;
      right: number;
      bottom: number;
      left: number;
    };
    includeTableOfContents: boolean;
    includePageNumbers: boolean;
    includeHeaderFooter: boolean;
  };
}

/**
 * Standard Investigation Report Template
 * Comprehensive accident reconstruction report
 */
export const STANDARD_INVESTIGATION: ReportTemplate = {
  id: 'standard-investigation',
  name: 'Standard Investigation Report',
  description: 'Comprehensive accident reconstruction report with full analysis',
  category: 'investigation',
  sections: [
    {
      id: 'executive-summary',
      title: 'Executive Summary',
      enabled: true,
      order: 1,
      subsections: [
        {
          id: 'key-findings',
          title: 'Key Findings',
          enabled: true,
          order: 1
        },
        {
          id: 'recommendations',
          title: 'Recommendations',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'incident-overview',
      title: 'Incident Overview',
      enabled: true,
      order: 2,
      subsections: [
        {
          id: 'date-time-location',
          title: 'Date, Time, and Location',
          enabled: true,
          order: 1
        },
        {
          id: 'environmental-conditions',
          title: 'Environmental Conditions',
          enabled: true,
          order: 2
        },
        {
          id: 'roadway-characteristics',
          title: 'Roadway Characteristics',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'vehicle-information',
      title: 'Vehicle Information',
      enabled: true,
      order: 3,
      subsections: [
        {
          id: 'vehicle-descriptions',
          title: 'Vehicle Descriptions',
          enabled: true,
          order: 1
        },
        {
          id: 'damage-analysis',
          title: 'Damage Analysis',
          enabled: true,
          order: 2
        },
        {
          id: 'mechanical-inspection',
          title: 'Mechanical Inspection Results',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'diagrams-photos',
      title: 'Diagrams and Photographs',
      enabled: true,
      order: 4,
      subsections: [
        {
          id: 'scene-diagrams',
          title: 'Scene Diagrams',
          enabled: true,
          order: 1
        },
        {
          id: 'vehicle-positions',
          title: 'Vehicle Positions',
          enabled: true,
          order: 2
        },
        {
          id: 'photographic-evidence',
          title: 'Photographic Evidence',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'physics-analysis',
      title: 'Physics Analysis',
      enabled: true,
      order: 5,
      subsections: [
        {
          id: 'speed-calculations',
          title: 'Speed Calculations',
          enabled: true,
          order: 1
        },
        {
          id: 'collision-dynamics',
          title: 'Collision Dynamics',
          enabled: true,
          order: 2
        },
        {
          id: 'trajectory-analysis',
          title: 'Trajectory Analysis',
          enabled: true,
          order: 3
        },
        {
          id: 'time-distance-study',
          title: 'Time-Distance Study',
          enabled: true,
          order: 4
        }
      ]
    },
    {
      id: 'witness-statements',
      title: 'Witness Statements',
      enabled: true,
      order: 6,
      subsections: [
        {
          id: 'driver-statements',
          title: 'Driver Statements',
          enabled: true,
          order: 1
        },
        {
          id: 'passenger-statements',
          title: 'Passenger Statements',
          enabled: true,
          order: 2
        },
        {
          id: 'third-party-witnesses',
          title: 'Third-Party Witnesses',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'conclusions',
      title: 'Conclusions and Findings',
      enabled: true,
      order: 7,
      subsections: [
        {
          id: 'sequence-of-events',
          title: 'Sequence of Events',
          enabled: true,
          order: 1
        },
        {
          id: 'causation-analysis',
          title: 'Causation Analysis',
          enabled: true,
          order: 2
        },
        {
          id: 'contributing-factors',
          title: 'Contributing Factors',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'appendices',
      title: 'Appendices',
      enabled: true,
      order: 8,
      subsections: [
        {
          id: 'calculations',
          title: 'Detailed Calculations',
          enabled: true,
          order: 1
        },
        {
          id: 'references',
          title: 'References and Citations',
          enabled: true,
          order: 2
        },
        {
          id: 'investigator-credentials',
          title: 'Investigator Credentials',
          enabled: true,
          order: 3
        }
      ]
    }
  ],
  metadata: {
    version: '1.0.0',
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01')
  },
  formatting: {
    pageSize: 'letter',
    orientation: 'portrait',
    margins: {
      top: 1.0,
      right: 1.0,
      bottom: 1.0,
      left: 1.0
    },
    includeTableOfContents: true,
    includePageNumbers: true,
    includeHeaderFooter: true
  }
};

/**
 * Insurance Summary Report Template
 * Concise report for insurance adjusters
 */
export const INSURANCE_SUMMARY: ReportTemplate = {
  id: 'insurance-summary',
  name: 'Insurance Summary Report',
  description: 'Brief summary report optimized for insurance adjusters',
  category: 'insurance',
  sections: [
    {
      id: 'claim-summary',
      title: 'Claim Summary',
      enabled: true,
      order: 1
    },
    {
      id: 'incident-details',
      title: 'Incident Details',
      enabled: true,
      order: 2,
      subsections: [
        {
          id: 'basic-facts',
          title: 'Basic Facts',
          enabled: true,
          order: 1
        },
        {
          id: 'parties-involved',
          title: 'Parties Involved',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'damage-assessment',
      title: 'Damage Assessment',
      enabled: true,
      order: 3
    },
    {
      id: 'liability-analysis',
      title: 'Liability Analysis',
      enabled: true,
      order: 4,
      subsections: [
        {
          id: 'fault-determination',
          title: 'Fault Determination',
          enabled: true,
          order: 1
        },
        {
          id: 'percentage-allocation',
          title: 'Percentage Allocation',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'supporting-diagrams',
      title: 'Supporting Diagrams',
      enabled: true,
      order: 5
    },
    {
      id: 'adjuster-recommendations',
      title: 'Recommendations',
      enabled: true,
      order: 6
    }
  ],
  metadata: {
    version: '1.0.0',
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01')
  },
  formatting: {
    pageSize: 'letter',
    orientation: 'portrait',
    margins: {
      top: 0.75,
      right: 0.75,
      bottom: 0.75,
      left: 0.75
    },
    includeTableOfContents: false,
    includePageNumbers: true,
    includeHeaderFooter: true
  }
};

/**
 * Police Supplement Report Template
 * Formatted for law enforcement use
 */
export const POLICE_SUPPLEMENT: ReportTemplate = {
  id: 'police-supplement',
  name: 'Police Supplemental Report',
  description: 'Technical supplement to police accident reports',
  category: 'law-enforcement',
  sections: [
    {
      id: 'case-reference',
      title: 'Case Reference Information',
      enabled: true,
      order: 1,
      subsections: [
        {
          id: 'report-number',
          title: 'Police Report Number',
          enabled: true,
          order: 1
        },
        {
          id: 'agency-info',
          title: 'Agency Information',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'technical-findings',
      title: 'Technical Findings',
      enabled: true,
      order: 2,
      subsections: [
        {
          id: 'speed-analysis',
          title: 'Speed Analysis',
          enabled: true,
          order: 1
        },
        {
          id: 'point-of-impact',
          title: 'Point of Impact',
          enabled: true,
          order: 2
        },
        {
          id: 'sight-distance',
          title: 'Sight Distance Analysis',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'reconstruction-diagrams',
      title: 'Reconstruction Diagrams',
      enabled: true,
      order: 3
    },
    {
      id: 'violations-identified',
      title: 'Traffic Violations Identified',
      enabled: true,
      order: 4
    },
    {
      id: 'supporting-evidence',
      title: 'Supporting Evidence',
      enabled: true,
      order: 5,
      subsections: [
        {
          id: 'physical-evidence',
          title: 'Physical Evidence',
          enabled: true,
          order: 1
        },
        {
          id: 'witness-corroboration',
          title: 'Witness Corroboration',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'expert-conclusions',
      title: 'Expert Conclusions',
      enabled: true,
      order: 6
    }
  ],
  metadata: {
    version: '1.0.0',
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01')
  },
  formatting: {
    pageSize: 'letter',
    orientation: 'portrait',
    margins: {
      top: 1.0,
      right: 1.0,
      bottom: 1.0,
      left: 1.0
    },
    includeTableOfContents: false,
    includePageNumbers: true,
    includeHeaderFooter: true
  }
};

/**
 * Litigation Support Report Template
 * Formatted for legal proceedings and expert testimony
 */
export const LITIGATION_SUPPORT: ReportTemplate = {
  id: 'litigation-support',
  name: 'Litigation Support Report',
  description: 'Comprehensive report formatted for legal proceedings',
  category: 'legal',
  sections: [
    {
      id: 'expert-qualifications',
      title: 'Expert Qualifications',
      enabled: true,
      order: 1,
      subsections: [
        {
          id: 'education',
          title: 'Education and Training',
          enabled: true,
          order: 1
        },
        {
          id: 'experience',
          title: 'Professional Experience',
          enabled: true,
          order: 2
        },
        {
          id: 'certifications',
          title: 'Certifications',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'scope-of-investigation',
      title: 'Scope of Investigation',
      enabled: true,
      order: 2,
      subsections: [
        {
          id: 'materials-reviewed',
          title: 'Materials Reviewed',
          enabled: true,
          order: 1
        },
        {
          id: 'investigation-methodology',
          title: 'Investigation Methodology',
          enabled: true,
          order: 2
        }
      ]
    },
    {
      id: 'factual-findings',
      title: 'Factual Findings',
      enabled: true,
      order: 3,
      subsections: [
        {
          id: 'undisputed-facts',
          title: 'Undisputed Facts',
          enabled: true,
          order: 1
        },
        {
          id: 'physical-evidence',
          title: 'Physical Evidence Analysis',
          enabled: true,
          order: 2
        },
        {
          id: 'documentary-evidence',
          title: 'Documentary Evidence',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'technical-analysis',
      title: 'Technical Analysis',
      enabled: true,
      order: 4,
      subsections: [
        {
          id: 'reconstruction-methodology',
          title: 'Reconstruction Methodology',
          enabled: true,
          order: 1
        },
        {
          id: 'calculations-simulations',
          title: 'Calculations and Simulations',
          enabled: true,
          order: 2
        },
        {
          id: 'scientific-principles',
          title: 'Scientific Principles Applied',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'visual-evidence',
      title: 'Visual Evidence',
      enabled: true,
      order: 5,
      subsections: [
        {
          id: 'diagrams',
          title: 'Reconstruction Diagrams',
          enabled: true,
          order: 1
        },
        {
          id: 'photographs',
          title: 'Photographs',
          enabled: true,
          order: 2
        },
        {
          id: 'animations',
          title: 'Animations and Simulations',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'expert-opinions',
      title: 'Expert Opinions',
      enabled: true,
      order: 6,
      subsections: [
        {
          id: 'primary-conclusions',
          title: 'Primary Conclusions',
          enabled: true,
          order: 1
        },
        {
          id: 'alternative-scenarios',
          title: 'Alternative Scenarios Considered',
          enabled: true,
          order: 2
        },
        {
          id: 'certainty-assessment',
          title: 'Degree of Certainty',
          enabled: true,
          order: 3
        }
      ]
    },
    {
      id: 'references-appendices',
      title: 'References and Appendices',
      enabled: true,
      order: 7,
      subsections: [
        {
          id: 'cited-references',
          title: 'Cited References',
          enabled: true,
          order: 1
        },
        {
          id: 'detailed-calculations',
          title: 'Detailed Calculations',
          enabled: true,
          order: 2
        },
        {
          id: 'cv-attachment',
          title: 'Curriculum Vitae',
          enabled: true,
          order: 3
        }
      ]
    }
  ],
  metadata: {
    version: '1.0.0',
    createdAt: new Date('2024-01-01'),
    updatedAt: new Date('2024-01-01')
  },
  formatting: {
    pageSize: 'letter',
    orientation: 'portrait',
    margins: {
      top: 1.0,
      right: 1.0,
      bottom: 1.0,
      left: 1.5
    },
    includeTableOfContents: true,
    includePageNumbers: true,
    includeHeaderFooter: true
  }
};

/**
 * Template Registry
 */
export class TemplateRegistry {
  private static templates: Map<string, ReportTemplate> = new Map([
    ['standard-investigation', STANDARD_INVESTIGATION],
    ['insurance-summary', INSURANCE_SUMMARY],
    ['police-supplement', POLICE_SUPPLEMENT],
    ['litigation-support', LITIGATION_SUPPORT]
  ]);

  private static customTemplates: Map<string, ReportTemplate> = new Map();

  /**
   * Get all available templates
   */
  static getAllTemplates(): ReportTemplate[] {
    return [
      ...Array.from(this.templates.values()),
      ...Array.from(this.customTemplates.values())
    ];
  }

  /**
   * Get template by ID
   */
  static getTemplate(templateId: string): ReportTemplate | null {
    return this.templates.get(templateId) || this.customTemplates.get(templateId) || null;
  }

  /**
   * Get templates by category
   */
  static getTemplatesByCategory(category: ReportTemplate['category']): ReportTemplate[] {
    return this.getAllTemplates().filter(t => t.category === category);
  }

  /**
   * Register a custom template
   */
  static registerCustomTemplate(template: ReportTemplate): void {
    if (this.templates.has(template.id)) {
      throw new Error(`Template ID ${template.id} conflicts with built-in template`);
    }
    template.category = 'custom';
    template.metadata.updatedAt = new Date();
    this.customTemplates.set(template.id, template);
  }

  /**
   * Update an existing custom template
   */
  static updateCustomTemplate(templateId: string, updates: Partial<ReportTemplate>): void {
    const template = this.customTemplates.get(templateId);
    if (!template) {
      throw new Error(`Custom template ${templateId} not found`);
    }
    const updated = { ...template, ...updates };
    updated.metadata.updatedAt = new Date();
    this.customTemplates.set(templateId, updated);
  }

  /**
   * Delete a custom template
   */
  static deleteCustomTemplate(templateId: string): boolean {
    return this.customTemplates.delete(templateId);
  }

  /**
   * Clone a template to create a custom version
   */
  static cloneTemplate(templateId: string, newId: string, newName: string): ReportTemplate {
    const source = this.getTemplate(templateId);
    if (!source) {
      throw new Error(`Template ${templateId} not found`);
    }

    const cloned: ReportTemplate = {
      ...JSON.parse(JSON.stringify(source)),
      id: newId,
      name: newName,
      category: 'custom',
      metadata: {
        ...source.metadata,
        createdAt: new Date(),
        updatedAt: new Date()
      }
    };

    this.registerCustomTemplate(cloned);
    return cloned;
  }
}
