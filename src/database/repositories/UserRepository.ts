import { Repository, DataSource } from 'typeorm';
import { User, UserRole } from '../entities/User';
import * as crypto from 'crypto';

/**
 * Custom repository for User entity with business logic methods
 */
export class UserRepository extends Repository<User> {
  constructor(private dataSource: DataSource) {
    super(User, dataSource.createEntityManager());
  }

  /**
   * Find user by email (case-insensitive)
   */
  async findByEmail(email: string): Promise<User | null> {
    return this.findOne({
      where: { email: email.toLowerCase() },
    });
  }

  /**
   * Find active users by role
   */
  async findByRole(role: UserRole): Promise<User[]> {
    return this.find({
      where: { role, isActive: true },
      order: { lastName: 'ASC', firstName: 'ASC' },
    });
  }

  /**
   * Find all active users
   */
  async findActive(): Promise<User[]> {
    return this.find({
      where: { isActive: true },
      order: { lastName: 'ASC', firstName: 'ASC' },
    });
  }

  /**
   * Hash password using SHA-256
   */
  hashPassword(password: string): string {
    return crypto.createHash('sha256').update(password).digest('hex');
  }

  /**
   * Verify password
   */
  verifyPassword(password: string, hashedPassword: string): boolean {
    const hash = this.hashPassword(password);
    return hash === hashedPassword;
  }

  /**
   * Create new user with hashed password
   */
  async createUser(userData: Partial<User>, plainPassword: string): Promise<User> {
    const user = this.create({
      ...userData,
      password: this.hashPassword(plainPassword),
      email: userData.email?.toLowerCase(),
    });

    return this.save(user);
  }

  /**
   * Update user password
   */
  async updatePassword(userId: string, newPassword: string): Promise<User> {
    const user = await this.findOne({ where: { id: userId } });
    if (!user) {
      throw new Error('User not found');
    }

    user.password = this.hashPassword(newPassword);
    user.resetPasswordToken = null;
    user.resetPasswordExpires = null;

    return this.save(user);
  }

  /**
   * Lock user account
   */
  async lockAccount(userId: string, durationMinutes: number = 30): Promise<User> {
    const user = await this.findOne({ where: { id: userId } });
    if (!user) {
      throw new Error('User not found');
    }

    user.lockedUntil = new Date(Date.now() + durationMinutes * 60000);
    return this.save(user);
  }

  /**
   * Unlock user account
   */
  async unlockAccount(userId: string): Promise<User> {
    const user = await this.findOne({ where: { id: userId } });
    if (!user) {
      throw new Error('User not found');
    }

    user.lockedUntil = null;
    user.loginAttempts = 0;
    return this.save(user);
  }

  /**
   * Get users with their case counts
   */
  async getUsersWithCaseCounts(): Promise<Array<User & { caseCount: number }>> {
    const users = await this.createQueryBuilder('user')
      .leftJoinAndSelect('user.cases', 'case')
      .loadRelationCountAndMap('user.caseCount', 'user.cases')
      .where('user.isActive = :isActive', { isActive: true })
      .orderBy('user.lastName', 'ASC')
      .getMany();

    return users as Array<User & { caseCount: number }>;
  }

  /**
   * Search users by name or email
   */
  async searchUsers(searchTerm: string): Promise<User[]> {
    return this.createQueryBuilder('user')
      .where('user.email LIKE :search', { search: `%${searchTerm}%` })
      .orWhere('user.firstName LIKE :search', { search: `%${searchTerm}%` })
      .orWhere('user.lastName LIKE :search', { search: `%${searchTerm}%` })
      .orderBy('user.lastName', 'ASC')
      .getMany();
  }

  /**
   * Deactivate user (soft delete)
   */
  async deactivateUser(userId: string): Promise<User> {
    const user = await this.findOne({ where: { id: userId } });
    if (!user) {
      throw new Error('User not found');
    }

    user.isActive = false;
    return this.save(user);
  }

  /**
   * Activate user
   */
  async activateUser(userId: string): Promise<User> {
    const user = await this.findOne({ where: { id: userId } });
    if (!user) {
      throw new Error('User not found');
    }

    user.isActive = true;
    user.lockedUntil = null;
    user.loginAttempts = 0;
    return this.save(user);
  }
}
