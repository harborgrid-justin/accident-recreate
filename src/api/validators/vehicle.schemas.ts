/**
 * Vehicle Validation Schemas
 * AccuScene Enterprise Accident Recreation Platform
 */

import { z } from 'zod';
import { VehicleType } from '../../vehicles/VehicleTypes';
import { uuidSchema } from '../middleware/validator';

/**
 * Create vehicle schema
 */
export const createVehicleSchema = z.object({
  accidentId: uuidSchema,
  type: z.nativeEnum(VehicleType),
  make: z.string().min(1).max(50).optional(),
  model: z.string().min(1).max(50).optional(),
  year: z.coerce.number().int().min(1900).max(new Date().getFullYear() + 1).optional(),
  color: z.string().min(1).max(30).optional(),
  licensePlate: z.string().min(1).max(20).optional(),
  vin: z.string().length(17).optional(),
  dimensions: z.object({
    length: z.number().positive(),
    width: z.number().positive(),
    height: z.number().positive(),
    wheelbase: z.number().positive(),
    groundClearance: z.number().positive(),
  }).optional(),
  weight: z.object({
    curb: z.number().positive(),
    gross: z.number().positive(),
    distribution: z.object({
      front: z.number().min(0).max(100),
      rear: z.number().min(0).max(100),
    }),
  }).optional(),
  position: z.object({
    x: z.number(),
    y: z.number(),
    angle: z.number().min(0).max(360).default(0),
  }),
  velocity: z.object({
    x: z.number().default(0),
    y: z.number().default(0),
    speed: z.number().min(0).default(0),
  }).optional(),
  driverName: z.string().min(1).max(100).optional(),
  occupants: z.coerce.number().int().min(0).default(1),
  damageDescription: z.string().optional(),
});

/**
 * Update vehicle schema
 */
export const updateVehicleSchema = z.object({
  type: z.nativeEnum(VehicleType).optional(),
  make: z.string().min(1).max(50).optional(),
  model: z.string().min(1).max(50).optional(),
  year: z.coerce.number().int().min(1900).max(new Date().getFullYear() + 1).optional(),
  color: z.string().min(1).max(30).optional(),
  licensePlate: z.string().min(1).max(20).optional(),
  vin: z.string().length(17).optional(),
  dimensions: z.object({
    length: z.number().positive(),
    width: z.number().positive(),
    height: z.number().positive(),
    wheelbase: z.number().positive(),
    groundClearance: z.number().positive(),
  }).optional(),
  weight: z.object({
    curb: z.number().positive(),
    gross: z.number().positive(),
    distribution: z.object({
      front: z.number().min(0).max(100),
      rear: z.number().min(0).max(100),
    }),
  }).optional(),
  position: z.object({
    x: z.number(),
    y: z.number(),
    angle: z.number().min(0).max(360),
  }).optional(),
  velocity: z.object({
    x: z.number(),
    y: z.number(),
    speed: z.number().min(0),
  }).optional(),
  driverName: z.string().min(1).max(100).optional(),
  occupants: z.coerce.number().int().min(0).optional(),
  damageDescription: z.string().optional(),
});

/**
 * Vehicle physics simulation schema
 */
export const vehiclePhysicsSchema = z.object({
  initialVelocity: z.object({
    x: z.number(),
    y: z.number(),
    speed: z.number().min(0),
  }),
  finalVelocity: z.object({
    x: z.number(),
    y: z.number(),
    speed: z.number().min(0),
  }).optional(),
  mass: z.number().positive(),
  coefficientOfFriction: z.number().min(0).max(1).default(0.7),
  brakingForce: z.number().min(0).optional(),
  timeStep: z.number().positive().default(0.016), // ~60fps
  duration: z.number().positive().default(5), // seconds
});

/**
 * Vehicle search schema
 */
export const vehicleSearchSchema = z.object({
  accidentId: uuidSchema.optional(),
  type: z.nativeEnum(VehicleType).optional(),
  q: z.string().optional(),
  page: z.coerce.number().int().min(1).default(1),
  limit: z.coerce.number().int().min(1).max(100).default(20),
});

/**
 * Vehicle ID param schema
 */
export const vehicleIdParamSchema = z.object({
  vehicleId: uuidSchema,
});
