//! Type conversions between JavaScript and Rust
//!
//! This module provides conversion utilities for passing data across
//! the FFI boundary with proper type safety.

use accuscene_core::types::*;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

/// Convert a Rust type to JSON string for JS
pub fn to_json_string<T: Serialize>(value: &T) -> Result<String> {
    serde_json::to_string(value).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Failed to serialize to JSON: {}", e),
        )
    })
}

/// Convert JSON string from JS to Rust type
pub fn from_json_string<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T> {
    serde_json::from_str(json).map_err(|e| {
        Error::new(
            Status::InvalidArg,
            format!("Failed to deserialize from JSON: {}", e),
        )
    })
}

/// JavaScript-compatible Vector2D
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsVector2D {
    pub x: f64,
    pub y: f64,
}

impl From<Vector2D> for JsVector2D {
    fn from(v: Vector2D) -> Self {
        Self { x: v.x, y: v.y }
    }
}

impl From<JsVector2D> for Vector2D {
    fn from(v: JsVector2D) -> Self {
        Self::new(v.x, v.y)
    }
}

/// JavaScript-compatible Vector3D
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsVector3D {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl From<Vector3D> for JsVector3D {
    fn from(v: Vector3D) -> Self {
        Self { x: v.x, y: v.y, z: v.z }
    }
}

impl From<JsVector3D> for Vector3D {
    fn from(v: JsVector3D) -> Self {
        Self::new(v.x, v.y, v.z)
    }
}

/// JavaScript-compatible vehicle metadata
#[napi(object)]
#[derive(Debug, Clone)]
pub struct JsVehicleMetadata {
    pub make: String,
    pub model: String,
    pub year: Option<u32>,
    pub color: Option<String>,
    pub license_plate: Option<String>,
    pub vin: Option<String>,
    pub notes: Option<String>,
}

impl From<VehicleMetadata> for JsVehicleMetadata {
    fn from(m: VehicleMetadata) -> Self {
        Self {
            make: m.make,
            model: m.model,
            year: m.year.map(|y| y as u32),
            color: m.color,
            license_plate: m.license_plate,
            vin: m.vin,
            notes: m.notes,
        }
    }
}

impl From<JsVehicleMetadata> for VehicleMetadata {
    fn from(m: JsVehicleMetadata) -> Self {
        Self {
            make: m.make,
            model: m.model,
            year: m.year.map(|y| y as u16),
            color: m.color,
            license_plate: m.license_plate,
            vin: m.vin,
            notes: m.notes,
        }
    }
}

/// JavaScript-compatible vehicle category
#[napi(string_enum)]
#[derive(Debug, Clone, Copy)]
pub enum JsVehicleCategory {
    Car,
    SUV,
    Truck,
    Motorcycle,
    Van,
    Commercial,
    Bus,
    Bicycle,
    Pedestrian,
    Other,
}

impl From<VehicleCategory> for JsVehicleCategory {
    fn from(c: VehicleCategory) -> Self {
        match c {
            VehicleCategory::Car => Self::Car,
            VehicleCategory::SUV => Self::SUV,
            VehicleCategory::Truck => Self::Truck,
            VehicleCategory::Motorcycle => Self::Motorcycle,
            VehicleCategory::Van => Self::Van,
            VehicleCategory::Commercial => Self::Commercial,
            VehicleCategory::Bus => Self::Bus,
            VehicleCategory::Bicycle => Self::Bicycle,
            VehicleCategory::Pedestrian => Self::Pedestrian,
            VehicleCategory::Other => Self::Other,
        }
    }
}

impl From<JsVehicleCategory> for VehicleCategory {
    fn from(c: JsVehicleCategory) -> Self {
        match c {
            JsVehicleCategory::Car => Self::Car,
            JsVehicleCategory::SUV => Self::SUV,
            JsVehicleCategory::Truck => Self::Truck,
            JsVehicleCategory::Motorcycle => Self::Motorcycle,
            JsVehicleCategory::Van => Self::Van,
            JsVehicleCategory::Commercial => Self::Commercial,
            JsVehicleCategory::Bus => Self::Bus,
            JsVehicleCategory::Bicycle => Self::Bicycle,
            JsVehicleCategory::Pedestrian => Self::Pedestrian,
            JsVehicleCategory::Other => Self::Other,
        }
    }
}

/// JavaScript-compatible weather condition
#[napi(string_enum)]
#[derive(Debug, Clone, Copy)]
pub enum JsWeatherCondition {
    Clear,
    PartlyCloudy,
    Cloudy,
    LightRain,
    HeavyRain,
    Fog,
    Snow,
    Ice,
    Windy,
    Unknown,
}

impl From<WeatherCondition> for JsWeatherCondition {
    fn from(w: WeatherCondition) -> Self {
        match w {
            WeatherCondition::Clear => Self::Clear,
            WeatherCondition::PartlyCloudy => Self::PartlyCloudy,
            WeatherCondition::Cloudy => Self::Cloudy,
            WeatherCondition::LightRain => Self::LightRain,
            WeatherCondition::HeavyRain => Self::HeavyRain,
            WeatherCondition::Fog => Self::Fog,
            WeatherCondition::Snow => Self::Snow,
            WeatherCondition::Ice => Self::Ice,
            WeatherCondition::Windy => Self::Windy,
            WeatherCondition::Unknown => Self::Unknown,
        }
    }
}

impl From<JsWeatherCondition> for WeatherCondition {
    fn from(w: JsWeatherCondition) -> Self {
        match w {
            JsWeatherCondition::Clear => Self::Clear,
            JsWeatherCondition::PartlyCloudy => Self::PartlyCloudy,
            JsWeatherCondition::Cloudy => Self::Cloudy,
            JsWeatherCondition::LightRain => Self::LightRain,
            JsWeatherCondition::HeavyRain => Self::HeavyRain,
            JsWeatherCondition::Fog => Self::Fog,
            JsWeatherCondition::Snow => Self::Snow,
            JsWeatherCondition::Ice => Self::Ice,
            JsWeatherCondition::Windy => Self::Windy,
            JsWeatherCondition::Unknown => Self::Unknown,
        }
    }
}

/// JavaScript-compatible road condition
#[napi(string_enum)]
#[derive(Debug, Clone, Copy)]
pub enum JsRoadCondition {
    Dry,
    Wet,
    Icy,
    Snowy,
    Gravel,
    Dirt,
    Construction,
    Damaged,
    Unknown,
}

impl From<RoadCondition> for JsRoadCondition {
    fn from(r: RoadCondition) -> Self {
        match r {
            RoadCondition::Dry => Self::Dry,
            RoadCondition::Wet => Self::Wet,
            RoadCondition::Icy => Self::Icy,
            RoadCondition::Snowy => Self::Snowy,
            RoadCondition::Gravel => Self::Gravel,
            RoadCondition::Dirt => Self::Dirt,
            RoadCondition::Construction => Self::Construction,
            RoadCondition::Damaged => Self::Damaged,
            RoadCondition::Unknown => Self::Unknown,
        }
    }
}

impl From<JsRoadCondition> for RoadCondition {
    fn from(r: JsRoadCondition) -> Self {
        match r {
            JsRoadCondition::Dry => Self::Dry,
            JsRoadCondition::Wet => Self::Wet,
            JsRoadCondition::Icy => Self::Icy,
            JsRoadCondition::Snowy => Self::Snowy,
            JsRoadCondition::Gravel => Self::Gravel,
            JsRoadCondition::Dirt => Self::Dirt,
            JsRoadCondition::Construction => Self::Construction,
            JsRoadCondition::Damaged => Self::Damaged,
            JsRoadCondition::Unknown => Self::Unknown,
        }
    }
}

/// JavaScript-compatible case status
#[napi(string_enum)]
#[derive(Debug, Clone, Copy)]
pub enum JsCaseStatus {
    Draft,
    Active,
    OnHold,
    UnderReview,
    Completed,
    Archived,
    Cancelled,
}

impl From<CaseStatus> for JsCaseStatus {
    fn from(s: CaseStatus) -> Self {
        match s {
            CaseStatus::Draft => Self::Draft,
            CaseStatus::Active => Self::Active,
            CaseStatus::OnHold => Self::OnHold,
            CaseStatus::UnderReview => Self::UnderReview,
            CaseStatus::Completed => Self::Completed,
            CaseStatus::Archived => Self::Archived,
            CaseStatus::Cancelled => Self::Cancelled,
        }
    }
}

impl From<JsCaseStatus> for CaseStatus {
    fn from(s: JsCaseStatus) -> Self {
        match s {
            JsCaseStatus::Draft => Self::Draft,
            JsCaseStatus::Active => Self::Active,
            JsCaseStatus::OnHold => Self::OnHold,
            JsCaseStatus::UnderReview => Self::UnderReview,
            JsCaseStatus::Completed => Self::Completed,
            JsCaseStatus::Archived => Self::Archived,
            JsCaseStatus::Cancelled => Self::Cancelled,
        }
    }
}

/// JavaScript-compatible evidence type
#[napi(string_enum)]
#[derive(Debug, Clone, Copy)]
pub enum JsEvidenceType {
    Photo,
    Video,
    Audio,
    Document,
    Physical,
    WitnessStatement,
    ExpertReport,
    Diagram,
    ThreeDModel,
    TelemetryData,
    MedicalReport,
    VehicleInspection,
    WeatherReport,
    Other,
}

impl From<EvidenceType> for JsEvidenceType {
    fn from(e: EvidenceType) -> Self {
        match e {
            EvidenceType::Photo => Self::Photo,
            EvidenceType::Video => Self::Video,
            EvidenceType::Audio => Self::Audio,
            EvidenceType::Document => Self::Document,
            EvidenceType::Physical => Self::Physical,
            EvidenceType::WitnessStatement => Self::WitnessStatement,
            EvidenceType::ExpertReport => Self::ExpertReport,
            EvidenceType::Diagram => Self::Diagram,
            EvidenceType::ThreeDModel => Self::ThreeDModel,
            EvidenceType::TelemetryData => Self::TelemetryData,
            EvidenceType::MedicalReport => Self::MedicalReport,
            EvidenceType::VehicleInspection => Self::VehicleInspection,
            EvidenceType::WeatherReport => Self::WeatherReport,
            EvidenceType::Other => Self::Other,
        }
    }
}

impl From<JsEvidenceType> for EvidenceType {
    fn from(e: JsEvidenceType) -> Self {
        match e {
            JsEvidenceType::Photo => Self::Photo,
            JsEvidenceType::Video => Self::Video,
            JsEvidenceType::Audio => Self::Audio,
            JsEvidenceType::Document => Self::Document,
            JsEvidenceType::Physical => Self::Physical,
            JsEvidenceType::WitnessStatement => Self::WitnessStatement,
            JsEvidenceType::ExpertReport => Self::ExpertReport,
            JsEvidenceType::Diagram => Self::Diagram,
            JsEvidenceType::ThreeDModel => Self::ThreeDModel,
            JsEvidenceType::TelemetryData => Self::TelemetryData,
            JsEvidenceType::MedicalReport => Self::MedicalReport,
            JsEvidenceType::VehicleInspection => Self::VehicleInspection,
            JsEvidenceType::WeatherReport => Self::WeatherReport,
            JsEvidenceType::Other => Self::Other,
        }
    }
}
