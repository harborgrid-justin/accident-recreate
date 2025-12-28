/**
 * Vehicle Renderer for 2D Canvas
 * AccuScene Enterprise - Accident Recreation Platform
 */

import { VehicleType, VehicleDimensions } from './VehicleTypes';
import { DamageZone, DamageSeverity, VehicleDamage } from './DamageModel';

export interface VehicleRenderOptions {
  x: number; // Center X position in pixels
  y: number; // Center Y position in pixels
  heading: number; // Heading in degrees (0 = North, 90 = East)
  color: string; // Vehicle color (hex or CSS color)
  showDamage: boolean; // Show damage overlay
  showDirection: boolean; // Show direction indicator
  showLabel: boolean; // Show vehicle label
  label?: string; // Custom label text
  scale: number; // Scale factor (pixels per foot)
  opacity: number; // Opacity 0-1
  selected: boolean; // Is vehicle selected
}

export const DEFAULT_RENDER_OPTIONS: VehicleRenderOptions = {
  x: 0,
  y: 0,
  heading: 0,
  color: '#4A90E2',
  showDamage: true,
  showDirection: true,
  showLabel: false,
  scale: 10,
  opacity: 1.0,
  selected: false,
};

/**
 * Render vehicle on 2D canvas
 */
export function renderVehicle(
  ctx: CanvasRenderingContext2D,
  type: VehicleType,
  dimensions: VehicleDimensions,
  options: Partial<VehicleRenderOptions> = {}
): void {
  const opts = { ...DEFAULT_RENDER_OPTIONS, ...options };

  ctx.save();

  // Translate to vehicle position
  ctx.translate(opts.x, opts.y);

  // Rotate to vehicle heading (canvas rotation is clockwise, degrees to radians)
  const headingRad = (opts.heading * Math.PI) / 180;
  ctx.rotate(headingRad);

  // Draw vehicle body
  drawVehicleBody(ctx, type, dimensions, opts);

  // Draw direction indicator
  if (opts.showDirection) {
    drawDirectionIndicator(ctx, dimensions, opts);
  }

  // Draw selection highlight
  if (opts.selected) {
    drawSelectionHighlight(ctx, dimensions, opts);
  }

  ctx.restore();

  // Draw label (after restore so it's not rotated)
  if (opts.showLabel && opts.label) {
    drawLabel(ctx, opts.x, opts.y, opts.label, dimensions, opts);
  }
}

/**
 * Draw vehicle body based on type
 */
function drawVehicleBody(
  ctx: CanvasRenderingContext2D,
  type: VehicleType,
  dimensions: VehicleDimensions,
  opts: VehicleRenderOptions
): void {
  const width = dimensions.width * opts.scale;
  const length = dimensions.length * opts.scale;

  ctx.globalAlpha = opts.opacity;

  switch (type) {
    case VehicleType.SEDAN:
    case VehicleType.SUV:
    case VehicleType.VAN:
      drawStandardVehicle(ctx, width, length, opts.color);
      break;

    case VehicleType.TRUCK:
      drawTruck(ctx, width, length, opts.color);
      break;

    case VehicleType.MOTORCYCLE:
      drawMotorcycle(ctx, width, length, opts.color);
      break;

    case VehicleType.BICYCLE:
      drawBicycle(ctx, width, length, opts.color);
      break;

    case VehicleType.PEDESTRIAN:
      drawPedestrian(ctx, opts.color);
      break;

    case VehicleType.COMMERCIAL:
      drawCommercialVehicle(ctx, width, length, opts.color);
      break;

    default:
      drawStandardVehicle(ctx, width, length, opts.color);
  }

  ctx.globalAlpha = 1.0;
}

/**
 * Draw standard vehicle (sedan, SUV, van)
 */
function drawStandardVehicle(
  ctx: CanvasRenderingContext2D,
  width: number,
  length: number,
  color: string
): void {
  const halfWidth = width / 2;
  const halfLength = length / 2;

  // Main body
  ctx.fillStyle = color;
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 2;

  ctx.beginPath();
  ctx.roundRect(-halfWidth, -halfLength, width, length, 4);
  ctx.fill();
  ctx.stroke();

  // Windshield
  ctx.fillStyle = 'rgba(135, 206, 235, 0.5)';
  const windshieldY = -halfLength + length * 0.25;
  ctx.fillRect(-halfWidth * 0.6, windshieldY, width * 0.6, length * 0.15);

  // Rear window
  const rearWindowY = halfLength - length * 0.35;
  ctx.fillRect(-halfWidth * 0.6, rearWindowY, width * 0.6, length * 0.1);

  // Side windows
  ctx.fillRect(-halfWidth + 2, -halfLength * 0.3, width * 0.15, length * 0.4);
  ctx.fillRect(halfWidth - 2 - width * 0.15, -halfLength * 0.3, width * 0.15, length * 0.4);
}

/**
 * Draw truck
 */
function drawTruck(
  ctx: CanvasRenderingContext2D,
  width: number,
  length: number,
  color: string
): void {
  const halfWidth = width / 2;
  const halfLength = length / 2;

  // Truck bed
  ctx.fillStyle = color;
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 2;
  ctx.fillRect(-halfWidth, -halfLength * 0.5, width, length * 1.2);
  ctx.strokeRect(-halfWidth, -halfLength * 0.5, width, length * 1.2);

  // Cab
  const cabLength = length * 0.4;
  ctx.fillStyle = color;
  ctx.fillRect(-halfWidth, -halfLength, width, cabLength);
  ctx.strokeRect(-halfWidth, -halfLength, width, cabLength);

  // Windshield
  ctx.fillStyle = 'rgba(135, 206, 235, 0.5)';
  ctx.fillRect(-halfWidth * 0.6, -halfLength + 5, width * 0.6, cabLength * 0.4);
}

/**
 * Draw motorcycle
 */
function drawMotorcycle(
  ctx: CanvasRenderingContext2D,
  width: number,
  length: number,
  color: string
): void {
  const halfWidth = width / 2;
  const halfLength = length / 2;

  // Body
  ctx.fillStyle = color;
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 2;

  ctx.beginPath();
  ctx.ellipse(0, 0, halfWidth, halfLength, 0, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // Front wheel
  ctx.fillStyle = '#333333';
  ctx.beginPath();
  ctx.arc(0, -halfLength * 0.6, width * 0.15, 0, Math.PI * 2);
  ctx.fill();

  // Rear wheel
  ctx.beginPath();
  ctx.arc(0, halfLength * 0.6, width * 0.15, 0, Math.PI * 2);
  ctx.fill();
}

/**
 * Draw bicycle
 */
function drawBicycle(
  ctx: CanvasRenderingContext2D,
  width: number,
  length: number,
  color: string
): void {
  const halfLength = length / 2;

  ctx.strokeStyle = color;
  ctx.lineWidth = 3;

  // Frame
  ctx.beginPath();
  ctx.moveTo(0, -halfLength);
  ctx.lineTo(0, halfLength);
  ctx.stroke();

  // Wheels
  ctx.fillStyle = '#333333';
  ctx.beginPath();
  ctx.arc(0, -halfLength * 0.7, width * 0.3, 0, Math.PI * 2);
  ctx.fill();

  ctx.beginPath();
  ctx.arc(0, halfLength * 0.7, width * 0.3, 0, Math.PI * 2);
  ctx.fill();
}

/**
 * Draw pedestrian
 */
function drawPedestrian(ctx: CanvasRenderingContext2D, color: string): void {
  ctx.fillStyle = color;
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 2;

  // Head
  ctx.beginPath();
  ctx.arc(0, -8, 6, 0, Math.PI * 2);
  ctx.fill();
  ctx.stroke();

  // Body
  ctx.beginPath();
  ctx.moveTo(0, -2);
  ctx.lineTo(0, 10);
  ctx.stroke();

  // Arms
  ctx.beginPath();
  ctx.moveTo(-8, 2);
  ctx.lineTo(8, 2);
  ctx.stroke();

  // Legs
  ctx.beginPath();
  ctx.moveTo(0, 10);
  ctx.lineTo(-5, 18);
  ctx.moveTo(0, 10);
  ctx.lineTo(5, 18);
  ctx.stroke();
}

/**
 * Draw commercial vehicle
 */
function drawCommercialVehicle(
  ctx: CanvasRenderingContext2D,
  width: number,
  length: number,
  color: string
): void {
  const halfWidth = width / 2;
  const halfLength = length / 2;

  // Box/trailer
  ctx.fillStyle = color;
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 2;
  ctx.fillRect(-halfWidth, -halfLength * 0.7, width, length * 1.5);
  ctx.strokeRect(-halfWidth, -halfLength * 0.7, width, length * 1.5);

  // Cab
  const cabLength = length * 0.3;
  ctx.fillStyle = darkenColor(color, 20);
  ctx.fillRect(-halfWidth, -halfLength, width, cabLength);
  ctx.strokeRect(-halfWidth, -halfLength, width, cabLength);

  // Windshield
  ctx.fillStyle = 'rgba(135, 206, 235, 0.5)';
  ctx.fillRect(-halfWidth * 0.6, -halfLength + 3, width * 0.6, cabLength * 0.5);
}

/**
 * Draw direction indicator
 */
function drawDirectionIndicator(
  ctx: CanvasRenderingContext2D,
  dimensions: VehicleDimensions,
  opts: VehicleRenderOptions
): void {
  const length = dimensions.length * opts.scale;
  const halfLength = length / 2;

  // Arrow pointing forward
  ctx.fillStyle = '#FFFF00';
  ctx.strokeStyle = '#000000';
  ctx.lineWidth = 1;

  const arrowSize = 8;
  const arrowY = -halfLength - 10;

  ctx.beginPath();
  ctx.moveTo(0, arrowY - arrowSize);
  ctx.lineTo(-arrowSize / 2, arrowY);
  ctx.lineTo(arrowSize / 2, arrowY);
  ctx.closePath();
  ctx.fill();
  ctx.stroke();
}

/**
 * Draw selection highlight
 */
function drawSelectionHighlight(
  ctx: CanvasRenderingContext2D,
  dimensions: VehicleDimensions,
  opts: VehicleRenderOptions
): void {
  const width = dimensions.width * opts.scale;
  const length = dimensions.length * opts.scale;
  const halfWidth = width / 2;
  const halfLength = length / 2;

  ctx.strokeStyle = '#FFD700';
  ctx.lineWidth = 3;
  ctx.setLineDash([5, 5]);

  ctx.strokeRect(-halfWidth - 5, -halfLength - 5, width + 10, length + 10);

  ctx.setLineDash([]);
}

/**
 * Draw label
 */
function drawLabel(
  ctx: CanvasRenderingContext2D,
  x: number,
  y: number,
  label: string,
  dimensions: VehicleDimensions,
  opts: VehicleRenderOptions
): void {
  const length = dimensions.length * opts.scale;
  const labelY = y + length / 2 + 20;

  ctx.font = 'bold 12px Arial';
  ctx.textAlign = 'center';
  ctx.textBaseline = 'top';

  // Background
  const metrics = ctx.measureText(label);
  const padding = 4;
  ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
  ctx.fillRect(
    x - metrics.width / 2 - padding,
    labelY - padding,
    metrics.width + padding * 2,
    16 + padding * 2
  );

  // Text
  ctx.fillStyle = '#000000';
  ctx.fillText(label, x, labelY);
}

/**
 * Render damage overlay
 */
export function renderDamageOverlay(
  ctx: CanvasRenderingContext2D,
  dimensions: VehicleDimensions,
  damage: VehicleDamage,
  options: Partial<VehicleRenderOptions> = {}
): void {
  const opts = { ...DEFAULT_RENDER_OPTIONS, ...options };

  ctx.save();
  ctx.translate(opts.x, opts.y);

  const headingRad = (opts.heading * Math.PI) / 180;
  ctx.rotate(headingRad);

  const width = dimensions.width * opts.scale;
  const length = dimensions.length * opts.scale;

  for (const zone of damage.zones) {
    drawDamageZone(ctx, zone.zone, zone.severity, width, length);
  }

  ctx.restore();
}

/**
 * Draw damage for specific zone
 */
function drawDamageZone(
  ctx: CanvasRenderingContext2D,
  zone: DamageZone,
  severity: DamageSeverity,
  width: number,
  length: number
): void {
  const halfWidth = width / 2;
  const halfLength = length / 2;

  const color = getDamageColor(severity);
  ctx.fillStyle = color;
  ctx.globalAlpha = 0.6;

  switch (zone) {
    case DamageZone.FRONT:
      ctx.fillRect(-halfWidth, -halfLength, width, length * 0.25);
      break;

    case DamageZone.FRONT_LEFT:
      ctx.fillRect(-halfWidth, -halfLength, width * 0.5, length * 0.4);
      break;

    case DamageZone.FRONT_RIGHT:
      ctx.fillRect(0, -halfLength, width * 0.5, length * 0.4);
      break;

    case DamageZone.LEFT:
      ctx.fillRect(-halfWidth, -halfLength * 0.3, width * 0.2, length * 0.6);
      break;

    case DamageZone.RIGHT:
      ctx.fillRect(halfWidth - width * 0.2, -halfLength * 0.3, width * 0.2, length * 0.6);
      break;

    case DamageZone.REAR:
      ctx.fillRect(-halfWidth, halfLength - length * 0.25, width, length * 0.25);
      break;

    case DamageZone.REAR_LEFT:
      ctx.fillRect(-halfWidth, halfLength - length * 0.4, width * 0.5, length * 0.4);
      break;

    case DamageZone.REAR_RIGHT:
      ctx.fillRect(0, halfLength - length * 0.4, width * 0.5, length * 0.4);
      break;

    case DamageZone.ROOF:
      ctx.beginPath();
      ctx.ellipse(0, 0, halfWidth * 0.5, halfLength * 0.5, 0, 0, Math.PI * 2);
      ctx.fill();
      break;

    case DamageZone.UNDERCARRIAGE:
      // Draw as dots/pattern
      for (let i = 0; i < 5; i++) {
        ctx.fillRect(-halfWidth + i * (width / 5), -5, 10, 10);
      }
      break;
  }

  ctx.globalAlpha = 1.0;
}

/**
 * Get damage color based on severity
 */
function getDamageColor(severity: DamageSeverity): string {
  const colors: Record<DamageSeverity, string> = {
    [DamageSeverity.NONE]: 'rgba(0, 0, 0, 0)',
    [DamageSeverity.MINOR]: '#FFEB3B',
    [DamageSeverity.MODERATE]: '#FF9800',
    [DamageSeverity.SEVERE]: '#FF5722',
    [DamageSeverity.MAJOR]: '#F44336',
    [DamageSeverity.CATASTROPHIC]: '#9C27B0',
  };
  return colors[severity];
}

/**
 * Darken a color by a percentage
 */
function darkenColor(color: string, percent: number): string {
  // Simple darkening - for hex colors
  if (color.startsWith('#')) {
    const num = parseInt(color.slice(1), 16);
    const r = Math.max(0, ((num >> 16) & 0xff) - percent);
    const g = Math.max(0, ((num >> 8) & 0xff) - percent);
    const b = Math.max(0, (num & 0xff) - percent);
    return `#${((r << 16) | (g << 8) | b).toString(16).padStart(6, '0')}`;
  }
  return color;
}

/**
 * Calculate vehicle bounds for collision detection
 */
export function getVehicleBounds(
  x: number,
  y: number,
  heading: number,
  dimensions: VehicleDimensions,
  scale: number
): { minX: number; minY: number; maxX: number; maxY: number } {
  const width = dimensions.width * scale;
  const length = dimensions.length * scale;
  const halfWidth = width / 2;
  const halfLength = length / 2;

  // Calculate rotated corners
  const headingRad = (heading * Math.PI) / 180;
  const cos = Math.cos(headingRad);
  const sin = Math.sin(headingRad);

  const corners = [
    { x: -halfWidth, y: -halfLength },
    { x: halfWidth, y: -halfLength },
    { x: halfWidth, y: halfLength },
    { x: -halfWidth, y: halfLength },
  ];

  const rotatedCorners = corners.map(corner => ({
    x: x + corner.x * cos - corner.y * sin,
    y: y + corner.x * sin + corner.y * cos,
  }));

  const xs = rotatedCorners.map(c => c.x);
  const ys = rotatedCorners.map(c => c.y);

  return {
    minX: Math.min(...xs),
    minY: Math.min(...ys),
    maxX: Math.max(...xs),
    maxY: Math.max(...ys),
  };
}

/**
 * Check if point is inside vehicle
 */
export function isPointInVehicle(
  pointX: number,
  pointY: number,
  vehicleX: number,
  vehicleY: number,
  heading: number,
  dimensions: VehicleDimensions,
  scale: number
): boolean {
  // Transform point to vehicle's local coordinate system
  const headingRad = (heading * Math.PI) / 180;
  const cos = Math.cos(-headingRad);
  const sin = Math.sin(-headingRad);

  const dx = pointX - vehicleX;
  const dy = pointY - vehicleY;

  const localX = dx * cos - dy * sin;
  const localY = dx * sin + dy * cos;

  const width = dimensions.width * scale;
  const length = dimensions.length * scale;
  const halfWidth = width / 2;
  const halfLength = length / 2;

  return (
    localX >= -halfWidth &&
    localX <= halfWidth &&
    localY >= -halfLength &&
    localY <= halfLength
  );
}
