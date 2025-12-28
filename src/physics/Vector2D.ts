/**
 * Vector2D - 2D Vector Mathematics
 * Provides comprehensive vector operations for physics calculations
 */

export class Vector2D {
  constructor(public x: number = 0, public y: number = 0) {}

  /**
   * Create a new vector from this one
   */
  clone(): Vector2D {
    return new Vector2D(this.x, this.y);
  }

  /**
   * Set vector components
   */
  set(x: number, y: number): Vector2D {
    this.x = x;
    this.y = y;
    return this;
  }

  /**
   * Add another vector to this one
   */
  add(v: Vector2D): Vector2D {
    return new Vector2D(this.x + v.x, this.y + v.y);
  }

  /**
   * Add scalar to both components
   */
  addScalar(s: number): Vector2D {
    return new Vector2D(this.x + s, this.y + s);
  }

  /**
   * Subtract another vector from this one
   */
  subtract(v: Vector2D): Vector2D {
    return new Vector2D(this.x - v.x, this.y - v.y);
  }

  /**
   * Multiply by scalar
   */
  multiply(scalar: number): Vector2D {
    return new Vector2D(this.x * scalar, this.y * scalar);
  }

  /**
   * Component-wise multiplication
   */
  multiplyVector(v: Vector2D): Vector2D {
    return new Vector2D(this.x * v.x, this.y * v.y);
  }

  /**
   * Divide by scalar
   */
  divide(scalar: number): Vector2D {
    if (scalar === 0) {
      console.warn('Division by zero in Vector2D');
      return new Vector2D(0, 0);
    }
    return new Vector2D(this.x / scalar, this.y / scalar);
  }

  /**
   * Calculate magnitude (length) of vector
   */
  magnitude(): number {
    return Math.sqrt(this.x * this.x + this.y * this.y);
  }

  /**
   * Calculate squared magnitude (more efficient when comparing lengths)
   */
  magnitudeSquared(): number {
    return this.x * this.x + this.y * this.y;
  }

  /**
   * Normalize vector to unit length
   */
  normalize(): Vector2D {
    const mag = this.magnitude();
    if (mag === 0) {
      return new Vector2D(0, 0);
    }
    return this.divide(mag);
  }

  /**
   * Calculate dot product with another vector
   * Returns: a·b = |a||b|cos(θ)
   */
  dot(v: Vector2D): number {
    return this.x * v.x + this.y * v.y;
  }

  /**
   * Calculate 2D cross product (returns scalar z-component)
   * Returns: a×b = |a||b|sin(θ)
   */
  cross(v: Vector2D): number {
    return this.x * v.y - this.y * v.x;
  }

  /**
   * Calculate distance to another vector
   */
  distanceTo(v: Vector2D): number {
    const dx = this.x - v.x;
    const dy = this.y - v.y;
    return Math.sqrt(dx * dx + dy * dy);
  }

  /**
   * Calculate squared distance (more efficient for comparisons)
   */
  distanceToSquared(v: Vector2D): number {
    const dx = this.x - v.x;
    const dy = this.y - v.y;
    return dx * dx + dy * dy;
  }

  /**
   * Rotate vector by angle (in radians)
   */
  rotate(angle: number): Vector2D {
    const cos = Math.cos(angle);
    const sin = Math.sin(angle);
    return new Vector2D(
      this.x * cos - this.y * sin,
      this.x * sin + this.y * cos
    );
  }

  /**
   * Calculate angle of vector from positive x-axis
   */
  angle(): number {
    return Math.atan2(this.y, this.x);
  }

  /**
   * Calculate angle between this vector and another
   */
  angleTo(v: Vector2D): number {
    const dot = this.dot(v);
    const magProduct = this.magnitude() * v.magnitude();

    if (magProduct === 0) {
      return 0;
    }

    // Clamp to avoid floating point errors with acos
    const cosAngle = Math.max(-1, Math.min(1, dot / magProduct));
    return Math.acos(cosAngle);
  }

  /**
   * Calculate signed angle between this vector and another
   */
  signedAngleTo(v: Vector2D): number {
    const angle = this.angleTo(v);
    const cross = this.cross(v);
    return cross < 0 ? -angle : angle;
  }

  /**
   * Project this vector onto another vector
   */
  projectOnto(v: Vector2D): Vector2D {
    const dotProduct = this.dot(v);
    const magnitudeSquared = v.magnitudeSquared();

    if (magnitudeSquared === 0) {
      return new Vector2D(0, 0);
    }

    const scalar = dotProduct / magnitudeSquared;
    return v.multiply(scalar);
  }

  /**
   * Reflect vector across a normal
   */
  reflect(normal: Vector2D): Vector2D {
    const normalUnit = normal.normalize();
    const dot = this.dot(normalUnit);
    return this.subtract(normalUnit.multiply(2 * dot));
  }

  /**
   * Get perpendicular vector (90 degrees counter-clockwise)
   */
  perpendicular(): Vector2D {
    return new Vector2D(-this.y, this.x);
  }

  /**
   * Negate vector
   */
  negate(): Vector2D {
    return new Vector2D(-this.x, -this.y);
  }

  /**
   * Linear interpolation between this vector and another
   */
  lerp(v: Vector2D, t: number): Vector2D {
    return new Vector2D(
      this.x + (v.x - this.x) * t,
      this.y + (v.y - this.y) * t
    );
  }

  /**
   * Clamp magnitude to max value
   */
  clampMagnitude(maxMagnitude: number): Vector2D {
    const mag = this.magnitude();
    if (mag > maxMagnitude) {
      return this.normalize().multiply(maxMagnitude);
    }
    return this.clone();
  }

  /**
   * Check if vectors are equal within tolerance
   */
  equals(v: Vector2D, tolerance: number = 0.0001): boolean {
    return Math.abs(this.x - v.x) < tolerance &&
           Math.abs(this.y - v.y) < tolerance;
  }

  /**
   * Convert to string for debugging
   */
  toString(): string {
    return `Vector2D(${this.x.toFixed(3)}, ${this.y.toFixed(3)})`;
  }

  /**
   * Convert to array [x, y]
   */
  toArray(): [number, number] {
    return [this.x, this.y];
  }

  /**
   * Convert to object {x, y}
   */
  toObject(): { x: number; y: number } {
    return { x: this.x, y: this.y };
  }

  // Static factory methods

  /**
   * Create zero vector
   */
  static zero(): Vector2D {
    return new Vector2D(0, 0);
  }

  /**
   * Create unit vector pointing right
   */
  static right(): Vector2D {
    return new Vector2D(1, 0);
  }

  /**
   * Create unit vector pointing up
   */
  static up(): Vector2D {
    return new Vector2D(0, 1);
  }

  /**
   * Create unit vector pointing left
   */
  static left(): Vector2D {
    return new Vector2D(-1, 0);
  }

  /**
   * Create unit vector pointing down
   */
  static down(): Vector2D {
    return new Vector2D(0, -1);
  }

  /**
   * Create vector from angle and magnitude
   */
  static fromAngle(angle: number, magnitude: number = 1): Vector2D {
    return new Vector2D(
      Math.cos(angle) * magnitude,
      Math.sin(angle) * magnitude
    );
  }

  /**
   * Create vector from array [x, y]
   */
  static fromArray(arr: [number, number]): Vector2D {
    return new Vector2D(arr[0], arr[1]);
  }

  /**
   * Create vector from object {x, y}
   */
  static fromObject(obj: { x: number; y: number }): Vector2D {
    return new Vector2D(obj.x, obj.y);
  }

  /**
   * Calculate distance between two vectors
   */
  static distance(v1: Vector2D, v2: Vector2D): number {
    return v1.distanceTo(v2);
  }

  /**
   * Linear interpolation between two vectors
   */
  static lerp(v1: Vector2D, v2: Vector2D, t: number): Vector2D {
    return v1.lerp(v2, t);
  }
}
