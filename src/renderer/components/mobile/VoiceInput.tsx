/**
 * AccuScene Enterprise v0.3.0
 * Voice Input Component
 *
 * Voice-to-text input with real-time transcription
 */

import React, { useState, useRef, useEffect, CSSProperties } from 'react';
import { VoiceRecognitionConfig, VoiceTranscript } from './types';
import { HapticFeedback } from './HapticFeedback';

export interface VoiceInputProps {
  onTranscript: (transcript: VoiceTranscript) => void;
  onError?: (error: string) => void;
  config?: VoiceRecognitionConfig;
  showVisualizer?: boolean;
  className?: string;
}

/**
 * Voice input component with speech recognition
 * Provides real-time transcription and interim results
 *
 * @example
 * ```tsx
 * <VoiceInput
 *   onTranscript={(transcript) => {
 *     if (transcript.isFinal) {
 *       setFieldValue(transcript.text);
 *     }
 *   }}
 *   config={{ language: 'en-US', continuous: true }}
 *   showVisualizer
 * />
 * ```
 */
export const VoiceInput: React.FC<VoiceInputProps> = ({
  onTranscript,
  onError,
  config = {},
  showVisualizer = true,
  className = '',
}) => {
  const [isListening, setIsListening] = useState(false);
  const [transcript, setTranscript] = useState('');
  const [interimTranscript, setInterimTranscript] = useState('');
  const [audioLevel, setAudioLevel] = useState(0);
  const [error, setError] = useState<string | null>(null);

  const recognitionRef = useRef<any>(null);
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const animationFrameRef = useRef<number | null>(null);

  const defaultConfig: Required<VoiceRecognitionConfig> = {
    language: 'en-US',
    continuous: true,
    interimResults: true,
    maxAlternatives: 1,
    ...config,
  };

  useEffect(() => {
    // Initialize speech recognition
    const SpeechRecognition =
      (window as any).SpeechRecognition ||
      (window as any).webkitSpeechRecognition;

    if (!SpeechRecognition) {
      const errorMsg = 'Speech recognition not supported in this browser';
      setError(errorMsg);
      onError?.(errorMsg);
      return;
    }

    const recognition = new SpeechRecognition();
    recognition.lang = defaultConfig.language;
    recognition.continuous = defaultConfig.continuous;
    recognition.interimResults = defaultConfig.interimResults;
    recognition.maxAlternatives = defaultConfig.maxAlternatives;

    recognition.onresult = (event: any) => {
      let interimText = '';
      let finalText = '';

      for (let i = event.resultIndex; i < event.results.length; i++) {
        const result = event.results[i];
        const text = result[0].transcript;

        if (result.isFinal) {
          finalText += text;
        } else {
          interimText += text;
        }
      }

      if (finalText) {
        setTranscript((prev) => prev + finalText);
        const transcriptData: VoiceTranscript = {
          text: finalText,
          confidence: event.results[event.results.length - 1][0].confidence,
          isFinal: true,
          timestamp: Date.now(),
        };
        onTranscript(transcriptData);
        HapticFeedback.light();
      }

      if (interimText) {
        setInterimTranscript(interimText);
        const transcriptData: VoiceTranscript = {
          text: interimText,
          confidence: 0,
          isFinal: false,
          timestamp: Date.now(),
        };
        onTranscript(transcriptData);
      }
    };

    recognition.onerror = (event: any) => {
      const errorMsg = `Speech recognition error: ${event.error}`;
      setError(errorMsg);
      onError?.(errorMsg);
      setIsListening(false);
      HapticFeedback.error();
    };

    recognition.onend = () => {
      if (isListening) {
        // Restart if continuous
        recognition.start();
      }
    };

    recognitionRef.current = recognition;

    return () => {
      if (recognitionRef.current) {
        recognitionRef.current.stop();
      }
      if (audioContextRef.current) {
        audioContextRef.current.close();
      }
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  const startListening = async () => {
    if (!recognitionRef.current) return;

    try {
      // Initialize audio visualization
      if (showVisualizer) {
        const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
        const audioContext = new AudioContext();
        const analyser = audioContext.createAnalyser();
        const microphone = audioContext.createMediaStreamSource(stream);

        analyser.fftSize = 256;
        microphone.connect(analyser);

        audioContextRef.current = audioContext;
        analyserRef.current = analyser;

        updateAudioLevel();
      }

      recognitionRef.current.start();
      setIsListening(true);
      setTranscript('');
      setInterimTranscript('');
      setError(null);
      HapticFeedback.medium();
    } catch (err) {
      const errorMsg = 'Failed to start voice recognition';
      setError(errorMsg);
      onError?.(errorMsg);
      HapticFeedback.error();
    }
  };

  const stopListening = () => {
    if (recognitionRef.current) {
      recognitionRef.current.stop();
    }

    if (audioContextRef.current) {
      audioContextRef.current.close();
      audioContextRef.current = null;
    }

    if (animationFrameRef.current) {
      cancelAnimationFrame(animationFrameRef.current);
      animationFrameRef.current = null;
    }

    setIsListening(false);
    setInterimTranscript('');
    setAudioLevel(0);
    HapticFeedback.medium();
  };

  const updateAudioLevel = () => {
    if (!analyserRef.current) return;

    const dataArray = new Uint8Array(analyserRef.current.frequencyBinCount);
    analyserRef.current.getByteFrequencyData(dataArray);

    const average = dataArray.reduce((a, b) => a + b) / dataArray.length;
    setAudioLevel(average / 255);

    animationFrameRef.current = requestAnimationFrame(updateAudioLevel);
  };

  const clear = () => {
    setTranscript('');
    setInterimTranscript('');
    HapticFeedback.light();
  };

  const containerStyles: CSSProperties = {
    display: 'flex',
    flexDirection: 'column',
    gap: '1rem',
    padding: '1rem',
    backgroundColor: '#ffffff',
    borderRadius: '12px',
    boxShadow: '0 2px 8px rgba(0, 0, 0, 0.1)',
  };

  const visualizerStyles: CSSProperties = {
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    height: '120px',
    backgroundColor: '#f8f8f8',
    borderRadius: '8px',
    position: 'relative',
    overflow: 'hidden',
  };

  const waveStyles: CSSProperties = {
    width: '120px',
    height: '120px',
    borderRadius: '50%',
    backgroundColor: isListening ? '#007AFF' : '#d0d0d0',
    opacity: isListening ? 0.3 + audioLevel * 0.7 : 0.3,
    transform: `scale(${isListening ? 1 + audioLevel * 0.5 : 1})`,
    transition: isListening ? 'none' : 'all 0.3s ease',
  };

  const transcriptStyles: CSSProperties = {
    minHeight: '60px',
    padding: '0.75rem',
    backgroundColor: '#f8f8f8',
    borderRadius: '8px',
    fontSize: '1rem',
    lineHeight: 1.5,
    overflow: 'auto',
  };

  const controlsStyles: CSSProperties = {
    display: 'flex',
    gap: '0.75rem',
    justifyContent: 'center',
  };

  const buttonStyles = (variant: 'primary' | 'secondary'): CSSProperties => ({
    padding: '0.75rem 1.5rem',
    fontSize: '1rem',
    fontWeight: 500,
    border: 'none',
    borderRadius: '8px',
    backgroundColor: variant === 'primary' ? '#007AFF' : '#f0f0f0',
    color: variant === 'primary' ? '#ffffff' : '#000000',
    cursor: 'pointer',
    minWidth: '44px',
    minHeight: '44px',
    transition: 'opacity 0.2s ease',
  });

  const micButtonStyles: CSSProperties = {
    width: '64px',
    height: '64px',
    borderRadius: '50%',
    border: 'none',
    backgroundColor: isListening ? '#FF3B30' : '#007AFF',
    color: '#ffffff',
    fontSize: '2rem',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    cursor: 'pointer',
    transition: 'transform 0.2s ease',
  };

  return (
    <div
      className={`voice-input ${className}`}
      style={containerStyles}
      data-testid="voice-input"
    >
      {error && (
        <div style={{ color: '#FF3B30', fontSize: '0.875rem', textAlign: 'center' }}>
          {error}
        </div>
      )}

      {showVisualizer && (
        <div className="voice-input__visualizer" style={visualizerStyles}>
          <div className="voice-input__wave" style={waveStyles} />
        </div>
      )}

      <div className="voice-input__transcript" style={transcriptStyles}>
        {transcript && <span style={{ color: '#000000' }}>{transcript}</span>}
        {interimTranscript && (
          <span style={{ color: '#8E8E93', fontStyle: 'italic' }}>
            {interimTranscript}
          </span>
        )}
        {!transcript && !interimTranscript && (
          <span style={{ color: '#a0a0a0' }}>
            {isListening ? 'Listening...' : 'Press microphone to start'}
          </span>
        )}
      </div>

      <div className="voice-input__controls" style={controlsStyles}>
        <button
          className="voice-input__mic"
          style={micButtonStyles}
          onClick={isListening ? stopListening : startListening}
          type="button"
          aria-label={isListening ? 'Stop listening' : 'Start listening'}
        >
          {isListening ? '⏹' : '🎤'}
        </button>
      </div>

      {transcript && (
        <button
          className="voice-input__clear"
          style={buttonStyles('secondary')}
          onClick={clear}
          type="button"
        >
          Clear
        </button>
      )}

      <style>{`
        .voice-input button:active {
          opacity: 0.8;
        }

        .voice-input__mic:active {
          transform: scale(0.95);
        }

        .voice-input button:focus-visible {
          outline: 2px solid #007AFF;
          outline-offset: 2px;
        }

        /* Dark mode support */
        @media (prefers-color-scheme: dark) {
          .voice-input {
            background-color: #1c1c1e;
          }

          .voice-input__visualizer,
          .voice-input__transcript {
            background-color: #2c2c2e;
          }

          .voice-input__transcript {
            color: #ffffff;
          }
        }

        /* Reduce motion */
        @media (prefers-reduced-motion: reduce) {
          .voice-input__wave,
          .voice-input button {
            transition: none !important;
          }
        }
      `}</style>
    </div>
  );
};

export default VoiceInput;
