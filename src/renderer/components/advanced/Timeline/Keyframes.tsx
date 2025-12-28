/**
 * Keyframes Component
 * Keyframe editor for animations
 */

import React, { useState } from 'react';
import { KeyframesProps } from '../types';
import './Keyframes.css';

export const Keyframes: React.FC<KeyframesProps> = ({
  keyframes,
  currentTime,
  duration,
  onKeyframeAdd,
  onKeyframeRemove,
  onKeyframeUpdate,
  selectedProperty,
}) => {
  const [expandedKeyframe, setExpandedKeyframe] = useState<string | null>(null);

  return (
    <div className="keyframes-editor">
      <div className="keyframes-header">
        <h4>Keyframes</h4>
        {selectedProperty && (
          <span className="selected-property">{selectedProperty}</span>
        )}
        {onKeyframeAdd && (
          <button
            className="add-keyframe-btn"
            onClick={() => onKeyframeAdd({
              id: `kf-${Date.now()}`,
              timestamp: currentTime,
              properties: {},
              interpolation: 'linear',
            })}
          >
            + Add Keyframe
          </button>
        )}
      </div>

      <div className="keyframes-track">
        <div className="track-timeline">
          {keyframes.map(keyframe => {
            const position = (keyframe.timestamp / duration) * 100;
            const isActive = Math.abs(keyframe.timestamp - currentTime) < 0.1;

            return (
              <div
                key={keyframe.id}
                className={`keyframe-marker ${isActive ? 'active' : ''}`}
                style={{ left: `${position}%` }}
                onClick={() => setExpandedKeyframe(
                  expandedKeyframe === keyframe.id ? null : keyframe.id
                )}
              >
                <div className="marker-diamond" />
                {expandedKeyframe === keyframe.id && (
                  <div className="keyframe-details">
                    <div className="details-header">
                      <strong>Keyframe</strong>
                      {onKeyframeRemove && (
                        <button
                          className="remove-keyframe-btn"
                          onClick={(e) => {
                            e.stopPropagation();
                            onKeyframeRemove(keyframe.id);
                            setExpandedKeyframe(null);
                          }}
                        >
                          ×
                        </button>
                      )}
                    </div>

                    <div className="details-content">
                      <div className="detail-row">
                        <label>Time:</label>
                        <span>{keyframe.timestamp.toFixed(2)}s</span>
                      </div>

                      <div className="detail-row">
                        <label>Interpolation:</label>
                        <select
                          value={keyframe.interpolation || 'linear'}
                          onChange={(e) => onKeyframeUpdate?.({
                            ...keyframe,
                            interpolation: e.target.value as any,
                          })}
                        >
                          <option value="linear">Linear</option>
                          <option value="ease">Ease</option>
                          <option value="ease-in">Ease In</option>
                          <option value="ease-out">Ease Out</option>
                          <option value="ease-in-out">Ease In-Out</option>
                        </select>
                      </div>

                      <div className="detail-row">
                        <label>Properties:</label>
                        <div className="properties-list">
                          {Object.entries(keyframe.properties).map(([key, value]) => (
                            <div key={key} className="property-item">
                              <span className="property-key">{key}:</span>
                              <span className="property-value">{JSON.stringify(value)}</span>
                            </div>
                          ))}
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </div>
            );
          })}
        </div>
      </div>

      <div className="keyframes-list">
        {keyframes.map(keyframe => {
          const isActive = Math.abs(keyframe.timestamp - currentTime) < 0.1;

          return (
            <div
              key={keyframe.id}
              className={`keyframe-item ${isActive ? 'active' : ''}`}
            >
              <div className="keyframe-time">{keyframe.timestamp.toFixed(2)}s</div>
              <div className="keyframe-interp">{keyframe.interpolation || 'linear'}</div>
              <div className="keyframe-props">
                {Object.keys(keyframe.properties).length} properties
              </div>
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default Keyframes;
