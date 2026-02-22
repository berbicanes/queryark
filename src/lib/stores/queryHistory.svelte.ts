import { load, type Store } from '@tauri-apps/plugin-store';
import { v4 as uuidv4 } from 'uuid';
import type { QueryHistoryEntry } from '$lib/types/query';

const MAX_ENTRIES = 500;

class QueryHistoryStore {
  entries = $state<QueryHistoryEntry[]>([]);
  private store: Store | null = null;
  private initialized = false;

  async init() {
    if (this.initialized) return;
    this.store = await load('query-history.json');
    const saved = await this.store.get<QueryHistoryEntry[]>('entries');
    if (saved) {
      this.entries = saved;
    }
    this.initialized = true;
  }

  private async persist() {
    if (this.store) {
      await this.store.set('entries', this.entries);
      await this.store.save();
    }
  }

  addEntry(entry: Omit<QueryHistoryEntry, 'id' | 'executedAt'>) {
    const newEntry: QueryHistoryEntry = {
      id: uuidv4(),
      executedAt: Date.now(),
      ...entry,
    };
    this.entries = [newEntry, ...this.entries].slice(0, MAX_ENTRIES);
    this.persist();
  }

  getEntries(connectionId?: string): QueryHistoryEntry[] {
    if (!connectionId) return this.entries;
    return this.entries.filter(e => e.connectionId === connectionId);
  }

  search(query: string, connectionId?: string): QueryHistoryEntry[] {
    const lower = query.toLowerCase();
    const base = connectionId ? this.getEntries(connectionId) : this.entries;
    return base.filter(e => e.sql.toLowerCase().includes(lower));
  }

  clear() {
    this.entries = [];
    this.persist();
  }
}

export const queryHistoryStore = new QueryHistoryStore();
