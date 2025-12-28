/**
 * AccuScene Enterprise v0.2.0 - Plugin Commands API
 * Command registration and execution API
 */

import { PluginCommands, PluginId, Command, Disposable } from '../types';

class PluginCommandsImpl implements PluginCommands {
  private commands = new Map<string, Command>();

  constructor(
    private pluginId: PluginId,
    private manager: any
  ) {}

  register(command: Command): Disposable {
    const id = `${this.pluginId}.${command.id}`;

    if (this.commands.has(id)) {
      throw new Error(`Command ${id} is already registered`);
    }

    this.commands.set(id, command);

    // Notify command system
    this.notifyCommandUpdate('register', { id, command });

    return {
      dispose: () => {
        this.commands.delete(id);
        this.notifyCommandUpdate('unregister', { id });
      },
    };
  }

  async execute(commandId: string, ...args: any[]): Promise<any> {
    // Try to find command with plugin prefix
    let command = this.commands.get(`${this.pluginId}.${commandId}`);

    // If not found, try global command ID
    if (!command) {
      command = this.commands.get(commandId);
    }

    if (!command) {
      // Try to execute from global command registry
      if (this.manager && this.manager.executeCommand) {
        return this.manager.executeCommand(commandId, ...args);
      }

      throw new Error(`Command ${commandId} not found`);
    }

    try {
      return await command.handler(...args);
    } catch (error) {
      throw new Error(`Error executing command ${commandId}: ${error}`);
    }
  }

  has(commandId: string): boolean {
    const fullId = `${this.pluginId}.${commandId}`;
    return this.commands.has(fullId) || this.commands.has(commandId);
  }

  /**
   * Get all registered commands
   */
  getCommands(): Command[] {
    return Array.from(this.commands.values());
  }

  /**
   * Get a specific command
   */
  getCommand(commandId: string): Command | undefined {
    const fullId = `${this.pluginId}.${commandId}`;
    return this.commands.get(fullId) || this.commands.get(commandId);
  }

  private notifyCommandUpdate(type: string, data: any): void {
    if (this.manager && this.manager.emit) {
      this.manager.emit(`command:${type}`, { pluginId: this.pluginId, data });
    }
  }
}

export const createPluginCommands = (pluginId: PluginId, manager: any): PluginCommands => {
  return new PluginCommandsImpl(pluginId, manager);
};
