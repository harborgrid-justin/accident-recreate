/**
 * Transfer module type definitions
 */

export type TransferFormat = 'csv' | 'excel' | 'json' | 'xml' | 'pdf' | 'archive';

export interface TransferConfig {
  maxFileSize: number;
  chunkSize: number;
  enableCompression: boolean;
  compressionLevel: number;
  csvDelimiter: string;
  csvHasHeader: boolean;
  excelSheetIndex: number;
  excelSheetName?: string;
  jsonPretty: boolean;
  xmlRootElement: string;
  includeMetadata: boolean;
  fieldMappings: Record<string, string>;
  dateFormat: string;
  timezone: string;
  skipValidation: boolean;
  continueOnError: boolean;
  errorThreshold: number;
}

export interface TransferProgress {
  id: string;
  total: number;
  processed: number;
  bytesProcessed: number;
  totalBytes?: number;
  status: ProgressStatus;
  stage: string;
  error?: string;
  etaSeconds?: number;
  startedAt: string;
  updatedAt: string;
}

export type ProgressStatus = 'pending' | 'running' | 'paused' | 'completed' | 'failed' | 'cancelled';

export interface TransferMetadata {
  sourceFormat?: TransferFormat;
  targetFormat: TransferFormat;
  recordCount: number;
  fieldCount: number;
  fileSize: number;
  createdAt: string;
  createdBy?: string;
  schemaVersion: string;
  custom: Record<string, string>;
}

export interface DataRecord {
  fields: Record<string, any>;
}

export interface FieldSchema {
  name: string;
  dataType: DataType;
  nullable: boolean;
  samples: string[];
  uniqueCount: number;
  nullCount: number;
  minValue?: number;
  maxValue?: number;
  minLength?: number;
  maxLength?: number;
  pattern?: string;
}

export type DataType = 'string' | 'integer' | 'float' | 'boolean' | 'date' | 'datetime' | 'time' | 'array' | 'object' | 'unknown';

export interface FieldMapping {
  source: string;
  target: string;
  transform?: Transform;
  default?: any;
  required: boolean;
}

export interface MappingProfile {
  name: string;
  description?: string;
  mappings: FieldMapping[];
  globalTransforms: Transform[];
}

export type Transform =
  | { type: 'to_upper_case' }
  | { type: 'to_lower_case' }
  | { type: 'trim' }
  | { type: 'replace'; from: string; to: string }
  | { type: 'format_number'; decimals: number }
  | { type: 'parse_number' }
  | { type: 'format_date'; from: string; to: string }
  | { type: 'prefix'; prefix: string }
  | { type: 'suffix'; suffix: string }
  | { type: 'substring'; start: number; end?: number }
  | { type: 'concat'; fields: string[]; separator: string }
  | { type: 'split'; delimiter: string; index: number }
  | { type: 'default_if_null'; default: any }
  | { type: 'convert_type'; targetType: string }
  | { type: 'custom'; function: string };

export interface ValidationRule {
  field: string;
  ruleType: ValidationRuleType;
  message?: string;
}

export type ValidationRuleType =
  | { type: 'required' }
  | { type: 'min_length'; min: number }
  | { type: 'max_length'; max: number }
  | { type: 'pattern'; regex: string }
  | { type: 'min_value'; min: number }
  | { type: 'max_value'; max: number }
  | { type: 'email' }
  | { type: 'url' }
  | { type: 'date'; format: string }
  | { type: 'enum'; values: string[] }
  | { type: 'custom'; validator: string };

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  warnings: ValidationWarning[];
  totalRecords: number;
  validRecords: number;
  invalidRecords: number;
}

export interface ValidationError {
  recordIndex?: number;
  field?: string;
  message: string;
  code: string;
}

export interface ValidationWarning {
  recordIndex?: number;
  field?: string;
  message: string;
}

export interface ExportRequest {
  format: TransferFormat;
  config: Partial<TransferConfig>;
  data: DataRecord[];
  mappingProfile?: MappingProfile;
}

export interface ImportRequest {
  file: File;
  format?: TransferFormat;
  config: Partial<TransferConfig>;
  mappingProfile?: MappingProfile;
  validate: boolean;
}

export interface ExportResponse {
  success: boolean;
  data?: Blob;
  metadata?: TransferMetadata;
  progress?: TransferProgress;
  error?: string;
}

export interface ImportResponse {
  success: boolean;
  data?: DataRecord[];
  metadata?: TransferMetadata;
  schema?: FieldSchema[];
  validationResult?: ValidationResult;
  progress?: TransferProgress;
  error?: string;
}

export interface WizardStep {
  id: string;
  title: string;
  description: string;
  component: React.ComponentType<any>;
  optional?: boolean;
  validate?: () => Promise<boolean>;
}

export interface ExportWizardState {
  currentStep: number;
  format?: TransferFormat;
  config: Partial<TransferConfig>;
  data: DataRecord[];
  mappingProfile?: MappingProfile;
  previewData?: DataRecord[];
  validationResult?: ValidationResult;
  progress?: TransferProgress;
}

export interface ImportWizardState {
  currentStep: number;
  file?: File;
  format?: TransferFormat;
  config: Partial<TransferConfig>;
  detectedSchema?: FieldSchema[];
  mappingProfile?: MappingProfile;
  previewData?: DataRecord[];
  validationResult?: ValidationResult;
  progress?: TransferProgress;
  importedData?: DataRecord[];
}

export interface FormatOption {
  format: TransferFormat;
  name: string;
  description: string;
  icon: string;
  supportsImport: boolean;
  supportsExport: boolean;
  fileExtension: string;
  mimeType: string;
}

export const DEFAULT_CONFIG: TransferConfig = {
  maxFileSize: 100 * 1024 * 1024, // 100MB
  chunkSize: 1024 * 1024, // 1MB
  enableCompression: true,
  compressionLevel: 6,
  csvDelimiter: ',',
  csvHasHeader: true,
  excelSheetIndex: 0,
  jsonPretty: true,
  xmlRootElement: 'data',
  includeMetadata: true,
  fieldMappings: {},
  dateFormat: 'YYYY-MM-DD HH:mm:ss',
  timezone: 'UTC',
  skipValidation: false,
  continueOnError: false,
  errorThreshold: 100,
};

export const FORMAT_OPTIONS: FormatOption[] = [
  {
    format: 'csv',
    name: 'CSV',
    description: 'Comma-Separated Values - Universal spreadsheet format',
    icon: '📊',
    supportsImport: true,
    supportsExport: true,
    fileExtension: 'csv',
    mimeType: 'text/csv',
  },
  {
    format: 'excel',
    name: 'Excel',
    description: 'Microsoft Excel (.xlsx) - Rich spreadsheet format',
    icon: '📈',
    supportsImport: true,
    supportsExport: true,
    fileExtension: 'xlsx',
    mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
  },
  {
    format: 'json',
    name: 'JSON',
    description: 'JavaScript Object Notation - Web-friendly data format',
    icon: '{}',
    supportsImport: true,
    supportsExport: true,
    fileExtension: 'json',
    mimeType: 'application/json',
  },
  {
    format: 'xml',
    name: 'XML',
    description: 'Extensible Markup Language - Enterprise data format',
    icon: '📄',
    supportsImport: true,
    supportsExport: true,
    fileExtension: 'xml',
    mimeType: 'application/xml',
  },
  {
    format: 'pdf',
    name: 'PDF',
    description: 'Portable Document Format - Print-ready reports',
    icon: '📑',
    supportsImport: false,
    supportsExport: true,
    fileExtension: 'pdf',
    mimeType: 'application/pdf',
  },
  {
    format: 'archive',
    name: 'ZIP Archive',
    description: 'Compressed archive with multiple formats',
    icon: '🗜️',
    supportsImport: true,
    supportsExport: true,
    fileExtension: 'zip',
    mimeType: 'application/zip',
  },
];
