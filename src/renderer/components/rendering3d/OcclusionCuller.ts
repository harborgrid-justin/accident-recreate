/**
 * AccuScene Enterprise v0.3.0
 * GPU Occlusion Culling with Queries
 */

import { RenderableObject, OcclusionQuery, WebGPUContext, WebGLContext } from './types';

export class OcclusionCuller {
  private queries: Map<string, OcclusionQuery> = new Map();
  private gpuContext?: WebGPUContext;
  private glContext?: WebGLContext;
  private querySet?: GPUQuerySet;
  private framesSinceLastCheck = 0;
  private checkInterval = 5; // Check every N frames

  constructor(gpuContext?: WebGPUContext, glContext?: WebGLContext) {
    this.gpuContext = gpuContext;
    this.glContext = glContext;
    this.initialize();
  }

  private initialize(): void {
    if (this.gpuContext) {
      this.initializeGPU();
    } else if (this.glContext) {
      this.initializeGL();
    }
  }

  private initializeGPU(): void {
    if (!this.gpuContext) return;

    const { device } = this.gpuContext;

    this.querySet = device.createQuerySet({
      type: 'occlusion',
      count: 256, // Max queries
    });
  }

  private initializeGL(): void {
    // WebGL occlusion queries initialized per-object
  }

  public registerObject(object: RenderableObject): void {
    if (this.glContext) {
      const { gl } = this.glContext;
      const query = gl.createQuery();

      if (query) {
        this.queries.set(object.id, {
          id: object.id,
          object,
          query,
          visible: true,
          lastChecked: 0,
        });
      }
    } else {
      // GPU queries managed differently
      this.queries.set(object.id, {
        id: object.id,
        object,
        visible: true,
        lastChecked: 0,
      });
    }
  }

  public unregisterObject(id: string): void {
    const query = this.queries.get(id);
    if (query && this.glContext) {
      this.glContext.gl.deleteQuery(query.query as WebGLQuery);
    }
    this.queries.delete(id);
  }

  public cullObjects(objects: RenderableObject[]): RenderableObject[] {
    this.framesSinceLastCheck++;

    if (this.framesSinceLastCheck >= this.checkInterval) {
      this.framesSinceLastCheck = 0;
      this.performOcclusionTest(objects);
    }

    // Filter based on previous visibility results
    return objects.filter(obj => {
      const query = this.queries.get(obj.id);
      return query ? query.visible : true;
    });
  }

  private performOcclusionTest(objects: RenderableObject[]): void {
    if (this.gpuContext) {
      this.performOcclusionTestGPU(objects);
    } else if (this.glContext) {
      this.performOcclusionTestGL(objects);
    }
  }

  private performOcclusionTestGPU(objects: RenderableObject[]): void {
    // Render bounding boxes with occlusion queries
    // Results available in next frame
  }

  private performOcclusionTestGL(objects: RenderableObject[]): void {
    if (!this.glContext) return;

    const { gl } = this.glContext;

    objects.forEach(obj => {
      const queryData = this.queries.get(obj.id);
      if (!queryData || !queryData.query) return;

      // Check previous query result
      const available = gl.getQueryParameter(queryData.query as WebGLQuery, gl.QUERY_RESULT_AVAILABLE);

      if (available) {
        const result = gl.getQueryParameter(queryData.query as WebGLQuery, gl.QUERY_RESULT);
        queryData.visible = result > 0;
        queryData.lastChecked = Date.now();
      }

      // Start new query
      gl.beginQuery(gl.ANY_SAMPLES_PASSED, queryData.query as WebGLQuery);

      // Render bounding box (simplified)
      this.renderBoundingBox(obj);

      gl.endQuery(gl.ANY_SAMPLES_PASSED);
    });
  }

  private renderBoundingBox(object: RenderableObject): void {
    // Render simplified bounding box for occlusion test
    // This should be fast and only write to depth buffer
  }

  public getVisibilityStats(): { total: number; visible: number; culled: number } {
    let visible = 0;
    let culled = 0;

    this.queries.forEach(query => {
      if (query.visible) {
        visible++;
      } else {
        culled++;
      }
    });

    return {
      total: this.queries.size,
      visible,
      culled,
    };
  }

  public dispose(): void {
    if (this.glContext) {
      this.queries.forEach(query => {
        if (query.query) {
          this.glContext!.gl.deleteQuery(query.query as WebGLQuery);
        }
      });
    }

    this.queries.clear();
  }
}
