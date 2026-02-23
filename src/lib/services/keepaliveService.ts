import { connectionStore } from '$lib/stores/connections.svelte';
import { schemaStore } from '$lib/stores/schema.svelte';
import { uiStore } from '$lib/stores/ui.svelte';
import * as tauri from '$lib/services/tauri';
import { DB_METADATA } from '$lib/types/database';

const PING_INTERVAL_MS = 30_000;
const MAX_RETRIES = 3;
const RETRY_DELAYS = [2000, 4000, 8000]; // exponential backoff

const intervals = new Map<string, ReturnType<typeof setInterval>>();
const reconnecting = new Set<string>();

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms));
}

async function attemptReconnect(connectionId: string) {
  if (reconnecting.has(connectionId)) return;
  reconnecting.add(connectionId);

  // Pause keepalive during reconnection
  stopKeepalive(connectionId);
  connectionStore.setStatus(connectionId, 'connecting');

  const conn = connectionStore.connections.find(c => c.config.id === connectionId);
  if (!conn) {
    reconnecting.delete(connectionId);
    return;
  }

  for (let attempt = 0; attempt < MAX_RETRIES; attempt++) {
    await sleep(RETRY_DELAYS[attempt]);

    try {
      // Disconnect old connection (best-effort)
      await tauri.disconnectDb(connectionId).catch(() => {});
      // Reconnect
      await tauri.connectDb(conn.config);
      connectionStore.setStatus(connectionId, 'connected');

      // Reload schema
      const meta = DB_METADATA[conn.config.db_type];
      const category = meta.category;
      if (category === 'Relational' || category === 'Analytics' || category === 'WideColumn') {
        const schemas = await tauri.getSchemas(connectionId);
        schemaStore.setSchemas(connectionId, schemas);
      } else {
        const containers = await tauri.getContainers(connectionId);
        schemaStore.setContainers(connectionId, containers);
      }

      uiStore.showSuccess(`Reconnected to ${conn.config.name}`);
      startKeepalive(connectionId);
      reconnecting.delete(connectionId);
      return;
    } catch {
      // Retry
    }
  }

  // All retries failed
  connectionStore.setStatus(connectionId, 'error', 'Connection lost — reconnection failed');
  reconnecting.delete(connectionId);
}

export function startKeepalive(connectionId: string) {
  // Don't start duplicate intervals
  if (intervals.has(connectionId)) return;

  const id = setInterval(async () => {
    try {
      await tauri.pingConnection(connectionId);
    } catch {
      attemptReconnect(connectionId);
    }
  }, PING_INTERVAL_MS);

  intervals.set(connectionId, id);
}

export function stopKeepalive(connectionId: string) {
  const id = intervals.get(connectionId);
  if (id) {
    clearInterval(id);
    intervals.delete(connectionId);
  }
  reconnecting.delete(connectionId);
}

export function stopAll() {
  for (const [connId, id] of intervals) {
    clearInterval(id);
    reconnecting.delete(connId);
  }
  intervals.clear();
}
