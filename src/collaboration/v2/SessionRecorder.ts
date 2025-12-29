/**
 * AccuScene Enterprise v0.3.0 - Session Recorder
 *
 * Record and playback editing sessions with full fidelity
 */

import { EventEmitter } from 'events';
import {
  SessionRecording,
  RecordedEvent,
  PlaybackState,
  SessionId,
  User,
  Operation
} from './types';

interface RecorderConfig {
  captureInterval: number;
  maxDuration: number;
  compressionEnabled: boolean;
  includeAudio: boolean;
  includeVideo: boolean;
}

export class SessionRecorder extends EventEmitter {
  private recording: SessionRecording | null = null;
  private isRecording = false;
  private isPaused = false;
  private startTime = 0;
  private pauseStartTime = 0;
  private totalPausedTime = 0;

  private config: RecorderConfig;
  private captureInterval: NodeJS.Timeout | null = null;
  private eventBuffer: RecordedEvent[] = [];

  // Playback state
  private playbackState: PlaybackState | null = null;
  private playbackInterval: NodeJS.Timeout | null = null;

  constructor(config: Partial<RecorderConfig> = {}) {
    super();

    this.config = {
      captureInterval: 100, // 10 FPS
      maxDuration: 3600000, // 1 hour
      compressionEnabled: true,
      includeAudio: false,
      includeVideo: false,
      ...config
    };
  }

  // ============================================================================
  // Recording Control
  // ============================================================================

  startRecording(sessionId: SessionId, participants: User[]): void {
    if (this.isRecording) {
      throw new Error('Already recording');
    }

    this.recording = {
      id: this.generateRecordingId(),
      sessionId,
      startTime: Date.now(),
      events: [],
      participants: [...participants],
      duration: 0,
      size: 0
    };

    this.isRecording = true;
    this.isPaused = false;
    this.startTime = Date.now();
    this.totalPausedTime = 0;
    this.eventBuffer = [];

    this.startCapture();
    this.emit('recordingStarted', this.recording);
  }

  pauseRecording(): void {
    if (!this.isRecording || this.isPaused) return;

    this.isPaused = true;
    this.pauseStartTime = Date.now();
    this.stopCapture();
    this.emit('recordingPaused');
  }

  resumeRecording(): void {
    if (!this.isRecording || !this.isPaused) return;

    this.totalPausedTime += Date.now() - this.pauseStartTime;
    this.isPaused = false;
    this.startCapture();
    this.emit('recordingResumed');
  }

  stopRecording(): SessionRecording | null {
    if (!this.isRecording) return null;

    this.isRecording = false;
    this.isPaused = false;
    this.stopCapture();

    // Flush buffer
    this.flushEventBuffer();

    if (this.recording) {
      this.recording.endTime = Date.now();
      this.recording.duration = this.getRecordingDuration();
      this.recording.size = this.calculateRecordingSize();

      this.emit('recordingStopped', this.recording);

      const result = this.recording;
      this.recording = null;
      return result;
    }

    return null;
  }

  // ============================================================================
  // Event Capture
  // ============================================================================

  captureOperation(operation: Operation): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'operation',
      timestamp: this.getRelativeTime(),
      userId: operation.userId,
      data: operation
    };

    this.addEvent(event);
  }

  captureUserAction(userId: string, action: string, data: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'user_action',
      timestamp: this.getRelativeTime(),
      userId,
      data: { action, ...data }
    };

    this.addEvent(event);
  }

  captureCursorMove(userId: string, position: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'cursor_move',
      timestamp: this.getRelativeTime(),
      userId,
      data: position
    };

    this.addEvent(event);
  }

  captureSelection(userId: string, selection: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'selection',
      timestamp: this.getRelativeTime(),
      userId,
      data: selection
    };

    this.addEvent(event);
  }

  captureChatMessage(userId: string, message: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'chat',
      timestamp: this.getRelativeTime(),
      userId,
      data: message
    };

    this.addEvent(event);
  }

  captureAnnotation(userId: string, annotation: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'annotation',
      timestamp: this.getRelativeTime(),
      userId,
      data: annotation
    };

    this.addEvent(event);
  }

  captureSnapshot(sceneData: any): void {
    if (!this.isRecording || this.isPaused) return;

    const event: RecordedEvent = {
      type: 'snapshot',
      timestamp: this.getRelativeTime(),
      userId: 'system',
      data: sceneData
    };

    this.addEvent(event);
  }

  private addEvent(event: RecordedEvent): void {
    this.eventBuffer.push(event);

    // Flush buffer periodically
    if (this.eventBuffer.length >= 100) {
      this.flushEventBuffer();
    }
  }

  private flushEventBuffer(): void {
    if (!this.recording || this.eventBuffer.length === 0) return;

    this.recording.events.push(...this.eventBuffer);
    this.eventBuffer = [];

    // Check max duration
    const duration = this.getRecordingDuration();
    if (duration >= this.config.maxDuration) {
      this.stopRecording();
    }
  }

  private startCapture(): void {
    this.captureInterval = setInterval(() => {
      this.flushEventBuffer();
    }, this.config.captureInterval);
  }

  private stopCapture(): void {
    if (this.captureInterval) {
      clearInterval(this.captureInterval);
      this.captureInterval = null;
    }
  }

  // ============================================================================
  // Playback Control
  // ============================================================================

  loadRecording(recording: SessionRecording): void {
    this.playbackState = {
      recording,
      currentTime: 0,
      playing: false,
      speed: 1.0,
      loop: false
    };

    this.emit('recordingLoaded', recording);
  }

  play(): void {
    if (!this.playbackState) {
      throw new Error('No recording loaded');
    }

    if (this.playbackState.playing) return;

    this.playbackState.playing = true;
    this.startPlayback();
    this.emit('playbackStarted');
  }

  pausePlayback(): void {
    if (!this.playbackState || !this.playbackState.playing) return;

    this.playbackState.playing = false;
    this.stopPlayback();
    this.emit('playbackPaused');
  }

  stopPlayback(): void {
    if (!this.playbackState) return;

    this.playbackState.playing = false;
    this.playbackState.currentTime = 0;
    this.stopPlayback();
    this.emit('playbackStopped');
  }

  seekTo(time: number): void {
    if (!this.playbackState) return;

    this.playbackState.currentTime = Math.max(
      0,
      Math.min(time, this.playbackState.recording.duration)
    );

    this.emit('playbackSeeked', this.playbackState.currentTime);
  }

  setPlaybackSpeed(speed: number): void {
    if (!this.playbackState) return;

    this.playbackState.speed = Math.max(0.1, Math.min(10, speed));

    // Restart playback if playing
    if (this.playbackState.playing) {
      this.stopPlayback();
      this.startPlayback();
    }

    this.emit('playbackSpeedChanged', this.playbackState.speed);
  }

  setLoop(loop: boolean): void {
    if (!this.playbackState) return;

    this.playbackState.loop = loop;
    this.emit('playbackLoopChanged', loop);
  }

  getPlaybackState(): PlaybackState | null {
    return this.playbackState;
  }

  private startPlayback(): void {
    if (!this.playbackState) return;

    const tickInterval = 50; // 20 FPS

    this.playbackInterval = setInterval(() => {
      if (!this.playbackState) return;

      const deltaTime = (tickInterval * this.playbackState.speed);
      this.playbackState.currentTime += deltaTime;

      // Get events at current time
      const events = this.getEventsAtTime(this.playbackState.currentTime);

      // Emit events
      for (const event of events) {
        this.emit('playbackEvent', event);
      }

      // Update progress
      const progress = this.playbackState.currentTime / this.playbackState.recording.duration;
      this.emit('playbackProgress', {
        currentTime: this.playbackState.currentTime,
        progress
      });

      // Check if finished
      if (this.playbackState.currentTime >= this.playbackState.recording.duration) {
        if (this.playbackState.loop) {
          this.playbackState.currentTime = 0;
        } else {
          this.stopPlayback();
        }
      }

    }, tickInterval);
  }

  private stopPlayback(): void {
    if (this.playbackInterval) {
      clearInterval(this.playbackInterval);
      this.playbackInterval = null;
    }
  }

  private getEventsAtTime(time: number): RecordedEvent[] {
    if (!this.playbackState) return [];

    const tolerance = 50; // ms
    return this.playbackState.recording.events.filter(
      event => Math.abs(event.timestamp - time) < tolerance
    );
  }

  // ============================================================================
  // Recording Management
  // ============================================================================

  getCurrentRecording(): SessionRecording | null {
    return this.recording;
  }

  isCurrentlyRecording(): boolean {
    return this.isRecording;
  }

  getRecordingDuration(): number {
    if (!this.recording) return 0;

    const elapsed = Date.now() - this.startTime;
    return elapsed - this.totalPausedTime;
  }

  private getRelativeTime(): number {
    return this.getRecordingDuration();
  }

  private calculateRecordingSize(): number {
    if (!this.recording) return 0;

    // Estimate size in bytes
    const jsonStr = JSON.stringify(this.recording);
    return jsonStr.length;
  }

  // ============================================================================
  // Export & Import
  // ============================================================================

  async exportRecording(recording: SessionRecording, format: 'json' | 'compressed' = 'json'): Promise<Blob> {
    if (format === 'compressed' && this.config.compressionEnabled) {
      return this.compressRecording(recording);
    }

    const json = JSON.stringify(recording, null, 2);
    return new Blob([json], { type: 'application/json' });
  }

  async importRecording(blob: Blob, format: 'json' | 'compressed' = 'json'): Promise<SessionRecording> {
    if (format === 'compressed') {
      return this.decompressRecording(blob);
    }

    const text = await blob.text();
    return JSON.parse(text);
  }

  private async compressRecording(recording: SessionRecording): Promise<Blob> {
    // Simple compression: remove redundant data and minify
    const compressed = {
      ...recording,
      events: recording.events.map(e => ({
        t: e.type,
        ts: e.timestamp,
        u: e.userId,
        d: e.data
      }))
    };

    const json = JSON.stringify(compressed);
    return new Blob([json], { type: 'application/json' });
  }

  private async decompressRecording(blob: Blob): Promise<SessionRecording> {
    const text = await blob.text();
    const compressed = JSON.parse(text);

    return {
      ...compressed,
      events: compressed.events.map((e: any) => ({
        type: e.t,
        timestamp: e.ts,
        userId: e.u,
        data: e.d
      }))
    };
  }

  // ============================================================================
  // Analysis
  // ============================================================================

  analyzeRecording(recording: SessionRecording): {
    totalEvents: number;
    eventsByType: Record<string, number>;
    eventsByUser: Record<string, number>;
    averageEventRate: number;
    peakEventRate: number;
    participants: number;
  } {
    const eventsByType: Record<string, number> = {};
    const eventsByUser: Record<string, number> = {};

    for (const event of recording.events) {
      eventsByType[event.type] = (eventsByType[event.type] || 0) + 1;
      eventsByUser[event.userId] = (eventsByUser[event.userId] || 0) + 1;
    }

    const durationSeconds = recording.duration / 1000;
    const averageEventRate = durationSeconds > 0 ? recording.events.length / durationSeconds : 0;

    // Calculate peak event rate (events per second in 10-second windows)
    let peakEventRate = 0;
    const windowSize = 10000; // 10 seconds

    for (let t = 0; t < recording.duration; t += windowSize) {
      const eventsInWindow = recording.events.filter(
        e => e.timestamp >= t && e.timestamp < t + windowSize
      ).length;
      const rate = eventsInWindow / (windowSize / 1000);
      peakEventRate = Math.max(peakEventRate, rate);
    }

    return {
      totalEvents: recording.events.length,
      eventsByType,
      eventsByUser,
      averageEventRate,
      peakEventRate,
      participants: recording.participants.length
    };
  }

  // ============================================================================
  // Helpers
  // ============================================================================

  private generateRecordingId(): string {
    return `rec-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  // ============================================================================
  // Cleanup
  // ============================================================================

  destroy(): void {
    this.stopRecording();
    this.stopPlayback();
    this.recording = null;
    this.playbackState = null;
    this.removeAllListeners();
  }
}
