import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  ManyToOne,
  JoinColumn,
  Index,
  BeforeInsert,
  BeforeUpdate,
} from 'typeorm';
import { Accident } from './Accident';

/**
 * Evidence type enum
 */
export enum EvidenceType {
  PHOTO = 'photo',
  VIDEO = 'video',
  AUDIO = 'audio',
  DOCUMENT = 'document',
  PHYSICAL = 'physical',
  DIGITAL = 'digital',
  FORENSIC = 'forensic',
  SURVEILLANCE = 'surveillance',
  DASHCAM = 'dashcam',
  POLICE_REPORT = 'police_report',
  MEDICAL_RECORD = 'medical_record',
  OTHER = 'other',
}

/**
 * Evidence source
 */
export enum EvidenceSource {
  SCENE = 'scene',
  WITNESS = 'witness',
  POLICE = 'police',
  INSURANCE = 'insurance',
  MEDICAL = 'medical',
  SURVEILLANCE = 'surveillance',
  VEHICLE = 'vehicle',
  THIRD_PARTY = 'third_party',
  INVESTIGATOR = 'investigator',
}

/**
 * Chain of custody status
 */
export enum CustodyStatus {
  COLLECTED = 'collected',
  STORED = 'stored',
  ANALYZED = 'analyzed',
  TRANSFERRED = 'transferred',
  ARCHIVED = 'archived',
  DISPOSED = 'disposed',
}

/**
 * Evidence entity - Represents physical and digital evidence
 * Maintains chain of custody and metadata
 */
@Entity('evidence')
export class Evidence {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 36 })
  @Index()
  accidentId: string;

  @Column({ type: 'varchar', length: 50, unique: true })
  @Index()
  evidenceNumber: string; // Unique tracking number (e.g., "EV-2024-00123-001")

  @Column({
    type: 'varchar',
    length: 30,
  })
  type: EvidenceType;

  @Column({
    type: 'varchar',
    length: 30,
    default: EvidenceSource.SCENE,
  })
  source: EvidenceSource;

  @Column({ type: 'varchar', length: 255 })
  description: string;

  @Column({ type: 'varchar', length: 500, nullable: true })
  filePath: string; // Path to stored file (for digital evidence)

  @Column({ type: 'varchar', length: 100, nullable: true })
  fileName: string;

  @Column({ type: 'varchar', length: 50, nullable: true })
  fileType: string; // MIME type or file extension

  @Column({ type: 'int', nullable: true })
  fileSize: number; // Size in bytes

  @Column({ type: 'varchar', length: 64, nullable: true })
  fileHash: string; // SHA-256 hash for integrity verification

  @Column({ type: 'datetime' })
  @Index()
  timestamp: Date; // When evidence was collected/created

  @Column({ type: 'varchar', length: 255 })
  collectedBy: string; // Person who collected the evidence

  @Column({ type: 'varchar', length: 500, nullable: true })
  collectionLocation: string;

  @Column({ type: 'text', nullable: true })
  collectionMethod: string;

  @Column({
    type: 'varchar',
    length: 30,
    default: CustodyStatus.COLLECTED,
  })
  custodyStatus: CustodyStatus;

  @Column({ type: 'varchar', length: 255, nullable: true })
  currentCustodian: string; // Current person/entity holding the evidence

  @Column({ type: 'varchar', length: 500, nullable: true })
  storageLocation: string;

  @Column({ type: 'simple-json', nullable: true })
  chainOfCustody: Array<{
    date: Date;
    from: string;
    to: string;
    reason: string;
    signature?: string;
  }>;

  @Column({ type: 'boolean', default: true })
  isOriginal: boolean;

  @Column({ type: 'int', default: 1 })
  copyNumber: number; // If not original, which copy is this

  @Column({ type: 'varchar', length: 36, nullable: true })
  originalEvidenceId: string; // Reference to original evidence if this is a copy

  @Column({ type: 'text', nullable: true })
  analysisNotes: string;

  @Column({ type: 'text', nullable: true })
  findings: string; // Results from analysis

  @Column({ type: 'datetime', nullable: true })
  analyzedDate: Date;

  @Column({ type: 'varchar', length: 255, nullable: true })
  analyzedBy: string;

  @Column({ type: 'boolean', default: true })
  isAdmissible: boolean; // Whether evidence is admissible in court

  @Column({ type: 'text', nullable: true })
  admissibilityNotes: string;

  @Column({ type: 'simple-json', nullable: true })
  tags: string[]; // Tags for categorization

  @Column({ type: 'simple-json', nullable: true })
  relatedVehicles: number[]; // Vehicle numbers this evidence relates to

  @Column({ type: 'simple-json', nullable: true })
  relatedWitnesses: string[]; // Witness IDs this evidence relates to

  @Column({ type: 'int', nullable: true })
  priority: number; // 1-5, with 5 being highest priority

  @Column({ type: 'boolean', default: false })
  requiresExpertAnalysis: boolean;

  @Column({ type: 'varchar', length: 255, nullable: true })
  expertAssigned: string;

  @Column({ type: 'simple-json', nullable: true })
  imageMetadata: {
    width?: number;
    height?: number;
    camera?: string;
    gpsLocation?: { lat: number; lng: number };
    exposureTime?: string;
    fNumber?: string;
    iso?: number;
  };

  @Column({ type: 'simple-json', nullable: true })
  videoMetadata: {
    duration?: number;
    width?: number;
    height?: number;
    codec?: string;
    fps?: number;
  };

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @ManyToOne(() => Accident, (accident) => accident.evidence, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'accidentId' })
  accident: Accident;

  // Virtual fields
  get isMedia(): boolean {
    return [
      EvidenceType.PHOTO,
      EvidenceType.VIDEO,
      EvidenceType.AUDIO,
    ].includes(this.type);
  }

  get isDigital(): boolean {
    return !!this.filePath;
  }

  get hasBeenAnalyzed(): boolean {
    return !!this.analyzedDate;
  }

  get fileSizeMB(): number | null {
    return this.fileSize ? this.fileSize / (1024 * 1024) : null;
  }

  /**
   * Validate evidence data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.description || this.description.trim().length === 0) {
      throw new Error('Evidence description is required');
    }

    if (!this.collectedBy || this.collectedBy.trim().length === 0) {
      throw new Error('Evidence collector is required');
    }

    if (!this.timestamp) {
      throw new Error('Evidence timestamp is required');
    }

    // Validate priority if provided
    if (this.priority !== null && this.priority !== undefined) {
      if (this.priority < 1 || this.priority > 5) {
        throw new Error('Priority must be between 1 and 5');
      }
    }

    // Validate copy number
    if (this.copyNumber < 1) {
      throw new Error('Copy number must be at least 1');
    }

    // Validate file size if provided
    if (this.fileSize !== null && this.fileSize !== undefined && this.fileSize < 0) {
      throw new Error('File size cannot be negative');
    }

    // Ensure arrays are initialized
    if (!this.chainOfCustody) {
      this.chainOfCustody = [];
    }

    if (!this.tags) {
      this.tags = [];
    }

    if (!this.relatedVehicles) {
      this.relatedVehicles = [];
    }

    if (!this.relatedWitnesses) {
      this.relatedWitnesses = [];
    }

    if (!this.metadata) {
      this.metadata = {};
    }
  }

  /**
   * Generate unique evidence number before insert
   */
  @BeforeInsert()
  async generateEvidenceNumber(): Promise<void> {
    if (!this.evidenceNumber) {
      const year = new Date().getFullYear();
      const random = Math.floor(Math.random() * 1000)
        .toString()
        .padStart(3, '0');
      const timestamp = Date.now().toString().slice(-6);
      this.evidenceNumber = `EV-${year}-${timestamp}-${random}`;
    }
  }

  /**
   * Add entry to chain of custody
   */
  addCustodyEntry(from: string, to: string, reason: string, signature?: string): void {
    if (!this.chainOfCustody) {
      this.chainOfCustody = [];
    }

    this.chainOfCustody.push({
      date: new Date(),
      from,
      to,
      reason,
      signature,
    });

    this.currentCustodian = to;
  }

  /**
   * Transfer custody to another person
   */
  transferCustody(to: string, reason: string, signature?: string): void {
    const from = this.currentCustodian || this.collectedBy;
    this.addCustodyEntry(from, to, reason, signature);
    this.custodyStatus = CustodyStatus.TRANSFERRED;
  }

  /**
   * Mark evidence as analyzed
   */
  markAnalyzed(analyzedBy: string, findings: string, notes?: string): void {
    this.analyzedDate = new Date();
    this.analyzedBy = analyzedBy;
    this.findings = findings;
    if (notes) {
      this.analysisNotes = notes;
    }
    this.custodyStatus = CustodyStatus.ANALYZED;
  }

  /**
   * Add tag to evidence
   */
  addTag(tag: string): void {
    if (!this.tags) {
      this.tags = [];
    }
    if (!this.tags.includes(tag)) {
      this.tags.push(tag);
    }
  }

  /**
   * Link evidence to vehicle
   */
  linkToVehicle(vehicleNumber: number): void {
    if (!this.relatedVehicles) {
      this.relatedVehicles = [];
    }
    if (!this.relatedVehicles.includes(vehicleNumber)) {
      this.relatedVehicles.push(vehicleNumber);
    }
  }

  /**
   * Link evidence to witness
   */
  linkToWitness(witnessId: string): void {
    if (!this.relatedWitnesses) {
      this.relatedWitnesses = [];
    }
    if (!this.relatedWitnesses.includes(witnessId)) {
      this.relatedWitnesses.push(witnessId);
    }
  }
}
