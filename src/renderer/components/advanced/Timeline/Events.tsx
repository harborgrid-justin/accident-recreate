/**
 * Events Component
 * Timeline event management
 */

import React from 'react';
import { EventsProps } from '../types';
import './Events.css';

export const Events: React.FC<EventsProps> = ({
  events,
  currentTime,
  onEventClick,
  editable = false,
  onEventAdd,
  onEventRemove,
  onEventUpdate,
}) => {
  return (
    <div className="timeline-events">
      <div className="events-header">
        <h4>Events</h4>
        {editable && onEventAdd && (
          <button
            className="add-event-btn"
            onClick={() => onEventAdd(currentTime)}
          >
            + Add Event
          </button>
        )}
      </div>

      <div className="events-list">
        {events.length === 0 ? (
          <div className="events-empty">No events recorded</div>
        ) : (
          events.map(event => (
            <div
              key={event.id}
              className={`event-item ${Math.abs(event.timestamp - currentTime) < 0.1 ? 'active' : ''}`}
              onClick={() => onEventClick?.(event)}
            >
              <div className="event-icon" style={{ backgroundColor: event.color || '#0088ff' }}>
                {event.icon || '•'}
              </div>

              <div className="event-info">
                <div className="event-title">{event.title}</div>
                <div className="event-meta">
                  <span className="event-time">{event.timestamp.toFixed(2)}s</span>
                  <span className="event-type">{event.type}</span>
                </div>
                {event.description && (
                  <div className="event-description">{event.description}</div>
                )}
              </div>

              {editable && onEventRemove && (
                <button
                  className="remove-event-btn"
                  onClick={(e) => {
                    e.stopPropagation();
                    onEventRemove(event.id);
                  }}
                >
                  ×
                </button>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default Events;
