/**
 * AR Overlay Component
 * Overlay UI elements for AR view
 */

import React from 'react';
import { AROverlayProps } from '../types';
import './Overlay.css';

export const AROverlay: React.FC<AROverlayProps> = ({
  visible,
  opacity = 0.8,
  elements = [],
}) => {
  if (!visible) return null;

  return (
    <div className="ar-overlay" style={{ opacity }}>
      {elements.map(element => (
        <div
          key={element.id}
          className={`ar-element ar-element-${element.type}`}
          style={{
            left: `${element.position[0]}px`,
            top: `${element.position[1]}px`,
            color: element.color || '#fff',
          }}
        >
          {typeof element.content === 'string' ? (
            <span>{element.content}</span>
          ) : (
            element.content
          )}
        </div>
      ))}
    </div>
  );
};

export default AROverlay;
