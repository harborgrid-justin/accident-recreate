//! Table definitions for AccuScene Enterprise database
//!
//! Provides sea-query Iden enums for type-safe query building.

use sea_query::Iden;

/// Users table
#[derive(Iden)]
#[iden = "users"]
pub enum Users {
    Table,
    Id,
    Email,
    Username,
    FullName,
    PasswordHash,
    Role,
    Organization,
    Phone,
    IsActive,
    LastLoginAt,
    CreatedAt,
    UpdatedAt,
}

/// Cases table
#[derive(Iden)]
#[iden = "cases"]
pub enum Cases {
    Table,
    Id,
    CaseNumber,
    Title,
    Description,
    Status,
    Priority,
    AssignedTo,
    CreatedBy,
    Organization,
    Tags,
    Metadata,
    ClosedAt,
    CreatedAt,
    UpdatedAt,
}

/// Accidents table
#[derive(Iden)]
#[iden = "accidents"]
pub enum Accidents {
    Table,
    Id,
    CaseId,
    AccidentDate,
    Location,
    LocationLat,
    LocationLng,
    WeatherConditions,
    RoadConditions,
    LightConditions,
    TrafficControl,
    Description,
    Severity,
    Fatalities,
    Injuries,
    PropertyDamageEstimate,
    PoliceReportNumber,
    PoliceDepartment,
    ReconstructionData,
    CreatedAt,
    UpdatedAt,
}

/// Vehicles table
#[derive(Iden)]
#[iden = "vehicles"]
pub enum Vehicles {
    Table,
    Id,
    AccidentId,
    VehicleNumber,
    Make,
    Model,
    Year,
    Color,
    Vin,
    LicensePlate,
    VehicleType,
    DamageDescription,
    Occupants,
    SpeedEstimate,
    DirectionOfTravel,
    FinalPositionLat,
    FinalPositionLng,
    AirbagDeployment,
    DriverInfo,
    InsuranceInfo,
    Metadata,
    CreatedAt,
    UpdatedAt,
}

/// Evidence table
#[derive(Iden)]
#[iden = "evidence"]
pub enum Evidence {
    Table,
    Id,
    CaseId,
    AccidentId,
    EvidenceType,
    Title,
    Description,
    FilePath,
    FileName,
    FileSize,
    FileMimeType,
    FileHash,
    CollectedBy,
    CollectedAt,
    Location,
    ChainOfCustody,
    Tags,
    Metadata,
    IsVerified,
    CreatedAt,
    UpdatedAt,
}

/// Audit log table
#[derive(Iden)]
#[iden = "audit_log"]
pub enum AuditLog {
    Table,
    Id,
    EntityType,
    EntityId,
    Action,
    UserId,
    UserEmail,
    Changes,
    IpAddress,
    UserAgent,
    Timestamp,
}

/// Cases full-text search table
#[derive(Iden)]
#[iden = "cases_fts"]
pub enum CasesFts {
    Table,
    CaseNumber,
    Title,
    Description,
    Tags,
}

/// Evidence full-text search table
#[derive(Iden)]
#[iden = "evidence_fts"]
pub enum EvidenceFts {
    Table,
    Title,
    Description,
    FileName,
    Tags,
}

/// Migrations table
#[derive(Iden)]
#[iden = "_migrations"]
pub enum Migrations {
    Table,
    Version,
    Name,
    AppliedAt,
    ExecutionTimeMs,
    Checksum,
}
