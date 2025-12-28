import { MigrationInterface, QueryRunner, Table, TableForeignKey, TableIndex } from 'typeorm';

/**
 * Initial database schema migration for AccuScene Enterprise Platform
 * Creates all tables, indexes, and foreign key relationships
 */
export class InitialSchema1703000000000 implements MigrationInterface {
  name = 'InitialSchema1703000000000';

  public async up(queryRunner: QueryRunner): Promise<void> {
    // Create users table
    await queryRunner.createTable(
      new Table({
        name: 'users',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'email',
            type: 'varchar',
            length: '255',
            isUnique: true,
          },
          {
            name: 'password',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'firstName',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'lastName',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'role',
            type: 'varchar',
            length: '20',
            default: "'viewer'",
          },
          {
            name: 'isActive',
            type: 'boolean',
            default: true,
          },
          {
            name: 'department',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'phoneNumber',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'avatar',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'lastLoginAt',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'resetPasswordToken',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'resetPasswordExpires',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'loginAttempts',
            type: 'int',
            default: 0,
          },
          {
            name: 'lockedUntil',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create cases table
    await queryRunner.createTable(
      new Table({
        name: 'cases',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'caseNumber',
            type: 'varchar',
            length: '50',
            isUnique: true,
          },
          {
            name: 'title',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'description',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'status',
            type: 'varchar',
            length: '20',
            default: "'draft'",
          },
          {
            name: 'priority',
            type: 'varchar',
            length: '20',
            default: "'medium'",
          },
          {
            name: 'userId',
            type: 'varchar',
            length: '36',
          },
          {
            name: 'clientName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'clientPhone',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'clientEmail',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'assignedTo',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'dueDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'closedAt',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'notes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'tags',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create accidents table
    await queryRunner.createTable(
      new Table({
        name: 'accidents',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'caseId',
            type: 'varchar',
            length: '36',
            isUnique: true,
          },
          {
            name: 'dateTime',
            type: 'datetime',
          },
          {
            name: 'location',
            type: 'varchar',
            length: '500',
          },
          {
            name: 'latitude',
            type: 'decimal',
            precision: 10,
            scale: 7,
            isNullable: true,
          },
          {
            name: 'longitude',
            type: 'decimal',
            precision: 10,
            scale: 7,
            isNullable: true,
          },
          {
            name: 'intersection',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'weather',
            type: 'varchar',
            length: '30',
            default: "'clear'",
          },
          {
            name: 'roadConditions',
            type: 'varchar',
            length: '30',
            default: "'dry'",
          },
          {
            name: 'lightConditions',
            type: 'varchar',
            length: '30',
            default: "'daylight'",
          },
          {
            name: 'temperature',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'speedLimit',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'roadType',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'numberOfLanes',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'trafficSignalsPresent',
            type: 'boolean',
            default: false,
          },
          {
            name: 'trafficSignsPresent',
            type: 'boolean',
            default: false,
          },
          {
            name: 'description',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'policeReportNumber',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'respondingOfficer',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'injuries',
            type: 'int',
            default: 0,
          },
          {
            name: 'fatalities',
            type: 'int',
            default: 0,
          },
          {
            name: 'estimatedDamage',
            type: 'decimal',
            precision: 12,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'diagram',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'environmentalFactors',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'preliminaryConclusion',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'finalConclusion',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create vehicles table
    await queryRunner.createTable(
      new Table({
        name: 'vehicles',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'accidentId',
            type: 'varchar',
            length: '36',
          },
          {
            name: 'vehicleNumber',
            type: 'int',
          },
          {
            name: 'type',
            type: 'varchar',
            length: '30',
            default: "'sedan'",
          },
          {
            name: 'make',
            type: 'varchar',
            length: '100',
          },
          {
            name: 'model',
            type: 'varchar',
            length: '100',
          },
          {
            name: 'year',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'color',
            type: 'varchar',
            length: '50',
            isNullable: true,
          },
          {
            name: 'licensePlate',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'licensePlateState',
            type: 'varchar',
            length: '2',
            isNullable: true,
          },
          {
            name: 'vin',
            type: 'varchar',
            length: '50',
            isNullable: true,
          },
          {
            name: 'driverName',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'driverPhone',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'driverLicense',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'driverLicenseState',
            type: 'varchar',
            length: '2',
            isNullable: true,
          },
          {
            name: 'ownerName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'insuranceCompany',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'insurancePolicyNumber',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'initialPosition',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'finalPosition',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'impactPoint',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'speed',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'estimatedSpeed',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'direction',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'damageSeverity',
            type: 'varchar',
            length: '20',
            default: "'none'",
          },
          {
            name: 'damage',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'damageAreas',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'estimatedDamage',
            type: 'decimal',
            precision: 12,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'airbagDeployed',
            type: 'boolean',
            default: false,
          },
          {
            name: 'seatbeltUsed',
            type: 'boolean',
            default: false,
          },
          {
            name: 'driverImpaired',
            type: 'boolean',
            default: false,
          },
          {
            name: 'impairedSubstance',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'driverDistracted',
            type: 'boolean',
            default: false,
          },
          {
            name: 'distractionType',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'driverStatement',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'occupants',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'injuredOccupants',
            type: 'int',
            default: 0,
          },
          {
            name: 'occupantDetails',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'towedFromScene',
            type: 'boolean',
            default: false,
          },
          {
            name: 'towCompany',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create witnesses table
    await queryRunner.createTable(
      new Table({
        name: 'witnesses',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'accidentId',
            type: 'varchar',
            length: '36',
          },
          {
            name: 'name',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'type',
            type: 'varchar',
            length: '20',
            default: "'eyewitness'",
          },
          {
            name: 'contact',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'email',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'address',
            type: 'varchar',
            length: '500',
            isNullable: true,
          },
          {
            name: 'age',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'occupation',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'statement',
            type: 'text',
          },
          {
            name: 'statementDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'statementTakenBy',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'observedDetails',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'witnessLocation',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'distanceFromAccident',
            type: 'decimal',
            precision: 5,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'hadClearView',
            type: 'boolean',
            default: true,
          },
          {
            name: 'viewObstructions',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'reliability',
            type: 'varchar',
            length: '20',
            default: "'moderate'",
          },
          {
            name: 'reliabilityNotes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'willingToTestify',
            type: 'boolean',
            default: false,
          },
          {
            name: 'contactedByInsurance',
            type: 'boolean',
            default: false,
          },
          {
            name: 'contactedByAttorney',
            type: 'boolean',
            default: false,
          },
          {
            name: 'observedVehicles',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'estimatedSpeed',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'audioRecordings',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'videoRecordings',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'photos',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'additionalNotes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create evidence table
    await queryRunner.createTable(
      new Table({
        name: 'evidence',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'accidentId',
            type: 'varchar',
            length: '36',
          },
          {
            name: 'evidenceNumber',
            type: 'varchar',
            length: '50',
            isUnique: true,
          },
          {
            name: 'type',
            type: 'varchar',
            length: '30',
          },
          {
            name: 'source',
            type: 'varchar',
            length: '30',
            default: "'scene'",
          },
          {
            name: 'description',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'filePath',
            type: 'varchar',
            length: '500',
            isNullable: true,
          },
          {
            name: 'fileName',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'fileType',
            type: 'varchar',
            length: '50',
            isNullable: true,
          },
          {
            name: 'fileSize',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'fileHash',
            type: 'varchar',
            length: '64',
            isNullable: true,
          },
          {
            name: 'timestamp',
            type: 'datetime',
          },
          {
            name: 'collectedBy',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'collectionLocation',
            type: 'varchar',
            length: '500',
            isNullable: true,
          },
          {
            name: 'collectionMethod',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'custodyStatus',
            type: 'varchar',
            length: '30',
            default: "'collected'",
          },
          {
            name: 'currentCustodian',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'storageLocation',
            type: 'varchar',
            length: '500',
            isNullable: true,
          },
          {
            name: 'chainOfCustody',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'isOriginal',
            type: 'boolean',
            default: true,
          },
          {
            name: 'copyNumber',
            type: 'int',
            default: 1,
          },
          {
            name: 'originalEvidenceId',
            type: 'varchar',
            length: '36',
            isNullable: true,
          },
          {
            name: 'analysisNotes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'findings',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'analyzedDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'analyzedBy',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'isAdmissible',
            type: 'boolean',
            default: true,
          },
          {
            name: 'admissibilityNotes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'tags',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'relatedVehicles',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'relatedWitnesses',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'priority',
            type: 'int',
            isNullable: true,
          },
          {
            name: 'requiresExpertAnalysis',
            type: 'boolean',
            default: false,
          },
          {
            name: 'expertAssigned',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'imageMetadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'videoMetadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create insurance_claims table
    await queryRunner.createTable(
      new Table({
        name: 'insurance_claims',
        columns: [
          {
            name: 'id',
            type: 'varchar',
            length: '36',
            isPrimary: true,
            isGenerated: true,
            generationStrategy: 'uuid',
          },
          {
            name: 'caseId',
            type: 'varchar',
            length: '36',
          },
          {
            name: 'claimNumber',
            type: 'varchar',
            length: '100',
            isUnique: true,
          },
          {
            name: 'type',
            type: 'varchar',
            length: '30',
          },
          {
            name: 'insurer',
            type: 'varchar',
            length: '255',
          },
          {
            name: 'insurerPolicyNumber',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'policyHolderName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'claimantName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'claimantPhone',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'claimantEmail',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'status',
            type: 'varchar',
            length: '30',
            default: "'draft'",
          },
          {
            name: 'amount',
            type: 'decimal',
            precision: 12,
            scale: 2,
          },
          {
            name: 'approvedAmount',
            type: 'decimal',
            precision: 12,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'paidAmount',
            type: 'decimal',
            precision: 12,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'deductible',
            type: 'decimal',
            precision: 12,
            scale: 2,
            isNullable: true,
          },
          {
            name: 'dateOfLoss',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'filedDate',
            type: 'datetime',
          },
          {
            name: 'submittedDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'reviewStartDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'decisionDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'settlementDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'closedDate',
            type: 'datetime',
            isNullable: true,
          },
          {
            name: 'adjusterName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'adjusterPhone',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'adjusterEmail',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'referenceNumber',
            type: 'varchar',
            length: '100',
            isNullable: true,
          },
          {
            name: 'description',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'denialReason',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'notes',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'documents',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'communications',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'payments',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'requiresLitigation',
            type: 'boolean',
            default: false,
          },
          {
            name: 'subrogate',
            type: 'boolean',
            default: false,
          },
          {
            name: 'attorneyName',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'attorneyPhone',
            type: 'varchar',
            length: '20',
            isNullable: true,
          },
          {
            name: 'attorneyEmail',
            type: 'varchar',
            length: '255',
            isNullable: true,
          },
          {
            name: 'numberOfVehicles',
            type: 'int',
            default: 0,
          },
          {
            name: 'numberOfInjuries',
            type: 'int',
            default: 0,
          },
          {
            name: 'totalLoss',
            type: 'boolean',
            default: false,
          },
          {
            name: 'metadata',
            type: 'text',
            isNullable: true,
          },
          {
            name: 'createdAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
          {
            name: 'updatedAt',
            type: 'datetime',
            default: 'CURRENT_TIMESTAMP',
          },
        ],
      }),
      true
    );

    // Create indexes
    await queryRunner.createIndex(
      'users',
      new TableIndex({
        name: 'IDX_USERS_EMAIL',
        columnNames: ['email'],
      })
    );

    await queryRunner.createIndex(
      'cases',
      new TableIndex({
        name: 'IDX_CASES_NUMBER',
        columnNames: ['caseNumber'],
      })
    );

    await queryRunner.createIndex(
      'cases',
      new TableIndex({
        name: 'IDX_CASES_STATUS',
        columnNames: ['status'],
      })
    );

    await queryRunner.createIndex(
      'cases',
      new TableIndex({
        name: 'IDX_CASES_USER',
        columnNames: ['userId'],
      })
    );

    await queryRunner.createIndex(
      'accidents',
      new TableIndex({
        name: 'IDX_ACCIDENTS_CASE',
        columnNames: ['caseId'],
      })
    );

    await queryRunner.createIndex(
      'accidents',
      new TableIndex({
        name: 'IDX_ACCIDENTS_DATETIME',
        columnNames: ['dateTime'],
      })
    );

    await queryRunner.createIndex(
      'vehicles',
      new TableIndex({
        name: 'IDX_VEHICLES_ACCIDENT',
        columnNames: ['accidentId'],
      })
    );

    await queryRunner.createIndex(
      'vehicles',
      new TableIndex({
        name: 'IDX_VEHICLES_LICENSE',
        columnNames: ['licensePlate'],
      })
    );

    await queryRunner.createIndex(
      'witnesses',
      new TableIndex({
        name: 'IDX_WITNESSES_ACCIDENT',
        columnNames: ['accidentId'],
      })
    );

    await queryRunner.createIndex(
      'witnesses',
      new TableIndex({
        name: 'IDX_WITNESSES_CONTACT',
        columnNames: ['contact'],
      })
    );

    await queryRunner.createIndex(
      'evidence',
      new TableIndex({
        name: 'IDX_EVIDENCE_ACCIDENT',
        columnNames: ['accidentId'],
      })
    );

    await queryRunner.createIndex(
      'evidence',
      new TableIndex({
        name: 'IDX_EVIDENCE_NUMBER',
        columnNames: ['evidenceNumber'],
      })
    );

    await queryRunner.createIndex(
      'evidence',
      new TableIndex({
        name: 'IDX_EVIDENCE_TIMESTAMP',
        columnNames: ['timestamp'],
      })
    );

    await queryRunner.createIndex(
      'insurance_claims',
      new TableIndex({
        name: 'IDX_CLAIMS_CASE',
        columnNames: ['caseId'],
      })
    );

    await queryRunner.createIndex(
      'insurance_claims',
      new TableIndex({
        name: 'IDX_CLAIMS_NUMBER',
        columnNames: ['claimNumber'],
      })
    );

    await queryRunner.createIndex(
      'insurance_claims',
      new TableIndex({
        name: 'IDX_CLAIMS_STATUS',
        columnNames: ['status'],
      })
    );

    await queryRunner.createIndex(
      'insurance_claims',
      new TableIndex({
        name: 'IDX_CLAIMS_FILED_DATE',
        columnNames: ['filedDate'],
      })
    );

    // Create foreign keys
    await queryRunner.createForeignKey(
      'cases',
      new TableForeignKey({
        columnNames: ['userId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'users',
        onDelete: 'CASCADE',
      })
    );

    await queryRunner.createForeignKey(
      'accidents',
      new TableForeignKey({
        columnNames: ['caseId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'cases',
        onDelete: 'CASCADE',
      })
    );

    await queryRunner.createForeignKey(
      'vehicles',
      new TableForeignKey({
        columnNames: ['accidentId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'accidents',
        onDelete: 'CASCADE',
      })
    );

    await queryRunner.createForeignKey(
      'witnesses',
      new TableForeignKey({
        columnNames: ['accidentId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'accidents',
        onDelete: 'CASCADE',
      })
    );

    await queryRunner.createForeignKey(
      'evidence',
      new TableForeignKey({
        columnNames: ['accidentId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'accidents',
        onDelete: 'CASCADE',
      })
    );

    await queryRunner.createForeignKey(
      'insurance_claims',
      new TableForeignKey({
        columnNames: ['caseId'],
        referencedColumnNames: ['id'],
        referencedTableName: 'cases',
        onDelete: 'CASCADE',
      })
    );
  }

  public async down(queryRunner: QueryRunner): Promise<void> {
    // Drop foreign keys
    const casesTable = await queryRunner.getTable('cases');
    const accidentsTable = await queryRunner.getTable('accidents');
    const vehiclesTable = await queryRunner.getTable('vehicles');
    const witnessesTable = await queryRunner.getTable('witnesses');
    const evidenceTable = await queryRunner.getTable('evidence');
    const insuranceClaimsTable = await queryRunner.getTable('insurance_claims');

    if (casesTable) {
      const casesForeignKeys = casesTable.foreignKeys;
      for (const foreignKey of casesForeignKeys) {
        await queryRunner.dropForeignKey('cases', foreignKey);
      }
    }

    if (accidentsTable) {
      const accidentsForeignKeys = accidentsTable.foreignKeys;
      for (const foreignKey of accidentsForeignKeys) {
        await queryRunner.dropForeignKey('accidents', foreignKey);
      }
    }

    if (vehiclesTable) {
      const vehiclesForeignKeys = vehiclesTable.foreignKeys;
      for (const foreignKey of vehiclesForeignKeys) {
        await queryRunner.dropForeignKey('vehicles', foreignKey);
      }
    }

    if (witnessesTable) {
      const witnessesForeignKeys = witnessesTable.foreignKeys;
      for (const foreignKey of witnessesForeignKeys) {
        await queryRunner.dropForeignKey('witnesses', foreignKey);
      }
    }

    if (evidenceTable) {
      const evidenceForeignKeys = evidenceTable.foreignKeys;
      for (const foreignKey of evidenceForeignKeys) {
        await queryRunner.dropForeignKey('evidence', foreignKey);
      }
    }

    if (insuranceClaimsTable) {
      const insuranceClaimsForeignKeys = insuranceClaimsTable.foreignKeys;
      for (const foreignKey of insuranceClaimsForeignKeys) {
        await queryRunner.dropForeignKey('insurance_claims', foreignKey);
      }
    }

    // Drop tables in reverse order
    await queryRunner.dropTable('insurance_claims');
    await queryRunner.dropTable('evidence');
    await queryRunner.dropTable('witnesses');
    await queryRunner.dropTable('vehicles');
    await queryRunner.dropTable('accidents');
    await queryRunner.dropTable('cases');
    await queryRunner.dropTable('users');
  }
}
