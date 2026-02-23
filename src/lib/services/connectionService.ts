import { connectionStore } from '$lib/stores/connections.svelte';
import { schemaStore } from '$lib/stores/schema.svelte';
import { tabStore } from '$lib/stores/tabs.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
import { settingsStore } from '$lib/stores/settings.svelte';
import * as tauri from '$lib/services/tauri';
import { startKeepalive, stopKeepalive } from '$lib/services/keepaliveService';
import type { ConnectionConfig, DatabaseCategory } from '$lib/types/connection';
import { DB_METADATA } from '$lib/types/database';

export async function connect(config: ConnectionConfig) {
  connectionStore.setStatus(config.id, 'connecting');
  try {
    await tauri.connectDb(config);
    connectionStore.setStatus(config.id, 'connected');
    connectionStore.setActive(config.id);
    settingsStore.setLastActiveConnectionId(config.id);
    startKeepalive(config.id);

    // Load schema based on database category
    const meta = DB_METADATA[config.db_type];
    const category = meta.category;

    if (category === 'Relational' || category === 'Analytics' || category === 'WideColumn') {
      // SQL-like: load schemas
      const schemas = await tauri.getSchemas(config.id);
      schemaStore.setSchemas(config.id, schemas);
    } else {
      // NoSQL: load generic containers
      const containers = await tauri.getContainers(config.id);
      schemaStore.setContainers(config.id, containers);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    connectionStore.setStatus(config.id, 'error', message);
    uiStore.showError(`Connection failed: ${message}`);
  }
}

export async function disconnect(connectionId: string) {
  try {
    stopKeepalive(connectionId);
    await tauri.disconnectDb(connectionId);
    connectionStore.setStatus(connectionId, 'disconnected');
    schemaStore.clearConnection(connectionId);
    if (connectionStore.activeConnectionId === connectionId) {
      connectionStore.setActive(null);
    }
    if (settingsStore.lastActiveConnectionId === connectionId) {
      settingsStore.setLastActiveConnectionId(null);
    }
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    uiStore.showError(`Disconnect failed: ${message}`);
  }
}

export async function testConnectionConfig(config: ConnectionConfig): Promise<boolean> {
  try {
    return await tauri.testConnection(config);
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    uiStore.showError(`Test failed: ${message}`);
    return false;
  }
}

export async function saveConnection(config: ConnectionConfig) {
  const existing = connectionStore.connections.find(c => c.config.id === config.id);
  if (existing) {
    await connectionStore.updateConnection(config);
  } else {
    await connectionStore.addConnection(config);
  }
}

export async function deleteConnection(connectionId: string) {
  const conn = connectionStore.connections.find(c => c.config.id === connectionId);
  if (conn?.status === 'connected') {
    await disconnect(connectionId);
  }
  await connectionStore.removeConnection(connectionId);
}
