import * as tauri from '$lib/services/tauri';
import { uiStore } from '$lib/stores/ui.svelte';

export async function scanKeys(connectionId: string, pattern: string = '*', count: number = 100): Promise<string[]> {
  try {
    return await tauri.scanKeys(connectionId, pattern, count);
  } catch (err) {
    uiStore.showError(`Failed to scan keys: ${err}`);
    return [];
  }
}

export async function getValue(connectionId: string, key: string): Promise<string | null> {
  try {
    return await tauri.getValue(connectionId, key);
  } catch (err) {
    uiStore.showError(`Failed to get value: ${err}`);
    return null;
  }
}

export async function getKeyType(connectionId: string, key: string): Promise<string> {
  try {
    return await tauri.getKeyType(connectionId, key);
  } catch (err) {
    uiStore.showError(`Failed to get key type: ${err}`);
    return 'unknown';
  }
}

export async function setValue(connectionId: string, key: string, value: string, ttl: number | null = null): Promise<boolean> {
  try {
    await tauri.setValue(connectionId, key, value, ttl);
    return true;
  } catch (err) {
    uiStore.showError(`Failed to set value: ${err}`);
    return false;
  }
}

export async function deleteKeys(connectionId: string, keys: string[]): Promise<number> {
  try {
    return await tauri.deleteKeys(connectionId, keys);
  } catch (err) {
    uiStore.showError(`Failed to delete keys: ${err}`);
    return 0;
  }
}
