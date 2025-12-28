//! Accident repository for managing accident records

use crate::error::{DatabaseError, DbResult};
use crate::repositories::Repository;
use rusqlite::{params, Connection, Row};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Accident {
    pub id: String,
    pub case_id: String,
    pub accident_date: String,
    pub location: String,
    pub location_lat: Option<f64>,
    pub location_lng: Option<f64>,
    pub weather_conditions: Option<String>,
    pub road_conditions: Option<String>,
    pub light_conditions: Option<String>,
    pub traffic_control: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub fatalities: i32,
    pub injuries: i32,
    pub property_damage_estimate: Option<f64>,
    pub police_report_number: Option<String>,
    pub police_department: Option<String>,
    pub reconstruction_data: Option<serde_json::Value>,
    pub created_at: String,
    pub updated_at: String,
}

impl Accident {
    fn from_row(row: &Row) -> rusqlite::Result<Self> {
        let reconstruction_json: Option<String> = row.get(17)?;
        let reconstruction_data = reconstruction_json.and_then(|s| serde_json::from_str(&s).ok());

        Ok(Self {
            id: row.get(0)?,
            case_id: row.get(1)?,
            accident_date: row.get(2)?,
            location: row.get(3)?,
            location_lat: row.get(4)?,
            location_lng: row.get(5)?,
            weather_conditions: row.get(6)?,
            road_conditions: row.get(7)?,
            light_conditions: row.get(8)?,
            traffic_control: row.get(9)?,
            description: row.get(10)?,
            severity: row.get(11)?,
            fatalities: row.get(12)?,
            injuries: row.get(13)?,
            property_damage_estimate: row.get(14)?,
            police_report_number: row.get(15)?,
            police_department: row.get(16)?,
            reconstruction_data,
            created_at: row.get(18)?,
            updated_at: row.get(19)?,
        })
    }
}

pub struct AccidentRepository;

impl AccidentRepository {
    pub fn new() -> Self {
        Self
    }

    pub fn find_by_case_id(&self, conn: &Connection, case_id: &str) -> DbResult<Vec<Accident>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_date, location, location_lat, location_lng,
                    weather_conditions, road_conditions, light_conditions, traffic_control,
                    description, severity, fatalities, injuries, property_damage_estimate,
                    police_report_number, police_department, reconstruction_data,
                    created_at, updated_at
             FROM accidents WHERE case_id = ? ORDER BY accident_date DESC",
        )?;

        let accidents = stmt
            .query_map([case_id], Accident::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accidents)
    }

    pub fn find_by_severity(&self, conn: &Connection, severity: &str) -> DbResult<Vec<Accident>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_date, location, location_lat, location_lng,
                    weather_conditions, road_conditions, light_conditions, traffic_control,
                    description, severity, fatalities, injuries, property_damage_estimate,
                    police_report_number, police_department, reconstruction_data,
                    created_at, updated_at
             FROM accidents WHERE severity = ? ORDER BY accident_date DESC",
        )?;

        let accidents = stmt
            .query_map([severity], Accident::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accidents)
    }
}

impl Default for AccidentRepository {
    fn default() -> Self {
        Self::new()
    }
}

impl Repository for AccidentRepository {
    type Entity = Accident;
    type Id = String;

    fn find_by_id(&self, conn: &Connection, id: &String) -> DbResult<Option<Accident>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_date, location, location_lat, location_lng,
                    weather_conditions, road_conditions, light_conditions, traffic_control,
                    description, severity, fatalities, injuries, property_damage_estimate,
                    police_report_number, police_department, reconstruction_data,
                    created_at, updated_at
             FROM accidents WHERE id = ?",
        )?;

        let mut rows = stmt.query_map([id], Accident::from_row)?;
        Ok(rows.next().transpose()?)
    }

    fn find_all(&self, conn: &Connection) -> DbResult<Vec<Accident>> {
        let mut stmt = conn.prepare(
            "SELECT id, case_id, accident_date, location, location_lat, location_lng,
                    weather_conditions, road_conditions, light_conditions, traffic_control,
                    description, severity, fatalities, injuries, property_damage_estimate,
                    police_report_number, police_department, reconstruction_data,
                    created_at, updated_at
             FROM accidents ORDER BY accident_date DESC",
        )?;

        let accidents = stmt
            .query_map([], Accident::from_row)?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(accidents)
    }

    fn create(&self, conn: &Connection, entity: &Accident) -> DbResult<()> {
        let reconstruction_json = entity.reconstruction_data.as_ref()
            .map(|d| serde_json::to_string(d).ok()).flatten();

        conn.execute(
            "INSERT INTO accidents (id, case_id, accident_date, location, location_lat, location_lng,
                                   weather_conditions, road_conditions, light_conditions, traffic_control,
                                   description, severity, fatalities, injuries, property_damage_estimate,
                                   police_report_number, police_department, reconstruction_data)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entity.id, entity.case_id, entity.accident_date, entity.location,
                entity.location_lat, entity.location_lng, entity.weather_conditions,
                entity.road_conditions, entity.light_conditions, entity.traffic_control,
                entity.description, entity.severity, entity.fatalities, entity.injuries,
                entity.property_damage_estimate, entity.police_report_number,
                entity.police_department, reconstruction_json,
            ],
        )?;
        Ok(())
    }

    fn update(&self, conn: &Connection, entity: &Accident) -> DbResult<()> {
        let reconstruction_json = entity.reconstruction_data.as_ref()
            .map(|d| serde_json::to_string(d).ok()).flatten();

        let affected = conn.execute(
            "UPDATE accidents SET case_id = ?, accident_date = ?, location = ?, location_lat = ?,
                                 location_lng = ?, weather_conditions = ?, road_conditions = ?,
                                 light_conditions = ?, traffic_control = ?, description = ?,
                                 severity = ?, fatalities = ?, injuries = ?, property_damage_estimate = ?,
                                 police_report_number = ?, police_department = ?, reconstruction_data = ?
             WHERE id = ?",
            params![
                entity.case_id, entity.accident_date, entity.location, entity.location_lat,
                entity.location_lng, entity.weather_conditions, entity.road_conditions,
                entity.light_conditions, entity.traffic_control, entity.description,
                entity.severity, entity.fatalities, entity.injuries, entity.property_damage_estimate,
                entity.police_report_number, entity.police_department, reconstruction_json, entity.id,
            ],
        )?;

        if affected == 0 {
            Err(DatabaseError::not_found("Accident", "id", &entity.id))
        } else {
            Ok(())
        }
    }

    fn delete(&self, conn: &Connection, id: &String) -> DbResult<()> {
        let affected = conn.execute("DELETE FROM accidents WHERE id = ?", [id])?;
        if affected == 0 {
            Err(DatabaseError::not_found("Accident", "id", id))
        } else {
            Ok(())
        }
    }

    fn count(&self, conn: &Connection) -> DbResult<i64> {
        let count = conn.query_row("SELECT COUNT(*) FROM accidents", [], |row| row.get(0))?;
        Ok(count)
    }
}
