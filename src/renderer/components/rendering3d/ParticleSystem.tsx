/**
 * AccuScene Enterprise v0.3.0
 * GPU Particle System (Debris, Glass, Smoke)
 */

import React, { useEffect, useRef } from 'react';
import { Particle, ParticleEmitter, Vec3, WebGPUContext, WebGLContext } from './types';

interface ParticleSystemProps {
  emitters: ParticleEmitter[];
  gpuContext?: WebGPUContext;
  glContext?: WebGLContext;
  maxParticles?: number;
}

export const ParticleSystem: React.FC<ParticleSystemProps> = ({
  emitters,
  gpuContext,
  glContext,
  maxParticles = 10000,
}) => {
  const particlesRef = useRef<Particle[]>([]);
  const lastUpdateRef = useRef(performance.now());

  useEffect(() => {
    let animationId: number;

    const update = () => {
      const now = performance.now();
      const deltaTime = (now - lastUpdateRef.current) / 1000;
      lastUpdateRef.current = now;

      updateParticles(deltaTime);
      animationId = requestAnimationFrame(update);
    };

    animationId = requestAnimationFrame(update);

    return () => {
      cancelAnimationFrame(animationId);
    };
  }, [emitters]);

  const updateParticles = (deltaTime: number) => {
    // Remove dead particles
    particlesRef.current = particlesRef.current.filter(
      particle => particle.life < particle.maxLife
    );

    // Emit new particles
    emitters.forEach(emitter => {
      const particlesToEmit = Math.floor(emitter.rate * deltaTime);

      for (let i = 0; i < particlesToEmit; i++) {
        if (particlesRef.current.length >= maxParticles) break;

        const particle = createParticle(emitter);
        particlesRef.current.push(particle);
      }
    });

    // Update existing particles
    particlesRef.current.forEach(particle => {
      particle.life += deltaTime;

      // Apply acceleration (gravity)
      particle.velocity.x += particle.acceleration.x * deltaTime;
      particle.velocity.y += particle.acceleration.y * deltaTime;
      particle.velocity.z += particle.acceleration.z * deltaTime;

      // Update position
      particle.position.x += particle.velocity.x * deltaTime;
      particle.position.y += particle.velocity.y * deltaTime;
      particle.position.z += particle.velocity.z * deltaTime;

      // Fade out over lifetime
      const lifeRatio = particle.life / particle.maxLife;
      particle.color.w = Math.max(0, 1 - lifeRatio);
    });

    // Render particles
    if (gpuContext) {
      renderParticlesGPU(particlesRef.current, gpuContext);
    } else if (glContext) {
      renderParticlesGL(particlesRef.current, glContext);
    }
  };

  const createParticle = (emitter: ParticleEmitter): Particle => {
    const randomVelocity = {
      x: emitter.initialVelocity.x + (Math.random() - 0.5) * emitter.velocityVariation.x,
      y: emitter.initialVelocity.y + (Math.random() - 0.5) * emitter.velocityVariation.y,
      z: emitter.initialVelocity.z + (Math.random() - 0.5) * emitter.velocityVariation.z,
    };

    const size = emitter.size + (Math.random() - 0.5) * emitter.sizeVariation;

    const color = {
      x: Math.max(0, Math.min(1, emitter.color.x + (Math.random() - 0.5) * emitter.colorVariation.x)),
      y: Math.max(0, Math.min(1, emitter.color.y + (Math.random() - 0.5) * emitter.colorVariation.y)),
      z: Math.max(0, Math.min(1, emitter.color.z + (Math.random() - 0.5) * emitter.colorVariation.z)),
      w: emitter.color.w,
    };

    return {
      position: { ...emitter.position },
      velocity: randomVelocity,
      acceleration: emitter.gravity,
      color,
      size,
      life: 0,
      maxLife: emitter.particleLife,
    };
  };

  const renderParticlesGPU = (particles: Particle[], context: WebGPUContext) => {
    // WebGPU particle rendering
    // Full implementation would create compute shader for particle update
    // and use instanced rendering for efficient drawing
  };

  const renderParticlesGL = (particles: Particle[], context: WebGLContext) => {
    // WebGL particle rendering using point sprites or instanced quads
  };

  return null; // This is a system component, no visual output
};

// Particle emitter presets
export const ParticlePresets = {
  debris: (position: Vec3): ParticleEmitter => ({
    id: `debris-${Date.now()}`,
    position,
    rate: 100,
    maxParticles: 500,
    particleLife: 2.0,
    initialVelocity: { x: 0, y: 5, z: 0 },
    velocityVariation: { x: 3, y: 2, z: 3 },
    gravity: { x: 0, y: -9.8, z: 0 },
    size: 0.1,
    sizeVariation: 0.05,
    color: { x: 0.3, y: 0.3, z: 0.3, w: 1.0 },
    colorVariation: { x: 0.1, y: 0.1, z: 0.1, w: 0 },
  }),

  glass: (position: Vec3): ParticleEmitter => ({
    id: `glass-${Date.now()}`,
    position,
    rate: 200,
    maxParticles: 1000,
    particleLife: 1.5,
    initialVelocity: { x: 0, y: 3, z: 0 },
    velocityVariation: { x: 4, y: 3, z: 4 },
    gravity: { x: 0, y: -9.8, z: 0 },
    size: 0.05,
    sizeVariation: 0.03,
    color: { x: 0.9, y: 0.95, z: 1.0, w: 0.8 },
    colorVariation: { x: 0.05, y: 0.05, z: 0.05, w: 0.2 },
  }),

  smoke: (position: Vec3): ParticleEmitter => ({
    id: `smoke-${Date.now()}`,
    position,
    rate: 50,
    maxParticles: 300,
    particleLife: 3.0,
    initialVelocity: { x: 0, y: 1, z: 0 },
    velocityVariation: { x: 0.5, y: 0.5, z: 0.5 },
    gravity: { x: 0, y: 0.5, z: 0 }, // Buoyancy
    size: 0.5,
    sizeVariation: 0.2,
    color: { x: 0.2, y: 0.2, z: 0.2, w: 0.6 },
    colorVariation: { x: 0.1, y: 0.1, z: 0.1, w: 0.2 },
  }),

  sparks: (position: Vec3): ParticleEmitter => ({
    id: `sparks-${Date.now()}`,
    position,
    rate: 150,
    maxParticles: 500,
    particleLife: 0.5,
    initialVelocity: { x: 0, y: 4, z: 0 },
    velocityVariation: { x: 5, y: 3, z: 5 },
    gravity: { x: 0, y: -15, z: 0 },
    size: 0.03,
    sizeVariation: 0.02,
    color: { x: 1.0, y: 0.8, z: 0.2, w: 1.0 },
    colorVariation: { x: 0.2, y: 0.2, z: 0.1, w: 0 },
  }),
};
