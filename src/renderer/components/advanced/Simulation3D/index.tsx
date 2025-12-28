/**
 * Simulation3D Component
 * Main 3D simulation viewer
 */

import React, { useEffect, useState, useCallback } from 'react';
import { Simulation3DProps } from '../types';
import './Simulation3D.css';

export const Simulation3D: React.FC<Simulation3DProps> = ({
  simulationData,
  currentTime,
  playing = false,
  playbackSpeed = 1,
  onTimeChange,
  onPlayingChange,
}) => {
  const [interpolatedStates, setInterpolatedStates] = useState<Map<string, any>>(new Map());

  // Interpolate vehicle positions based on current time
  const interpolateVehicleStates = useCallback(() => {
    const states = new Map();

    simulationData.vehicles.forEach(vehicle => {
      const { keyframes } = vehicle;

      // Find surrounding keyframes
      let prevKeyframe = keyframes[0];
      let nextKeyframe = keyframes[keyframes.length - 1];

      for (let i = 0; i < keyframes.length - 1; i++) {
        if (
          keyframes[i].timestamp <= currentTime &&
          keyframes[i + 1].timestamp > currentTime
        ) {
          prevKeyframe = keyframes[i];
          nextKeyframe = keyframes[i + 1];
          break;
        }
      }

      // Interpolate between keyframes
      const t =
        (currentTime - prevKeyframe.timestamp) /
        (nextKeyframe.timestamp - prevKeyframe.timestamp);

      const interpolatedState = {
        position: [
          prevKeyframe.position[0] + (nextKeyframe.position[0] - prevKeyframe.position[0]) * t,
          prevKeyframe.position[1] + (nextKeyframe.position[1] - prevKeyframe.position[1]) * t,
          prevKeyframe.position[2] + (nextKeyframe.position[2] - prevKeyframe.position[2]) * t,
        ],
        rotation: [
          prevKeyframe.rotation[0] + (nextKeyframe.rotation[0] - prevKeyframe.rotation[0]) * t,
          prevKeyframe.rotation[1] + (nextKeyframe.rotation[1] - prevKeyframe.rotation[1]) * t,
          prevKeyframe.rotation[2] + (nextKeyframe.rotation[2] - prevKeyframe.rotation[2]) * t,
        ],
        velocity:
          prevKeyframe.velocity + (nextKeyframe.velocity - prevKeyframe.velocity) * t,
      };

      states.set(vehicle.vehicleId, interpolatedState);
    });

    setInterpolatedStates(states);
  }, [simulationData, currentTime]);

  useEffect(() => {
    interpolateVehicleStates();
  }, [interpolateVehicleStates]);

  // Playback loop
  useEffect(() => {
    if (!playing) return;

    const interval = setInterval(() => {
      const newTime = currentTime + (1 / simulationData.frameRate) * playbackSpeed;

      if (newTime >= simulationData.duration) {
        onPlayingChange?.(false);
        onTimeChange?.(simulationData.duration);
      } else {
        onTimeChange?.(newTime);
      }
    }, (1000 / simulationData.frameRate) / playbackSpeed);

    return () => clearInterval(interval);
  }, [playing, currentTime, playbackSpeed, simulationData, onTimeChange, onPlayingChange]);

  return (
    <div className="simulation-3d-container">
      <div className="simulation-viewport">
        {/* Scene would render here */}
        <div className="simulation-overlay">
          <div className="simulation-time">
            Time: {currentTime.toFixed(2)}s / {simulationData.duration.toFixed(2)}s
          </div>
          {playing && <div className="simulation-playing">Playing at {playbackSpeed}x</div>}
        </div>
      </div>

      <div className="simulation-info">
        <div className="simulation-vehicles">
          <h4>Vehicles</h4>
          {simulationData.vehicles.map(vehicle => {
            const state = interpolatedStates.get(vehicle.vehicleId);
            return (
              <div key={vehicle.vehicleId} className="vehicle-info">
                <div className="vehicle-name">{vehicle.vehicleId}</div>
                {state && (
                  <div className="vehicle-state">
                    <small>Velocity: {state.velocity.toFixed(2)} m/s</small>
                  </div>
                )}
              </div>
            );
          })}
        </div>

        <div className="simulation-events">
          <h4>Events</h4>
          {simulationData.events
            .filter(event => Math.abs(event.timestamp - currentTime) < 0.1)
            .map((event, index) => (
              <div key={index} className="event-notification">
                <strong>{event.type}</strong>
                <p>{event.description}</p>
              </div>
            ))}
        </div>
      </div>
    </div>
  );
};

export default Simulation3D;
