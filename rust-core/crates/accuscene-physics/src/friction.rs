//! Surface friction models for various road conditions.

use serde::{Deserialize, Serialize};

/// Surface type for friction calculations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Dry asphalt
    AsphaltDry,
    /// Wet asphalt
    AsphaltWet,
    /// Dry concrete
    ConcreteDry,
    /// Wet concrete
    ConcreteWet,
    /// Gravel
    Gravel,
    /// Dirt
    Dirt,
    /// Ice
    Ice,
    /// Snow (packed)
    SnowPacked,
    /// Snow (loose)
    SnowLoose,
    /// Grass (dry)
    GrassDry,
    /// Grass (wet)
    GrassWet,
    /// Custom surface
    Custom(u32),
}

impl SurfaceType {
    /// Returns the static friction coefficient for this surface.
    pub fn static_friction(&self) -> f64 {
        match self {
            Self::AsphaltDry => 0.9,
            Self::AsphaltWet => 0.6,
            Self::ConcreteDry => 0.85,
            Self::ConcreteWet => 0.55,
            Self::Gravel => 0.6,
            Self::Dirt => 0.5,
            Self::Ice => 0.15,
            Self::SnowPacked => 0.25,
            Self::SnowLoose => 0.2,
            Self::GrassDry => 0.4,
            Self::GrassWet => 0.3,
            Self::Custom(_) => 0.7, // Default for custom surfaces
        }
    }

    /// Returns the kinetic friction coefficient for this surface.
    pub fn kinetic_friction(&self) -> f64 {
        // Kinetic friction is typically 70-80% of static friction
        self.static_friction() * 0.75
    }

    /// Returns a descriptive name for the surface.
    pub fn name(&self) -> &str {
        match self {
            Self::AsphaltDry => "Dry Asphalt",
            Self::AsphaltWet => "Wet Asphalt",
            Self::ConcreteDry => "Dry Concrete",
            Self::ConcreteWet => "Wet Concrete",
            Self::Gravel => "Gravel",
            Self::Dirt => "Dirt",
            Self::Ice => "Ice",
            Self::SnowPacked => "Packed Snow",
            Self::SnowLoose => "Loose Snow",
            Self::GrassDry => "Dry Grass",
            Self::GrassWet => "Wet Grass",
            Self::Custom(id) => return "Custom Surface",
        }
    }
}

impl Default for SurfaceType {
    fn default() -> Self {
        Self::AsphaltDry
    }
}

/// Friction model for calculating forces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrictionModel {
    /// Surface type
    pub surface: SurfaceType,
    /// Temperature adjustment factor (1.0 = nominal)
    pub temperature_factor: f64,
    /// Wear adjustment factor (1.0 = no wear, 0.0 = completely worn)
    pub wear_factor: f64,
    /// Contamination factor (1.0 = clean, 0.0 = heavily contaminated)
    pub contamination_factor: f64,
}

impl FrictionModel {
    /// Creates a new friction model for the given surface.
    pub fn new(surface: SurfaceType) -> Self {
        Self {
            surface,
            temperature_factor: 1.0,
            wear_factor: 1.0,
            contamination_factor: 1.0,
        }
    }

    /// Sets the temperature factor.
    pub fn with_temperature_factor(mut self, factor: f64) -> Self {
        self.temperature_factor = factor.clamp(0.5, 1.5);
        self
    }

    /// Sets the wear factor.
    pub fn with_wear_factor(mut self, factor: f64) -> Self {
        self.wear_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Sets the contamination factor.
    pub fn with_contamination_factor(mut self, factor: f64) -> Self {
        self.contamination_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Calculates the effective static friction coefficient.
    pub fn effective_static_friction(&self) -> f64 {
        let base_friction = self.surface.static_friction();
        base_friction
            * self.temperature_factor
            * self.wear_factor
            * self.contamination_factor
    }

    /// Calculates the effective kinetic friction coefficient.
    pub fn effective_kinetic_friction(&self) -> f64 {
        let base_friction = self.surface.kinetic_friction();
        base_friction
            * self.temperature_factor
            * self.wear_factor
            * self.contamination_factor
    }

    /// Calculates friction force given normal force and velocity.
    pub fn friction_force(&self, normal_force: f64, velocity: f64) -> f64 {
        let friction_coefficient = if velocity.abs() < 0.1 {
            self.effective_static_friction()
        } else {
            self.effective_kinetic_friction()
        };

        friction_coefficient * normal_force
    }

    /// Adjusts friction based on temperature (Celsius).
    pub fn adjust_for_temperature(&mut self, temperature: f64) {
        // Friction generally decreases at very high temperatures and very low temperatures
        self.temperature_factor = match self.surface {
            SurfaceType::Ice | SurfaceType::SnowPacked | SurfaceType::SnowLoose => {
                // Ice/snow: friction increases as temperature approaches 0°C
                if temperature < -20.0 {
                    0.7
                } else if temperature < 0.0 {
                    1.0 + (temperature / 20.0) * 0.3
                } else {
                    1.0 // Melting, different physics apply
                }
            }
            _ => {
                // Other surfaces: optimal around 20°C
                if temperature < -10.0 {
                    0.8
                } else if temperature > 50.0 {
                    0.9
                } else {
                    1.0
                }
            }
        };
    }

    /// Adjusts friction based on road contamination (0.0 = clean, 1.0 = heavily contaminated).
    pub fn adjust_for_contamination(&mut self, level: f64) {
        self.contamination_factor = 1.0 - (level.clamp(0.0, 1.0) * 0.5);
    }
}

impl Default for FrictionModel {
    fn default() -> Self {
        Self::new(SurfaceType::default())
    }
}

/// Calculates drag coefficient based on road surface.
pub fn surface_rolling_resistance(surface: SurfaceType) -> f64 {
    match surface {
        SurfaceType::AsphaltDry | SurfaceType::AsphaltWet => 0.015,
        SurfaceType::ConcreteDry | SurfaceType::ConcreteWet => 0.012,
        SurfaceType::Gravel => 0.03,
        SurfaceType::Dirt => 0.04,
        SurfaceType::Ice => 0.01,
        SurfaceType::SnowPacked => 0.025,
        SurfaceType::SnowLoose => 0.05,
        SurfaceType::GrassDry => 0.08,
        SurfaceType::GrassWet => 0.1,
        SurfaceType::Custom(_) => 0.02,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surface_friction_coefficients() {
        assert!(SurfaceType::AsphaltDry.static_friction() > SurfaceType::AsphaltWet.static_friction());
        assert!(SurfaceType::AsphaltDry.static_friction() > SurfaceType::Ice.static_friction());
        assert!(SurfaceType::Ice.static_friction() < 0.2);
    }

    #[test]
    fn test_kinetic_friction() {
        let asphalt = SurfaceType::AsphaltDry;
        assert!(asphalt.kinetic_friction() < asphalt.static_friction());
    }

    #[test]
    fn test_friction_model() {
        let model = FrictionModel::new(SurfaceType::AsphaltDry);
        assert_eq!(model.effective_static_friction(), 0.9);

        let worn_model = FrictionModel::new(SurfaceType::AsphaltDry)
            .with_wear_factor(0.8);
        assert!(worn_model.effective_static_friction() < 0.9);
    }

    #[test]
    fn test_temperature_adjustment() {
        let mut model = FrictionModel::new(SurfaceType::AsphaltDry);
        let original = model.effective_static_friction();

        model.adjust_for_temperature(60.0); // Hot surface
        assert!(model.effective_static_friction() < original);
    }

    #[test]
    fn test_contamination_adjustment() {
        let mut model = FrictionModel::new(SurfaceType::AsphaltDry);
        let original = model.effective_static_friction();

        model.adjust_for_contamination(0.5); // 50% contaminated
        assert!(model.effective_static_friction() < original);
    }

    #[test]
    fn test_friction_force() {
        let model = FrictionModel::new(SurfaceType::AsphaltDry);
        let normal_force = 10000.0; // 10 kN

        let force_static = model.friction_force(normal_force, 0.0);
        let force_kinetic = model.friction_force(normal_force, 10.0);

        assert!(force_static > force_kinetic);
    }
}
