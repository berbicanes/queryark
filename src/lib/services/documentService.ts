import * as tauri from '$lib/services/tauri';
import { uiStore } from '$lib/stores/ui.svelte';
import type { QueryResponse } from '$lib/types/query';

export async function loadDocuments(connectionId: string, container: string, collection: string, limit: number, offset: number): Promise<QueryResponse | null> {
  try {
    return await tauri.getItemData(connectionId, container, collection, limit, offset);
  } catch (err) {
    uiStore.showError(`Failed to load documents: ${err}`);
    return null;
  }
}

export async function getDocumentCount(connectionId: string, container: string, collection: string): Promise<number> {
  try {
    return await tauri.getItemCount(connectionId, container, collection);
  } catch (err) {
    uiStore.showError(`Failed to get document count: ${err}`);
    return 0;
  }
}

export async function insertDocument(connectionId: string, container: string, collection: string, document: string): Promise<string | null> {
  try {
    const id = await tauri.insertDocument(connectionId, container, collection, document);
    uiStore.showError(''); // clear any previous error
    return id;
  } catch (err) {
    uiStore.showError(`Failed to insert document: ${err}`);
    return null;
  }
}

export async function updateDocument(connectionId: string, container: string, collection: string, filter: string, update: string): Promise<number> {
  try {
    return await tauri.updateDocument(connectionId, container, collection, filter, update);
  } catch (err) {
    uiStore.showError(`Failed to update document: ${err}`);
    return 0;
  }
}

export async function deleteDocuments(connectionId: string, container: string, collection: string, filter: string): Promise<number> {
  try {
    return await tauri.deleteDocuments(connectionId, container, collection, filter);
  } catch (err) {
    uiStore.showError(`Failed to delete documents: ${err}`);
    return 0;
  }
}
