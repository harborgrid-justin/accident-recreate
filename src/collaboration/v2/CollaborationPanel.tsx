/**
 * AccuScene Enterprise v0.3.0 - Collaboration Panel
 *
 * Main collaboration sidebar panel with tabs
 */

import React, { useState } from 'react';
import { Presence, ChatMessage, CollaborationTab } from './types';
import { UserListItem } from './UserAvatars';

interface CollaborationPanelProps {
  presences: Presence[];
  messages: ChatMessage[];
  onSendMessage?: (content: string) => void;
  defaultTab?: CollaborationTab;
}

export const CollaborationPanel: React.FC<CollaborationPanelProps> = ({
  presences,
  messages,
  onSendMessage,
  defaultTab = CollaborationTab.PARTICIPANTS
}) => {
  const [activeTab, setActiveTab] = useState<CollaborationTab>(defaultTab);
  const [messageInput, setMessageInput] = useState('');

  const handleSendMessage = () => {
    if (messageInput.trim() && onSendMessage) {
      onSendMessage(messageInput);
      setMessageInput('');
    }
  };

  const tabs = [
    { id: CollaborationTab.PARTICIPANTS, label: 'Participants', count: presences.length },
    { id: CollaborationTab.CHAT, label: 'Chat', count: messages.length },
    { id: CollaborationTab.HISTORY, label: 'History' },
    { id: CollaborationTab.BRANCHES, label: 'Branches' },
    { id: CollaborationTab.ANNOTATIONS, label: 'Annotations' },
    { id: CollaborationTab.SETTINGS, label: 'Settings' }
  ];

  return (
    <div style={{
      width: '320px',
      height: '100%',
      backgroundColor: 'white',
      borderLeft: '1px solid #e5e7eb',
      display: 'flex',
      flexDirection: 'column'
    }}>
      {/* Tabs */}
      <div style={{
        display: 'flex',
        borderBottom: '1px solid #e5e7eb',
        overflowX: 'auto'
      }}>
        {tabs.map(tab => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            style={{
              padding: '12px 16px',
              border: 'none',
              background: 'none',
              borderBottom: activeTab === tab.id ? '2px solid #3b82f6' : '2px solid transparent',
              color: activeTab === tab.id ? '#3b82f6' : '#6b7280',
              fontWeight: activeTab === tab.id ? 600 : 400,
              cursor: 'pointer',
              whiteSpace: 'nowrap'
            }}
          >
            {tab.label}
            {tab.count !== undefined && (
              <span style={{
                marginLeft: '4px',
                padding: '2px 6px',
                backgroundColor: '#f3f4f6',
                borderRadius: '10px',
                fontSize: '11px'
              }}>
                {tab.count}
              </span>
            )}
          </button>
        ))}
      </div>

      {/* Content */}
      <div style={{ flex: 1, overflow: 'auto', padding: '16px' }}>
        {activeTab === CollaborationTab.PARTICIPANTS && (
          <div>
            {presences.map(presence => (
              <UserListItem key={presence.userId} presence={presence} />
            ))}
          </div>
        )}

        {activeTab === CollaborationTab.CHAT && (
          <div style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ flex: 1, overflow: 'auto', marginBottom: '16px' }}>
              {messages.map(message => (
                <div key={message.id} style={{ marginBottom: '12px' }}>
                  <div style={{ fontSize: '12px', color: '#6b7280', marginBottom: '4px' }}>
                    {message.userId} • {new Date(message.timestamp).toLocaleTimeString()}
                  </div>
                  <div>{message.content}</div>
                </div>
              ))}
            </div>
            <div style={{ display: 'flex', gap: '8px' }}>
              <input
                type="text"
                value={messageInput}
                onChange={(e) => setMessageInput(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && handleSendMessage()}
                placeholder="Type a message..."
                style={{
                  flex: 1,
                  padding: '8px 12px',
                  border: '1px solid #e5e7eb',
                  borderRadius: '6px',
                  outline: 'none'
                }}
              />
              <button
                onClick={handleSendMessage}
                style={{
                  padding: '8px 16px',
                  backgroundColor: '#3b82f6',
                  color: 'white',
                  border: 'none',
                  borderRadius: '6px',
                  cursor: 'pointer'
                }}
              >
                Send
              </button>
            </div>
          </div>
        )}

        {/* Other tabs would go here */}
      </div>
    </div>
  );
};
