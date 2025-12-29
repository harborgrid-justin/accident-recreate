//! Tire models for vehicle dynamics.
//!
//! Implements the Pacejka "Magic Formula" tire model for:
//! - Longitudinal force (acceleration/braking)
//! - Lateral force (cornering)
//! - Combined slip conditions

use serde::{Deserialize, Serialize};

/// Tire parameters using Pacejka Magic Formula.
///
/// The Magic Formula:
/// Y(x) = D * sin(C * arctan(B*x - E*(B*x - arctan(B*x))))
///
/// Where:
/// - B: Stiffness factor
/// - C: Shape factor
/// - D: Peak value
/// - E: Curvature factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TireParameters {
    /// Tire radius (m).
    pub radius: f64,

    /// Tire mass (kg).
    pub mass: f64,

    /// --- Longitudinal coefficients (Fx) ---

    /// Longitudinal stiffness factor.
    pub b_long: f64,

    /// Longitudinal shape factor.
    pub c_long: f64,

    /// Longitudinal peak friction coefficient.
    pub d_long: f64,

    /// Longitudinal curvature factor.
    pub e_long: f64,

    /// --- Lateral coefficients (Fy) ---

    /// Lateral stiffness factor.
    pub b_lat: f64,

    /// Lateral shape factor.
    pub c_lat: f64,

    /// Lateral peak friction coefficient.
    pub d_lat: f64,

    /// Lateral curvature factor.
    pub e_lat: f64,

    /// --- Additional parameters ---

    /// Rolling resistance coefficient.
    pub rolling_resistance: f64,
}

impl TireParameters {
    /// Creates tire parameters for a typical passenger car.
    pub fn passenger_car() -> Self {
        Self {
            radius: 0.33,    // 33 cm (typical for 205/55R16)
            mass: 10.0,      // kg (tire + wheel)

            // Longitudinal (dry asphalt)
            b_long: 10.0,
            c_long: 1.9,
            d_long: 1.0,
            e_long: 0.97,

            // Lateral (dry asphalt)
            b_lat: 8.0,
            c_lat: 1.3,
            d_lat: 0.9,
            e_lat: -1.6,

            rolling_resistance: 0.015,
        }
    }

    /// Creates tire parameters for a performance/sports car.
    pub fn performance_car() -> Self {
        Self {
            radius: 0.35,
            mass: 12.0,

            // Higher grip coefficients
            b_long: 12.0,
            c_long: 1.9,
            d_long: 1.2,
            e_long: 0.97,

            b_lat: 10.0,
            c_lat: 1.3,
            d_lat: 1.1,
            e_lat: -1.6,

            rolling_resistance: 0.012,
        }
    }

    /// Creates tire parameters for an SUV/truck.
    pub fn suv_truck() -> Self {
        Self {
            radius: 0.40,
            mass: 20.0,

            // Lower performance than passenger car
            b_long: 8.0,
            c_long: 1.9,
            d_long: 0.9,
            e_long: 0.97,

            b_lat: 6.0,
            c_lat: 1.3,
            d_lat: 0.8,
            e_lat: -1.6,

            rolling_resistance: 0.018,
        }
    }

    /// Creates tire parameters for wet conditions.
    pub fn wet_conditions() -> Self {
        let mut params = Self::passenger_car();

        // Reduce peak friction
        params.d_long *= 0.7;
        params.d_lat *= 0.7;

        // Increase stiffness (earlier loss of grip)
        params.b_long *= 1.2;
        params.b_lat *= 1.2;

        params
    }

    /// Creates tire parameters for icy conditions.
    pub fn icy_conditions() -> Self {
        let mut params = Self::passenger_car();

        // Dramatically reduce friction
        params.d_long *= 0.2;
        params.d_lat *= 0.2;

        // Very soft response
        params.b_long *= 0.5;
        params.b_lat *= 0.5;

        params
    }

    /// Computes longitudinal tire force using Pacejka Magic Formula.
    ///
    /// F_x = D_x * sin(C_x * arctan(B_x * κ - E_x * (B_x * κ - arctan(B_x * κ))))
    ///
    /// Where κ is the slip ratio.
    pub fn longitudinal_force(&self, slip_ratio: f64, normal_force: f64) -> f64 {
        if normal_force < 1e-6 {
            return 0.0;
        }

        let kappa = slip_ratio.clamp(-1.0, 1.0);
        let b_x = self.b_long;
        let c_x = self.c_long;
        let d_x = self.d_long * normal_force;
        let e_x = self.e_long;

        let bx_kappa = b_x * kappa;
        let atan_term = bx_kappa.atan();

        let fx = d_x * (c_x * (bx_kappa - e_x * (bx_kappa - atan_term))).sin();

        // Add rolling resistance
        let rolling_resistance_force = -self.rolling_resistance * normal_force * kappa.signum();

        fx + rolling_resistance_force
    }

    /// Computes lateral tire force using Pacejka Magic Formula.
    ///
    /// F_y = D_y * sin(C_y * arctan(B_y * α - E_y * (B_y * α - arctan(B_y * α))))
    ///
    /// Where α is the slip angle (radians).
    pub fn lateral_force(&self, slip_angle: f64, normal_force: f64) -> f64 {
        if normal_force < 1e-6 {
            return 0.0;
        }

        let alpha = slip_angle.clamp(-0.5, 0.5); // Clamp to ±30 degrees
        let b_y = self.b_lat;
        let c_y = self.c_lat;
        let d_y = self.d_lat * normal_force;
        let e_y = self.e_lat;

        let by_alpha = b_y * alpha;
        let atan_term = by_alpha.atan();

        d_y * (c_y * (by_alpha - e_y * (by_alpha - atan_term))).sin()
    }

    /// Computes combined slip force (simplified).
    ///
    /// When both longitudinal and lateral slip exist, the tire force is limited
    /// by the friction circle.
    pub fn combined_force(
        &self,
        slip_ratio: f64,
        slip_angle: f64,
        normal_force: f64,
    ) -> (f64, f64) {
        let fx_pure = self.longitudinal_force(slip_ratio, normal_force);
        let fy_pure = self.lateral_force(slip_angle, normal_force);

        // Friction circle approximation
        let max_force = self.d_long.max(self.d_lat) * normal_force;
        let combined_magnitude = (fx_pure * fx_pure + fy_pure * fy_pure).sqrt();

        if combined_magnitude > max_force {
            let scale = max_force / combined_magnitude;
            (fx_pure * scale, fy_pure * scale)
        } else {
            (fx_pure, fy_pure)
        }
    }

    /// Computes the maximum longitudinal force (peak grip).
    pub fn max_longitudinal_force(&self, normal_force: f64) -> f64 {
        self.d_long * normal_force
    }

    /// Computes the maximum lateral force (peak cornering grip).
    pub fn max_lateral_force(&self, normal_force: f64) -> f64 {
        self.d_lat * normal_force
    }

    /// Computes the optimal slip ratio for maximum longitudinal force.
    pub fn optimal_slip_ratio(&self) -> f64 {
        // Approximate optimal slip ratio (typically 10-20%)
        0.15
    }

    /// Computes the optimal slip angle for maximum lateral force.
    pub fn optimal_slip_angle(&self) -> f64 {
        // Approximate optimal slip angle (typically 5-10 degrees)
        0.1 // radians (~5.7 degrees)
    }
}

impl Default for TireParameters {
    fn default() -> Self {
        Self::passenger_car()
    }
}

/// Tire condition affecting performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TireCondition {
    /// New tires (100% performance).
    New,

    /// Good condition (95% performance).
    Good,

    /// Worn (80% performance).
    Worn,

    /// Severely worn (60% performance).
    SeverelyWorn,
}

impl TireCondition {
    /// Returns the performance multiplier for this condition.
    pub fn performance_factor(&self) -> f64 {
        match self {
            Self::New => 1.0,
            Self::Good => 0.95,
            Self::Worn => 0.80,
            Self::SeverelyWorn => 0.60,
        }
    }
}

/// Road surface condition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadCondition {
    /// Dry asphalt.
    DryAsphalt,

    /// Wet asphalt.
    WetAsphalt,

    /// Snow covered.
    Snow,

    /// Ice.
    Ice,

    /// Gravel.
    Gravel,
}

impl RoadCondition {
    /// Returns the friction multiplier for this surface.
    pub fn friction_factor(&self) -> f64 {
        match self {
            Self::DryAsphalt => 1.0,
            Self::WetAsphalt => 0.7,
            Self::Snow => 0.3,
            Self::Ice => 0.2,
            Self::Gravel => 0.6,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_tire_parameters_creation() {
        let tire = TireParameters::passenger_car();
        assert_eq!(tire.radius, 0.33);
        assert!(tire.d_long > 0.0);
    }

    #[test]
    fn test_longitudinal_force() {
        let tire = TireParameters::passenger_car();
        let normal_force = 5000.0; // N

        // Zero slip should give near-zero force (except rolling resistance)
        let f0 = tire.longitudinal_force(0.0, normal_force);
        assert!(f0.abs() < 100.0);

        // Optimal slip should give significant force
        let f_opt = tire.longitudinal_force(0.15, normal_force);
        assert!(f_opt > 1000.0);

        // Full slip (locked wheel) should give less force
        let f_locked = tire.longitudinal_force(1.0, normal_force);
        assert!(f_locked < f_opt);
    }

    #[test]
    fn test_lateral_force() {
        let tire = TireParameters::passenger_car();
        let normal_force = 5000.0; // N

        // Zero slip angle should give zero force
        let f0 = tire.lateral_force(0.0, normal_force);
        assert_relative_eq!(f0, 0.0, epsilon = 1.0);

        // Small slip angle should give force
        let f_small = tire.lateral_force(0.1, normal_force);
        assert!(f_small.abs() > 100.0);
    }

    #[test]
    fn test_road_conditions() {
        assert_eq!(RoadCondition::DryAsphalt.friction_factor(), 1.0);
        assert_eq!(RoadCondition::WetAsphalt.friction_factor(), 0.7);
        assert_eq!(RoadCondition::Ice.friction_factor(), 0.2);
    }
}
