import { load, Store } from '@tauri-apps/plugin-store';
import { checkKeychainAvailable, storeKeychainPassword, storeKeychainSecret } from '$lib/services/tauri';
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

  /** Sorted unique group names */
  get groups(): string[] {
    const names = new Set<string>();
    for (const c of this.connections) {
      if (c.config.group) names.add(c.config.group);
    }
    return Array.from(names).sort();
  }

  getConnectionsByGroup(group: string | null): ConnectionState[] {
    return this.connections.filter(c =>
      group === null ? !c.config.group : c.config.group === group
    );
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
    await this.migrateToKeychain();
  }

  /** Migrate plaintext passwords to OS keychain for connections that haven't opted in yet. */
  private async migrateToKeychain() {
    let keychainAvailable: boolean;
    try {
      keychainAvailable = await checkKeychainAvailable();
    } catch {
      return; // Keychain not available, skip migration
    }
    if (!keychainAvailable) return;

    let changed = false;
    for (const conn of this.connections) {
      const config = conn.config;
      if (config.use_keychain) continue; // Already using keychain

      try {
        let migrated = false;

        if (config.password) {
          await storeKeychainPassword(config.id, config.password);
          config.password = undefined;
          migrated = true;
        }
        if (config.ssh_password) {
          await storeKeychainSecret(config.id, 'ssh_password', config.ssh_password);
          config.ssh_password = undefined;
          migrated = true;
        }
        if (config.ssh_passphrase) {
          await storeKeychainSecret(config.id, 'ssh_passphrase', config.ssh_passphrase);
          config.ssh_passphrase = undefined;
          migrated = true;
        }
        if (config.cloud_auth?.AwsCredentials?.secret_key) {
          await storeKeychainSecret(config.id, 'aws_secret_key', config.cloud_auth.AwsCredentials.secret_key);
          config.cloud_auth.AwsCredentials.secret_key = '';
          migrated = true;
        }
        if (config.cloud_auth?.GcpServiceAccount?.credentials_json) {
          await storeKeychainSecret(config.id, 'credentials_json', config.cloud_auth.GcpServiceAccount.credentials_json);
          config.cloud_auth.GcpServiceAccount.credentials_json = '';
          migrated = true;
        }

        if (migrated) {
          config.use_keychain = true;
          changed = true;
        }
      } catch {
        // Per-connection failure: leave plaintext as-is
      }
    }

    if (changed) {
      await this.persist();
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
