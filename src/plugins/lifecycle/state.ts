/**
 * AccuScene Enterprise v0.2.0 - Plugin State Machine
 * State machine for plugin lifecycle management
 */

import { PluginState, PluginId } from '../types';

export interface StateTransition {
  from: PluginState;
  to: PluginState;
  timestamp: number;
}

export class PluginStateMachine {
  private currentState: PluginState = PluginState.UNLOADED;
  private history: StateTransition[] = [];
  private listeners = new Set<StateChangeListener>();

  constructor(
    private pluginId: PluginId,
    initialState: PluginState = PluginState.UNLOADED
  ) {
    this.currentState = initialState;
  }

  /**
   * Get the current state
   */
  getState(): PluginState {
    return this.currentState;
  }

  /**
   * Transition to a new state
   */
  transition(to: PluginState): void {
    if (!this.canTransition(this.currentState, to)) {
      throw new Error(
        `Invalid state transition for plugin ${this.pluginId}: ${this.currentState} -> ${to}`
      );
    }

    const from = this.currentState;
    this.currentState = to;

    const transition: StateTransition = {
      from,
      to,
      timestamp: Date.now(),
    };

    this.history.push(transition);
    this.notifyListeners(transition);
  }

  /**
   * Check if a transition is valid
   */
  canTransition(from: PluginState, to: PluginState): boolean {
    const validTransitions: Record<PluginState, PluginState[]> = {
      [PluginState.UNLOADED]: [PluginState.LOADING],
      [PluginState.LOADING]: [PluginState.LOADED, PluginState.ERROR],
      [PluginState.LOADED]: [
        PluginState.INITIALIZING,
        PluginState.UNLOADING,
        PluginState.ERROR,
      ],
      [PluginState.INITIALIZING]: [PluginState.ACTIVE, PluginState.ERROR],
      [PluginState.ACTIVE]: [
        PluginState.PAUSED,
        PluginState.LOADED,
        PluginState.ERROR,
        PluginState.UNLOADING,
      ],
      [PluginState.PAUSED]: [PluginState.ACTIVE, PluginState.LOADED, PluginState.ERROR],
      [PluginState.ERROR]: [
        PluginState.LOADED,
        PluginState.UNLOADING,
        PluginState.INITIALIZING,
      ],
      [PluginState.UNLOADING]: [PluginState.UNLOADED],
    };

    return validTransitions[from]?.includes(to) ?? false;
  }

  /**
   * Listen to state changes
   */
  onChange(listener: StateChangeListener): () => void {
    this.listeners.add(listener);

    return () => {
      this.listeners.delete(listener);
    };
  }

  /**
   * Get state history
   */
  getHistory(): StateTransition[] {
    return [...this.history];
  }

  /**
   * Get the duration in a specific state
   */
  getStateDuration(state: PluginState): number {
    const transitions = this.history.filter(t => t.to === state);

    if (transitions.length === 0) {
      return 0;
    }

    let duration = 0;

    for (let i = 0; i < transitions.length; i++) {
      const enter = transitions[i].timestamp;
      const exit =
        this.history.find((t, idx) => idx > i && t.from === state)?.timestamp || Date.now();

      duration += exit - enter;
    }

    return duration;
  }

  /**
   * Check if the plugin has ever been in a state
   */
  hasBeenInState(state: PluginState): boolean {
    return this.history.some(t => t.to === state);
  }

  /**
   * Reset the state machine
   */
  reset(): void {
    this.currentState = PluginState.UNLOADED;
    this.history = [];
  }

  private notifyListeners(transition: StateTransition): void {
    this.listeners.forEach(listener => {
      try {
        listener(transition);
      } catch (error) {
        console.error(`Error in state change listener for ${this.pluginId}:`, error);
      }
    });
  }
}

export type StateChangeListener = (transition: StateTransition) => void;

export const createStateMachine = (
  pluginId: PluginId,
  initialState?: PluginState
): PluginStateMachine => {
  return new PluginStateMachine(pluginId, initialState);
};
