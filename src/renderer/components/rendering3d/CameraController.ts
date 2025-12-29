/**
 * AccuScene Enterprise v0.3.0
 * Camera Controller with Orbit, Pan, Zoom and Smooth Interpolation
 */

import { Camera, Vec3, CameraControlState } from './types';

export class CameraController {
  private state: CameraControlState;
  private camera: Camera;
  private isDragging = false;
  private isPanning = false;
  private lastMouseX = 0;
  private lastMouseY = 0;
  private targetDistance: number;
  private targetAzimuth: number;
  private targetElevation: number;
  private targetPosition: Vec3;

  constructor(camera: Camera, initialState?: Partial<CameraControlState>) {
    this.camera = camera;
    this.state = {
      distance: 10,
      azimuth: 0,
      elevation: 30,
      target: { x: 0, y: 0, z: 0 },
      damping: 0.1,
      zoomSpeed: 1.0,
      rotateSpeed: 1.0,
      panSpeed: 1.0,
      ...initialState,
    };

    this.targetDistance = this.state.distance;
    this.targetAzimuth = this.state.azimuth;
    this.targetElevation = this.state.elevation;
    this.targetPosition = { ...this.state.target };

    this.updateCamera();
  }

  // Handle mouse down
  public onMouseDown(event: MouseEvent): void {
    if (event.button === 0) { // Left click
      this.isDragging = true;
      this.lastMouseX = event.clientX;
      this.lastMouseY = event.clientY;
    } else if (event.button === 2) { // Right click
      this.isPanning = true;
      this.lastMouseX = event.clientX;
      this.lastMouseY = event.clientY;
    }
  }

  // Handle mouse move
  public onMouseMove(event: MouseEvent): void {
    const deltaX = event.clientX - this.lastMouseX;
    const deltaY = event.clientY - this.lastMouseY;

    if (this.isDragging) {
      // Orbit camera
      this.targetAzimuth -= deltaX * 0.5 * this.state.rotateSpeed;
      this.targetElevation = Math.max(
        -89,
        Math.min(89, this.targetElevation - deltaY * 0.5 * this.state.rotateSpeed)
      );
    } else if (this.isPanning) {
      // Pan camera
      const panSpeed = this.state.distance * 0.001 * this.state.panSpeed;
      const right = this.getCameraRight();
      const up = this.getCameraUp();

      this.targetPosition.x -= right.x * deltaX * panSpeed;
      this.targetPosition.y += up.y * deltaY * panSpeed;
      this.targetPosition.z -= right.z * deltaX * panSpeed;
    }

    this.lastMouseX = event.clientX;
    this.lastMouseY = event.clientY;
  }

  // Handle mouse up
  public onMouseUp(): void {
    this.isDragging = false;
    this.isPanning = false;
  }

  // Handle mouse wheel
  public onWheel(event: WheelEvent): void {
    event.preventDefault();

    const zoomDelta = event.deltaY * 0.01 * this.state.zoomSpeed;
    this.targetDistance = Math.max(0.1, this.targetDistance + zoomDelta);
  }

  // Handle touch events for mobile
  private touchStartX = 0;
  private touchStartY = 0;
  private initialPinchDistance = 0;

  public onTouchStart(event: TouchEvent): void {
    if (event.touches.length === 1) {
      this.isDragging = true;
      this.touchStartX = event.touches[0].clientX;
      this.touchStartY = event.touches[0].clientY;
    } else if (event.touches.length === 2) {
      this.isDragging = false;
      const dx = event.touches[1].clientX - event.touches[0].clientX;
      const dy = event.touches[1].clientY - event.touches[0].clientY;
      this.initialPinchDistance = Math.sqrt(dx * dx + dy * dy);
    }
  }

  public onTouchMove(event: TouchEvent): void {
    event.preventDefault();

    if (event.touches.length === 1 && this.isDragging) {
      const deltaX = event.touches[0].clientX - this.touchStartX;
      const deltaY = event.touches[0].clientY - this.touchStartY;

      this.targetAzimuth -= deltaX * 0.5 * this.state.rotateSpeed;
      this.targetElevation = Math.max(
        -89,
        Math.min(89, this.targetElevation - deltaY * 0.5 * this.state.rotateSpeed)
      );

      this.touchStartX = event.touches[0].clientX;
      this.touchStartY = event.touches[0].clientY;
    } else if (event.touches.length === 2) {
      const dx = event.touches[1].clientX - event.touches[0].clientX;
      const dy = event.touches[1].clientY - event.touches[0].clientY;
      const distance = Math.sqrt(dx * dx + dy * dy);
      const delta = distance - this.initialPinchDistance;

      this.targetDistance = Math.max(0.1, this.targetDistance - delta * 0.01);
      this.initialPinchDistance = distance;
    }
  }

  public onTouchEnd(): void {
    this.isDragging = false;
  }

  // Update camera position with smooth interpolation
  public update(deltaTime: number): void {
    const dampingFactor = 1.0 - Math.pow(1.0 - this.state.damping, deltaTime * 60);

    // Smooth interpolation
    this.state.distance += (this.targetDistance - this.state.distance) * dampingFactor;
    this.state.azimuth += (this.targetAzimuth - this.state.azimuth) * dampingFactor;
    this.state.elevation += (this.targetElevation - this.state.elevation) * dampingFactor;

    this.state.target.x += (this.targetPosition.x - this.state.target.x) * dampingFactor;
    this.state.target.y += (this.targetPosition.y - this.state.target.y) * dampingFactor;
    this.state.target.z += (this.targetPosition.z - this.state.target.z) * dampingFactor;

    this.updateCamera();
  }

  // Update camera based on current state
  private updateCamera(): void {
    const azimuthRad = (this.state.azimuth * Math.PI) / 180;
    const elevationRad = (this.state.elevation * Math.PI) / 180;

    // Calculate camera position
    const x = this.state.distance * Math.cos(elevationRad) * Math.sin(azimuthRad);
    const y = this.state.distance * Math.sin(elevationRad);
    const z = this.state.distance * Math.cos(elevationRad) * Math.cos(azimuthRad);

    this.camera.position = {
      x: this.state.target.x + x,
      y: this.state.target.y + y,
      z: this.state.target.z + z,
    };

    this.camera.target = { ...this.state.target };
  }

  // Get camera right vector
  private getCameraRight(): Vec3 {
    const forward = this.normalize({
      x: this.camera.target.x - this.camera.position.x,
      y: this.camera.target.y - this.camera.position.y,
      z: this.camera.target.z - this.camera.position.z,
    });

    const right = this.cross(forward, this.camera.up);
    return this.normalize(right);
  }

  // Get camera up vector
  private getCameraUp(): Vec3 {
    const forward = this.normalize({
      x: this.camera.target.x - this.camera.position.x,
      y: this.camera.target.y - this.camera.position.y,
      z: this.camera.target.z - this.camera.position.z,
    });

    const right = this.getCameraRight();
    return this.normalize(this.cross(right, forward));
  }

  // Focus on target position
  public focusOn(target: Vec3, distance?: number): void {
    this.targetPosition = { ...target };
    if (distance !== undefined) {
      this.targetDistance = distance;
    }
  }

  // Set camera angles
  public setAngles(azimuth: number, elevation: number): void {
    this.targetAzimuth = azimuth;
    this.targetElevation = Math.max(-89, Math.min(89, elevation));
  }

  // Get current state
  public getState(): CameraControlState {
    return { ...this.state };
  }

  // Set state
  public setState(newState: Partial<CameraControlState>): void {
    this.state = { ...this.state, ...newState };
    this.targetDistance = this.state.distance;
    this.targetAzimuth = this.state.azimuth;
    this.targetElevation = this.state.elevation;
    this.targetPosition = { ...this.state.target };
  }

  // Helper functions
  private normalize(v: Vec3): Vec3 {
    const len = Math.sqrt(v.x * v.x + v.y * v.y + v.z * v.z);
    if (len === 0) return { x: 0, y: 1, z: 0 };
    return { x: v.x / len, y: v.y / len, z: v.z / len };
  }

  private cross(a: Vec3, b: Vec3): Vec3 {
    return {
      x: a.y * b.z - a.z * b.y,
      y: a.z * b.x - a.x * b.z,
      z: a.x * b.y - a.y * b.x,
    };
  }
}
