/**
 * AccuScene Enterprise v0.3.0 - Annotation Sync
 *
 * Synchronized annotations and comments across collaboration session
 */

import { EventEmitter } from 'events';
import {
  Annotation,
  AnnotationId,
  AnnotationReply,
  SessionId,
  UserId,
  User
} from './types';

export class AnnotationSync extends EventEmitter {
  private annotations: Map<AnnotationId, Annotation> = new Map();
  private sessionId: SessionId | null = null;
  private currentUser: User | null = null;

  initialize(sessionId: SessionId, user: User): void {
    this.sessionId = sessionId;
    this.currentUser = user;
  }

  async createAnnotation(
    position: { x: number; y: number; z?: number },
    content: string,
    type: Annotation['type'],
    attachedToObjectId?: string
  ): Promise<Annotation> {
    if (!this.currentUser || !this.sessionId) {
      throw new Error('Not initialized');
    }

    const annotation: Annotation = {
      id: this.generateId(),
      sessionId: this.sessionId,
      sceneId: this.sessionId,
      userId: this.currentUser.id,
      position,
      content,
      type,
      attachedToObjectId,
      resolved: false,
      createdAt: Date.now(),
      updatedAt: Date.now(),
      replies: []
    };

    this.annotations.set(annotation.id, annotation);
    this.emit('annotationCreated', annotation);

    return annotation;
  }

  async updateAnnotation(annotationId: AnnotationId, updates: Partial<Annotation>): Promise<void> {
    const annotation = this.annotations.get(annotationId);
    if (!annotation) {
      throw new Error(`Annotation not found: ${annotationId}`);
    }

    Object.assign(annotation, updates, { updatedAt: Date.now() });
    this.annotations.set(annotationId, annotation);
    this.emit('annotationUpdated', annotation);
  }

  async deleteAnnotation(annotationId: AnnotationId): Promise<void> {
    const annotation = this.annotations.get(annotationId);
    if (annotation) {
      this.annotations.delete(annotationId);
      this.emit('annotationDeleted', { annotationId, annotation });
    }
  }

  async addReply(annotationId: AnnotationId, content: string): Promise<void> {
    if (!this.currentUser) {
      throw new Error('No current user');
    }

    const annotation = this.annotations.get(annotationId);
    if (!annotation) {
      throw new Error(`Annotation not found: ${annotationId}`);
    }

    const reply: AnnotationReply = {
      id: this.generateId(),
      userId: this.currentUser.id,
      content,
      timestamp: Date.now()
    };

    if (!annotation.replies) {
      annotation.replies = [];
    }

    annotation.replies.push(reply);
    annotation.updatedAt = Date.now();

    this.emit('replyAdded', { annotation, reply });
  }

  async resolveAnnotation(annotationId: AnnotationId): Promise<void> {
    await this.updateAnnotation(annotationId, { resolved: true });
  }

  async unresolveAnnotation(annotationId: AnnotationId): Promise<void> {
    await this.updateAnnotation(annotationId, { resolved: false });
  }

  getAnnotation(annotationId: AnnotationId): Annotation | null {
    return this.annotations.get(annotationId) || null;
  }

  getAllAnnotations(): Annotation[] {
    return Array.from(this.annotations.values());
  }

  getAnnotationsByObject(objectId: string): Annotation[] {
    return Array.from(this.annotations.values())
      .filter(a => a.attachedToObjectId === objectId);
  }

  getUnresolvedAnnotations(): Annotation[] {
    return Array.from(this.annotations.values())
      .filter(a => !a.resolved);
  }

  private generateId(): string {
    return `ann-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }
}
