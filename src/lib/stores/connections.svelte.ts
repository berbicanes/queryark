import { load, Store } from '@tauri-apps/plugin-store';
import type { ConnectionConfig, ConnectionState, ConnectionStatus } from '$lib/types/connection';

// Create a class-based store using Svelte 5 runes
class ConnectionStore {
  connections = $state<ConnectionState[]>([]);
  activeConnectionId = $state<string | null>(null);
  private store: Store | null = null;

  get activeConnection(): ConnectionState | undefined {
    // Use $derived behavior - find the active connection
    return this.connections.find(c => c.config.id === this.activeConnectionId);
  }

  get connectedConnections(): ConnectionState[] {
    return this.connections.filter(c => c.status === 'connected');
  }

  async init() {
    this.store = await load('connections.json');
    const saved = await this.store.get<ConnectionConfig[]>('connections');
    if (saved) {
      this.connections = saved.map(config => ({
        config,
        status: 'disconnected' as ConnectionStatus
      }));
    }
  }

  private async persist() {
    if (this.store) {
      await this.store.set('connections', this.connections.map(c => c.config));
      await this.store.save();
    }
  }

  async addConnection(config: ConnectionConfig) {
    this.connections.push({ config, status: 'disconnected' });
    await this.persist();
  }

  async updateConnection(config: ConnectionConfig) {
    const idx = this.connections.findIndex(c => c.config.id === config.id);
    if (idx >= 0) {
      this.connections[idx].config = config;
      await this.persist();
    }
  }

  async removeConnection(id: string) {
    this.connections = this.connections.filter(c => c.config.id !== id);
    if (this.activeConnectionId === id) {
      this.activeConnectionId = null;
    }
    await this.persist();
  }

  setStatus(id: string, status: ConnectionStatus, error?: string) {
    const conn = this.connections.find(c => c.config.id === id);
    if (conn) {
      conn.status = status;
      conn.error = error;
    }
  }

  setActive(id: string | null) {
    this.activeConnectionId = id;
  }
}

export const connectionStore = new ConnectionStore();
