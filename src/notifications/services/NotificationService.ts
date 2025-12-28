// Notification API service

import {
  Notification,
  NotificationFilter,
  NotificationPage,
  NotificationPreferences,
  NotificationStats,
} from '../types';

const API_BASE_URL = process.env.REACT_APP_API_URL || 'http://localhost:8080/api';

export class NotificationService {
  private baseUrl: string;
  private token: string | null = null;

  constructor(baseUrl: string = API_BASE_URL) {
    this.baseUrl = baseUrl;
  }

  /**
   * Set authentication token
   */
  setToken(token: string) {
    this.token = token;
  }

  /**
   * Get request headers
   */
  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.token) {
      headers['Authorization'] = `Bearer ${this.token}`;
    }

    return headers;
  }

  /**
   * Handle API response
   */
  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: response.statusText }));
      throw new Error(error.message || 'API request failed');
    }

    return response.json();
  }

  /**
   * Get notifications for current user
   */
  async getNotifications(
    page: number = 1,
    pageSize: number = 20,
    filter?: NotificationFilter
  ): Promise<NotificationPage> {
    const params = new URLSearchParams({
      page: page.toString(),
      pageSize: pageSize.toString(),
    });

    if (filter) {
      if (filter.level) params.append('level', filter.level.join(','));
      if (filter.category) params.append('category', filter.category.join(','));
      if (filter.read !== undefined) params.append('read', filter.read.toString());
      if (filter.archived !== undefined) params.append('archived', filter.archived.toString());
      if (filter.startDate) params.append('startDate', filter.startDate);
      if (filter.endDate) params.append('endDate', filter.endDate);
    }

    const response = await fetch(`${this.baseUrl}/notifications?${params}`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<NotificationPage>(response);
  }

  /**
   * Get unread notifications
   */
  async getUnread(limit: number = 20): Promise<Notification[]> {
    const response = await fetch(`${this.baseUrl}/notifications/unread?limit=${limit}`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<Notification[]>(response);
  }

  /**
   * Get notification by ID
   */
  async getNotification(id: string): Promise<Notification> {
    const response = await fetch(`${this.baseUrl}/notifications/${id}`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<Notification>(response);
  }

  /**
   * Mark notification as read
   */
  async markRead(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/notifications/${id}/read`, {
      method: 'POST',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Mark notification as unread
   */
  async markUnread(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/notifications/${id}/unread`, {
      method: 'POST',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Mark all notifications as read
   */
  async markAllRead(): Promise<{ count: number }> {
    const response = await fetch(`${this.baseUrl}/notifications/read-all`, {
      method: 'POST',
      headers: this.getHeaders(),
    });

    return this.handleResponse<{ count: number }>(response);
  }

  /**
   * Archive notification
   */
  async archive(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/notifications/${id}/archive`, {
      method: 'POST',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Delete notification
   */
  async delete(id: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/notifications/${id}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Get notification statistics
   */
  async getStats(): Promise<NotificationStats> {
    const response = await fetch(`${this.baseUrl}/notifications/stats`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<NotificationStats>(response);
  }

  /**
   * Get user preferences
   */
  async getPreferences(): Promise<NotificationPreferences> {
    const response = await fetch(`${this.baseUrl}/notifications/preferences`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<NotificationPreferences>(response);
  }

  /**
   * Update user preferences
   */
  async updatePreferences(preferences: Partial<NotificationPreferences>): Promise<void> {
    const response = await fetch(`${this.baseUrl}/notifications/preferences`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: JSON.stringify(preferences),
    });

    await this.handleResponse<void>(response);
  }

  /**
   * Send a notification
   */
  async send(
    notification: Omit<Notification, 'id' | 'createdAt' | 'read' | 'archived'>,
    channels: string[] = []
  ): Promise<{ id: string }> {
    const response = await fetch(`${this.baseUrl}/notifications`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ notification, channels }),
    });

    return this.handleResponse<{ id: string }>(response);
  }

  /**
   * Execute notification action
   */
  async executeAction(notificationId: string, actionId: string): Promise<void> {
    const response = await fetch(
      `${this.baseUrl}/notifications/${notificationId}/actions/${actionId}`,
      {
        method: 'POST',
        headers: this.getHeaders(),
      }
    );

    await this.handleResponse<void>(response);
  }

  /**
   * Get WebSocket URL
   */
  getWebSocketUrl(): string {
    const wsProtocol = this.baseUrl.startsWith('https') ? 'wss' : 'ws';
    const baseUrlWithoutProtocol = this.baseUrl.replace(/^https?:\/\//, '');
    return `${wsProtocol}://${baseUrlWithoutProtocol}/ws/notifications`;
  }
}

// Export singleton instance
export const notificationService = new NotificationService();
