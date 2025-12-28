import { Repository, DataSource, Between, In } from 'typeorm';
import { Case, CaseStatus, CasePriority } from '../entities/Case';

/**
 * Custom repository for Case entity with business logic methods
 */
export class CaseRepository extends Repository<Case> {
  constructor(private dataSource: DataSource) {
    super(Case, dataSource.createEntityManager());
  }

  /**
   * Find case by case number
   */
  async findByCaseNumber(caseNumber: string): Promise<Case | null> {
    return this.findOne({
      where: { caseNumber },
      relations: ['accident', 'insuranceClaims', 'user'],
    });
  }

  /**
   * Find cases by status
   */
  async findByStatus(status: CaseStatus | CaseStatus[]): Promise<Case[]> {
    const statuses = Array.isArray(status) ? status : [status];
    return this.find({
      where: { status: In(statuses) },
      relations: ['accident', 'user'],
      order: { updatedAt: 'DESC' },
    });
  }

  /**
   * Find cases by user ID
   */
  async findByUserId(userId: string): Promise<Case[]> {
    return this.find({
      where: { userId },
      relations: ['accident', 'insuranceClaims'],
      order: { createdAt: 'DESC' },
    });
  }

  /**
   * Find active cases (not closed or archived)
   */
  async findActiveCases(): Promise<Case[]> {
    return this.createQueryBuilder('case')
      .where('case.status NOT IN (:...statuses)', {
        statuses: [CaseStatus.CLOSED, CaseStatus.ARCHIVED],
      })
      .leftJoinAndSelect('case.accident', 'accident')
      .leftJoinAndSelect('case.user', 'user')
      .orderBy('case.priority', 'DESC')
      .addOrderBy('case.updatedAt', 'DESC')
      .getMany();
  }

  /**
   * Find overdue cases
   */
  async findOverdueCases(): Promise<Case[]> {
    return this.createQueryBuilder('case')
      .where('case.dueDate < :now', { now: new Date() })
      .andWhere('case.status != :closed', { closed: CaseStatus.CLOSED })
      .leftJoinAndSelect('case.user', 'user')
      .orderBy('case.dueDate', 'ASC')
      .getMany();
  }

  /**
   * Find cases by priority
   */
  async findByPriority(priority: CasePriority): Promise<Case[]> {
    return this.find({
      where: { priority },
      relations: ['accident', 'user'],
      order: { dueDate: 'ASC' },
    });
  }

  /**
   * Search cases by title, description, or case number
   */
  async searchCases(searchTerm: string): Promise<Case[]> {
    return this.createQueryBuilder('case')
      .where('case.caseNumber LIKE :search', { search: `%${searchTerm}%` })
      .orWhere('case.title LIKE :search', { search: `%${searchTerm}%` })
      .orWhere('case.description LIKE :search', { search: `%${searchTerm}%` })
      .orWhere('case.clientName LIKE :search', { search: `%${searchTerm}%` })
      .leftJoinAndSelect('case.accident', 'accident')
      .leftJoinAndSelect('case.user', 'user')
      .orderBy('case.updatedAt', 'DESC')
      .getMany();
  }

  /**
   * Find cases by tag
   */
  async findByTag(tag: string): Promise<Case[]> {
    return this.createQueryBuilder('case')
      .where("case.tags LIKE :tag", { tag: `%${tag}%` })
      .leftJoinAndSelect('case.accident', 'accident')
      .orderBy('case.updatedAt', 'DESC')
      .getMany();
  }

  /**
   * Find cases created within date range
   */
  async findByDateRange(startDate: Date, endDate: Date): Promise<Case[]> {
    return this.find({
      where: {
        createdAt: Between(startDate, endDate),
      },
      relations: ['accident', 'user'],
      order: { createdAt: 'DESC' },
    });
  }

  /**
   * Get case statistics
   */
  async getCaseStatistics(): Promise<{
    total: number;
    byStatus: Record<CaseStatus, number>;
    byPriority: Record<CasePriority, number>;
    overdueCount: number;
    averageDaysOpen: number;
  }> {
    const cases = await this.find();

    const byStatus = {} as Record<CaseStatus, number>;
    const byPriority = {} as Record<CasePriority, number>;

    // Initialize counts
    Object.values(CaseStatus).forEach(status => {
      byStatus[status] = 0;
    });
    Object.values(CasePriority).forEach(priority => {
      byPriority[priority] = 0;
    });

    let totalDaysOpen = 0;
    let overdueCount = 0;

    cases.forEach(caseItem => {
      byStatus[caseItem.status]++;
      byPriority[caseItem.priority]++;
      totalDaysOpen += caseItem.daysOpen;
      if (caseItem.isOverdue) overdueCount++;
    });

    return {
      total: cases.length,
      byStatus,
      byPriority,
      overdueCount,
      averageDaysOpen: cases.length > 0 ? totalDaysOpen / cases.length : 0,
    };
  }

  /**
   * Get case with full details (all relations)
   */
  async findOneWithFullDetails(caseId: string): Promise<Case | null> {
    return this.createQueryBuilder('case')
      .where('case.id = :caseId', { caseId })
      .leftJoinAndSelect('case.user', 'user')
      .leftJoinAndSelect('case.accident', 'accident')
      .leftJoinAndSelect('accident.vehicles', 'vehicles')
      .leftJoinAndSelect('accident.witnesses', 'witnesses')
      .leftJoinAndSelect('accident.evidence', 'evidence')
      .leftJoinAndSelect('case.insuranceClaims', 'insuranceClaims')
      .getOne();
  }

  /**
   * Update case status
   */
  async updateCaseStatus(caseId: string, status: CaseStatus): Promise<Case> {
    const caseItem = await this.findOne({ where: { id: caseId } });
    if (!caseItem) {
      throw new Error('Case not found');
    }

    caseItem.updateStatus(status);
    return this.save(caseItem);
  }

  /**
   * Assign case to user
   */
  async assignCase(caseId: string, userId: string, assignedTo?: string): Promise<Case> {
    const caseItem = await this.findOne({ where: { id: caseId } });
    if (!caseItem) {
      throw new Error('Case not found');
    }

    caseItem.userId = userId;
    if (assignedTo) {
      caseItem.assignedTo = assignedTo;
    }

    return this.save(caseItem);
  }
}
