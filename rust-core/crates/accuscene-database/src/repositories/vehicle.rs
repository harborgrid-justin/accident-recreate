//! Vehicle repository for managing vehicle records

use crate::error::{DatabaseError, DbResult};
use crate::repositories::Repository;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vehicle {
    pub id: String,
    pub accident_id: String,
    pub vehicle_number: i32,
    pub make: Option<String>,
    pub model: Option<String>,
    pub year: Option<i32>,
    pub color: Option<String>,
    pub vin: Option<String>,
    pub license_plate: Option<String>,
    pub vehicle_type: Option<String>,
    pub damage_description: Option<String>,
    pub occupants: Option<i32>,
    pub speed_estimate: Option<f64>,
    pub direction_of_travel: Option<f64>,
    pub final_position_lat: Option<f64>,
    pub final_position_lng: Option<f64>,
    pub airbag_deployment: Option<i32>,
    pub driver_info: Option<serde_json::Value>,
    pub insurance_info: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

impl Vehicle {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let driver_json: Option<String> = row.get(17)?;
        let driver_info = driver_json.and_then(|s| serde_json::from_str(&s).ok());

        let insurance_json: Option<String> = row.get(18)?;
        let insurance_info = insurance_json.and_then(|s| serde_json::from_str(&s).ok());

        let metadata_json: Option<String> = row.get(19)?;
        let metadata = metadata_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Self {
            id: row.get(0)?,
            accident_id: row.get(1)?,
            vehicle_number: row.get(2)?,
            make: row.get(3)?,
            model: row.get(4)?,
            year: row.get(5)?,
            color: row.get(6)?,
            vin: row.get(7)?,
            license_plate: row.get(8)?,
            vehicle_type: row.get(9)?,
            damage_description: row.get(10)?,
            occupants: row.get(11)?,
            speed_estimate: row.get(12)?,
            direction_of_travel: row.get(13)?,
            final_position_lat: row.get(14)?,
            final_position_lng: row.get(15)?,
            airbag_deployment: row.get(16)?,
            driver_info,
            insurance_info,
            metadata,
            created_at: row.get(20)?,
            updated_at: row.get(21)?,
        })
    }
}

pub struct VehicleRepository;

impl VehicleRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn find_by_accident_id(&self, conn: &Connection, accident_id: &str) -> DbResult<Vec<Vehicle>> {
        let mut stmt = conn.prepare(
            "SELECT id, accident_id, vehicle_number, make, model, year, color, vin,
                    license_plate, vehicle_type, damage_description, occupants,
                    speed_estimate, direction_of_travel, final_position_lat, final_position_lng,
                    airbag_deployment, driver_info, insurance_info, metadata,
                    created_at, updated_at
             FROM vehicles WHERE accident_id = ? ORDER BY vehicle_number",
        )?;

        let vehicles = stmt
            .query_map([accident_id], Vehicle::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(vehicles)
    }

    pub fn find_by_vin(&self, conn: &Connection, vin: &str) -> DbResult<Option<Vehicle>> {
        let mut stmt = conn.prepare(
            "SELECT id, accident_id, vehicle_number, make, model, year, color, vin,
                    license_plate, vehicle_type, damage_description, occupants,
                    speed_estimate, direction_of_travel, final_position_lat, final_position_lng,
                    airbag_deployment, driver_info, insurance_info, metadata,
                    created_at, updated_at
             FROM vehicles WHERE vin = ?",
        )?;

        let mut rows = stmt.query_map([vin], Vehicle::from_row)?;
        Ok(rows.next().transpose()?)
    }
}

impl Default for VehicleRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for VehicleRepository {
    type Entity = Vehicle;
    type Id = String;

    fn find_by_id(&self, conn: &Connection, id: &String) -> DbResult<Option<Vehicle>> {
        let mut stmt = conn.prepare(
            "SELECT id, accident_id, vehicle_number, make, model, year, color, vin,
                    license_plate, vehicle_type, damage_description, occupants,
                    speed_estimate, direction_of_travel, final_position_lat, final_position_lng,
                    airbag_deployment, driver_info, insurance_info, metadata,
                    created_at, updated_at
             FROM vehicles WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], Vehicle::from_row)?;
        Ok(rows.next().transpose()?)
    }

    fn find_all(&self, conn: &Connection) -> DbResult<Vec<Vehicle>> {
        let mut stmt = conn.prepare(
            "SELECT id, accident_id, vehicle_number, make, model, year, color, vin,
                    license_plate, vehicle_type, damage_description, occupants,
                    speed_estimate, direction_of_travel, final_position_lat, final_position_lng,
                    airbag_deployment, driver_info, insurance_info, metadata,
                    created_at, updated_at
             FROM vehicles ORDER BY created_at DESC",
        )?;

        let vehicles = stmt
            .query_map([], Vehicle::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(vehicles)
    }

    fn create(&self, conn: &Connection, entity: &Vehicle) -> DbResult<()> {
        let driver_json = entity.driver_info.as_ref().map(|d| serde_json::to_string(d).ok()).flatten();
        let insurance_json = entity.insurance_info.as_ref().map(|i| serde_json::to_string(i).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        conn.execute(
            "INSERT INTO vehicles (id, accident_id, vehicle_number, make, model, year, color, vin,
                                  license_plate, vehicle_type, damage_description, occupants,
                                  speed_estimate, direction_of_travel, final_position_lat, final_position_lng,
                                  airbag_deployment, driver_info, insurance_info, metadata)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entity.id, entity.accident_id, entity.vehicle_number, entity.make, entity.model,
                entity.year, entity.color, entity.vin, entity.license_plate, entity.vehicle_type,
                entity.damage_description, entity.occupants, entity.speed_estimate,
                entity.direction_of_travel, entity.final_position_lat, entity.final_position_lng,
                entity.airbag_deployment, driver_json, insurance_json, metadata_json,
            ],
        )?;
        Ok(())
    }

    fn update(&self, conn: &Connection, entity: &Vehicle) -> DbResult<()> {
        let driver_json = entity.driver_info.as_ref().map(|d| serde_json::to_string(d).ok()).flatten();
        let insurance_json = entity.insurance_info.as_ref().map(|i| serde_json::to_string(i).ok()).flatten();
        let metadata_json = entity.metadata.as_ref().map(|m| serde_json::to_string(m).ok()).flatten();

        let affected = conn.execute(
            "UPDATE vehicles SET accident_id = ?, vehicle_number = ?, make = ?, model = ?, year = ?,
                                color = ?, vin = ?, license_plate = ?, vehicle_type = ?,
                                damage_description = ?, occupants = ?, speed_estimate = ?,
                                direction_of_travel = ?, final_position_lat = ?, final_position_lng = ?,
                                airbag_deployment = ?, driver_info = ?, insurance_info = ?, metadata = ?
             WHERE id = ?",
            params![
                entity.accident_id, entity.vehicle_number, entity.make, entity.model, entity.year,
                entity.color, entity.vin, entity.license_plate, entity.vehicle_type,
                entity.damage_description, entity.occupants, entity.speed_estimate,
                entity.direction_of_travel, entity.final_position_lat, entity.final_position_lng,
                entity.airbag_deployment, driver_json, insurance_json, metadata_json, entity.id,
            ],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Vehicle", "id", &entity.id))
        } else {
            Ok(())
        }
    }

    fn delete(&self, conn: &Connection, id: &String) -> DbResult<()> {
        let affected = conn.execute("DELETE FROM vehicles WHERE id = ?", [id])?;
        if affected == 0 {
            Err(DatabaseError::not_found("Vehicle", "id", id))
        } else {
            Ok(())
        }
    }

    fn count(&self, conn: &Connection) -> DbResult<i64> {
        let count = conn.query_row("SELECT COUNT(*) FROM vehicles", [], |row| row.get(0))?;
        Ok(count)
    }
}
