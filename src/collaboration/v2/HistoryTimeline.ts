/**
 * AccuScene Enterprise v0.3.0 - History Timeline
 *
 * Visual timeline of all changes with playback and inspection
 */

import { EventEmitter } from 'events';
import {
  Commit,
  CommitId,
  Operation,
  User,
  UserId
} from './types';

interface TimelineEvent {
  id: string;
  timestamp: number;
  type: TimelineEventType;
  userId: UserId;
  user?: User;
  data: any;
  commit?: Commit;
}

enum TimelineEventType {
  COMMIT = 'commit',
  OPERATION = 'operation',
  MERGE = 'merge',
  BRANCH = 'branch',
  USER_JOIN = 'user_join',
  USER_LEAVE = 'user_leave',
  ANNOTATION = 'annotation',
  CHAT = 'chat'
}

interface TimelineSegment {
  startTime: number;
  endTime: number;
  events: TimelineEvent[];
  commits: Commit[];
  activeUsers: Set<UserId>;
}

interface TimelineFilter {
  eventTypes?: TimelineEventType[];
  userIds?: UserId[];
  startTime?: number;
  endTime?: number;
  searchQuery?: string;
}

export class HistoryTimeline extends EventEmitter {
  private events: TimelineEvent[] = [];
  private commits: Map<CommitId, Commit> = new Map();
  private users: Map<UserId, User> = new Map();
  private segments: TimelineSegment[] = [];

  // Playback state
  private playbackPosition = 0;
  private isPlaying = false;
  private playbackSpeed = 1.0;
  private playbackInterval: NodeJS.Timeout | null = null;

  // Configuration
  private readonly SEGMENT_DURATION = 300000; // 5 minutes

  constructor() {
    super();
  }

  // ============================================================================
  // Event Recording
  // ============================================================================

  recordCommit(commit: Commit, user?: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: commit.timestamp,
      type: TimelineEventType.COMMIT,
      userId: commit.author,
      user,
      data: commit,
      commit
    };

    this.addEvent(event);
    this.commits.set(commit.id, commit);

    if (user) {
      this.users.set(user.id, user);
    }
  }

  recordOperation(operation: Operation, user?: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: operation.timestamp,
      type: TimelineEventType.OPERATION,
      userId: operation.userId,
      user,
      data: operation
    };

    this.addEvent(event);

    if (user) {
      this.users.set(user.id, user);
    }
  }

  recordMerge(sourceId: string, targetId: string, user: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: Date.now(),
      type: TimelineEventType.MERGE,
      userId: user.id,
      user,
      data: { sourceId, targetId }
    };

    this.addEvent(event);
    this.users.set(user.id, user);
  }

  recordUserJoin(user: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: Date.now(),
      type: TimelineEventType.USER_JOIN,
      userId: user.id,
      user,
      data: { user }
    };

    this.addEvent(event);
    this.users.set(user.id, user);
  }

  recordUserLeave(user: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: Date.now(),
      type: TimelineEventType.USER_LEAVE,
      userId: user.id,
      user,
      data: { user }
    };

    this.addEvent(event);
  }

  recordAnnotation(annotation: any, user: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: annotation.createdAt,
      type: TimelineEventType.ANNOTATION,
      userId: user.id,
      user,
      data: annotation
    };

    this.addEvent(event);
    this.users.set(user.id, user);
  }

  recordChatMessage(message: any, user: User): void {
    const event: TimelineEvent = {
      id: this.generateEventId(),
      timestamp: message.timestamp,
      type: TimelineEventType.CHAT,
      userId: user.id,
      user,
      data: message
    };

    this.addEvent(event);
    this.users.set(user.id, user);
  }

  private addEvent(event: TimelineEvent): void {
    // Insert event in chronological order
    const insertIndex = this.findInsertIndex(event.timestamp);
    this.events.splice(insertIndex, 0, event);

    // Update segments
    this.updateSegments();

    this.emit('eventAdded', event);
  }

  private findInsertIndex(timestamp: number): number {
    // Binary search for insertion point
    let left = 0;
    let right = this.events.length;

    while (left < right) {
      const mid = Math.floor((left + right) / 2);
      if (this.events[mid].timestamp < timestamp) {
        left = mid + 1;
      } else {
        right = mid;
      }
    }

    return left;
  }

  // ============================================================================
  // Timeline Segmentation
  // ============================================================================

  private updateSegments(): void {
    if (this.events.length === 0) return;

    this.segments = [];
    let currentSegment: TimelineEvent[] = [];
    let segmentStart = this.events[0].timestamp;

    for (const event of this.events) {
      if (event.timestamp - segmentStart > this.SEGMENT_DURATION) {
        // Create segment
        if (currentSegment.length > 0) {
          this.segments.push(this.createSegment(currentSegment));
        }

        // Start new segment
        currentSegment = [event];
        segmentStart = event.timestamp;
      } else {
        currentSegment.push(event);
      }
    }

    // Add final segment
    if (currentSegment.length > 0) {
      this.segments.push(this.createSegment(currentSegment));
    }
  }

  private createSegment(events: TimelineEvent[]): TimelineSegment {
    const startTime = events[0].timestamp;
    const endTime = events[events.length - 1].timestamp;

    const commits = events
      .filter(e => e.commit)
      .map(e => e.commit!);

    const activeUsers = new Set(events.map(e => e.userId));

    return {
      startTime,
      endTime,
      events: [...events],
      commits,
      activeUsers
    };
  }

  getSegments(): TimelineSegment[] {
    return [...this.segments];
  }

  getSegmentAt(timestamp: number): TimelineSegment | null {
    return this.segments.find(
      s => timestamp >= s.startTime && timestamp <= s.endTime
    ) || null;
  }

  // ============================================================================
  // Timeline Querying
  // ============================================================================

  getEvents(filter?: TimelineFilter): TimelineEvent[] {
    let filtered = [...this.events];

    if (filter) {
      if (filter.eventTypes) {
        filtered = filtered.filter(e => filter.eventTypes!.includes(e.type));
      }

      if (filter.userIds) {
        filtered = filtered.filter(e => filter.userIds!.includes(e.userId));
      }

      if (filter.startTime !== undefined) {
        filtered = filtered.filter(e => e.timestamp >= filter.startTime!);
      }

      if (filter.endTime !== undefined) {
        filtered = filtered.filter(e => e.timestamp <= filter.endTime!);
      }

      if (filter.searchQuery) {
        const query = filter.searchQuery.toLowerCase();
        filtered = filtered.filter(e =>
          JSON.stringify(e.data).toLowerCase().includes(query)
        );
      }
    }

    return filtered;
  }

  getEventById(eventId: string): TimelineEvent | null {
    return this.events.find(e => e.id === eventId) || null;
  }

  getEventsByType(type: TimelineEventType): TimelineEvent[] {
    return this.events.filter(e => e.type === type);
  }

  getEventsByUser(userId: UserId): TimelineEvent[] {
    return this.events.filter(e => e.userId === userId);
  }

  getEventsBetween(startTime: number, endTime: number): TimelineEvent[] {
    return this.events.filter(
      e => e.timestamp >= startTime && e.timestamp <= endTime
    );
  }

  getCommits(): Commit[] {
    return Array.from(this.commits.values());
  }

  getCommit(commitId: CommitId): Commit | null {
    return this.commits.get(commitId) || null;
  }

  // ============================================================================
  // Playback Control
  // ============================================================================

  play(): void {
    if (this.isPlaying) return;

    this.isPlaying = true;
    this.startPlayback();
    this.emit('playbackStarted');
  }

  pause(): void {
    if (!this.isPlaying) return;

    this.isPlaying = false;
    this.stopPlayback();
    this.emit('playbackPaused');
  }

  stop(): void {
    this.isPlaying = false;
    this.playbackPosition = 0;
    this.stopPlayback();
    this.emit('playbackStopped');
  }

  seek(position: number): void {
    this.playbackPosition = Math.max(0, Math.min(position, this.events.length - 1));
    this.emit('playbackSeeked', this.playbackPosition);
  }

  seekToTime(timestamp: number): void {
    const index = this.events.findIndex(e => e.timestamp >= timestamp);
    if (index !== -1) {
      this.seek(index);
    }
  }

  setSpeed(speed: number): void {
    this.playbackSpeed = Math.max(0.1, Math.min(10, speed));
    this.emit('playbackSpeedChanged', this.playbackSpeed);

    // Restart playback if playing
    if (this.isPlaying) {
      this.stopPlayback();
      this.startPlayback();
    }
  }

  getPlaybackState() {
    return {
      position: this.playbackPosition,
      isPlaying: this.isPlaying,
      speed: this.playbackSpeed,
      currentEvent: this.events[this.playbackPosition] || null,
      progress: this.events.length > 0 ? this.playbackPosition / this.events.length : 0
    };
  }

  private startPlayback(): void {
    // Calculate interval based on average time between events
    const avgInterval = this.calculateAverageInterval();
    const interval = avgInterval / this.playbackSpeed;

    this.playbackInterval = setInterval(() => {
      if (this.playbackPosition >= this.events.length - 1) {
        this.stop();
        return;
      }

      this.playbackPosition++;
      const event = this.events[this.playbackPosition];

      this.emit('playbackEvent', event);
      this.emit('playbackProgress', {
        position: this.playbackPosition,
        event,
        progress: this.playbackPosition / this.events.length
      });

    }, interval);
  }

  private stopPlayback(): void {
    if (this.playbackInterval) {
      clearInterval(this.playbackInterval);
      this.playbackInterval = null;
    }
  }

  private calculateAverageInterval(): number {
    if (this.events.length < 2) return 1000;

    let totalInterval = 0;
    for (let i = 1; i < this.events.length; i++) {
      totalInterval += this.events[i].timestamp - this.events[i - 1].timestamp;
    }

    return totalInterval / (this.events.length - 1);
  }

  // ============================================================================
  // Timeline Analysis
  // ============================================================================

  getActivityHeatmap(bucketSize = 3600000): Map<number, number> {
    // Create heatmap of activity (1 hour buckets by default)
    const heatmap = new Map<number, number>();

    for (const event of this.events) {
      const bucket = Math.floor(event.timestamp / bucketSize) * bucketSize;
      heatmap.set(bucket, (heatmap.get(bucket) || 0) + 1);
    }

    return heatmap;
  }

  getUserActivityStats(userId: UserId): {
    totalEvents: number;
    commits: number;
    operations: number;
    annotations: number;
    chatMessages: number;
    firstActivity: number;
    lastActivity: number;
  } | null {
    const userEvents = this.getEventsByUser(userId);

    if (userEvents.length === 0) return null;

    return {
      totalEvents: userEvents.length,
      commits: userEvents.filter(e => e.type === TimelineEventType.COMMIT).length,
      operations: userEvents.filter(e => e.type === TimelineEventType.OPERATION).length,
      annotations: userEvents.filter(e => e.type === TimelineEventType.ANNOTATION).length,
      chatMessages: userEvents.filter(e => e.type === TimelineEventType.CHAT).length,
      firstActivity: userEvents[0].timestamp,
      lastActivity: userEvents[userEvents.length - 1].timestamp
    };
  }

  getSessionDuration(): number {
    if (this.events.length < 2) return 0;
    return this.events[this.events.length - 1].timestamp - this.events[0].timestamp;
  }

  getEventFrequency(): number {
    const duration = this.getSessionDuration();
    return duration > 0 ? this.events.length / (duration / 1000) : 0; // events per second
  }

  getPeakActivityPeriod(windowSize = 3600000): {
    startTime: number;
    endTime: number;
    eventCount: number;
  } | null {
    if (this.events.length === 0) return null;

    let maxCount = 0;
    let maxStart = 0;
    let maxEnd = 0;

    for (let i = 0; i < this.events.length; i++) {
      const windowStart = this.events[i].timestamp;
      const windowEnd = windowStart + windowSize;

      const count = this.events.filter(
        e => e.timestamp >= windowStart && e.timestamp < windowEnd
      ).length;

      if (count > maxCount) {
        maxCount = count;
        maxStart = windowStart;
        maxEnd = windowEnd;
      }
    }

    return {
      startTime: maxStart,
      endTime: maxEnd,
      eventCount: maxCount
    };
  }

  // ============================================================================
  // Export & Import
  // ============================================================================

  export(): {
    events: TimelineEvent[];
    commits: Commit[];
    users: User[];
    metadata: any;
  } {
    return {
      events: [...this.events],
      commits: Array.from(this.commits.values()),
      users: Array.from(this.users.values()),
      metadata: {
        totalEvents: this.events.length,
        totalCommits: this.commits.size,
        totalUsers: this.users.size,
        duration: this.getSessionDuration(),
        startTime: this.events[0]?.timestamp,
        endTime: this.events[this.events.length - 1]?.timestamp
      }
    };
  }

  import(data: {
    events: TimelineEvent[];
    commits: Commit[];
    users: User[];
  }): void {
    this.clear();

    this.events = [...data.events].sort((a, b) => a.timestamp - b.timestamp);

    for (const commit of data.commits) {
      this.commits.set(commit.id, commit);
    }

    for (const user of data.users) {
      this.users.set(user.id, user);
    }

    this.updateSegments();
    this.emit('imported', { eventCount: this.events.length });
  }

  clear(): void {
    this.events = [];
    this.commits.clear();
    this.users.clear();
    this.segments = [];
    this.playbackPosition = 0;
    this.stop();
    this.emit('cleared');
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  private generateEventId(): string {
    return `event-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  getStatistics() {
    const eventsByType = this.events.reduce((acc, e) => {
      acc[e.type] = (acc[e.type] || 0) + 1;
      return acc;
    }, {} as Record<string, number>);

    return {
      totalEvents: this.events.length,
      totalCommits: this.commits.size,
      totalUsers: this.users.size,
      totalSegments: this.segments.length,
      eventsByType,
      duration: this.getSessionDuration(),
      frequency: this.getEventFrequency(),
      playback: this.getPlaybackState()
    };
  }
}
