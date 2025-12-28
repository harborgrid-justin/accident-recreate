//! Index definitions for AccuScene Enterprise database
//!
//! Defines all database indexes for optimal query performance.

use sea_query::Iden;

/// Index names
#[derive(Iden)]
pub enum IndexName {
    // Users indexes
    #[iden = "idx_users_email"]
    UsersEmail,
    #[iden = "idx_users_username"]
    UsersUsername,
    #[iden = "idx_users_organization"]
    UsersOrganization,
    #[iden = "idx_users_created_at"]
    UsersCreatedAt,

    // Cases indexes
    #[iden = "idx_cases_case_number"]
    CasesCaseNumber,
    #[iden = "idx_cases_status"]
    CasesStatus,
    #[iden = "idx_cases_assigned_to"]
    CasesAssignedTo,
    #[iden = "idx_cases_created_by"]
    CasesCreatedBy,
    #[iden = "idx_cases_organization"]
    CasesOrganization,
    #[iden = "idx_cases_created_at"]
    CasesCreatedAt,

    // Accidents indexes
    #[iden = "idx_accidents_case_id"]
    AccidentsCaseId,
    #[iden = "idx_accidents_accident_date"]
    AccidentsAccidentDate,
    #[iden = "idx_accidents_location"]
    AccidentsLocation,
    #[iden = "idx_accidents_severity"]
    AccidentsSeverity,
    #[iden = "idx_accidents_created_at"]
    AccidentsCreatedAt,

    // Vehicles indexes
    #[iden = "idx_vehicles_accident_id"]
    VehiclesAccidentId,
    #[iden = "idx_vehicles_vin"]
    VehiclesVin,
    #[iden = "idx_vehicles_license_plate"]
    VehiclesLicensePlate,
    #[iden = "idx_vehicles_created_at"]
    VehiclesCreatedAt,

    // Evidence indexes
    #[iden = "idx_evidence_case_id"]
    EvidenceCaseId,
    #[iden = "idx_evidence_accident_id"]
    EvidenceAccidentId,
    #[iden = "idx_evidence_evidence_type"]
    EvidenceEvidenceType,
    #[iden = "idx_evidence_collected_by"]
    EvidenceCollectedBy,
    #[iden = "idx_evidence_collected_at"]
    EvidenceCollectedAt,
    #[iden = "idx_evidence_created_at"]
    EvidenceCreatedAt,

    // Audit log indexes
    #[iden = "idx_audit_log_entity"]
    AuditLogEntity,
    #[iden = "idx_audit_log_user_id"]
    AuditLogUserId,
    #[iden = "idx_audit_log_action"]
    AuditLogAction,
    #[iden = "idx_audit_log_timestamp"]
    AuditLogTimestamp,

    // Migrations indexes
    #[iden = "idx_migrations_applied_at"]
    MigrationsAppliedAt,
}
