import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  ManyToOne,
  OneToOne,
  OneToMany,
  JoinColumn,
  Index,
  BeforeInsert,
  BeforeUpdate,
} from 'typeorm';
import { User } from './User';
import { Accident } from './Accident';
import { InsuranceClaim } from './InsuranceClaim';

/**
 * Case status enum
 */
export enum CaseStatus {
  DRAFT = 'draft',
  ACTIVE = 'active',
  UNDER_REVIEW = 'under_review',
  PENDING_APPROVAL = 'pending_approval',
  COMPLETED = 'completed',
  ARCHIVED = 'archived',
  CLOSED = 'closed',
}

/**
 * Case priority levels
 */
export enum CasePriority {
  LOW = 'low',
  MEDIUM = 'medium',
  HIGH = 'high',
  CRITICAL = 'critical',
}

/**
 * Case entity - Represents an accident investigation case
 * Central entity linking users, accidents, and insurance claims
 */
@Entity('cases')
export class Case {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 50, unique: true })
  @Index()
  caseNumber: string; // e.g., "ACC-2024-00123"

  @Column({ type: 'varchar', length: 255 })
  title: string;

  @Column({ type: 'text', nullable: true })
  description: string;

  @Column({
    type: 'varchar',
    length: 20,
    default: CaseStatus.DRAFT,
  })
  @Index()
  status: CaseStatus;

  @Column({
    type: 'varchar',
    length: 20,
    default: CasePriority.MEDIUM,
  })
  priority: CasePriority;

  @Column({ type: 'varchar', length: 36 })
  @Index()
  userId: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  clientName: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  clientPhone: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  clientEmail: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  assignedTo: string; // Name or ID of assigned investigator

  @Column({ type: 'datetime', nullable: true })
  dueDate: Date;

  @Column({ type: 'datetime', nullable: true })
  closedAt: Date;

  @Column({ type: 'text', nullable: true })
  notes: string;

  @Column({ type: 'simple-json', nullable: true })
  tags: string[]; // Tags for categorization and search

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @ManyToOne(() => User, (user) => user.cases, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'userId' })
  user: User;

  @OneToOne(() => Accident, (accident) => accident.case, {
    cascade: true,
    eager: true,
  })
  accident: Accident;

  @OneToMany(() => InsuranceClaim, (claim) => claim.case, {
    cascade: true,
  })
  insuranceClaims: InsuranceClaim[];

  // Virtual fields
  get isOverdue(): boolean {
    if (!this.dueDate || this.status === CaseStatus.CLOSED) {
      return false;
    }
    return new Date() > this.dueDate;
  }

  get daysOpen(): number {
    const endDate = this.closedAt || new Date();
    const startDate = this.createdAt;
    const diffTime = Math.abs(endDate.getTime() - startDate.getTime());
    return Math.ceil(diffTime / (1000 * 60 * 60 * 24));
  }

  /**
   * Generate unique case number before insert
   */
  @BeforeInsert()
  async generateCaseNumber(): Promise<void> {
    if (!this.caseNumber) {
      const year = new Date().getFullYear();
      const random = Math.floor(Math.random() * 100000)
        .toString()
        .padStart(5, '0');
      this.caseNumber = `ACC-${year}-${random}`;
    }
  }

  /**
   * Validate case data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.title || this.title.trim().length === 0) {
      throw new Error('Case title is required');
    }

    if (this.title.length > 255) {
      throw new Error('Case title must be less than 255 characters');
    }

    // Validate email if provided
    if (this.clientEmail) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.clientEmail)) {
        throw new Error('Invalid client email format');
      }
    }

    // Ensure tags is always an array
    if (!this.tags) {
      this.tags = [];
    }

    // Ensure metadata is always an object
    if (!this.metadata) {
      this.metadata = {};
    }
  }

  /**
   * Update case status with timestamp tracking
   */
  updateStatus(newStatus: CaseStatus): void {
    this.status = newStatus;
    if (newStatus === CaseStatus.CLOSED || newStatus === CaseStatus.ARCHIVED) {
      this.closedAt = new Date();
    }
  }

  /**
   * Add a tag to the case
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
   * Remove a tag from the case
   */
  removeTag(tag: string): void {
    if (this.tags) {
      this.tags = this.tags.filter((t) => t !== tag);
    }
  }
}
