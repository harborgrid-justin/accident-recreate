/**
 * AccuScene Enterprise - Operational Transformation
 * v0.2.0
 *
 * Operational transformation for concurrent operation conflict resolution
 */

import { Operation, OperationType } from '../types';

/**
 * Operational Transformation Engine
 */
export class OperationalTransform {
  /**
   * Transform operation A against operation B
   * Returns the transformed version of A that can be applied after B
   */
  static transform(opA: Operation, opB: Operation): Operation {
    // If operations don't conflict, return opA unchanged
    if (!this.doOperationsConflict(opA, opB)) {
      return opA;
    }

    // Transform based on operation types
    if (this.isObjectOperation(opA) && this.isObjectOperation(opB)) {
      return this.transformObjectOperations(opA, opB);
    }

    if (this.isAnnotationOperation(opA) && this.isAnnotationOperation(opB)) {
      return this.transformAnnotationOperations(opA, opB);
    }

    if (this.isMeasurementOperation(opA) && this.isMeasurementOperation(opB)) {
      return this.transformMeasurementOperations(opA, opB);
    }

    // Default: return opA with tracking
    return {
      ...opA,
      transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
    };
  }

  /**
   * Transform multiple operations against each other
   */
  static transformMultiple(ops: Operation[], against: Operation[]): Operation[] {
    const transformed: Operation[] = [];

    for (let op of ops) {
      for (const againstOp of against) {
        op = this.transform(op, againstOp);
      }
      transformed.push(op);
    }

    return transformed;
  }

  /**
   * Check if two operations conflict
   */
  private static doOperationsConflict(opA: Operation, opB: Operation): boolean {
    // Operations from the same client don't conflict
    if (opA.clientId === opB.clientId) {
      return false;
    }

    // Check if they target the same object
    return this.getOperationTarget(opA) === this.getOperationTarget(opB);
  }

  /**
   * Get the target identifier of an operation
   */
  private static getOperationTarget(op: Operation): string {
    const data = op.data as any;

    if (data.objectId) return `object:${data.objectId}`;
    if (data.annotationId) return `annotation:${data.annotationId}`;
    if (data.measurementId) return `measurement:${data.measurementId}`;
    if (data.sceneId) return `scene:${data.sceneId}`;

    return `unknown:${op.id}`;
  }

  /**
   * Check if operation is an object operation
   */
  private static isObjectOperation(op: Operation): boolean {
    return op.type.toString().startsWith('object:');
  }

  /**
   * Check if operation is an annotation operation
   */
  private static isAnnotationOperation(op: Operation): boolean {
    return op.type.toString().startsWith('annotation:');
  }

  /**
   * Check if operation is a measurement operation
   */
  private static isMeasurementOperation(op: Operation): boolean {
    return op.type.toString().startsWith('measurement:');
  }

  /**
   * Transform object operations
   */
  private static transformObjectOperations(opA: Operation, opB: Operation): Operation {
    const dataA = opA.data as any;
    const dataB = opB.data as any;

    // If B deletes the object, A becomes a no-op
    if (opB.type === OperationType.OBJECT_DELETE) {
      return {
        ...opA,
        type: OperationType.CUSTOM,
        data: { ...dataA, deleted: true },
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // If A deletes and B updates, A wins (delete takes precedence)
    if (opA.type === OperationType.OBJECT_DELETE) {
      return {
        ...opA,
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // For updates, merge properties
    if (
      opA.type === OperationType.OBJECT_UPDATE &&
      opB.type === OperationType.OBJECT_UPDATE
    ) {
      return {
        ...opA,
        data: {
          ...dataA,
          properties: {
            ...dataB.properties,
            ...dataA.properties,
          },
        },
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // For move/transform operations, adjust position based on B
    if (
      (opA.type === OperationType.OBJECT_MOVE ||
        opA.type === OperationType.OBJECT_TRANSFORM) &&
      (opB.type === OperationType.OBJECT_MOVE ||
        opB.type === OperationType.OBJECT_TRANSFORM)
    ) {
      // Keep A's intention, but track that it was transformed
      return {
        ...opA,
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    return {
      ...opA,
      transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
    };
  }

  /**
   * Transform annotation operations
   */
  private static transformAnnotationOperations(opA: Operation, opB: Operation): Operation {
    const dataA = opA.data as any;
    const dataB = opB.data as any;

    // If B deletes the annotation, A becomes a no-op
    if (opB.type === OperationType.ANNOTATION_DELETE) {
      return {
        ...opA,
        type: OperationType.CUSTOM,
        data: { ...dataA, deleted: true },
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // If A deletes and B updates, A wins
    if (opA.type === OperationType.ANNOTATION_DELETE) {
      return {
        ...opA,
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // For updates, merge properties
    if (
      opA.type === OperationType.ANNOTATION_UPDATE &&
      opB.type === OperationType.ANNOTATION_UPDATE
    ) {
      return {
        ...opA,
        data: {
          ...dataA,
          text: dataA.text || dataB.text,
          position: dataA.position || dataB.position,
          style: {
            ...dataB.style,
            ...dataA.style,
          },
        },
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    return {
      ...opA,
      transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
    };
  }

  /**
   * Transform measurement operations
   */
  private static transformMeasurementOperations(opA: Operation, opB: Operation): Operation {
    const dataA = opA.data as any;
    const dataB = opB.data as any;

    // If B deletes the measurement, A becomes a no-op
    if (opB.type === OperationType.MEASUREMENT_DELETE) {
      return {
        ...opA,
        type: OperationType.CUSTOM,
        data: { ...dataA, deleted: true },
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // If A deletes and B updates, A wins
    if (opA.type === OperationType.MEASUREMENT_DELETE) {
      return {
        ...opA,
        transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
      };
    }

    // For updates, A's data takes precedence
    return {
      ...opA,
      transformedAgainst: [...(opA.transformedAgainst || []), opB.id],
    };
  }

  /**
   * Check if an operation has been transformed
   */
  static isTransformed(op: Operation): boolean {
    return (op.transformedAgainst?.length || 0) > 0;
  }

  /**
   * Get the list of operations this was transformed against
   */
  static getTransformedAgainst(op: Operation): string[] {
    return op.transformedAgainst || [];
  }
}
