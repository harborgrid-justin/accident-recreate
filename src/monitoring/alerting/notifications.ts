/**
 * AccuScene Enterprise v0.2.0
 * Alert Notifications
 *
 * Notification system for alerts
 */

import { Alert, AlertNotification, AlertHandler, AlertSeverity } from '../types';

/**
 * Alert notification manager
 */
export class AlertNotificationManager {
  private handlers: Map<string, AlertHandler> = new Map();
  private notificationQueue: AlertNotification[] = [];
  private isProcessing: boolean = false;

  /**
   * Register a notification handler
   */
  public registerHandler(name: string, handler: AlertHandler): void {
    this.handlers.set(name, handler);
  }

  /**
   * Unregister a notification handler
   */
  public unregisterHandler(name: string): void {
    this.handlers.delete(name);
  }

  /**
   * Send alert notification
   */
  public async notify(alert: Alert): Promise<void> {
    const notification: AlertNotification = {
      alert,
      timestamp: Date.now()
    };

    this.notificationQueue.push(notification);

    if (!this.isProcessing) {
      await this.processQueue();
    }
  }

  /**
   * Process notification queue
   */
  private async processQueue(): Promise<void> {
    if (this.isProcessing || this.notificationQueue.length === 0) {
      return;
    }

    this.isProcessing = true;

    while (this.notificationQueue.length > 0) {
      const notification = this.notificationQueue.shift();

      if (notification) {
        await this.sendNotification(notification);
      }
    }

    this.isProcessing = false;
  }

  /**
   * Send notification to all handlers
   */
  private async sendNotification(notification: AlertNotification): Promise<void> {
    const promises: Promise<void>[] = [];

    for (const handler of this.handlers.values()) {
      promises.push(
        handler(notification).catch(error => {
          console.error('Notification handler failed:', error);
        })
      );
    }

    await Promise.all(promises);
  }

  /**
   * Get pending notification count
   */
  public getPendingCount(): number {
    return this.notificationQueue.length;
  }
}

/**
 * Console notification handler
 */
export const consoleNotificationHandler: AlertHandler = async (
  notification: AlertNotification
): Promise<void> => {
  const { alert } = notification;
  const severityEmoji = {
    [AlertSeverity.INFO]: 'ℹ️',
    [AlertSeverity.WARNING]: '⚠️',
    [AlertSeverity.ERROR]: '❌',
    [AlertSeverity.CRITICAL]: '🚨'
  };

  console.log(`
${severityEmoji[alert.rule.severity]} ALERT: ${alert.rule.name}
  Severity: ${alert.rule.severity}
  State: ${alert.state}
  Value: ${alert.value}
  Description: ${alert.rule.description}
  Started: ${new Date(alert.startsAt).toISOString()}
  ${alert.endsAt ? `Ended: ${new Date(alert.endsAt).toISOString()}` : ''}
  `);
};

/**
 * Email notification handler
 */
export class EmailNotificationHandler {
  private emailEndpoint: string;
  private from: string;
  private to: string[];

  constructor(emailEndpoint: string, from: string, to: string[]) {
    this.emailEndpoint = emailEndpoint;
    this.from = from;
    this.to = to;
  }

  public async send(notification: AlertNotification): Promise<void> {
    const { alert } = notification;

    const emailBody = {
      from: this.from,
      to: this.to,
      subject: `[${alert.rule.severity.toUpperCase()}] ${alert.rule.name}`,
      html: this.generateEmailHTML(alert)
    };

    try {
      const response = await fetch(this.emailEndpoint, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(emailBody)
      });

      if (!response.ok) {
        throw new Error(`Email notification failed: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to send email notification:', error);
      throw error;
    }
  }

  private generateEmailHTML(alert: Alert): string {
    const severityColor = {
      [AlertSeverity.INFO]: '#0066cc',
      [AlertSeverity.WARNING]: '#ff9900',
      [AlertSeverity.ERROR]: '#cc0000',
      [AlertSeverity.CRITICAL]: '#990000'
    };

    return `
      <html>
        <body style="font-family: Arial, sans-serif;">
          <h2 style="color: ${severityColor[alert.rule.severity]};">
            ${alert.rule.name}
          </h2>
          <p><strong>Severity:</strong> ${alert.rule.severity}</p>
          <p><strong>State:</strong> ${alert.state}</p>
          <p><strong>Value:</strong> ${alert.value}</p>
          <p><strong>Description:</strong> ${alert.rule.description}</p>
          <p><strong>Started:</strong> ${new Date(alert.startsAt).toISOString()}</p>
          ${alert.endsAt ? `<p><strong>Ended:</strong> ${new Date(alert.endsAt).toISOString()}</p>` : ''}
          ${Object.keys(alert.annotations).length > 0 ? `
            <h3>Details</h3>
            <ul>
              ${Object.entries(alert.annotations)
                .map(([key, value]) => `<li><strong>${key}:</strong> ${value}</li>`)
                .join('')}
            </ul>
          ` : ''}
        </body>
      </html>
    `;
  }
}

/**
 * Webhook notification handler
 */
export class WebhookNotificationHandler {
  private webhookUrl: string;
  private headers: Record<string, string>;

  constructor(webhookUrl: string, headers?: Record<string, string>) {
    this.webhookUrl = webhookUrl;
    this.headers = headers || {};
  }

  public async send(notification: AlertNotification): Promise<void> {
    const payload = this.formatPayload(notification);

    try {
      const response = await fetch(this.webhookUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...this.headers
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`Webhook notification failed: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to send webhook notification:', error);
      throw error;
    }
  }

  private formatPayload(notification: AlertNotification): any {
    const { alert } = notification;

    return {
      alert_id: alert.id,
      rule_name: alert.rule.name,
      severity: alert.rule.severity,
      state: alert.state,
      value: alert.value,
      description: alert.rule.description,
      starts_at: new Date(alert.startsAt).toISOString(),
      ends_at: alert.endsAt ? new Date(alert.endsAt).toISOString() : null,
      labels: alert.labels,
      annotations: alert.annotations,
      timestamp: new Date(notification.timestamp).toISOString()
    };
  }
}

/**
 * Slack notification handler
 */
export class SlackNotificationHandler {
  private webhookUrl: string;

  constructor(webhookUrl: string) {
    this.webhookUrl = webhookUrl;
  }

  public async send(notification: AlertNotification): Promise<void> {
    const { alert } = notification;

    const color = {
      [AlertSeverity.INFO]: '#36a64f',
      [AlertSeverity.WARNING]: '#ff9900',
      [AlertSeverity.ERROR]: '#cc0000',
      [AlertSeverity.CRITICAL]: '#990000'
    };

    const payload = {
      attachments: [
        {
          color: color[alert.rule.severity],
          title: alert.rule.name,
          text: alert.rule.description,
          fields: [
            {
              title: 'Severity',
              value: alert.rule.severity,
              short: true
            },
            {
              title: 'State',
              value: alert.state,
              short: true
            },
            {
              title: 'Value',
              value: alert.value.toString(),
              short: true
            },
            {
              title: 'Started',
              value: new Date(alert.startsAt).toISOString(),
              short: true
            }
          ],
          footer: 'AccuScene Monitoring',
          ts: Math.floor(notification.timestamp / 1000)
        }
      ]
    };

    try {
      const response = await fetch(this.webhookUrl, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify(payload)
      });

      if (!response.ok) {
        throw new Error(`Slack notification failed: ${response.statusText}`);
      }
    } catch (error) {
      console.error('Failed to send Slack notification:', error);
      throw error;
    }
  }
}

/**
 * Alert aggregation to prevent notification spam
 */
export class AlertAggregator {
  private aggregationWindow: number = 300000; // 5 minutes
  private recentAlerts: Map<string, AlertNotification[]> = new Map();

  constructor(aggregationWindow?: number) {
    if (aggregationWindow) {
      this.aggregationWindow = aggregationWindow;
    }
  }

  /**
   * Check if alert should be sent or aggregated
   */
  public shouldSend(notification: AlertNotification): boolean {
    const ruleId = notification.alert.rule.id;
    const recent = this.recentAlerts.get(ruleId) || [];

    // Filter out old notifications
    const cutoff = Date.now() - this.aggregationWindow;
    const validRecent = recent.filter(n => n.timestamp >= cutoff);

    this.recentAlerts.set(ruleId, validRecent);

    // Only send if no recent notifications for this rule
    if (validRecent.length === 0) {
      validRecent.push(notification);
      this.recentAlerts.set(ruleId, validRecent);
      return true;
    }

    return false;
  }

  /**
   * Clear aggregation history
   */
  public clear(): void {
    this.recentAlerts.clear();
  }
}
