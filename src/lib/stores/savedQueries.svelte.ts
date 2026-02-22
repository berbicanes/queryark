import { load, type Store } from '@tauri-apps/plugin-store';
import { v4 as uuidv4 } from 'uuid';
import type { SavedQuery } from '$lib/types/query';

class SavedQueriesStore {
  queries = $state<SavedQuery[]>([]);
  private store: Store | null = null;
  private initialized = false;

  async init() {
    if (this.initialized) return;
    this.store = await load('saved-queries.json');
    const saved = await this.store.get<SavedQuery[]>('queries');
    if (saved) {
      this.queries = saved;
    }
    this.initialized = true;
  }

  private async persist() {
    if (this.store) {
      await this.store.set('queries', this.queries);
      await this.store.save();
    }
  }

  save(name: string, connectionId: string, sql: string) {
    const existing = this.queries.find(
      q => q.name === name && q.connectionId === connectionId
    );
    if (existing) {
      existing.sql = sql;
      existing.updatedAt = Date.now();
      this.queries = [...this.queries]; // trigger reactivity
    } else {
      const newQuery: SavedQuery = {
        id: uuidv4(),
        name,
        connectionId,
        sql,
        createdAt: Date.now(),
        updatedAt: Date.now(),
      };
      this.queries = [newQuery, ...this.queries];
    }
    this.persist();
  }

  remove(id: string) {
    this.queries = this.queries.filter(q => q.id !== id);
    this.persist();
  }

  rename(id: string, newName: string) {
    const query = this.queries.find(q => q.id === id);
    if (query) {
      query.name = newName;
      query.updatedAt = Date.now();
      this.queries = [...this.queries]; // trigger reactivity
      this.persist();
    }
  }

  getByConnection(connectionId: string): SavedQuery[] {
    return this.queries.filter(q => q.connectionId === connectionId);
  }
}

export const savedQueriesStore = new SavedQueriesStore();
