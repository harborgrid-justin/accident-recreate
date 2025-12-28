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
import { Case } from './Case';

/**
 * Insurance claim status enum
 */
export enum ClaimStatus {
  DRAFT = 'draft',
  SUBMITTED = 'submitted',
  UNDER_REVIEW = 'under_review',
  ADDITIONAL_INFO_REQUIRED = 'additional_info_required',
  APPROVED = 'approved',
  PARTIALLY_APPROVED = 'partially_approved',
  DENIED = 'denied',
  APPEALED = 'appealed',
  SETTLED = 'settled',
  CLOSED = 'closed',
}

/**
 * Claim type enum
 */
export enum ClaimType {
  PROPERTY_DAMAGE = 'property_damage',
  BODILY_INJURY = 'bodily_injury',
  COMPREHENSIVE = 'comprehensive',
  COLLISION = 'collision',
  LIABILITY = 'liability',
  UNINSURED_MOTORIST = 'uninsured_motorist',
  PERSONAL_INJURY_PROTECTION = 'personal_injury_protection',
  MEDICAL_PAYMENTS = 'medical_payments',
}

/**
 * InsuranceClaim entity - Represents insurance claims related to the case
 * Tracks claim status, amounts, and communications with insurers
 */
@Entity('insurance_claims')
export class InsuranceClaim {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 36 })
  @Index()
  caseId: string;

  @Column({ type: 'varchar', length: 100, unique: true })
  @Index()
  claimNumber: string; // Insurer's claim number

  @Column({
    type: 'varchar',
    length: 30,
  })
  type: ClaimType;

  @Column({ type: 'varchar', length: 255 })
  insurer: string; // Insurance company name

  @Column({ type: 'varchar', length: 100, nullable: true })
  insurerPolicyNumber: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  policyHolderName: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  claimantName: string; // Person filing the claim

  @Column({ type: 'varchar', length: 20, nullable: true })
  claimantPhone: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  claimantEmail: string;

  @Column({
    type: 'varchar',
    length: 30,
    default: ClaimStatus.DRAFT,
  })
  @Index()
  status: ClaimStatus;

  @Column({ type: 'decimal', precision: 12, scale: 2 })
  amount: number; // Claimed amount in USD

  @Column({ type: 'decimal', precision: 12, scale: 2, nullable: true })
  approvedAmount: number; // Amount approved by insurer

  @Column({ type: 'decimal', precision: 12, scale: 2, nullable: true })
  paidAmount: number; // Amount actually paid

  @Column({ type: 'decimal', precision: 12, scale: 2, nullable: true })
  deductible: number;

  @Column({ type: 'datetime', nullable: true })
  dateOfLoss: Date; // Date of accident (might differ from case date)

  @Column({ type: 'datetime' })
  @Index()
  filedDate: Date; // When claim was filed

  @Column({ type: 'datetime', nullable: true })
  submittedDate: Date; // When claim was submitted to insurer

  @Column({ type: 'datetime', nullable: true })
  reviewStartDate: Date;

  @Column({ type: 'datetime', nullable: true })
  decisionDate: Date; // When insurer made decision

  @Column({ type: 'datetime', nullable: true })
  settlementDate: Date;

  @Column({ type: 'datetime', nullable: true })
  closedDate: Date;

  @Column({ type: 'varchar', length: 255, nullable: true })
  adjusterName: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  adjusterPhone: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  adjusterEmail: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  referenceNumber: string; // Internal reference number

  @Column({ type: 'text', nullable: true })
  description: string; // Description of damages/injuries claimed

  @Column({ type: 'text', nullable: true })
  denialReason: string; // Reason if claim was denied

  @Column({ type: 'text', nullable: true })
  notes: string;

  @Column({ type: 'simple-json', nullable: true })
  documents: Array<{
    name: string;
    path: string;
    type: string;
    uploadDate: Date;
  }>;

  @Column({ type: 'simple-json', nullable: true })
  communications: Array<{
    date: Date;
    type: 'call' | 'email' | 'letter' | 'meeting';
    with: string;
    summary: string;
    followUpRequired: boolean;
  }>;

  @Column({ type: 'simple-json', nullable: true })
  payments: Array<{
    date: Date;
    amount: number;
    checkNumber?: string;
    method: string;
    notes?: string;
  }>;

  @Column({ type: 'boolean', default: false })
  requiresLitigation: boolean;

  @Column({ type: 'boolean', default: false })
  subrogate: boolean; // Whether insurer will pursue subrogation

  @Column({ type: 'varchar', length: 255, nullable: true })
  attorneyName: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  attorneyPhone: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  attorneyEmail: string;

  @Column({ type: 'int', default: 0 })
  numberOfVehicles: number; // Vehicles involved in this claim

  @Column({ type: 'int', default: 0 })
  numberOfInjuries: number; // Injuries in this claim

  @Column({ type: 'boolean', default: false })
  totalLoss: boolean; // Whether vehicle is total loss

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @ManyToOne(() => Case, (caseEntity) => caseEntity.insuranceClaims, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'caseId' })
  case: Case;

  // Virtual fields
  get isPending(): boolean {
    return [
      ClaimStatus.SUBMITTED,
      ClaimStatus.UNDER_REVIEW,
      ClaimStatus.ADDITIONAL_INFO_REQUIRED,
    ].includes(this.status);
  }

  get isResolved(): boolean {
    return [
      ClaimStatus.APPROVED,
      ClaimStatus.PARTIALLY_APPROVED,
      ClaimStatus.DENIED,
      ClaimStatus.SETTLED,
      ClaimStatus.CLOSED,
    ].includes(this.status);
  }

  get daysInReview(): number | null {
    if (!this.reviewStartDate) {
      return null;
    }
    const endDate = this.decisionDate || new Date();
    const diffTime = Math.abs(endDate.getTime() - this.reviewStartDate.getTime());
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  get outstandingAmount(): number {
    const approved = this.approvedAmount || this.amount;
    const paid = this.paidAmount || 0;
    return Math.max(0, approved - paid);
  }

  get recoveryPercentage(): number {
    if (!this.amount || this.amount === 0) {
      return 0;
    }
    const paid = this.paidAmount || 0;
    return (paid / this.amount) * 100;
  }

  /**
   * Validate insurance claim data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.claimNumber || this.claimNumber.trim().length === 0) {
      throw new Error('Claim number is required');
    }

    if (!this.insurer || this.insurer.trim().length === 0) {
      throw new Error('Insurer is required');
    }

    if (!this.amount || this.amount <= 0) {
      throw new Error('Claim amount must be greater than zero');
    }

    // Validate amounts
    if (this.approvedAmount !== null && this.approvedAmount !== undefined && this.approvedAmount < 0) {
      throw new Error('Approved amount cannot be negative');
    }

    if (this.paidAmount !== null && this.paidAmount !== undefined && this.paidAmount < 0) {
      throw new Error('Paid amount cannot be negative');
    }

    if (this.deductible !== null && this.deductible !== undefined && this.deductible < 0) {
      throw new Error('Deductible cannot be negative');
    }

    // Validate that paid amount doesn't exceed approved/claimed amount
    if (this.paidAmount && this.approvedAmount && this.paidAmount > this.approvedAmount) {
      throw new Error('Paid amount cannot exceed approved amount');
    }

    // Validate email if provided
    if (this.claimantEmail) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.claimantEmail)) {
        throw new Error('Invalid claimant email format');
      }
    }

    if (this.adjusterEmail) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.adjusterEmail)) {
        throw new Error('Invalid adjuster email format');
      }
    }

    if (this.attorneyEmail) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.attorneyEmail)) {
        throw new Error('Invalid attorney email format');
      }
    }

    // Validate counts
    if (this.numberOfVehicles < 0) {
      throw new Error('Number of vehicles cannot be negative');
    }

    if (this.numberOfInjuries < 0) {
      throw new Error('Number of injuries cannot be negative');
    }

    // Ensure arrays are initialized
    if (!this.documents) {
      this.documents = [];
    }

    if (!this.communications) {
      this.communications = [];
    }

    if (!this.payments) {
      this.payments = [];
    }

    if (!this.metadata) {
      this.metadata = {};
    }

    // Set filed date if not set
    if (!this.filedDate) {
      this.filedDate = new Date();
    }
  }

  /**
   * Update claim status with appropriate timestamp
   */
  updateStatus(newStatus: ClaimStatus): void {
    this.status = newStatus;

    switch (newStatus) {
      case ClaimStatus.SUBMITTED:
        if (!this.submittedDate) {
          this.submittedDate = new Date();
        }
        break;
      case ClaimStatus.UNDER_REVIEW:
        if (!this.reviewStartDate) {
          this.reviewStartDate = new Date();
        }
        break;
      case ClaimStatus.APPROVED:
      case ClaimStatus.PARTIALLY_APPROVED:
      case ClaimStatus.DENIED:
        if (!this.decisionDate) {
          this.decisionDate = new Date();
        }
        break;
      case ClaimStatus.SETTLED:
        if (!this.settlementDate) {
          this.settlementDate = new Date();
        }
        break;
      case ClaimStatus.CLOSED:
        if (!this.closedDate) {
          this.closedDate = new Date();
        }
        break;
    }
  }

  /**
   * Add document to claim
   */
  addDocument(name: string, path: string, type: string): void {
    if (!this.documents) {
      this.documents = [];
    }
    this.documents.push({
      name,
      path,
      type,
      uploadDate: new Date(),
    });
  }

  /**
   * Add communication entry
   */
  addCommunication(
    type: 'call' | 'email' | 'letter' | 'meeting',
    with_: string,
    summary: string,
    followUpRequired: boolean = false
  ): void {
    if (!this.communications) {
      this.communications = [];
    }
    this.communications.push({
      date: new Date(),
      type,
      with: with_,
      summary,
      followUpRequired,
    });
  }

  /**
   * Record payment
   */
  recordPayment(amount: number, method: string, checkNumber?: string, notes?: string): void {
    if (!this.payments) {
      this.payments = [];
    }

    this.payments.push({
      date: new Date(),
      amount,
      checkNumber,
      method,
      notes,
    });

    // Update paid amount
    this.paidAmount = (this.paidAmount || 0) + amount;
  }

  /**
   * Calculate days since filing
   */
  getDaysSinceFiling(): number {
    const endDate = this.closedDate || new Date();
    const diffTime = Math.abs(endDate.getTime() - this.filedDate.getTime());
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }
}
