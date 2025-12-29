//! Vector mathematics for physics calculations
//!
//! This module provides 2D and 3D vector types with complete
//! mathematical operations for physics simulations.

use crate::error::{AccuSceneError, Result};
use crate::traits::{MemoryFootprint, Serializable, Validatable};
use serde::{Deserialize, Serialize};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

/// 2D Vector for position, velocity, and acceleration
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector2D {
    /// X component
    pub x: f64,
    /// Y component
    pub y: f64,
}

impl Vector2D {
    /// Create a new 2D vector
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Create a zero vector
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }

    /// Create a unit vector in the X direction
    pub fn unit_x() -> Self {
        Self { x: 1.0, y: 0.0 }
    }

    /// Create a unit vector in the Y direction
    pub fn unit_y() -> Self {
        Self { x: 0.0, y: 1.0 }
    }

    /// Create a vector from polar coordinates (magnitude and angle in radians)
    pub fn from_polar(magnitude: f64, angle: f64) -> Self {
        Self {
            x: magnitude * angle.cos(),
            y: magnitude * angle.sin(),
        }
    }

    /// Calculate the magnitude (length) of the vector
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }

    /// Calculate the squared magnitude (avoiding sqrt for performance)
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    /// Calculate the angle in radians
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }

    /// Normalize the vector to unit length
    pub fn normalize(&self) -> Result<Self> {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Err(AccuSceneError::math("Cannot normalize zero vector"));
        }
        Ok(Self {
            x: self.x / mag,
            y: self.y / mag,
        })
    }

    /// Normalize the vector, returning zero if magnitude is zero
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or_else(|_| Self::zero())
    }

    /// Calculate dot product with another vector
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y
    }

    /// Calculate cross product magnitude (scalar in 2D)
    pub fn cross(&self, other: &Self) -> f64 {
        self.x * other.y - self.y * other.x
    }

    /// Calculate distance to another vector
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    /// Calculate squared distance to another vector
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (*self - *other).magnitude_squared()
    }

    /// Linearly interpolate to another vector
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
        }
    }

    /// Rotate the vector by an angle in radians
    pub fn rotate(&self, angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            x: self.x * cos - self.y * sin,
            y: self.x * sin + self.y * cos,
        }
    }

    /// Project this vector onto another vector
    pub fn project(&self, other: &Self) -> Result<Self> {
        let mag_sq = other.magnitude_squared();
        if mag_sq == 0.0 {
            return Err(AccuSceneError::math("Cannot project onto zero vector"));
        }
        let scalar = self.dot(other) / mag_sq;
        Ok(*other * scalar)
    }

    /// Reflect this vector across a normal vector
    pub fn reflect(&self, normal: &Self) -> Result<Self> {
        let n = normal.normalize()?;
        Ok(*self - n * (2.0 * self.dot(&n)))
    }

    /// Check if vector is approximately zero
    pub fn is_zero(&self, epsilon: f64) -> bool {
        self.x.abs() < epsilon && self.y.abs() < epsilon
    }

    /// Clamp vector components to a range
    pub fn clamp(&self, min: f64, max: f64) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
        }
    }

    /// Get perpendicular vector (rotated 90 degrees counter-clockwise)
    pub fn perpendicular(&self) -> Self {
        Self {
            x: -self.y,
            y: self.x,
        }
    }
}

// Arithmetic operations for Vector2D
impl Add for Vector2D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl AddAssign for Vector2D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
    }
}

impl Sub for Vector2D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}

impl SubAssign for Vector2D {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
    }
}

impl Mul<f64> for Vector2D {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
        }
    }
}

impl MulAssign<f64> for Vector2D {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
    }
}

impl Div<f64> for Vector2D {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
        }
    }
}

impl DivAssign<f64> for Vector2D {
    fn div_assign(&mut self, scalar: f64) {
        self.x /= scalar;
        self.y /= scalar;
    }
}

impl Neg for Vector2D {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl Validatable for Vector2D {
    fn validate(&self) -> Result<()> {
        if !self.x.is_finite() || !self.y.is_finite() {
            return Err(AccuSceneError::validation(
                "Vector components must be finite",
            ));
        }
        Ok(())
    }
}

impl Serializable for Vector2D {}

impl MemoryFootprint for Vector2D {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

/// 3D Vector for advanced physics calculations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Vector3D {
    /// X component
    pub x: f64,
    /// Y component
    pub y: f64,
    /// Z component
    pub z: f64,
}

impl Vector3D {
    /// Create a new 3D vector
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    /// Create a zero vector
    pub fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Create unit vectors
    pub fn unit_x() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /// Create a unit vector pointing in the Y direction (0, 1, 0)
    pub fn unit_y() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    /// Create a unit vector pointing in the Z direction (0, 0, 1)
    pub fn unit_z() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 1.0,
        }
    }

    /// Create from a 2D vector (z = 0)
    pub fn from_2d(v: Vector2D) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: 0.0,
        }
    }

    /// Convert to 2D vector (dropping z component)
    pub fn to_2d(&self) -> Vector2D {
        Vector2D::new(self.x, self.y)
    }

    /// Calculate magnitude
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    /// Calculate squared magnitude
    pub fn magnitude_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    /// Normalize to unit length
    pub fn normalize(&self) -> Result<Self> {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Err(AccuSceneError::math("Cannot normalize zero vector"));
        }
        Ok(Self {
            x: self.x / mag,
            y: self.y / mag,
            z: self.z / mag,
        })
    }

    /// Normalize or return zero
    pub fn normalize_or_zero(&self) -> Self {
        self.normalize().unwrap_or_else(|_| Self::zero())
    }

    /// Dot product
    pub fn dot(&self, other: &Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Cross product
    pub fn cross(&self, other: &Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    /// Distance to another vector
    pub fn distance(&self, other: &Self) -> f64 {
        (*self - *other).magnitude()
    }

    /// Squared distance to another vector
    pub fn distance_squared(&self, other: &Self) -> f64 {
        (*self - *other).magnitude_squared()
    }

    /// Linear interpolation
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    /// Project onto another vector
    pub fn project(&self, other: &Self) -> Result<Self> {
        let mag_sq = other.magnitude_squared();
        if mag_sq == 0.0 {
            return Err(AccuSceneError::math("Cannot project onto zero vector"));
        }
        let scalar = self.dot(other) / mag_sq;
        Ok(*other * scalar)
    }

    /// Reflect across a normal
    pub fn reflect(&self, normal: &Self) -> Result<Self> {
        let n = normal.normalize()?;
        Ok(*self - n * (2.0 * self.dot(&n)))
    }

    /// Check if approximately zero
    pub fn is_zero(&self, epsilon: f64) -> bool {
        self.x.abs() < epsilon && self.y.abs() < epsilon && self.z.abs() < epsilon
    }

    /// Clamp components
    pub fn clamp(&self, min: f64, max: f64) -> Self {
        Self {
            x: self.x.clamp(min, max),
            y: self.y.clamp(min, max),
            z: self.z.clamp(min, max),
        }
    }
}

// Arithmetic operations for Vector3D
impl Add for Vector3D {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vector3D {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vector3D {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vector3D {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl Mul<f64> for Vector3D {
    type Output = Self;
    fn mul(self, scalar: f64) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl MulAssign<f64> for Vector3D {
    fn mul_assign(&mut self, scalar: f64) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Div<f64> for Vector3D {
    type Output = Self;
    fn div(self, scalar: f64) -> Self {
        Self {
            x: self.x / scalar,
            y: self.y / scalar,
            z: self.z / scalar,
        }
    }
}

impl DivAssign<f64> for Vector3D {
    fn div_assign(&mut self, scalar: f64) {
        self.x /= scalar;
        self.y /= scalar;
        self.z /= scalar;
    }
}

impl Neg for Vector3D {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Validatable for Vector3D {
    fn validate(&self) -> Result<()> {
        if !self.x.is_finite() || !self.y.is_finite() || !self.z.is_finite() {
            return Err(AccuSceneError::validation(
                "Vector components must be finite",
            ));
        }
        Ok(())
    }
}

impl Serializable for Vector3D {}

impl MemoryFootprint for Vector3D {
    fn memory_footprint(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector2d_basic() {
        let v = Vector2D::new(3.0, 4.0);
        assert_eq!(v.magnitude(), 5.0);
        assert_eq!(v.magnitude_squared(), 25.0);
    }

    #[test]
    fn test_vector2d_normalize() {
        let v = Vector2D::new(3.0, 4.0);
        let n = v.normalize().unwrap();
        assert!((n.magnitude() - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_vector2d_operations() {
        let v1 = Vector2D::new(1.0, 2.0);
        let v2 = Vector2D::new(3.0, 4.0);

        let sum = v1 + v2;
        assert_eq!(sum, Vector2D::new(4.0, 6.0));

        let diff = v2 - v1;
        assert_eq!(diff, Vector2D::new(2.0, 2.0));

        let scaled = v1 * 2.0;
        assert_eq!(scaled, Vector2D::new(2.0, 4.0));
    }

    #[test]
    fn test_vector2d_dot_cross() {
        let v1 = Vector2D::new(1.0, 0.0);
        let v2 = Vector2D::new(0.0, 1.0);

        assert_eq!(v1.dot(&v2), 0.0);
        assert_eq!(v1.cross(&v2), 1.0);
    }

    #[test]
    fn test_vector3d_basic() {
        let v = Vector3D::new(1.0, 2.0, 2.0);
        assert_eq!(v.magnitude(), 3.0);
    }

    #[test]
    fn test_vector3d_cross() {
        let v1 = Vector3D::unit_x();
        let v2 = Vector3D::unit_y();
        let cross = v1.cross(&v2);
        assert_eq!(cross, Vector3D::unit_z());
    }

    #[test]
    fn test_vector_serialization() {
        let v = Vector2D::new(1.5, 2.5);
        let json = v.to_json().unwrap();
        let deserialized = Vector2D::from_json(&json).unwrap();
        assert_eq!(v, deserialized);
    }
}
