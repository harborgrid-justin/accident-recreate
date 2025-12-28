import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  OneToOne,
  OneToMany,
  JoinColumn,
  Index,
  BeforeInsert,
  BeforeUpdate,
} from 'typeorm';
import { Case } from './Case';
import { Vehicle } from './Vehicle';
import { Witness } from './Witness';
import { Evidence } from './Evidence';

/**
 * Weather conditions enum
 */
export enum WeatherCondition {
  CLEAR = 'clear',
  CLOUDY = 'cloudy',
  RAIN = 'rain',
  HEAVY_RAIN = 'heavy_rain',
  SNOW = 'snow',
  SLEET = 'sleet',
  FOG = 'fog',
  WIND = 'wind',
  HAIL = 'hail',
}

/**
 * Road conditions enum
 */
export enum RoadCondition {
  DRY = 'dry',
  WET = 'wet',
  ICY = 'icy',
  SNOW_COVERED = 'snow_covered',
  MUDDY = 'muddy',
  UNDER_CONSTRUCTION = 'under_construction',
  DAMAGED = 'damaged',
  DEBRIS = 'debris',
}

/**
 * Light conditions enum
 */
export enum LightCondition {
  DAYLIGHT = 'daylight',
  DAWN = 'dawn',
  DUSK = 'dusk',
  DARK_STREET_LIGHTS = 'dark_street_lights',
  DARK_NO_LIGHTS = 'dark_no_lights',
}

/**
 * Accident entity - Core entity containing all accident scene details
 * Links to vehicles, witnesses, and evidence
 */
@Entity('accidents')
export class Accident {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 36, unique: true })
  @Index()
  caseId: string;

  @Column({ type: 'datetime' })
  @Index()
  dateTime: Date;

  @Column({ type: 'varchar', length: 500 })
  location: string; // Full address or description

  @Column({ type: 'decimal', precision: 10, scale: 7, nullable: true })
  latitude: number;

  @Column({ type: 'decimal', precision: 10, scale: 7, nullable: true })
  longitude: number;

  @Column({ type: 'varchar', length: 255, nullable: true })
  intersection: string; // Specific intersection name

  @Column({
    type: 'varchar',
    length: 30,
    default: WeatherCondition.CLEAR,
  })
  weather: WeatherCondition;

  @Column({
    type: 'varchar',
    length: 30,
    default: RoadCondition.DRY,
  })
  roadConditions: RoadCondition;

  @Column({
    type: 'varchar',
    length: 30,
    default: LightCondition.DAYLIGHT,
  })
  lightConditions: LightCondition;

  @Column({ type: 'int', nullable: true })
  temperature: number; // In Fahrenheit

  @Column({ type: 'int', nullable: true })
  speedLimit: number; // In MPH

  @Column({ type: 'varchar', length: 100, nullable: true })
  roadType: string; // Highway, residential, etc.

  @Column({ type: 'int', nullable: true })
  numberOfLanes: number;

  @Column({ type: 'boolean', default: false })
  trafficSignalsPresent: boolean;

  @Column({ type: 'boolean', default: false })
  trafficSignsPresent: boolean;

  @Column({ type: 'text', nullable: true })
  description: string; // Detailed narrative of the accident

  @Column({ type: 'text', nullable: true })
  policeReportNumber: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  respondingOfficer: string;

  @Column({ type: 'int', default: 0 })
  injuries: number; // Number of injuries

  @Column({ type: 'int', default: 0 })
  fatalities: number; // Number of fatalities

  @Column({ type: 'decimal', precision: 12, scale: 2, nullable: true })
  estimatedDamage: number; // Total estimated damage in USD

  @Column({ type: 'simple-json', nullable: true })
  diagram: any; // JSON representation of accident diagram/scene

  @Column({ type: 'simple-json', nullable: true })
  environmentalFactors: string[]; // Additional factors like glare, construction, etc.

  @Column({ type: 'text', nullable: true })
  preliminaryConclusion: string;

  @Column({ type: 'text', nullable: true })
  finalConclusion: string;

  @Column({ type: 'simple-json', nullable: true })
  metadata: Record<string, any>; // Additional flexible data

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @OneToOne(() => Case, (caseEntity) => caseEntity.accident, {
    onDelete: 'CASCADE',
    nullable: false,
  })
  @JoinColumn({ name: 'caseId' })
  case: Case;

  @OneToMany(() => Vehicle, (vehicle) => vehicle.accident, {
    cascade: true,
    eager: true,
  })
  vehicles: Vehicle[];

  @OneToMany(() => Witness, (witness) => witness.accident, {
    cascade: true,
  })
  witnesses: Witness[];

  @OneToMany(() => Evidence, (evidence) => evidence.accident, {
    cascade: true,
  })
  evidence: Evidence[];

  // Virtual fields
  get totalVehicles(): number {
    return this.vehicles?.length || 0;
  }

  get totalWitnesses(): number {
    return this.witnesses?.length || 0;
  }

  get totalEvidence(): number {
    return this.evidence?.length || 0;
  }

  get severity(): 'minor' | 'moderate' | 'severe' | 'catastrophic' {
    if (this.fatalities > 0) return 'catastrophic';
    if (this.injuries > 3) return 'severe';
    if (this.injuries > 0) return 'moderate';
    return 'minor';
  }

  /**
   * Validate accident data before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validate(): void {
    if (!this.location || this.location.trim().length === 0) {
      throw new Error('Accident location is required');
    }

    if (!this.dateTime) {
      throw new Error('Accident date/time is required');
    }

    // Validate coordinates if provided
    if (this.latitude !== null && this.latitude !== undefined) {
      if (this.latitude < -90 || this.latitude > 90) {
        throw new Error('Invalid latitude value');
      }
    }

    if (this.longitude !== null && this.longitude !== undefined) {
      if (this.longitude < -180 || this.longitude > 180) {
        throw new Error('Invalid longitude value');
      }
    }

    // Validate non-negative numbers
    if (this.injuries < 0) {
      throw new Error('Injuries count cannot be negative');
    }

    if (this.fatalities < 0) {
      throw new Error('Fatalities count cannot be negative');
    }

    if (this.estimatedDamage !== null && this.estimatedDamage !== undefined && this.estimatedDamage < 0) {
      throw new Error('Estimated damage cannot be negative');
    }

    // Ensure arrays are initialized
    if (!this.environmentalFactors) {
      this.environmentalFactors = [];
    }

    if (!this.metadata) {
      this.metadata = {};
    }
  }

  /**
   * Calculate total estimated damage from all vehicles
   */
  calculateTotalDamage(): number {
    if (!this.vehicles || this.vehicles.length === 0) {
      return this.estimatedDamage || 0;
    }

    const vehicleDamageTotal = this.vehicles.reduce((sum, vehicle) => {
      return sum + (vehicle.estimatedDamage || 0);
    }, 0);

    return vehicleDamageTotal;
  }

  /**
   * Add environmental factor
   */
  addEnvironmentalFactor(factor: string): void {
    if (!this.environmentalFactors) {
      this.environmentalFactors = [];
    }
    if (!this.environmentalFactors.includes(factor)) {
      this.environmentalFactors.push(factor);
    }
  }

  /**
   * Get formatted location string with coordinates if available
   */
  getFormattedLocation(): string {
    let location = this.location;
    if (this.latitude && this.longitude) {
      location += ` (${this.latitude.toFixed(6)}, ${this.longitude.toFixed(6)})`;
    }
    return location;
  }
}
