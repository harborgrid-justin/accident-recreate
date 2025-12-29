/**
 * AccuScene Enterprise v0.3.0 - Selection Highlight
 *
 * Remote selection highlighting for collaborative editing
 */

import React from 'react';
import { Presence, UserId } from './types';

interface SelectionHighlightProps {
  presence: Presence;
}

export const SelectionHighlight: React.FC<SelectionHighlightProps> = ({ presence }) => {
  if (!presence.selection || !presence.selection.bounds) return null;

  const { bounds } = presence.selection;

  return (
    <div
      style={{
        position: 'absolute',
        left: bounds.min.x,
        top: bounds.min.y,
        width: bounds.max.x - bounds.min.x,
        height: bounds.max.y - bounds.min.y,
        border: `2px solid ${presence.user.color}`,
        borderRadius: '4px',
        backgroundColor: `${presence.user.color}20`,
        pointerEvents: 'none',
        zIndex: 999
      }}
    >
      <div
        style={{
          position: 'absolute',
          top: -24,
          left: 0,
          backgroundColor: presence.user.color,
          color: 'white',
          padding: '2px 8px',
          borderRadius: '4px 4px 0 0',
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

interface SelectionOverlayProps {
  presences: Presence[];
  currentUserId?: UserId;
}

export const SelectionOverlay: React.FC<SelectionOverlayProps> = ({ presences, currentUserId }) => {
  return (
    <div style={{ position: 'absolute', inset: 0, pointerEvents: 'none', overflow: 'hidden' }}>
      {presences
        .filter(p => p.userId !== currentUserId && p.selection)
        .map(presence => (
          <SelectionHighlight key={presence.userId} presence={presence} />
        ))}
    </div>
  );
};
