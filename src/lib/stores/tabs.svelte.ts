import { v4 as uuidv4 } from 'uuid';
import type { Tab, TabType } from '$lib/types/tabs';

class TabStore {
  tabs = $state<Tab[]>([]);
  activeTabId = $state<string | null>(null);

  get activeTab(): Tab | undefined {
    return this.tabs.find(t => t.id === this.activeTabId);
  }

  openTab(tab: Omit<Tab, 'id'>) {
    // Check for existing table tab with same connection+schema+table
    if (tab.type === 'table') {
      const existing = this.tabs.find(
        t => t.type === 'table' && t.connectionId === tab.connectionId &&
             t.schema === tab.schema && t.table === tab.table
      );
      if (existing) {
        this.activeTabId = existing.id;
        return existing.id;
      }
    }

    // Check for existing document/keyvalue/graph tab with same connection+container+item
    if (tab.type === 'document' || tab.type === 'keyvalue' || tab.type === 'graph') {
      const existing = this.tabs.find(
        t => t.type === tab.type && t.connectionId === tab.connectionId &&
             t.container === tab.container && t.item === tab.item
      );
      if (existing) {
        this.activeTabId = existing.id;
        return existing.id;
      }
    }

    const id = uuidv4();
    const newTab: Tab = { ...tab, id };
    this.tabs.push(newTab);
    this.activeTabId = id;
    return id;
  }

  closeTab(id: string) {
    const idx = this.tabs.findIndex(t => t.id === id);
    if (idx < 0) return;
    this.tabs.splice(idx, 1);
    if (this.activeTabId === id) {
      // Activate adjacent tab
      if (this.tabs.length > 0) {
        const newIdx = Math.min(idx, this.tabs.length - 1);
        this.activeTabId = this.tabs[newIdx].id;
      } else {
        this.activeTabId = null;
      }
    }
  }

  setActive(id: string) {
    this.activeTabId = id;
  }

  updateTabSql(id: string, sql: string) {
    const tab = this.tabs.find(t => t.id === id);
    if (tab) tab.sql = sql;
  }

  newQueryTab(connectionId: string) {
    const queryCount = this.tabs.filter(t => t.type === 'query').length + 1;
    return this.openTab({
      type: 'query',
      title: `Query ${queryCount}`,
      connectionId,
      sql: ''
    });
  }
}

export const tabStore = new TabStore();
