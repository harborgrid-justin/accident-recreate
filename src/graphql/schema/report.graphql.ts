/**
 * Report GraphQL Schema
 * AccuScene Enterprise v0.2.0 - GraphQL Federation API
 */

import { gql } from 'graphql-tag';

export const reportTypeDefs = gql`
  """
  Represents a generated report for a case
  """
  type Report @key(fields: "id") {
    id: ID!
    caseId: ID!
    case: Case
    type: ReportType!
    format: ReportFormat!
    status: ReportStatus!
    title: String!
    content: String
    fileUrl: URL
    fileSize: Int
    generatedBy: ID!
    generatedByUser: User
    generatedAt: DateTime
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Report type enumeration
  """
  enum ReportType {
    PRELIMINARY
    TECHNICAL
    EXECUTIVE
    EXPERT_WITNESS
    INSURANCE
  }

  """
  Report format enumeration
  """
  enum ReportFormat {
    PDF
    DOCX
    HTML
    JSON
  }

  """
  Report status enumeration
  """
  enum ReportStatus {
    DRAFT
    GENERATING
    COMPLETED
    FAILED
  }

  """
  Input type for generating a report
  """
  input GenerateReportInput {
    caseId: ID!
    type: ReportType!
    format: ReportFormat!
    options: ReportOptionsInput
  }

  """
  Report generation options
  """
  input ReportOptionsInput {
    includeSimulations: Boolean = true
    includePhotos: Boolean = true
    includeMetrics: Boolean = true
    customSections: [String!]
    template: String
  }

  """
  Input type for updating report
  """
  input UpdateReportInput {
    title: String
    content: String
    status: ReportStatus
  }

  """
  Report template
  """
  type ReportTemplate {
    id: ID!
    name: String!
    type: ReportType!
    format: ReportFormat!
    sections: [ReportSection!]!
    variables: [String!]!
    createdAt: DateTime!
    updatedAt: DateTime!
  }

  """
  Report section definition
  """
  type ReportSection {
    id: ID!
    title: String!
    order: Int!
    content: String
    required: Boolean!
    variables: [String!]!
  }

  extend type Query {
    """
    Get a single report by ID
    """
    report(id: ID!): Report @auth(requires: [CASE_READ])

    """
    Get all reports for a case
    """
    reportsByCase(caseId: ID!): [Report!]! @auth(requires: [CASE_READ])

    """
    Get report templates
    """
    reportTemplates(type: ReportType): [ReportTemplate!]! @auth(requires: [REPORT_GENERATE])

    """
    Get report template by ID
    """
    reportTemplate(id: ID!): ReportTemplate @auth(requires: [REPORT_GENERATE])

    """
    Preview report before generation
    """
    previewReport(input: GenerateReportInput!): Report @auth(requires: [REPORT_GENERATE])
  }

  extend type Mutation {
    """
    Generate a new report
    """
    generateReport(input: GenerateReportInput!): Report! @auth(requires: [REPORT_GENERATE])

    """
    Update report content
    """
    updateReport(id: ID!, input: UpdateReportInput!): Report! @auth(requires: [REPORT_GENERATE])

    """
    Delete a report
    """
    deleteReport(id: ID!): Boolean! @auth(requires: [CASE_DELETE])

    """
    Regenerate an existing report
    """
    regenerateReport(id: ID!): Report! @auth(requires: [REPORT_GENERATE])

    """
    Export report to different format
    """
    exportReport(id: ID!, format: ReportFormat!): Report! @auth(requires: [REPORT_GENERATE])

    """
    Share report via email
    """
    shareReport(reportId: ID!, recipients: [Email!]!, message: String): Boolean! @auth(requires: [REPORT_GENERATE])

    """
    Create custom report template
    """
    createReportTemplate(input: CreateReportTemplateInput!): ReportTemplate! @auth(requires: [USER_MANAGE])

    """
    Update report template
    """
    updateReportTemplate(id: ID!, input: UpdateReportTemplateInput!): ReportTemplate! @auth(requires: [USER_MANAGE])

    """
    Delete report template
    """
    deleteReportTemplate(id: ID!): Boolean! @auth(requires: [USER_MANAGE])
  }

  """
  Input type for creating report template
  """
  input CreateReportTemplateInput {
    name: String!
    type: ReportType!
    format: ReportFormat!
    sections: [ReportSectionInput!]!
  }

  """
  Input type for updating report template
  """
  input UpdateReportTemplateInput {
    name: String
    sections: [ReportSectionInput!]
  }

  """
  Input type for report section
  """
  input ReportSectionInput {
    title: String!
    order: Int!
    content: String
    required: Boolean!
    variables: [String!]
  }

  extend type Subscription {
    """
    Subscribe to report generation progress
    """
    reportProgress(reportId: ID!): ReportProgressPayload! @auth(requires: [REPORT_GENERATE])

    """
    Subscribe to report completion
    """
    reportCompleted(caseId: ID): Report! @auth(requires: [REPORT_GENERATE])
  }

  """
  Report progress subscription payload
  """
  type ReportProgressPayload {
    report: Report!
    progress: Float!
    status: ReportStatus!
    currentSection: String
  }
`;
