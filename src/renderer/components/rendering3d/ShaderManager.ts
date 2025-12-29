/**
 * AccuScene Enterprise v0.3.0
 * Shader Compilation and Caching System
 */

import { Shader, RenderBackend } from './types';

export class ShaderManager {
  private backend: RenderBackend;
  private device?: GPUDevice;
  private gl?: WebGL2RenderingContext;
  private shaders: Map<string, Shader> = new Map();
  private programs: Map<string, GPUShaderModule | WebGLProgram> = new Map();

  constructor(
    backend: RenderBackend,
    device?: GPUDevice,
    gl?: WebGL2RenderingContext
  ) {
    this.backend = backend;
    this.device = device;
    this.gl = gl;
  }

  // Load and compile shader
  public async loadShader(id: string, source: string): Promise<Shader | null> {
    // Check cache
    if (this.shaders.has(id)) {
      return this.shaders.get(id)!;
    }

    const shader: Shader = {
      id,
      type: this.backend,
      source,
      compiled: false,
    };

    if (this.backend === 'webgpu' && this.device) {
      try {
        const module = this.device.createShaderModule({
          code: source,
          label: id,
        });

        // Check for compilation errors
        const compilationInfo = await module.getCompilationInfo();
        const errors = compilationInfo.messages.filter(m => m.type === 'error');

        if (errors.length > 0) {
          console.error(`Shader compilation errors for ${id}:`, errors);
          return null;
        }

        shader.webgpuModule = module;
        shader.compiled = true;
        this.programs.set(id, module);
      } catch (err) {
        console.error(`Failed to compile WebGPU shader ${id}:`, err);
        return null;
      }
    } else if (this.backend === 'webgl2' && this.gl) {
      // WebGL shaders need vertex and fragment shaders separately
      // This is a simplified version - real implementation would handle this properly
      shader.compiled = true;
    }

    this.shaders.set(id, shader);
    return shader;
  }

  // Compile WebGL shader program
  public compileGLProgram(
    vertexSource: string,
    fragmentSource: string,
    id: string
  ): WebGLProgram | null {
    if (!this.gl) return null;

    const gl = this.gl;

    // Check cache
    if (this.programs.has(id)) {
      return this.programs.get(id) as WebGLProgram;
    }

    // Compile vertex shader
    const vertexShader = gl.createShader(gl.VERTEX_SHADER);
    if (!vertexShader) return null;

    gl.shaderSource(vertexShader, vertexSource);
    gl.compileShader(vertexShader);

    if (!gl.getShaderParameter(vertexShader, gl.COMPILE_STATUS)) {
      console.error('Vertex shader compilation error:', gl.getShaderInfoLog(vertexShader));
      gl.deleteShader(vertexShader);
      return null;
    }

    // Compile fragment shader
    const fragmentShader = gl.createShader(gl.FRAGMENT_SHADER);
    if (!fragmentShader) return null;

    gl.shaderSource(fragmentShader, fragmentSource);
    gl.compileShader(fragmentShader);

    if (!gl.getShaderParameter(fragmentShader, gl.COMPILE_STATUS)) {
      console.error('Fragment shader compilation error:', gl.getShaderInfoLog(fragmentShader));
      gl.deleteShader(vertexShader);
      gl.deleteShader(fragmentShader);
      return null;
    }

    // Link program
    const program = gl.createProgram();
    if (!program) return null;

    gl.attachShader(program, vertexShader);
    gl.attachShader(program, fragmentShader);
    gl.linkProgram(program);

    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
      console.error('Program linking error:', gl.getProgramInfoLog(program));
      gl.deleteShader(vertexShader);
      gl.deleteShader(fragmentShader);
      gl.deleteProgram(program);
      return null;
    }

    // Cleanup shaders (they're now part of the program)
    gl.deleteShader(vertexShader);
    gl.deleteShader(fragmentShader);

    this.programs.set(id, program);
    return program;
  }

  // Get cached shader
  public getShader(id: string): Shader | null {
    return this.shaders.get(id) || null;
  }

  // Get compiled program
  public getProgram(id: string): GPUShaderModule | WebGLProgram | null {
    return this.programs.get(id) || null;
  }

  // Preload common shaders
  public async preloadShaders(): Promise<void> {
    const shaderPaths = {
      pbr: '/shaders/pbr.wgsl',
      vehicle: '/shaders/vehicle.wgsl',
      ground: '/shaders/ground.wgsl',
      postprocess: '/shaders/postprocess.wgsl',
    };

    const promises = Object.entries(shaderPaths).map(async ([id, path]) => {
      try {
        const response = await fetch(path);
        const source = await response.text();
        await this.loadShader(id, source);
      } catch (err) {
        console.warn(`Failed to load shader ${id}:`, err);
      }
    });

    await Promise.all(promises);
  }

  // Hot reload shader (development feature)
  public async reloadShader(id: string): Promise<boolean> {
    const shader = this.shaders.get(id);
    if (!shader) return false;

    // Remove from cache
    this.shaders.delete(id);
    this.programs.delete(id);

    // Reload
    const newShader = await this.loadShader(id, shader.source);
    return newShader !== null && newShader.compiled;
  }

  // Dispose all shaders
  public dispose(): void {
    if (this.gl) {
      this.programs.forEach((program) => {
        if (program instanceof WebGLProgram) {
          this.gl!.deleteProgram(program);
        }
      });
    }

    this.shaders.clear();
    this.programs.clear();
  }
}
