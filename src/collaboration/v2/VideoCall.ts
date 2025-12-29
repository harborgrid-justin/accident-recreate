/**
 * AccuScene Enterprise v0.3.0 - Video Call
 *
 * WebRTC video call with screen sharing and quality adaptation
 */

import { EventEmitter } from 'events';
import {
  VideoConnection,
  UserId,
  SessionId,
  VideoQuality,
  RTCStats
} from './types';

interface VideoConfig {
  stunServers: RTCIceServer[];
  turnServers: RTCIceServer[];
  maxConnections: number;
  defaultQuality: VideoQuality;
  adaptiveQuality: boolean;
}

const QUALITY_CONSTRAINTS: Record<VideoQuality, MediaTrackConstraints> = {
  [VideoQuality.LOW]: { width: 320, height: 240, frameRate: 15 },
  [VideoQuality.MEDIUM]: { width: 640, height: 480, frameRate: 24 },
  [VideoQuality.HIGH]: { width: 1280, height: 720, frameRate: 30 },
  [VideoQuality.HD]: { width: 1920, height: 1080, frameRate: 30 }
};

export class VideoCall extends EventEmitter {
  private localStream: MediaStream | null = null;
  private screenStream: MediaStream | null = null;
  private connections: Map<UserId, VideoConnection> = new Map();
  private peerConnections: Map<UserId, RTCPeerConnection> = new Map();

  private config: VideoConfig;
  private sessionId: SessionId | null = null;
  private localUserId: UserId | null = null;

  // State
  private isConnected = false;
  private videoEnabled = true;
  private audioEnabled = true;
  private isScreenSharing = false;
  private currentQuality: VideoQuality;

  // Stats tracking
  private statsInterval: NodeJS.Timeout | null = null;
  private stats: Map<UserId, RTCStats> = new Map();

  constructor(config: Partial<VideoConfig> = {}) {
    super();

    this.config = {
      stunServers: [{ urls: 'stun:stun.l.google.com:19302' }],
      turnServers: [],
      maxConnections: 16,
      defaultQuality: VideoQuality.MEDIUM,
      adaptiveQuality: true,
      ...config
    };

    this.currentQuality = this.config.defaultQuality;
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  async connect(sessionId: SessionId, userId: UserId, quality?: VideoQuality): Promise<void> {
    this.sessionId = sessionId;
    this.localUserId = userId;

    if (quality) {
      this.currentQuality = quality;
    }

    // Get local media stream
    await this.initializeLocalStream();

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

    // Stop local streams
    this.stopLocalStream();
    if (this.isScreenSharing) {
      await this.stopScreenShare();
    }

    this.stopStatsTracking();
    this.isConnected = false;

    this.emit('disconnected');
  }

  private async initializeLocalStream(): Promise<void> {
    try {
      const constraints = QUALITY_CONSTRAINTS[this.currentQuality];

      this.localStream = await navigator.mediaDevices.getUserMedia({
        video: constraints,
        audio: {
          echoCancellation: true,
          noiseSuppression: true,
          autoGainControl: true
        }
      });

      this.emit('localStreamReady', this.localStream);
    } catch (error) {
      this.emit('error', {
        type: 'media_access',
        message: 'Failed to access camera/microphone',
        error
      });
      throw error;
    }
  }

  private stopLocalStream(): void {
    if (this.localStream) {
      this.localStream.getTracks().forEach(track => track.stop());
      this.localStream = null;
    }
  }

  // ============================================================================
  // Peer Connection Management
  // ============================================================================

  async addPeer(userId: UserId, isInitiator = false): Promise<void> {
    if (this.peerConnections.has(userId)) {
      throw new Error(`Peer already exists: ${userId}`);
    }

    if (this.peerConnections.size >= this.config.maxConnections) {
      throw new Error('Maximum connections reached');
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

    // Add screen share tracks if active
    if (this.screenStream) {
      this.screenStream.getTracks().forEach(track => {
        pc.addTrack(track, this.screenStream!);
      });
    }

    // Handle incoming tracks
    pc.ontrack = (event) => {
      this.handleRemoteTrack(userId, event.streams[0], event.track);
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

  private handleRemoteTrack(userId: UserId, stream: MediaStream, track: MediaStreamTrack): void {
    let connection = this.connections.get(userId);

    if (!connection) {
      connection = {
        userId,
        peerId: userId,
        stream,
        videoEnabled: true,
        audioEnabled: true,
        screenSharing: false,
        quality: this.currentQuality
      };
      this.connections.set(userId, connection);
    }

    // Update connection based on track type
    if (track.kind === 'video') {
      // Check if this is a screen share track
      const isScreenShare = track.label.includes('screen') || track.label.includes('window');
      connection.screenSharing = isScreenShare;
    }

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
  // Video/Audio Control
  // ============================================================================

  enableVideo(): void {
    if (!this.localStream) return;

    this.localStream.getVideoTracks().forEach(track => {
      track.enabled = true;
    });

    this.videoEnabled = true;
    this.emit('videoEnabled');
  }

  disableVideo(): void {
    if (!this.localStream) return;

    this.localStream.getVideoTracks().forEach(track => {
      track.enabled = false;
    });

    this.videoEnabled = false;
    this.emit('videoDisabled');
  }

  toggleVideo(): void {
    if (this.videoEnabled) {
      this.disableVideo();
    } else {
      this.enableVideo();
    }
  }

  enableAudio(): void {
    if (!this.localStream) return;

    this.localStream.getAudioTracks().forEach(track => {
      track.enabled = true;
    });

    this.audioEnabled = true;
    this.emit('audioEnabled');
  }

  disableAudio(): void {
    if (!this.localStream) return;

    this.localStream.getAudioTracks().forEach(track => {
      track.enabled = false;
    });

    this.audioEnabled = false;
    this.emit('audioDisabled');
  }

  toggleAudio(): void {
    if (this.audioEnabled) {
      this.disableAudio();
    } else {
      this.enableAudio();
    }
  }

  // ============================================================================
  // Screen Sharing
  // ============================================================================

  async startScreenShare(): Promise<void> {
    if (this.isScreenSharing) return;

    try {
      this.screenStream = await navigator.mediaDevices.getDisplayMedia({
        video: {
          cursor: 'always'
        },
        audio: false
      });

      // Handle screen share stopped by user
      this.screenStream.getVideoTracks()[0].onended = () => {
        this.stopScreenShare();
      };

      // Replace video track in all peer connections
      const screenTrack = this.screenStream.getVideoTracks()[0];

      for (const pc of this.peerConnections.values()) {
        const sender = pc.getSenders().find(s => s.track?.kind === 'video');
        if (sender) {
          await sender.replaceTrack(screenTrack);
        }
      }

      this.isScreenSharing = true;
      this.emit('screenShareStarted', this.screenStream);

    } catch (error) {
      this.emit('error', {
        type: 'screen_share',
        message: 'Failed to start screen sharing',
        error
      });
      throw error;
    }
  }

  async stopScreenShare(): Promise<void> {
    if (!this.isScreenSharing || !this.screenStream) return;

    // Stop screen stream
    this.screenStream.getTracks().forEach(track => track.stop());
    this.screenStream = null;

    // Restore camera track in all peer connections
    if (this.localStream) {
      const videoTrack = this.localStream.getVideoTracks()[0];

      for (const pc of this.peerConnections.values()) {
        const sender = pc.getSenders().find(s => s.track?.kind === 'video');
        if (sender && videoTrack) {
          await sender.replaceTrack(videoTrack);
        }
      }
    }

    this.isScreenSharing = false;
    this.emit('screenShareStopped');
  }

  // ============================================================================
  // Quality Management
  // ============================================================================

  async setQuality(quality: VideoQuality): Promise<void> {
    if (this.currentQuality === quality) return;

    this.currentQuality = quality;
    const constraints = QUALITY_CONSTRAINTS[quality];

    // Update local stream constraints
    if (this.localStream) {
      const videoTrack = this.localStream.getVideoTracks()[0];
      if (videoTrack) {
        await videoTrack.applyConstraints(constraints);
      }
    }

    this.emit('qualityChanged', quality);
  }

  private async adaptQuality(stats: RTCStats): Promise<void> {
    if (!this.config.adaptiveQuality) return;

    // Adapt quality based on network conditions
    if (stats.packetLoss > 5 && this.currentQuality !== VideoQuality.LOW) {
      // High packet loss - reduce quality
      const qualities = [VideoQuality.HD, VideoQuality.HIGH, VideoQuality.MEDIUM, VideoQuality.LOW];
      const currentIndex = qualities.indexOf(this.currentQuality);
      if (currentIndex < qualities.length - 1) {
        await this.setQuality(qualities[currentIndex + 1]);
      }
    } else if (stats.packetLoss < 1 && stats.latency < 100 && this.currentQuality !== VideoQuality.HD) {
      // Good conditions - increase quality
      const qualities = [VideoQuality.LOW, VideoQuality.MEDIUM, VideoQuality.HIGH, VideoQuality.HD];
      const currentIndex = qualities.indexOf(this.currentQuality);
      if (currentIndex < qualities.length - 1) {
        await this.setQuality(qualities[currentIndex + 1]);
      }
    }
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
        if (report.type === 'inbound-rtp') {
          bitrate = report.bytesReceived * 8 / 1000; // kbps
          packetLoss = report.packetsLost / (report.packetsReceived + report.packetsLost) * 100;
          jitter = report.jitter * 1000; // ms
        }

        if (report.type === 'candidate-pair' && report.state === 'succeeded') {
          latency = report.currentRoundTripTime * 1000; // ms
        }
      });

      const rtcStats: RTCStats = {
        userId,
        bitrate,
        packetLoss,
        latency,
        jitter
      };

      this.stats.set(userId, rtcStats);

      // Adapt quality if enabled
      await this.adaptQuality(rtcStats);
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

  getConnections(): VideoConnection[] {
    return Array.from(this.connections.values());
  }

  getConnection(userId: UserId): VideoConnection | null {
    return this.connections.get(userId) || null;
  }

  isVideoEnabled(): boolean {
    return this.videoEnabled;
  }

  isAudioEnabled(): boolean {
    return this.audioEnabled;
  }

  isCurrentlyScreenSharing(): boolean {
    return this.isScreenSharing;
  }

  getLocalStream(): MediaStream | null {
    return this.localStream;
  }

  getCurrentQuality(): VideoQuality {
    return this.currentQuality;
  }
}
