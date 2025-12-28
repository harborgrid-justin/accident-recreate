import {
  Entity,
  PrimaryGeneratedColumn,
  Column,
  CreateDateColumn,
  UpdateDateColumn,
  OneToMany,
  Index,
  BeforeInsert,
  BeforeUpdate,
} from 'typeorm';
import { Case } from './Case';
import * as crypto from 'crypto';

/**
 * User roles in the AccuScene system
 */
export enum UserRole {
  ADMIN = 'admin',
  INVESTIGATOR = 'investigator',
  ANALYST = 'analyst',
  VIEWER = 'viewer',
}

/**
 * User entity - Represents system users with authentication and authorization
 * Includes role-based access control for the enterprise platform
 */
@Entity('users')
export class User {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ type: 'varchar', length: 255, unique: true })
  @Index()
  email: string;

  @Column({ type: 'varchar', length: 255 })
  password: string; // Hashed password

  @Column({ type: 'varchar', length: 100, nullable: true })
  firstName: string;

  @Column({ type: 'varchar', length: 100, nullable: true })
  lastName: string;

  @Column({
    type: 'varchar',
    length: 20,
    default: UserRole.VIEWER,
  })
  role: UserRole;

  @Column({ type: 'boolean', default: true })
  isActive: boolean;

  @Column({ type: 'varchar', length: 255, nullable: true })
  department: string;

  @Column({ type: 'varchar', length: 20, nullable: true })
  phoneNumber: string;

  @Column({ type: 'varchar', length: 255, nullable: true })
  avatar: string; // Path to avatar image

  @Column({ type: 'datetime', nullable: true })
  lastLoginAt: Date;

  @Column({ type: 'varchar', length: 255, nullable: true })
  resetPasswordToken: string;

  @Column({ type: 'datetime', nullable: true })
  resetPasswordExpires: Date;

  @Column({ type: 'int', default: 0 })
  loginAttempts: number;

  @Column({ type: 'datetime', nullable: true })
  lockedUntil: Date;

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;

  // Relations
  @OneToMany(() => Case, (caseEntity) => caseEntity.user, {
    cascade: true,
  })
  cases: Case[];

  // Virtual fields (not stored in database)
  get fullName(): string {
    if (this.firstName && this.lastName) {
      return `${this.firstName} ${this.lastName}`;
    }
    return this.email;
  }

  get isLocked(): boolean {
    return !!(this.lockedUntil && this.lockedUntil > new Date());
  }

  /**
   * Validate email format before insert/update
   */
  @BeforeInsert()
  @BeforeUpdate()
  validateEmail(): void {
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(this.email)) {
      throw new Error('Invalid email format');
    }
    this.email = this.email.toLowerCase().trim();
  }

  /**
   * Validate password strength before insert/update
   * Password should be hashed before this validation
   */
  @BeforeInsert()
  @BeforeUpdate()
  validatePassword(): void {
    if (!this.password || this.password.length < 8) {
      throw new Error('Password must be at least 8 characters long');
    }
  }

  /**
   * Generate password reset token
   */
  generateResetToken(): string {
    const token = crypto.randomBytes(32).toString('hex');
    this.resetPasswordToken = crypto
      .createHash('sha256')
      .update(token)
      .digest('hex');
    this.resetPasswordExpires = new Date(Date.now() + 3600000); // 1 hour
    return token;
  }

  /**
   * Record successful login
   */
  recordLogin(): void {
    this.lastLoginAt = new Date();
    this.loginAttempts = 0;
    this.lockedUntil = null;
  }

  /**
   * Record failed login attempt
   */
  recordFailedLogin(): void {
    this.loginAttempts += 1;
    if (this.loginAttempts >= 5) {
      this.lockedUntil = new Date(Date.now() + 1800000); // Lock for 30 minutes
    }
  }

  /**
   * Sanitize user data for API responses (remove sensitive fields)
   */
  toJSON(): Partial<User> {
    const { password, resetPasswordToken, resetPasswordExpires, ...sanitized } = this;
    return sanitized;
  }
}
