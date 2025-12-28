/**
 * AR Markers Component
 * AR marker detection and visualization
 */

import React, { useEffect, useRef } from 'react';
import { ARMarkersProps } from '../types';
import './Markers.css';

export const ARMarkers: React.FC<ARMarkersProps> = ({
  markers,
  showLabels = true,
  onMarkerDetected,
}) => {
  const markersRef = useRef<Map<string, any>>(new Map());

  useEffect(() => {
    // Initialize marker tracking
    markers.forEach(marker => {
      if (!markersRef.current.has(marker.id)) {
        // In production, this would set up actual marker tracking
        markersRef.current.set(marker.id, {
          detected: false,
          position: marker.position,
        });
      }
    });

    // Cleanup removed markers
    const markerIds = new Set(markers.map(m => m.id));
    markersRef.current.forEach((_, id) => {
      if (!markerIds.has(id)) {
        markersRef.current.delete(id);
      }
    });
  }, [markers]);

  return (
    <div className="ar-markers">
      {markers.map(marker => (
        <div
          key={marker.id}
          className={`ar-marker ar-marker-${marker.type}`}
          style={{
            left: `${marker.position[0]}px`,
            top: `${marker.position[1]}px`,
          }}
        >
          {marker.imageUrl && (
            <img
              src={marker.imageUrl}
              alt={`Marker ${marker.id}`}
              className="marker-image"
              style={{
                width: `${marker.scale * 100}px`,
                height: `${marker.scale * 100}px`,
              }}
            />
          )}

          {showLabels && (
            <div className="marker-label">
              {marker.id}
            </div>
          )}
        </div>
      ))}
    </div>
  );
};

export default ARMarkers;
