import { Repository, DataSource, Between } from 'typeorm';
import { Accident, WeatherCondition, RoadCondition } from '../entities/Accident';

/**
 * Custom repository for Accident entity with business logic methods
 */
export class AccidentRepository extends Repository<Accident> {
  constructor(private dataSource: DataSource) {
    super(Accident, dataSource.createEntityManager());
  }

  /**
   * Find accident by case ID
   */
  async findByCaseId(caseId: string): Promise<Accident | null> {
    return this.findOne({
      where: { caseId },
      relations: ['vehicles', 'witnesses', 'evidence', 'case'],
    });
  }

  /**
   * Find accidents by location (partial match)
   */
  async findByLocation(location: string): Promise<Accident[]> {
    return this.createQueryBuilder('accident')
      .where('accident.location LIKE :location', { location: `%${location}%` })
      .leftJoinAndSelect('accident.case', 'case')
      .orderBy('accident.dateTime', 'DESC')
      .getMany();
  }

  /**
   * Find accidents by date range
   */
  async findByDateRange(startDate: Date, endDate: Date): Promise<Accident[]> {
    return this.find({
      where: {
        dateTime: Between(startDate, endDate),
      },
      relations: ['case', 'vehicles'],
      order: { dateTime: 'DESC' },
    });
  }

  /**
   * Find accidents by weather condition
   */
  async findByWeather(weather: WeatherCondition): Promise<Accident[]> {
    return this.find({
      where: { weather },
      relations: ['case', 'vehicles'],
      order: { dateTime: 'DESC' },
    });
  }

  /**
   * Find accidents by road condition
   */
  async findByRoadCondition(roadCondition: RoadCondition): Promise<Accident[]> {
    return this.find({
      where: { roadConditions: roadCondition },
      relations: ['case', 'vehicles'],
      order: { dateTime: 'DESC' },
    });
  }

  /**
   * Find accidents with injuries
   */
  async findWithInjuries(): Promise<Accident[]> {
    return this.createQueryBuilder('accident')
      .where('accident.injuries > 0')
      .leftJoinAndSelect('accident.case', 'case')
      .leftJoinAndSelect('accident.vehicles', 'vehicles')
      .orderBy('accident.injuries', 'DESC')
      .addOrderBy('accident.dateTime', 'DESC')
      .getMany();
  }

  /**
   * Find accidents with fatalities
   */
  async findWithFatalities(): Promise<Accident[]> {
    return this.createQueryBuilder('accident')
      .where('accident.fatalities > 0')
      .leftJoinAndSelect('accident.case', 'case')
      .leftJoinAndSelect('accident.vehicles', 'vehicles')
      .orderBy('accident.fatalities', 'DESC')
      .addOrderBy('accident.dateTime', 'DESC')
      .getMany();
  }

  /**
   * Find accidents by severity
   */
  async findBySeverity(severity: 'minor' | 'moderate' | 'severe' | 'catastrophic'): Promise<Accident[]> {
    const accidents = await this.find({
      relations: ['case', 'vehicles'],
      order: { dateTime: 'DESC' },
    });

    return accidents.filter(accident => accident.severity === severity);
  }

  /**
   * Find accidents with multiple vehicles
   */
  async findMultiVehicle(): Promise<Accident[]> {
    return this.createQueryBuilder('accident')
      .leftJoinAndSelect('accident.vehicles', 'vehicles')
      .leftJoinAndSelect('accident.case', 'case')
      .having('COUNT(vehicles.id) > 1')
      .groupBy('accident.id')
      .orderBy('accident.dateTime', 'DESC')
      .getMany();
  }

  /**
   * Find accidents near coordinates (within radius in miles)
   */
  async findNearLocation(lat: number, lng: number, radiusMiles: number = 10): Promise<Accident[]> {
    // Approximate: 1 degree latitude ≈ 69 miles
    const latRange = radiusMiles / 69;
    const lngRange = radiusMiles / (69 * Math.cos(lat * Math.PI / 180));

    return this.createQueryBuilder('accident')
      .where('accident.latitude BETWEEN :latMin AND :latMax', {
        latMin: lat - latRange,
        latMax: lat + latRange,
      })
      .andWhere('accident.longitude BETWEEN :lngMin AND :lngMax', {
        lngMin: lng - lngRange,
        lngMax: lng + lngRange,
      })
      .leftJoinAndSelect('accident.case', 'case')
      .orderBy('accident.dateTime', 'DESC')
      .getMany();
  }

  /**
   * Get accident statistics
   */
  async getAccidentStatistics(): Promise<{
    total: number;
    byWeather: Record<string, number>;
    byRoadCondition: Record<string, number>;
    bySeverity: Record<string, number>;
    totalInjuries: number;
    totalFatalities: number;
    averageVehicles: number;
    totalDamage: number;
  }> {
    const accidents = await this.find({ relations: ['vehicles'] });

    const byWeather: Record<string, number> = {};
    const byRoadCondition: Record<string, number> = {};
    const bySeverity: Record<string, number> = {};

    let totalInjuries = 0;
    let totalFatalities = 0;
    let totalVehicles = 0;
    let totalDamage = 0;

    accidents.forEach(accident => {
      // Weather
      byWeather[accident.weather] = (byWeather[accident.weather] || 0) + 1;

      // Road condition
      byRoadCondition[accident.roadConditions] = (byRoadCondition[accident.roadConditions] || 0) + 1;

      // Severity
      bySeverity[accident.severity] = (bySeverity[accident.severity] || 0) + 1;

      // Totals
      totalInjuries += accident.injuries;
      totalFatalities += accident.fatalities;
      totalVehicles += accident.totalVehicles;
      totalDamage += accident.calculateTotalDamage();
    });

    return {
      total: accidents.length,
      byWeather,
      byRoadCondition,
      bySeverity,
      totalInjuries,
      totalFatalities,
      averageVehicles: accidents.length > 0 ? totalVehicles / accidents.length : 0,
      totalDamage,
    };
  }

  /**
   * Get accident with full details (all relations)
   */
  async findOneWithFullDetails(accidentId: string): Promise<Accident | null> {
    return this.createQueryBuilder('accident')
      .where('accident.id = :accidentId', { accidentId })
      .leftJoinAndSelect('accident.case', 'case')
      .leftJoinAndSelect('accident.vehicles', 'vehicles')
      .leftJoinAndSelect('accident.witnesses', 'witnesses')
      .leftJoinAndSelect('accident.evidence', 'evidence')
      .getOne();
  }

  /**
   * Search accidents by police report number
   */
  async findByPoliceReport(reportNumber: string): Promise<Accident | null> {
    return this.findOne({
      where: { policeReportNumber: reportNumber },
      relations: ['case', 'vehicles', 'witnesses'],
    });
  }
}
