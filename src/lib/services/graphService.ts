import * as tauri from '$lib/services/tauri';
import { uiStore } from '$lib/stores/ui.svelte';
import type { QueryResponse } from '$lib/types/query';

export async function getLabels(connectionId: string): Promise<string[]> {
  try {
    return await tauri.getLabels(connectionId);
  } catch (err) {
    uiStore.showError(`Failed to get labels: ${err}`);
    return [];
  }
}

export async function getRelationshipTypes(connectionId: string): Promise<string[]> {
  try {
    return await tauri.getRelationshipTypes(connectionId);
  } catch (err) {
    uiStore.showError(`Failed to get relationship types: ${err}`);
    return [];
  }
}

export async function getNodeProperties(connectionId: string, label: string): Promise<string[]> {
  try {
    return await tauri.getNodeProperties(connectionId, label);
  } catch (err) {
    uiStore.showError(`Failed to get node properties: ${err}`);
    return [];
  }
}

export async function getNodes(connectionId: string, label: string, limit: number = 50, skip: number = 0): Promise<QueryResponse | null> {
  try {
    return await tauri.getNodes(connectionId, label, limit, skip);
  } catch (err) {
    uiStore.showError(`Failed to get nodes: ${err}`);
    return null;
  }
}
