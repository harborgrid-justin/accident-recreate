/**
 * AccuScene Enterprise v0.3.0 - User Avatars
 *
 * Collaborator avatars with status indicators
 */

import React from 'react';
import { User, UserStatus, Presence } from './types';

interface UserAvatarProps {
  user: User;
  status?: UserStatus;
  size?: number;
  showStatus?: boolean;
}

export const UserAvatar: React.FC<UserAvatarProps> = ({
  user,
  status = UserStatus.ONLINE,
  size = 32,
  showStatus = true
}) => {
  const getStatusColor = () => {
    switch (status) {
      case UserStatus.ONLINE: return '#10b981';
      case UserStatus.AWAY: return '#f59e0b';
      case UserStatus.BUSY: return '#ef4444';
      case UserStatus.OFFLINE: return '#6b7280';
      default: return '#6b7280';
    }
  };

  const initials = user.name
    .split(' ')
    .map(n => n[0])
    .join('')
    .toUpperCase()
    .substring(0, 2);

  return (
    <div style={{ position: 'relative', display: 'inline-block' }}>
      {user.avatar ? (
        <img
          src={user.avatar}
          alt={user.name}
          style={{
            width: size,
            height: size,
            borderRadius: '50%',
            border: `2px solid ${user.color}`
          }}
        />
      ) : (
        <div
          style={{
            width: size,
            height: size,
            borderRadius: '50%',
            backgroundColor: user.color,
            color: '#ffffff',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: size * 0.4,
            fontWeight: 600,
            border: `2px solid ${user.color}`
          }}
        >
          {initials}
        </div>
      )}
      {showStatus && (
        <div
          style={{
            position: 'absolute',
            bottom: 0,
            right: 0,
            width: size * 0.3,
            height: size * 0.3,
            borderRadius: '50%',
            backgroundColor: getStatusColor(),
            border: '2px solid white'
          }}
        />
      )}
    </div>
  );
};

interface UserAvatarStackProps {
  presences: Presence[];
  max?: number;
  size?: number;
}

export const UserAvatarStack: React.FC<UserAvatarStackProps> = ({
  presences,
  max = 5,
  size = 32
}) => {
  const visible = presences.slice(0, max);
  const remaining = presences.length - max;

  return (
    <div style={{ display: 'flex', alignItems: 'center' }}>
      {visible.map((presence, index) => (
        <div
          key={presence.userId}
          style={{ marginLeft: index > 0 ? -size * 0.3 : 0 }}
        >
          <UserAvatar
            user={presence.user}
            status={presence.status}
            size={size}
          />
        </div>
      ))}
      {remaining > 0 && (
        <div
          style={{
            width: size,
            height: size,
            borderRadius: '50%',
            backgroundColor: '#6b7280',
            color: '#ffffff',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontSize: size * 0.4,
            fontWeight: 600,
            marginLeft: -size * 0.3
          }}
        >
          +{remaining}
        </div>
      )}
    </div>
  );
};

interface UserListItemProps {
  presence: Presence;
  showActivity?: boolean;
}

export const UserListItem: React.FC<UserListItemProps> = ({ presence, showActivity = true }) => {
  return (
    <div style={{
      display: 'flex',
      alignItems: 'center',
      gap: '12px',
      padding: '8px',
      borderRadius: '6px',
      hover: { backgroundColor: '#f3f4f6' }
    }}>
      <UserAvatar user={presence.user} status={presence.status} size={40} />
      <div style={{ flex: 1 }}>
        <div style={{ fontWeight: 500 }}>{presence.user.name}</div>
        <div style={{ fontSize: '12px', color: '#6b7280' }}>
          {presence.isTyping ? 'Typing...' : presence.currentTool || 'Idle'}
        </div>
      </div>
      {showActivity && presence.status === UserStatus.ONLINE && (
        <div style={{ fontSize: '12px', color: '#10b981' }}>Active</div>
      )}
    </div>
  );
};
