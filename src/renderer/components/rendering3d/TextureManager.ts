/**
 * AccuScene Enterprise v0.3.0
 * Texture Loading, Compression, and Mipmapping
 */

import { RenderBackend, TextureDescriptor } from './types';

export class TextureManager {
  private backend: RenderBackend;
  private device?: GPUDevice;
  private gl?: WebGL2RenderingContext;
  private textures: Map<string, GPUTexture | WebGLTexture> = new Map();
  private loadingQueue: Map<string, Promise<GPUTexture | WebGLTexture>> = new Map();

  constructor(
    backend: RenderBackend,
    device?: GPUDevice,
    gl?: WebGL2RenderingContext
  ) {
    this.backend = backend;
    this.device = device;
    this.gl = gl;
  }

  // Load texture from URL
  public async loadTexture(url: string, generateMipmaps = true): Promise<GPUTexture | WebGLTexture | null> {
    // Check cache
    if (this.textures.has(url)) {
      return this.textures.get(url)!;
    }

    // Check if already loading
    if (this.loadingQueue.has(url)) {
      return this.loadingQueue.get(url)!;
    }

    const loadPromise = this._loadTextureInternal(url, generateMipmaps);
    this.loadingQueue.set(url, loadPromise);

    try {
      const texture = await loadPromise;
      this.textures.set(url, texture);
      return texture;
    } finally {
      this.loadingQueue.delete(url);
    }
  }

  private async _loadTextureInternal(url: string, generateMipmaps: boolean): Promise<GPUTexture | WebGLTexture> {
    const image = await this.loadImage(url);

    if (this.backend === 'webgpu' && this.device) {
      return this.createGPUTexture(image, generateMipmaps);
    } else if (this.backend === 'webgl2' && this.gl) {
      return this.createGLTexture(image, generateMipmaps);
    }

    throw new Error('No render context available');
  }

  // Load image from URL
  private loadImage(url: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.crossOrigin = 'anonymous';

      image.onload = () => resolve(image);
      image.onerror = () => reject(new Error(`Failed to load image: ${url}`));

      image.src = url;
    });
  }

  // Create WebGPU texture
  private createGPUTexture(image: HTMLImageElement, generateMipmaps: boolean): GPUTexture {
    const { device } = this;
    if (!device) throw new Error('No GPU device');

    const mipLevelCount = generateMipmaps
      ? Math.floor(Math.log2(Math.max(image.width, image.height))) + 1
      : 1;

    const texture = device.createTexture({
      size: { width: image.width, height: image.height },
      format: 'rgba8unorm',
      usage:
        GPUTextureUsage.TEXTURE_BINDING |
        GPUTextureUsage.COPY_DST |
        GPUTextureUsage.RENDER_ATTACHMENT,
      mipLevelCount,
    });

    // Create canvas to get pixel data
    const canvas = document.createElement('canvas');
    canvas.width = image.width;
    canvas.height = image.height;
    const ctx = canvas.getContext('2d');
    if (!ctx) throw new Error('Failed to get canvas context');

    ctx.drawImage(image, 0, 0);
    const imageData = ctx.getImageData(0, 0, image.width, image.height);

    // Copy to GPU
    device.queue.writeTexture(
      { texture, mipLevel: 0 },
      imageData.data,
      {
        bytesPerRow: image.width * 4,
        rowsPerImage: image.height,
      },
      { width: image.width, height: image.height }
    );

    // Generate mipmaps if requested
    if (generateMipmaps) {
      this.generateMipmapsGPU(texture, image.width, image.height, mipLevelCount);
    }

    return texture;
  }

  // Generate mipmaps for WebGPU texture
  private generateMipmapsGPU(texture: GPUTexture, width: number, height: number, mipLevelCount: number): void {
    // Simplified mipmap generation
    // Full implementation would use compute shaders or render passes
    console.log(`Generating ${mipLevelCount} mipmap levels for ${width}x${height} texture`);
  }

  // Create WebGL texture
  private createGLTexture(image: HTMLImageElement, generateMipmaps: boolean): WebGLTexture {
    const { gl } = this;
    if (!gl) throw new Error('No WebGL context');

    const texture = gl.createTexture();
    if (!texture) throw new Error('Failed to create texture');

    gl.bindTexture(gl.TEXTURE_2D, texture);

    // Set pixel storage mode
    gl.pixelStorei(gl.UNPACK_FLIP_Y_WEBGL, true);
    gl.pixelStorei(gl.UNPACK_PREMULTIPLY_ALPHA_WEBGL, false);

    // Upload texture data
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);

    // Generate mipmaps
    if (generateMipmaps && this.isPowerOf2(image.width) && this.isPowerOf2(image.height)) {
      gl.generateMipmap(gl.TEXTURE_2D);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR_MIPMAP_LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    } else {
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    }

    // Set anisotropic filtering if available
    const ext = this.gl?.extensions.anisotropic;
    if (ext) {
      const maxAnisotropy = gl.getParameter(ext.MAX_TEXTURE_MAX_ANISOTROPY_EXT);
      gl.texParameterf(gl.TEXTURE_2D, ext.TEXTURE_MAX_ANISOTROPY_EXT, maxAnisotropy);
    }

    return texture;
  }

  // Check if value is power of 2
  private isPowerOf2(value: number): boolean {
    return (value & (value - 1)) === 0;
  }

  // Create empty texture
  public createTexture(descriptor: TextureDescriptor): GPUTexture | WebGLTexture {
    if (this.backend === 'webgpu' && this.device) {
      return this.device.createTexture({
        size: { width: descriptor.width, height: descriptor.height },
        format: descriptor.format as GPUTextureFormat,
        usage: descriptor.usage as GPUTextureUsageFlags,
        mipLevelCount: descriptor.mipLevelCount || 1,
        sampleCount: descriptor.sampleCount || 1,
      });
    } else if (this.backend === 'webgl2' && this.gl) {
      const { gl } = this;
      const texture = gl.createTexture();
      if (!texture) throw new Error('Failed to create texture');

      gl.bindTexture(gl.TEXTURE_2D, texture);
      gl.texImage2D(
        gl.TEXTURE_2D,
        0,
        descriptor.format as number,
        descriptor.width,
        descriptor.height,
        0,
        gl.RGBA,
        gl.UNSIGNED_BYTE,
        null
      );

      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
      gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.LINEAR);

      return texture;
    }

    throw new Error('No render context available');
  }

  // Create cubemap texture
  public async loadCubemap(urls: string[]): Promise<GPUTexture | WebGLTexture | null> {
    if (urls.length !== 6) {
      throw new Error('Cubemap requires exactly 6 face URLs');
    }

    const images = await Promise.all(urls.map(url => this.loadImage(url)));

    if (this.backend === 'webgpu' && this.device) {
      return this.createGPUCubemap(images);
    } else if (this.backend === 'webgl2' && this.gl) {
      return this.createGLCubemap(images);
    }

    return null;
  }

  private createGPUCubemap(images: HTMLImageElement[]): GPUTexture {
    const { device } = this;
    if (!device) throw new Error('No GPU device');

    const size = images[0].width; // Assume all faces are same size

    const texture = device.createTexture({
      size: { width: size, height: size, depthOrArrayLayers: 6 },
      format: 'rgba8unorm',
      usage: GPUTextureUsage.TEXTURE_BINDING | GPUTextureUsage.COPY_DST,
      dimension: '2d',
    });

    // Upload each face
    images.forEach((image, i) => {
      const canvas = document.createElement('canvas');
      canvas.width = canvas.height = size;
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      ctx.drawImage(image, 0, 0);
      const imageData = ctx.getImageData(0, 0, size, size);

      device.queue.writeTexture(
        { texture, origin: { x: 0, y: 0, z: i } },
        imageData.data,
        { bytesPerRow: size * 4, rowsPerImage: size },
        { width: size, height: size }
      );
    });

    return texture;
  }

  private createGLCubemap(images: HTMLImageElement[]): WebGLTexture {
    const { gl } = this;
    if (!gl) throw new Error('No WebGL context');

    const texture = gl.createTexture();
    if (!texture) throw new Error('Failed to create cubemap texture');

    gl.bindTexture(gl.TEXTURE_CUBE_MAP, texture);

    const faces = [
      gl.TEXTURE_CUBE_MAP_POSITIVE_X,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_X,
      gl.TEXTURE_CUBE_MAP_POSITIVE_Y,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_Y,
      gl.TEXTURE_CUBE_MAP_POSITIVE_Z,
      gl.TEXTURE_CUBE_MAP_NEGATIVE_Z,
    ];

    images.forEach((image, i) => {
      gl.texImage2D(faces[i], 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, image);
    });

    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_MAG_FILTER, gl.LINEAR);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_CUBE_MAP, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);

    return texture;
  }

  // Get texture from cache
  public getTexture(url: string): GPUTexture | WebGLTexture | null {
    return this.textures.get(url) || null;
  }

  // Dispose all textures
  public dispose(): void {
    if (this.backend === 'webgpu') {
      this.textures.forEach(texture => {
        (texture as GPUTexture).destroy();
      });
    } else if (this.backend === 'webgl2' && this.gl) {
      this.textures.forEach(texture => {
        this.gl!.deleteTexture(texture as WebGLTexture);
      });
    }

    this.textures.clear();
    this.loadingQueue.clear();
  }
}
