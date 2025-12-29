/**
 * AccuScene Enterprise v0.3.0 - Audit Logger
 *
 * Detailed audit trail of all collaboration actions
 */

import { EventEmitter } from 'events';
import { AuditEvent, AuditSeverity, UserId, SessionId, Resource } from './types';

export class AuditLogger extends EventEmitter {
  private events: AuditEvent[] = [];
  private maxEvents = 10000;

  log(event: AuditEvent): void {
    this.events.push(event);

    if (this.events.length > this.maxEvents) {
      this.events.shift();
    }

    this.emit('logged', event);

    if (event.severity === AuditSeverity.CRITICAL || event.severity === AuditSeverity.ERROR) {
      this.emit('alert', event);
    }
  }

  getEvents(filter?: {
    userId?: UserId;
    sessionId?: SessionId;
    resource?: Resource;
    severity?: AuditSeverity;
    startTime?: number;
    endTime?: number;
  }): AuditEvent[] {
    let filtered = [...this.events];

    if (filter) {
      if (filter.userId) filtered = filtered.filter(e => e.userId === filter.userId);
      if (filter.sessionId) filtered = filtered.filter(e => e.sessionId === filter.sessionId);
      if (filter.resource) filtered = filtered.filter(e => e.resource === filter.resource);
      if (filter.severity) filtered = filtered.filter(e => e.severity === filter.severity);
      if (filter.startTime) filtered = filtered.filter(e => e.timestamp >= filter.startTime!);
      if (filter.endTime) filtered = filtered.filter(e => e.timestamp <= filter.endTime!);
    }

    return filtered;
  }

  export(): AuditEvent[] {
    return [...this.events];
  }

  clear(): void {
    this.events = [];
    this.emit('cleared');
  }
}
