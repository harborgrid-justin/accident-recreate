/**
 * Loading Component - Loading spinner and overlay
 */

import React from 'react';

export interface LoadingProps {
  message?: string;
  fullScreen?: boolean;
  size?: 'small' | 'medium' | 'large';
}

export const Loading: React.FC<LoadingProps> = ({
  message,
  fullScreen = false,
  size = 'medium',
}) => {
  if (fullScreen) {
    return (
      <div className="loading-overlay">
        <div className="loading-content">
          <div className={`loading-spinner loading-spinner-${size}`}></div>
          {message && <p className="loading-message">{message}</p>}
        </div>
      </div>
    );
  }

  return (
    <div className="loading-inline">
      <div className={`loading-spinner loading-spinner-${size}`}></div>
      {message && <span className="loading-message">{message}</span>}
    </div>
  );
};

export default Loading;
