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
 * Witness type enum
 */
export enum WitnessType {
  EYEWITNESS = 'eyewitness',
  PASSENGER = 'passenger',
  FIRST_RESPONDER = 'first_responder',
  EXPERT = 'expert',
  OTHER = 'other',
}

/**
 * Witness reliability rating
 */
export enum WitnessReliability {
  VERY_HIGH = 'very_high',
  HIGH = 'high',
  MODERATE = 'moderate',
  LOW = 'low',
  QUESTIONABLE = 'questionable',
}

/**
 * Witness entity - Represents witnesses to the accident
 * Stores contact information, statements, and credibility assessment
 */
@Entity('witnesses')
export class Witness {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 36 })
  @Index()
  accidentId: string;

  @Column({ type: 'varchar', length: 255 })
  name: string;

  @Column({
    type: 'varchar',
    length: 20,
    default: WitnessType.EYEWITNESS,
  })
  type: WitnessType;

  @Column({ type: 'varchar', length: 20, nullable: true })
  @Index()
  contact: string; // Phone number

  @Column({ type: 'varchar', length: 255, nullable: true })
  email: string;

  @Column({ type: 'varchar', length: 500, nullable: true })
  address: string;

  @Column({ type: 'int', nullable: true })
  age: number;

  @Column({ type: 'varchar', length: 100, nullable: true })
  occupation: string;

  @Column({ type: 'text' })
  statement: string; // Witness statement/testimony

  @Column({ type: 'datetime', nullable: true })
  statementDate: Date; // When statement was given

  @Column({ type: 'varchar', length: 255, nullable: true })
  statementTakenBy: string; // Officer/investigator who recorded statement

  @Column({ type: 'text', nullable: true })
  observedDetails: string; // Specific details observed

  @Column({ type: 'varchar', length: 255, nullable: true })
  witnessLocation: string; // Where witness was positioned during accident

  @Column({ type: 'decimal', precision: 5, scale: 2, nullable: true })
  distanceFromAccident: number; // Distance in feet/meters

  @Column({ type: 'boolean', default: true })
  hadClearView: boolean;

  @Column({ type: 'text', nullable: true })
  viewObstructions: string; // Any obstructions to view

  @Column({
    type: 'varchar',
    length: 20,
    default: WitnessReliability.MODERATE,
  })
  reliability: WitnessReliability;

  @Column({ type: 'text', nullable: true })
  reliabilityNotes: string; // Why witness is rated as reliable/unreliable

  @Column({ type: 'boolean', default: false })
  willingToTestify: boolean;

  @Column({ type: 'boolean', default: false })
  contactedByInsurance: boolean;

  @Column({ type: 'boolean', default: false })
  contactedByAttorney: boolean;

  @Column({ type: 'simple-json', nullable: true })
  observedVehicles: Array<{
    vehicleNumber: number;
    observations: string;
  }>;

  @Column({ type: 'int', nullable: true })
  estimatedSpeed: number; // Witness estimate of vehicle speed

  @Column({ type: 'simple-json', nullable: true })
  audioRecordings: string[]; // Paths to audio recordings of statement

  @Column({ type: 'simple-json', nullable: true })
  videoRecordings: string[]; // Paths to video recordings

  @Column({ type: 'simple-json', nullable: true })
  photos: string[]; // Photos taken by witness

  @Column({ type: 'text', nullable: true })
  additionalNotes: string;

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @ManyToOne(() => Accident, (accident) => accident.witnesses, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'accidentId' })
  accident: Accident;

  // Virtual fields
  get hasRecordings(): boolean {
    return (
      (this.audioRecordings && this.audioRecordings.length > 0) ||
      (this.videoRecordings && this.videoRecordings.length > 0)
    );
  }

  get hasPhotos(): boolean {
    return this.photos && this.photos.length > 0;
  }

  get isReliable(): boolean {
    return (
      this.reliability === WitnessReliability.HIGH ||
      this.reliability === WitnessReliability.VERY_HIGH
    );
  }

  get contactInfo(): string {
    const info = [];
    if (this.contact) info.push(this.contact);
    if (this.email) info.push(this.email);
    return info.join(' | ') || 'No contact information';
  }

  /**
   * Validate witness data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.name || this.name.trim().length === 0) {
      throw new Error('Witness name is required');
    }

    if (!this.statement || this.statement.trim().length === 0) {
      throw new Error('Witness statement is required');
    }

    // Validate email if provided
    if (this.email) {
      const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
      if (!emailRegex.test(this.email)) {
        throw new Error('Invalid witness email format');
      }
      this.email = this.email.toLowerCase().trim();
    }

    // Validate age if provided
    if (this.age !== null && this.age !== undefined) {
      if (this.age < 0 || this.age > 150) {
        throw new Error('Invalid age value');
      }
    }

    // Validate distance if provided
    if (this.distanceFromAccident !== null && this.distanceFromAccident !== undefined) {
      if (this.distanceFromAccident < 0) {
        throw new Error('Distance from accident cannot be negative');
      }
    }

    // Validate speed estimate if provided
    if (this.estimatedSpeed !== null && this.estimatedSpeed !== undefined) {
      if (this.estimatedSpeed < 0 || this.estimatedSpeed > 300) {
        throw new Error('Invalid estimated speed value');
      }
    }

    // Ensure arrays are initialized
    if (!this.observedVehicles) {
      this.observedVehicles = [];
    }

    if (!this.audioRecordings) {
      this.audioRecordings = [];
    }

    if (!this.videoRecordings) {
      this.videoRecordings = [];
    }

    if (!this.photos) {
      this.photos = [];
    }

    if (!this.metadata) {
      this.metadata = {};
    }
  }

  /**
   * Add observed vehicle details
   */
  addObservedVehicle(vehicleNumber: number, observations: string): void {
    if (!this.observedVehicles) {
      this.observedVehicles = [];
    }

    const existing = this.observedVehicles.find(
      (v) => v.vehicleNumber === vehicleNumber
    );

    if (existing) {
      existing.observations = observations;
    } else {
      this.observedVehicles.push({ vehicleNumber, observations });
    }
  }

  /**
   * Add audio recording path
   */
  addAudioRecording(path: string): void {
    if (!this.audioRecordings) {
      this.audioRecordings = [];
    }
    if (!this.audioRecordings.includes(path)) {
      this.audioRecordings.push(path);
    }
  }

  /**
   * Add video recording path
   */
  addVideoRecording(path: string): void {
    if (!this.videoRecordings) {
      this.videoRecordings = [];
    }
    if (!this.videoRecordings.includes(path)) {
      this.videoRecordings.push(path);
    }
  }

  /**
   * Add photo path
   */
  addPhoto(path: string): void {
    if (!this.photos) {
      this.photos = [];
    }
    if (!this.photos.includes(path)) {
      this.photos.push(path);
    }
  }

  /**
   * Update reliability rating with notes
   */
  updateReliability(rating: WitnessReliability, notes?: string): void {
    this.reliability = rating;
    if (notes) {
      this.reliabilityNotes = notes;
    }
  }
}
