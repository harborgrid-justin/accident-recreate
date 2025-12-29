/**
 * AccuScene Enterprise v0.3.0 - Cursor Overlay
 *
 * Remote cursor visualization with smooth interpolation
 */

import React, { useEffect, useRef } from 'react';
import { Presence, UserId } from './types';

interface CursorProps {
  presence: Presence;
}

export const Cursor: React.FC<CursorProps> = ({ presence }) => {
  const cursorRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (cursorRef.current && presence.cursor) {
      // Smooth transition
      cursorRef.current.style.transform = `translate(${presence.cursor.x}px, ${presence.cursor.y}px)`;
    }
  }, [presence.cursor]);

  if (!presence.cursor) return null;

  return (
    <div
      ref={cursorRef}
      style={{
        position: 'absolute',
        left: 0,
        top: 0,
        pointerEvents: 'none',
        transition: 'transform 100ms ease-out',
        zIndex: 9999
      }}
    >
      {/* Cursor pointer */}
      <svg width="24" height="24" viewBox="0 0 24 24" fill="none">
        <path
          d="M5 3L19 12L12 13L9 20L5 3Z"
          fill={presence.user.color}
          stroke="white"
          strokeWidth="1.5"
        />
      </svg>
      {/* User label */}
      <div
        style={{
          position: 'absolute',
          left: '24px',
          top: '0px',
          backgroundColor: presence.user.color,
          color: 'white',
          padding: '2px 8px',
          borderRadius: '4px',
          fontSize: '12px',
          fontWeight: 500,
          whiteSpace: 'nowrap'
        }}
      >
        {presence.user.name}
      </div>
    </div>
  );
};

interface CursorOverlayProps {
  presences: Presence[];
  currentUserId?: UserId;
}

export const CursorOverlay: React.FC<CursorOverlayProps> = ({ presences, currentUserId }) => {
  return (
    <div
      style={{
        position: 'absolute',
        inset: 0,
        pointerEvents: 'none',
        overflow: 'hidden'
      }}
    >
      {presences
        .filter(p => p.userId !== currentUserId && p.cursor)
        .map(presence => (
          <Cursor key={presence.userId} presence={presence} />
        ))}
    </div>
  );
};
