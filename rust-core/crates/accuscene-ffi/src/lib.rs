//! AccuScene FFI - Node.js bindings using napi-rs
//!
//! This crate provides FFI bindings to expose AccuScene Core functionality
//! to Node.js/JavaScript through napi-rs.
//!
//! # Usage from Node.js
//!
//! ```javascript
//! const accuscene = require('accuscene-ffi');
//!
//! // Create a vehicle
//! const vehicle = accuscene.createVehicle('Car');
//! console.log(accuscene.getVehicleInfo(vehicle));
//!
//! // Vector operations
//! const v1 = { x: 3.0, y: 4.0 };
//! const magnitude = accuscene.vector2dMagnitude(v1);
//! console.log('Magnitude:', magnitude);
//! ```

#![warn(clippy::all)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

mod conversions;
mod error;

use accuscene_core::prelude::*;
use accuscene_core::utils;
use conversions::*;
use error::to_ffi_result;
use napi::bindgen_prelude::*;
use napi_derive::napi;

// Explicitly import napi::Result to avoid ambiguity
type NapiResult<T> = napi::Result<T>;

/// Get library version
#[napi]
pub fn version() -> String {
    accuscene_core::version()
}

/// Generate a new UUID
#[napi]
pub fn generate_id() -> String {
    utils::generate_id()
}

/// Generate a short ID (8 characters)
#[napi]
pub fn generate_short_id() -> String {
    utils::generate_short_id()
}

// ============================================================================
// Vector Operations
// ============================================================================

/// Calculate magnitude of a 2D vector
#[napi]
pub fn vector2d_magnitude(vector: JsVector2D) -> f64 {
    let v: Vector2D = vector.into();
    v.magnitude()
}

/// Normalize a 2D vector
#[napi]
pub fn vector2d_normalize(vector: JsVector2D) -> NapiResult<JsVector2D> {
    let v: Vector2D = vector.into();
    let normalized = to_ffi_result(v.normalize())?;
    Ok(normalized.into())
}

/// Add two 2D vectors
#[napi]
pub fn vector2d_add(v1: JsVector2D, v2: JsVector2D) -> JsVector2D {
    let a: Vector2D = v1.into();
    let b: Vector2D = v2.into();
    (a + b).into()
}

/// Subtract two 2D vectors
#[napi]
pub fn vector2d_subtract(v1: JsVector2D, v2: JsVector2D) -> JsVector2D {
    let a: Vector2D = v1.into();
    let b: Vector2D = v2.into();
    (a - b).into()
}

/// Multiply 2D vector by scalar
#[napi]
pub fn vector2d_multiply(vector: JsVector2D, scalar: f64) -> JsVector2D {
    let v: Vector2D = vector.into();
    (v * scalar).into()
}

/// Dot product of two 2D vectors
#[napi]
pub fn vector2d_dot(v1: JsVector2D, v2: JsVector2D) -> f64 {
    let a: Vector2D = v1.into();
    let b: Vector2D = v2.into();
    a.dot(&b)
}

/// Distance between two 2D vectors
#[napi]
pub fn vector2d_distance(v1: JsVector2D, v2: JsVector2D) -> f64 {
    let a: Vector2D = v1.into();
    let b: Vector2D = v2.into();
    a.distance(&b)
}

/// Rotate a 2D vector by angle in radians
#[napi]
pub fn vector2d_rotate(vector: JsVector2D, angle: f64) -> JsVector2D {
    let v: Vector2D = vector.into();
    v.rotate(angle).into()
}

/// Calculate magnitude of a 3D vector
#[napi]
pub fn vector3d_magnitude(vector: JsVector3D) -> f64 {
    let v: Vector3D = vector.into();
    v.magnitude()
}

/// Cross product of two 3D vectors
#[napi]
pub fn vector3d_cross(v1: JsVector3D, v2: JsVector3D) -> JsVector3D {
    let a: Vector3D = v1.into();
    let b: Vector3D = v2.into();
    a.cross(&b).into()
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Convert degrees to radians
#[napi]
pub fn deg_to_rad(degrees: f64) -> f64 {
    utils::deg_to_rad(degrees)
}

/// Convert radians to degrees
#[napi]
pub fn rad_to_deg(radians: f64) -> f64 {
    utils::rad_to_deg(radians)
}

/// Convert m/s to km/h
#[napi]
pub fn ms_to_kmh(ms: f64) -> f64 {
    utils::ms_to_kmh(ms)
}

/// Convert km/h to m/s
#[napi]
pub fn kmh_to_ms(kmh: f64) -> f64 {
    utils::kmh_to_ms(kmh)
}

/// Convert m/s to mph
#[napi]
pub fn ms_to_mph(ms: f64) -> f64 {
    utils::ms_to_mph(ms)
}

/// Convert mph to m/s
#[napi]
pub fn mph_to_ms(mph: f64) -> f64 {
    utils::mph_to_ms(mph)
}

/// Calculate kinetic energy
#[napi]
pub fn kinetic_energy(mass_kg: f64, velocity_ms: f64) -> f64 {
    utils::kinetic_energy(mass_kg, velocity_ms)
}

/// Calculate momentum
#[napi]
pub fn momentum(mass_kg: f64, velocity_ms: f64) -> f64 {
    utils::momentum(mass_kg, velocity_ms)
}

/// Calculate stopping distance
#[napi]
pub fn stopping_distance(initial_velocity_ms: f64, deceleration_ms2: f64) -> NapiResult<f64> {
    to_ffi_result(utils::stopping_distance(initial_velocity_ms, deceleration_ms2))
}

/// Clamp a value between min and max
#[napi]
pub fn clamp(value: f64, min: f64, max: f64) -> f64 {
    utils::clamp(value, min, max)
}

/// Linear interpolation
#[napi]
pub fn lerp(a: f64, b: f64, t: f64) -> f64 {
    utils::lerp(a, b, t)
}

// ============================================================================
// Vehicle Operations
// ============================================================================

/// Create a new vehicle (returns JSON string)
#[napi]
pub fn create_vehicle(category: JsVehicleCategory) -> String {
    let cat: VehicleCategory = category.into();
    let vehicle = Vehicle::new(cat);
    to_json_string(&vehicle).unwrap_or_else(|_| "{}".to_string())
}

/// Create a vehicle with metadata (returns JSON string)
#[napi]
pub fn create_vehicle_with_metadata(
    category: JsVehicleCategory,
    metadata: JsVehicleMetadata,
) -> String {
    let cat: VehicleCategory = category.into();
    let meta: VehicleMetadata = metadata.into();
    let vehicle = Vehicle::with_metadata(cat, meta);
    to_json_string(&vehicle).unwrap_or_else(|_| "{}".to_string())
}

/// Parse vehicle from JSON string
#[napi]
pub fn parse_vehicle(json: String) -> NapiResult<String> {
    let vehicle: Vehicle = from_json_string(&json)?;
    to_ffi_result(vehicle.validate())?;
    to_json_string(&vehicle)
}

/// Calculate vehicle kinetic energy
#[napi]
pub fn vehicle_kinetic_energy(vehicle_json: String) -> NapiResult<f64> {
    let vehicle: Vehicle = from_json_string(&vehicle_json)?;
    Ok(vehicle.kinetic_energy())
}

/// Calculate vehicle speed in km/h
#[napi]
pub fn vehicle_speed_kmh(vehicle_json: String) -> NapiResult<f64> {
    let vehicle: Vehicle = from_json_string(&vehicle_json)?;
    Ok(vehicle.speed_kmh())
}

/// Update vehicle position by time step
#[napi]
pub fn vehicle_update_position(vehicle_json: String, dt: f64) -> NapiResult<String> {
    let mut vehicle: Vehicle = from_json_string(&vehicle_json)?;
    vehicle.update_position(dt);
    to_json_string(&vehicle)
}

// ============================================================================
// Accident Scene Operations
// ============================================================================

/// Create a new accident scene (returns JSON string)
#[napi]
pub fn create_accident_scene(name: String) -> String {
    let scene = AccidentScene::new(name);
    to_json_string(&scene).unwrap_or_else(|_| "{}".to_string())
}

/// Parse accident scene from JSON
#[napi]
pub fn parse_accident_scene(json: String) -> NapiResult<String> {
    let scene: AccidentScene = from_json_string(&json)?;
    to_ffi_result(scene.validate())?;
    to_json_string(&scene)
}

/// Add vehicle to scene
#[napi]
pub fn scene_add_vehicle(scene_json: String, vehicle_json: String) -> NapiResult<String> {
    let mut scene: AccidentScene = from_json_string(&scene_json)?;
    let vehicle: Vehicle = from_json_string(&vehicle_json)?;
    to_ffi_result(scene.add_vehicle(vehicle))?;
    to_json_string(&scene)
}

/// Remove vehicle from scene
#[napi]
pub fn scene_remove_vehicle(scene_json: String, vehicle_id: String) -> NapiResult<String> {
    let mut scene: AccidentScene = from_json_string(&scene_json)?;
    to_ffi_result(scene.remove_vehicle(&vehicle_id))?;
    to_json_string(&scene)
}

/// Calculate total kinetic energy in scene
#[napi]
pub fn scene_total_kinetic_energy(scene_json: String) -> NapiResult<f64> {
    let scene: AccidentScene = from_json_string(&scene_json)?;
    Ok(scene.total_kinetic_energy())
}

/// Get scene statistics
#[napi]
pub fn scene_statistics(scene_json: String) -> NapiResult<String> {
    let scene: AccidentScene = from_json_string(&scene_json)?;
    let stats = scene.statistics();
    to_json_string(&stats)
}

/// Step scene simulation
#[napi]
pub fn scene_step_simulation(scene_json: String, dt: f64) -> NapiResult<String> {
    let mut scene: AccidentScene = from_json_string(&scene_json)?;
    to_ffi_result(scene.step_simulation(dt))?;
    to_json_string(&scene)
}

/// Calculate effective friction for scene
#[napi]
pub fn scene_effective_friction(scene_json: String) -> NapiResult<f64> {
    let scene: AccidentScene = from_json_string(&scene_json)?;
    Ok(scene.effective_friction())
}

// ============================================================================
// Case Operations
// ============================================================================

/// Create a new case (returns JSON string)
#[napi]
pub fn create_case(title: String) -> String {
    let case = Case::new(title);
    to_json_string(&case).unwrap_or_else(|_| "{}".to_string())
}

/// Parse case from JSON
#[napi]
pub fn parse_case(json: String) -> NapiResult<String> {
    let case: Case = from_json_string(&json)?;
    to_ffi_result(case.validate())?;
    to_json_string(&case)
}

/// Set case status
#[napi]
pub fn case_set_status(case_json: String, status: JsCaseStatus) -> NapiResult<String> {
    let mut case: Case = from_json_string(&case_json)?;
    let s: CaseStatus = status.into();
    to_ffi_result(case.set_status(s))?;
    to_json_string(&case)
}

/// Add tag to case
#[napi]
pub fn case_add_tag(case_json: String, tag: String) -> NapiResult<String> {
    let mut case: Case = from_json_string(&case_json)?;
    case.add_tag(tag);
    to_json_string(&case)
}

/// Get case summary
#[napi]
pub fn case_summary(case_json: String) -> NapiResult<String> {
    let case: Case = from_json_string(&case_json)?;
    let summary = case.summary();
    to_json_string(&summary)
}

/// Check if case is overdue
#[napi]
pub fn case_is_overdue(case_json: String) -> NapiResult<bool> {
    let case: Case = from_json_string(&case_json)?;
    Ok(case.is_overdue())
}

// ============================================================================
// Evidence Operations
// ============================================================================

/// Create new evidence (returns JSON string)
#[napi]
pub fn create_evidence(title: String, evidence_type: JsEvidenceType) -> String {
    let etype: EvidenceType = evidence_type.into();
    let evidence = Evidence::new(title, etype);
    to_json_string(&evidence).unwrap_or_else(|_| "{}".to_string())
}

/// Parse evidence from JSON
#[napi]
pub fn parse_evidence(json: String) -> NapiResult<String> {
    let evidence: Evidence = from_json_string(&json)?;
    to_ffi_result(evidence.validate())?;
    to_json_string(&evidence)
}

/// Transfer custody of evidence
#[napi]
pub fn evidence_transfer_custody(
    evidence_json: String,
    custodian: String,
    purpose: String,
) -> NapiResult<String> {
    let mut evidence: Evidence = from_json_string(&evidence_json)?;
    evidence.transfer_custody(custodian, purpose);
    to_json_string(&evidence)
}

/// Set evidence relevance score
#[napi]
pub fn evidence_set_relevance(evidence_json: String, score: u32) -> NapiResult<String> {
    let mut evidence: Evidence = from_json_string(&evidence_json)?;
    to_ffi_result(evidence.set_relevance(score as u8))?;
    to_json_string(&evidence)
}

/// Attach file to evidence
#[napi]
pub fn evidence_attach_file(
    evidence_json: String,
    file_path: String,
    file_size: u32,
    format: String,
) -> NapiResult<String> {
    let mut evidence: Evidence = from_json_string(&evidence_json)?;
    evidence.attach_file(file_path, file_size as u64, format);
    to_json_string(&evidence)
}

/// Get evidence summary
#[napi]
pub fn evidence_summary(evidence_json: String) -> NapiResult<String> {
    let evidence: Evidence = from_json_string(&evidence_json)?;
    let summary = evidence.summary();
    to_json_string(&summary)
}

// ============================================================================
// Configuration Operations
// ============================================================================

/// Get default configuration as JSON
#[napi]
pub fn get_default_config() -> String {
    let config = Config::default();
    to_json_string(&config).unwrap_or_else(|_| "{}".to_string())
}

/// Validate configuration JSON
#[napi]
pub fn validate_config(config_json: String) -> NapiResult<bool> {
    let config: Config = from_json_string(&config_json)?;
    to_ffi_result(config.validate())?;
    Ok(true)
}

// ============================================================================
// Validation
// ============================================================================

/// Validate any AccuScene object by type
#[napi]
pub fn validate_object(object_json: String, object_type: String) -> NapiResult<bool> {
    match object_type.as_str() {
        "vehicle" => {
            let obj: Vehicle = from_json_string(&object_json)?;
            to_ffi_result(obj.validate())?;
        }
        "scene" => {
            let obj: AccidentScene = from_json_string(&object_json)?;
            to_ffi_result(obj.validate())?;
        }
        "case" => {
            let obj: Case = from_json_string(&object_json)?;
            to_ffi_result(obj.validate())?;
        }
        "evidence" => {
            let obj: Evidence = from_json_string(&object_json)?;
            to_ffi_result(obj.validate())?;
        }
        "config" => {
            let obj: Config = from_json_string(&object_json)?;
            to_ffi_result(obj.validate())?;
        }
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                format!("Unknown object type: {}", object_type),
            ))
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let ver = version();
        assert!(ver.contains("accuscene-core"));
    }

    #[test]
    fn test_vector_operations() {
        let v1 = JsVector2D { x: 3.0, y: 4.0 };
        let mag = vector2d_magnitude(v1.clone());
        assert_eq!(mag, 5.0);
    }

    #[test]
    fn test_create_vehicle() {
        let vehicle_json = create_vehicle(JsVehicleCategory::Car);
        assert!(vehicle_json.contains("\"category\""));
    }
}
