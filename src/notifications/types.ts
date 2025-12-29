// TypeScript interfaces for notifications

export enum NotificationLevel {
  Info = 'info',
  Success = 'success',
  Warning = 'warning',
  Error = 'error',
  Alert = 'alert',
}

export enum Priority {
  Low = 1,
  Normal = 2,
  High = 3,
  Urgent = 4,
  Critical = 5,
}

export enum NotificationCategory {
  System = 'system',
  Case = 'case',
  Collaboration = 'collaboration',
  Analysis = 'analysis',
  Report = 'report',
  Security = 'security',
  Billing = 'billing',
  Marketing = 'marketing',
}

export enum DeliveryState {
  Pending = 'pending',
  Processing = 'processing',
  Delivered = 'delivered',
  Failed = 'failed',
  Cancelled = 'cancelled',
}

export interface NotificationAction {
  id: string;
  label: string;
  url?: string;
  actionType: 'navigate' | 'api_call' | 'dismiss' | string;
  metadata: Record<string, any>;
}

export interface NotificationSender {
  id: string;
  name: string;
  avatarUrl?: string;
  senderType: 'user' | 'system' | 'agent' | 'integration';
}

export interface Notification {
  id: string;
  userId: string;
  organizationId?: string;
  level: NotificationLevel;
  priority: Priority;
  category: NotificationCategory;
  title: string;
  message: string;
  htmlMessage?: string;
  actions: NotificationAction[];
  metadata: Record<string, any>;
  relatedEntityId?: string;
  relatedEntityType?: string;
  read: boolean;
  readAt?: string;
  archived: boolean;
  createdAt: string;
  expiresAt?: string;
  sender?: NotificationSender;
  templateId?: string;
  templateVars?: Record<string, any>;
}

export interface DeliveryStatus {
  notificationId: string;
  channel: string;
  status: DeliveryState;
  attempts: number;
  lastAttemptAt?: string;
  deliveredAt?: string;
  errorMessage?: string;
}

export interface NotificationStats {
  total: number;
  unread: number;
  byLevel: Record<string, number>;
  byCategory: Record<string, number>;
  byChannel: Record<string, number>;
}

export interface NotificationPreferences {
  userId: string;
  enabledChannels: string[];
  quietHours?: QuietHours;
  categoryPreferences: Record<string, CategoryPreference>;
  levelPreferences: Record<string, boolean>;
  digestEnabled: boolean;
  digestFrequency: 'hourly' | 'daily' | 'weekly';
}

export interface QuietHours {
  enabled: boolean;
  startHour: number; // 0-23
  endHour: number; // 0-23
  timezone: string;
  days: number[]; // 0-6 (Sunday-Saturday)
}

export interface CategoryPreference {
  enabled: boolean;
  channels: string[];
  minPriority: number; // 1-5
}

export interface NotificationFilter {
  level?: NotificationLevel[];
  category?: NotificationCategory[];
  read?: boolean;
  archived?: boolean;
  startDate?: string;
  endDate?: string;
}

export interface NotificationPage {
  notifications: Notification[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface WebSocketMessage {
  type: 'notification' | 'read' | 'archived' | 'deleted' | 'stats';
  payload: any;
}
