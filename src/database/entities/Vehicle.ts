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
 * Vehicle type enum
 */
export enum VehicleType {
  SEDAN = 'sedan',
  SUV = 'suv',
  TRUCK = 'truck',
  VAN = 'van',
  MOTORCYCLE = 'motorcycle',
  BUS = 'bus',
  COMMERCIAL = 'commercial',
  BICYCLE = 'bicycle',
  PEDESTRIAN = 'pedestrian',
  OTHER = 'other',
}

/**
 * Damage severity enum
 */
export enum DamageSeverity {
  NONE = 'none',
  MINOR = 'minor',
  MODERATE = 'moderate',
  SEVERE = 'severe',
  TOTALED = 'totaled',
}

/**
 * Vehicle entity - Represents vehicles involved in the accident
 * Stores physical properties, damage, and position information
 */
@Entity('vehicles')
export class Vehicle {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 36 })
  @Index()
  accidentId: string;

  @Column({ type: 'int' })
  vehicleNumber: number; // Sequential number for identification (Vehicle 1, Vehicle 2, etc.)

  @Column({
    type: 'varchar',
    length: 30,
    default: VehicleType.SEDAN,
  })
  type: VehicleType;

  @Column({ type: 'varchar', length: 100 })
  make: string; // e.g., Toyota, Ford

  @Column({ type: 'varchar', length: 100 })
  model: string; // e.g., Camry, F-150

  @Column({ type: 'int', nullable: true })
  year: number;

  @Column({ type: 'varchar', length: 50, nullable: true })
  color: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  @Index()
  licensePlate: string;

  @Column({ type: 'varchar', length: 2, nullable: true })
  licensePlateState: string;

  @Column({ type: 'varchar', length: 50, nullable: true })
  vin: string; // Vehicle Identification Number

  @Column({ type: 'varchar', length: 255 })
  driverName: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  driverPhone: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  driverLicense: string;

  @Column({ type: 'varchar', length: 2, nullable: true })
  driverLicenseState: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  ownerName: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  insuranceCompany: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  insurancePolicyNumber: string;

  @Column({ type: 'simple-json', nullable: true })
  initialPosition: {
    x: number;
    y: number;
    angle: number; // Degrees
  };

  @Column({ type: 'simple-json', nullable: true })
  finalPosition: {
    x: number;
    y: number;
    angle: number; // Degrees
  };

  @Column({ type: 'simple-json', nullable: true })
  impactPoint: {
    x: number;
    y: number;
  };

  @Column({ type: 'int', nullable: true })
  speed: number; // Speed in MPH at time of impact (estimated)

  @Column({ type: 'int', nullable: true })
  estimatedSpeed: number; // Calculated from physics simulation

  @Column({ type: 'varchar', length: 100, nullable: true })
  direction: string; // e.g., "Northbound", "Turning left"

  @Column({
    type: 'varchar',
    length: 20,
    default: DamageSeverity.NONE,
  })
  damageSeverity: DamageSeverity;

  @Column({ type: 'text', nullable: true })
  damage: string; // Detailed damage description

  @Column({ type: 'simple-json', nullable: true })
  damageAreas: string[]; // e.g., ["front", "driver_side", "windshield"]

  @Column({ type: 'decimal', precision: 12, scale: 2, nullable: true })
  estimatedDamage: number; // Estimated repair cost in USD

  @Column({ type: 'boolean', default: false })
  airbagDeployed: boolean;

  @Column({ type: 'boolean', default: false })
  seatbeltUsed: boolean;

  @Column({ type: 'boolean', default: false })
  driverImpaired: boolean;

  @Column({ type: 'text', nullable: true })
  impairedSubstance: string; // Alcohol, drugs, etc.

  @Column({ type: 'boolean', default: false })
  driverDistracted: boolean;

  @Column({ type: 'text', nullable: true })
  distractionType: string; // Phone, passengers, etc.

  @Column({ type: 'text', nullable: true })
  driverStatement: string;

  @Column({ type: 'int', nullable: true })
  occupants: number; // Total number of occupants

  @Column({ type: 'int', default: 0 })
  injuredOccupants: number;

  @Column({ type: 'simple-json', nullable: true })
  occupantDetails: Array<{
    name: string;
    age?: number;
    position: string; // driver, front_passenger, etc.
    injured: boolean;
    injuryDescription?: string;
  }>;

  @Column({ type: 'boolean', default: false })
  towedFromScene: boolean;

  @Column({ type: 'varchar', length: 255, nullable: true })
  towCompany: string;

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @ManyToOne(() => Accident, (accident) => accident.vehicles, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'accidentId' })
  accident: Accident;

  // Virtual fields
  get vehicleDescription(): string {
    const parts = [];
    if (this.year) parts.push(this.year);
    if (this.color) parts.push(this.color);
    if (this.make) parts.push(this.make);
    if (this.model) parts.push(this.model);
    return parts.join(' ') || 'Unknown Vehicle';
  }

  get hasInjuries(): boolean {
    return this.injuredOccupants > 0;
  }

  /**
   * Validate vehicle data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.make || this.make.trim().length === 0) {
      throw new Error('Vehicle make is required');
    }

    if (!this.model || this.model.trim().length === 0) {
      throw new Error('Vehicle model is required');
    }

    if (!this.driverName || this.driverName.trim().length === 0) {
      throw new Error('Driver name is required');
    }

    // Validate year if provided
    if (this.year !== null && this.year !== undefined) {
      const currentYear = new Date().getFullYear();
      if (this.year < 1900 || this.year > currentYear + 2) {
        throw new Error(`Invalid vehicle year: ${this.year}`);
      }
    }

    // Validate speed if provided
    if (this.speed !== null && this.speed !== undefined) {
      if (this.speed < 0 || this.speed > 300) {
        throw new Error('Invalid speed value');
      }
    }

    // Validate occupants
    if (this.occupants !== null && this.occupants !== undefined && this.occupants < 0) {
      throw new Error('Occupants count cannot be negative');
    }

    if (this.injuredOccupants < 0) {
      throw new Error('Injured occupants count cannot be negative');
    }

    if (this.occupants !== null && this.injuredOccupants > this.occupants) {
      throw new Error('Injured occupants cannot exceed total occupants');
    }

    // Validate damage amount
    if (this.estimatedDamage !== null && this.estimatedDamage !== undefined && this.estimatedDamage < 0) {
      throw new Error('Estimated damage cannot be negative');
    }

    // Ensure arrays are initialized
    if (!this.damageAreas) {
      this.damageAreas = [];
    }

    if (!this.occupantDetails) {
      this.occupantDetails = [];
    }

    if (!this.metadata) {
      this.metadata = {};
    }
  }

  /**
   * Calculate displacement from initial to final position
   */
  calculateDisplacement(): number | null {
    if (!this.initialPosition || !this.finalPosition) {
      return null;
    }

    const dx = this.finalPosition.x - this.initialPosition.x;
    const dy = this.finalPosition.y - this.initialPosition.y;

    return Math.sqrt(dx * dx + dy * dy);
  }

  /**
   * Calculate angle change from initial to final position
   */
  calculateAngleChange(): number | null {
    if (!this.initialPosition || !this.finalPosition) {
      return null;
    }

    let angleDiff = this.finalPosition.angle - this.initialPosition.angle;

    // Normalize to -180 to 180
    while (angleDiff > 180) angleDiff -= 360;
    while (angleDiff < -180) angleDiff += 360;

    return angleDiff;
  }

  /**
   * Add damage area
   */
  addDamageArea(area: string): void {
    if (!this.damageAreas) {
      this.damageAreas = [];
    }
    if (!this.damageAreas.includes(area)) {
      this.damageAreas.push(area);
    }
  }
}
