/**
 * AccuScene Enterprise v0.2.0
 * Alert Rules Engine
 *
 * Rule-based alerting system
 */

import { AlertRule, Alert, AlertState, AlertCondition, MetricLabels } from '../types';
import { v4 as uuidv4 } from 'uuid';

export class AlertRulesEngine {
  private rules: Map<string, AlertRule> = new Map();
  private activeAlerts: Map<string, Alert> = new Map();
  private pendingAlerts: Map<string, { rule: AlertRule; since: number }> = new Map();
  private evaluationInterval: number = 15000; // 15 seconds
  private intervalId?: NodeJS.Timeout;

  constructor(evaluationInterval?: number) {
    if (evaluationInterval) {
      this.evaluationInterval = evaluationInterval;
    }
  }

  /**
   * Add an alert rule
   */
  public addRule(rule: AlertRule): void {
    this.rules.set(rule.id, rule);
  }

  /**
   * Remove an alert rule
   */
  public removeRule(ruleId: string): void {
    this.rules.delete(ruleId);

    // Remove any related alerts
    for (const [alertId, alert] of this.activeAlerts.entries()) {
      if (alert.rule.id === ruleId) {
        this.activeAlerts.delete(alertId);
      }
    }
  }

  /**
   * Get a rule by ID
   */
  public getRule(ruleId: string): AlertRule | undefined {
    return this.rules.get(ruleId);
  }

  /**
   * Get all rules
   */
  public getRules(): AlertRule[] {
    return Array.from(this.rules.values());
  }

  /**
   * Get active alerts
   */
  public getActiveAlerts(): Alert[] {
    return Array.from(this.activeAlerts.values());
  }

  /**
   * Get alerts for a specific rule
   */
  public getAlertsByRule(ruleId: string): Alert[] {
    return Array.from(this.activeAlerts.values()).filter(
      alert => alert.rule.id === ruleId
    );
  }

  /**
   * Start alert evaluation
   */
  public start(metricGetter: (metric: string) => number): void {
    if (this.intervalId) {
      return;
    }

    this.intervalId = setInterval(() => {
      this.evaluate(metricGetter);
    }, this.evaluationInterval);

    // Immediate first evaluation
    this.evaluate(metricGetter);
  }

  /**
   * Stop alert evaluation
   */
  public stop(): void {
    if (this.intervalId) {
      clearInterval(this.intervalId);
      this.intervalId = undefined;
    }
  }

  /**
   * Evaluate all rules
   */
  public evaluate(metricGetter: (metric: string) => number): void {
    const now = Date.now();

    for (const rule of this.rules.values()) {
      const value = metricGetter(rule.condition.metric);
      const isFiring = this.evaluateCondition(rule.condition, value);

      if (isFiring) {
        this.handleFiringRule(rule, value, now);
      } else {
        this.handleResolvedRule(rule, now);
      }
    }
  }

  /**
   * Evaluate a single condition
   */
  private evaluateCondition(condition: AlertCondition, value: number): boolean {
    const { operator, threshold } = condition;

    switch (operator) {
      case 'gt':
        return value > threshold;

      case 'gte':
        return value >= threshold;

      case 'lt':
        return value < threshold;

      case 'lte':
        return value <= threshold;

      case 'eq':
        return value === threshold;

      case 'neq':
        return value !== threshold;

      default:
        return false;
    }
  }

  /**
   * Handle a firing rule
   */
  private handleFiringRule(rule: AlertRule, value: number, now: number): void {
    const pendingKey = rule.id;

    // Check if already pending
    const pending = this.pendingAlerts.get(pendingKey);

    if (pending) {
      // Check if duration threshold met
      const duration = rule.duration || 0;

      if (now - pending.since >= duration) {
        // Fire the alert
        this.fireAlert(rule, value, pending.since);
        this.pendingAlerts.delete(pendingKey);
      }
    } else {
      // Start pending
      this.pendingAlerts.set(pendingKey, { rule, since: now });

      // If no duration requirement, fire immediately
      if (!rule.duration) {
        this.fireAlert(rule, value, now);
        this.pendingAlerts.delete(pendingKey);
      }
    }
  }

  /**
   * Handle a resolved rule
   */
  private handleResolvedRule(rule: AlertRule, now: number): void {
    const pendingKey = rule.id;

    // Remove from pending
    this.pendingAlerts.delete(pendingKey);

    // Resolve any active alerts
    for (const [alertId, alert] of this.activeAlerts.entries()) {
      if (alert.rule.id === rule.id && alert.state === AlertState.FIRING) {
        alert.state = AlertState.RESOLVED;
        alert.endsAt = now;
      }
    }
  }

  /**
   * Fire an alert
   */
  private fireAlert(rule: AlertRule, value: number, startsAt: number): void {
    const alertId = uuidv4();

    const alert: Alert = {
      id: alertId,
      rule,
      state: AlertState.FIRING,
      value,
      startsAt,
      labels: rule.labels || {},
      annotations: rule.annotations || {}
    };

    this.activeAlerts.set(alertId, alert);
  }

  /**
   * Manually fire an alert
   */
  public manualFire(ruleId: string, value: number): Alert | undefined {
    const rule = this.rules.get(ruleId);

    if (!rule) {
      return undefined;
    }

    const alertId = uuidv4();
    const now = Date.now();

    const alert: Alert = {
      id: alertId,
      rule,
      state: AlertState.FIRING,
      value,
      startsAt: now,
      labels: rule.labels || {},
      annotations: rule.annotations || {}
    };

    this.activeAlerts.set(alertId, alert);

    return alert;
  }

  /**
   * Manually resolve an alert
   */
  public manualResolve(alertId: string): void {
    const alert = this.activeAlerts.get(alertId);

    if (alert && alert.state === AlertState.FIRING) {
      alert.state = AlertState.RESOLVED;
      alert.endsAt = Date.now();
    }
  }

  /**
   * Clear resolved alerts older than retention period
   */
  public clearResolvedAlerts(retentionMs: number = 3600000): void {
    const cutoff = Date.now() - retentionMs;

    for (const [alertId, alert] of this.activeAlerts.entries()) {
      if (
        alert.state === AlertState.RESOLVED &&
        alert.endsAt &&
        alert.endsAt < cutoff
      ) {
        this.activeAlerts.delete(alertId);
      }
    }
  }

  /**
   * Clear all alerts
   */
  public clearAll(): void {
    this.activeAlerts.clear();
    this.pendingAlerts.clear();
  }
}

/**
 * Alert rule builder
 */
export class AlertRuleBuilder {
  private rule: Partial<AlertRule> = {};

  public id(id: string): this {
    this.rule.id = id;
    return this;
  }

  public name(name: string): this {
    this.rule.name = name;
    return this;
  }

  public description(description: string): this {
    this.rule.description = description;
    return this;
  }

  public severity(severity: any): this {
    this.rule.severity = severity;
    return this;
  }

  public metric(metric: string): this {
    if (!this.rule.condition) {
      this.rule.condition = {} as AlertCondition;
    }
    this.rule.condition.metric = metric;
    return this;
  }

  public operator(operator: AlertCondition['operator']): this {
    if (!this.rule.condition) {
      this.rule.condition = {} as AlertCondition;
    }
    this.rule.condition.operator = operator;
    return this;
  }

  public threshold(threshold: number): this {
    if (!this.rule.condition) {
      this.rule.condition = {} as AlertCondition;
    }
    this.rule.condition.threshold = threshold;
    return this;
  }

  public duration(duration: number): this {
    this.rule.duration = duration;
    return this;
  }

  public labels(labels: MetricLabels): this {
    this.rule.labels = labels;
    return this;
  }

  public annotations(annotations: Record<string, string>): this {
    this.rule.annotations = annotations;
    return this;
  }

  public build(): AlertRule {
    if (!this.rule.id || !this.rule.name || !this.rule.condition) {
      throw new Error('Missing required fields for alert rule');
    }

    return this.rule as AlertRule;
  }
}
