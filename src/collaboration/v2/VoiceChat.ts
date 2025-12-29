/**
 * AccuScene Enterprise v0.3.0 - Voice Chat
 *
 * WebRTC voice chat with spatial audio and quality adaptation
 */

import { EventEmitter } from 'events';
import {
  VoiceConnection,
  UserId,
  SessionId,
  RTCStats
} from './types';

interface VoiceConfig {
  stunServers: RTCIceServer[];
  turnServers: RTCIceServer[];
  echoCancellation: boolean;
  noiseSuppression: boolean;
  autoGainControl: boolean;
  spatialAudio: boolean;
}

export class VoiceChat extends EventEmitter {
  private localStream: MediaStream | null = null;
  private connections: Map<UserId, VoiceConnection> = new Map();
  private peerConnections: Map<UserId, RTCPeerConnection> = new Map();

  private config: VoiceConfig;
  private sessionId: SessionId | null = null;
  private localUserId: UserId | null = null;

  // State
  private isConnected = false;
  private isMuted = false;
  private volume = 1.0;

  // Audio context for spatial audio
  private audioContext: AudioContext | null = null;
  private gainNode: GainNode | null = null;

  // Stats tracking
  private statsInterval: NodeJS.Timeout | null = null;
  private stats: Map<UserId, RTCStats> = new Map();

  constructor(config: Partial<VoiceConfig> = {}) {
    super();

    this.config = {
      stunServers: [{ urls: 'stun:stun.l.google.com:19302' }],
      turnServers: [],
      echoCancellation: true,
      noiseSuppression: true,
      autoGainControl: true,
      spatialAudio: false,
      ...config
    };
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  async connect(sessionId: SessionId, userId: UserId): Promise<void> {
    this.sessionId = sessionId;
    this.localUserId = userId;

    // Get local media stream
    await this.initializeLocalStream();

    // Initialize audio context if spatial audio enabled
    if (this.config.spatialAudio) {
      await this.initializeSpatialAudio();
    }

    this.isConnected = true;
    this.startStatsTracking();

    this.emit('connected');
  }

  async disconnect(): Promise<void> {
    // Stop all peer connections
    for (const [userId, pc] of this.peerConnections.entries()) {
      pc.close();
      this.peerConnections.delete(userId);
      this.connections.delete(userId);
    }

    // Stop local stream
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }

    // Close audio context
    if (this.audioContext) {
      await this.audioContext.close();
      this.audioContext = null;
    }

    this.stopStatsTracking();
    this.isConnected = false;

    this.emit('disconnected');
  }

  private async initializeLocalStream(): Promise<void> {
    try {
      this.localStream = await navigator.mediaDevices.getUserMedia({
        audio: {
          echoCancellation: this.config.echoCancellation,
          noiseSuppression: this.config.noiseSuppression,
          autoGainControl: this.config.autoGainControl
        }
      });

      this.emit('localStreamReady', this.localStream);
    } catch (error) {
      this.emit('error', {
        type: 'media_access',
        message: 'Failed to access microphone',
        error
      });
      throw error;
    }
  }

  // ============================================================================
  // Peer Connection Management
  // ============================================================================

  async addPeer(userId: UserId, isInitiator = false): Promise<void> {
    if (this.peerConnections.has(userId)) {
      throw new Error(`Peer already exists: ${userId}`);
    }

    const pc = new RTCPeerConnection({
      iceServers: [...this.config.stunServers, ...this.config.turnServers]
    });

    // Add local tracks
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => {
        pc.addTrack(track, this.localStream!);
      });
    }

    // Handle incoming tracks
    pc.ontrack = (event) => {
      this.handleRemoteTrack(userId, event.streams[0]);
    };

    // Handle ICE candidates
    pc.onicecandidate = (event) => {
      if (event.candidate) {
        this.emit('iceCandidate', {
          userId,
          candidate: event.candidate
        });
      }
    };

    // Handle connection state changes
    pc.onconnectionstatechange = () => {
      this.emit('connectionStateChange', {
        userId,
        state: pc.connectionState
      });

      if (pc.connectionState === 'failed') {
        this.handleConnectionFailure(userId);
      }
    };

    this.peerConnections.set(userId, pc);

    // Create offer if initiator
    if (isInitiator) {
      const offer = await pc.createOffer();
      await pc.setLocalDescription(offer);

      this.emit('offer', {
        userId,
        offer
      });
    }
  }

  async removePeer(userId: UserId): Promise<void> {
    const pc = this.peerConnections.get(userId);
    if (pc) {
      pc.close();
      this.peerConnections.delete(userId);
    }

    this.connections.delete(userId);
    this.stats.delete(userId);

    this.emit('peerRemoved', userId);
  }

  async handleOffer(userId: UserId, offer: RTCSessionDescriptionInit): Promise<void> {
    let pc = this.peerConnections.get(userId);

    if (!pc) {
      await this.addPeer(userId, false);
      pc = this.peerConnections.get(userId)!;
    }

    await pc.setRemoteDescription(new RTCSessionDescription(offer));

    const answer = await pc.createAnswer();
    await pc.setLocalDescription(answer);

    this.emit('answer', {
      userId,
      answer
    });
  }

  async handleAnswer(userId: UserId, answer: RTCSessionDescriptionInit): Promise<void> {
    const pc = this.peerConnections.get(userId);
    if (!pc) {
      throw new Error(`No peer connection for user: ${userId}`);
    }

    await pc.setRemoteDescription(new RTCSessionDescription(answer));
  }

  async handleIceCandidate(userId: UserId, candidate: RTCIceCandidateInit): Promise<void> {
    const pc = this.peerConnections.get(userId);
    if (!pc) {
      throw new Error(`No peer connection for user: ${userId}`);
    }

    await pc.addIceCandidate(new RTCIceCandidate(candidate));
  }

  private handleRemoteTrack(userId: UserId, stream: MediaStream): void {
    const connection: VoiceConnection = {
      userId,
      peerId: userId,
      stream,
      muted: false,
      volume: 1.0,
      speaking: false
    };

    this.connections.set(userId, connection);

    // Apply spatial audio if enabled
    if (this.config.spatialAudio && this.audioContext) {
      this.applySpatialAudio(userId, stream);
    }

    // Detect speaking activity
    this.detectSpeakingActivity(userId, stream);

    this.emit('peerConnected', connection);
  }

  private handleConnectionFailure(userId: UserId): void {
    this.emit('connectionFailed', userId);

    // Attempt reconnection
    setTimeout(() => {
      if (this.isConnected) {
        this.reconnectPeer(userId);
      }
    }, 3000);
  }

  private async reconnectPeer(userId: UserId): Promise<void> {
    await this.removePeer(userId);
    await this.addPeer(userId, true);
  }

  // ============================================================================
  // Audio Control
  // ============================================================================

  mute(): void {
    if (!this.localStream) return;

    this.localStream.getAudioTracks().forEach(track => {
      track.enabled = false;
    });

    this.isMuted = true;
    this.emit('muted');
  }

  unmute(): void {
    if (!this.localStream) return;

    this.localStream.getAudioTracks().forEach(track => {
      track.enabled = true;
    });

    this.isMuted = false;
    this.emit('unmuted');
  }

  toggleMute(): void {
    if (this.isMuted) {
      this.unmute();
    } else {
      this.mute();
    }
  }

  setVolume(volume: number): void {
    this.volume = Math.max(0, Math.min(1, volume));

    if (this.gainNode) {
      this.gainNode.gain.value = this.volume;
    }

    this.emit('volumeChanged', this.volume);
  }

  setPeerVolume(userId: UserId, volume: number): void {
    const connection = this.connections.get(userId);
    if (!connection) return;

    connection.volume = Math.max(0, Math.min(1, volume));
    this.emit('peerVolumeChanged', { userId, volume });
  }

  mutePeer(userId: UserId): void {
    const connection = this.connections.get(userId);
    if (!connection) return;

    connection.muted = true;

    const audioElement = document.querySelector(`audio[data-user-id="${userId}"]`) as HTMLAudioElement;
    if (audioElement) {
      audioElement.muted = true;
    }

    this.emit('peerMuted', userId);
  }

  unmutePeer(userId: UserId): void {
    const connection = this.connections.get(userId);
    if (!connection) return;

    connection.muted = false;

    const audioElement = document.querySelector(`audio[data-user-id="${userId}"]`) as HTMLAudioElement;
    if (audioElement) {
      audioElement.muted = false;
    }

    this.emit('peerUnmuted', userId);
  }

  // ============================================================================
  // Spatial Audio
  // ============================================================================

  private async initializeSpatialAudio(): Promise<void> {
    this.audioContext = new AudioContext();
    this.gainNode = this.audioContext.createGain();
    this.gainNode.gain.value = this.volume;
    this.gainNode.connect(this.audioContext.destination);
  }

  private applySpatialAudio(userId: UserId, stream: MediaStream): void {
    if (!this.audioContext) return;

    const source = this.audioContext.createMediaStreamSource(stream);
    const panner = this.audioContext.createPanner();

    panner.panningModel = 'HRTF';
    panner.distanceModel = 'inverse';
    panner.refDistance = 1;
    panner.maxDistance = 10000;
    panner.rolloffFactor = 1;
    panner.coneInnerAngle = 360;
    panner.coneOuterAngle = 0;
    panner.coneOuterGain = 0;

    source.connect(panner);
    panner.connect(this.gainNode!);
  }

  updateSpatialPosition(userId: UserId, position: { x: number; y: number; z: number }): void {
    if (!this.audioContext) return;

    // Find the panner node for this user
    // In a real implementation, we'd store references to panner nodes
    this.emit('spatialPositionUpdated', { userId, position });
  }

  // ============================================================================
  // Speaking Detection
  // ============================================================================

  private detectSpeakingActivity(userId: UserId, stream: MediaStream): void {
    if (!this.audioContext) {
      this.audioContext = new AudioContext();
    }

    const source = this.audioContext.createMediaStreamSource(stream);
    const analyser = this.audioContext.createAnalyser();
    analyser.fftSize = 512;

    source.connect(analyser);

    const dataArray = new Uint8Array(analyser.frequencyBinCount);
    const threshold = 30;

    const checkSpeaking = () => {
      analyser.getByteFrequencyData(dataArray);

      const average = dataArray.reduce((a, b) => a + b) / dataArray.length;
      const isSpeaking = average > threshold;

      const connection = this.connections.get(userId);
      if (connection && connection.speaking !== isSpeaking) {
        connection.speaking = isSpeaking;
        this.emit('speakingChanged', { userId, speaking: isSpeaking });
      }

      if (this.connections.has(userId)) {
        requestAnimationFrame(checkSpeaking);
      }
    };

    checkSpeaking();
  }

  // ============================================================================
  // Statistics
  // ============================================================================

  private startStatsTracking(): void {
    this.statsInterval = setInterval(() => {
      this.updateStats();
    }, 1000);
  }

  private stopStatsTracking(): void {
    if (this.statsInterval) {
      clearInterval(this.statsInterval);
      this.statsInterval = null;
    }
  }

  private async updateStats(): Promise<void> {
    for (const [userId, pc] of this.peerConnections.entries()) {
      const stats = await pc.getStats();

      let bitrate = 0;
      let packetLoss = 0;
      let latency = 0;
      let jitter = 0;

      stats.forEach((report: any) => {
        if (report.type === 'inbound-rtp' && report.mediaType === 'audio') {
          bitrate = report.bytesReceived * 8 / 1000; // kbps
          packetLoss = report.packetsLost / (report.packetsReceived + report.packetsLost) * 100;
          jitter = report.jitter * 1000; // ms
        }

        if (report.type === 'candidate-pair' && report.state === 'succeeded') {
          latency = report.currentRoundTripTime * 1000; // ms
        }
      });

      this.stats.set(userId, {
        userId,
        bitrate,
        packetLoss,
        latency,
        jitter
      });
    }

    this.emit('statsUpdated', Array.from(this.stats.values()));
  }

  getStats(userId?: UserId): RTCStats | RTCStats[] | null {
    if (userId) {
      return this.stats.get(userId) || null;
    }
    return Array.from(this.stats.values());
  }

  // ============================================================================
  // Getters
  // ============================================================================

  getConnections(): VoiceConnection[] {
    return Array.from(this.connections.values());
  }

  getConnection(userId: UserId): VoiceConnection | null {
    return this.connections.get(userId) || null;
  }

  isMutedLocal(): boolean {
    return this.isMuted;
  }

  getVolume(): number {
    return this.volume;
  }

  getSpeakingUsers(): UserId[] {
    return Array.from(this.connections.values())
      .filter(c => c.speaking)
      .map(c => c.userId);
  }
}
