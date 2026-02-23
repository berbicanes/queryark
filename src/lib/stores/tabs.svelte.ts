import { v4 as uuidv4 } from 'uuid';
import { load, type Store } from '@tauri-apps/plugin-store';
import type { Tab, TabType } from '$lib/types/tabs';

class TabStore {
  tabs = $state<Tab[]>([]);
  activeTabId = $state<string | null>(null);

  // Persistence
  private store: Store | null = null;
  private initialized = false;
  private persistTimer: ReturnType<typeof setTimeout> | null = null;

  // Split pane state
  splitMode = $state(false);
  leftPaneTabs = $state<string[]>([]);
  rightPaneTabs = $state<string[]>([]);
  activeLeftTabId = $state<string | null>(null);
  activeRightTabId = $state<string | null>(null);
  activePaneId = $state<'left' | 'right'>('left');
  splitRatio = $state(0.5);

  get activeTab(): Tab | undefined {
    return this.tabs.find(t => t.id === this.activeTabId);
  }

  async init() {
    if (this.initialized) return;
    this.store = await load('session.json');
    const savedTabs = await this.store.get<Tab[]>('tabs');
    if (savedTabs && savedTabs.length > 0) {
      this.tabs = savedTabs;
    }
    const savedActiveId = await this.store.get<string | null>('activeTabId');
    if (savedActiveId && this.tabs.some(t => t.id === savedActiveId)) {
      this.activeTabId = savedActiveId;
    } else if (this.tabs.length > 0) {
      this.activeTabId = this.tabs[0].id;
    }
    this.initialized = true;
  }

  private async persist() {
    if (!this.store) return;
    await this.store.set('tabs', this.tabs.map(t => ({ ...t })));
    await this.store.set('activeTabId', this.activeTabId);
    await this.store.save();
  }

  private debouncedPersist() {
    if (this.persistTimer) clearTimeout(this.persistTimer);
    this.persistTimer = setTimeout(() => this.persist(), 500);
  }

  /** Returns true if the query tab with the given id has non-empty SQL */
  hasContent(id: string): boolean {
    const tab = this.tabs.find(t => t.id === id);
    return tab?.type === 'query' && !!tab.sql?.trim();
  }

  /** Returns all tabs for a given connection */
  tabsForConnection(connectionId: string): Tab[] {
    return this.tabs.filter(t => t.connectionId === connectionId);
  }

  /** Sorted view: pinned tabs first, then unpinned, preserving relative order within each group */
  get sortedTabs(): Tab[] {
    const pinned = this.tabs.filter(t => t.pinned);
    const unpinned = this.tabs.filter(t => !t.pinned);
    return [...pinned, ...unpinned];
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
        if (this.splitMode) this._focusTabInPane(existing.id);
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
        if (this.splitMode) this._focusTabInPane(existing.id);
        return existing.id;
      }
    }

    // Dedup diagram tabs by connection + sorted schemas
    if (tab.type === 'diagram') {
      const sortedSchemas = [...(tab.diagramSchemas ?? [])].sort().join(',');
      const existing = this.tabs.find(
        t => t.type === 'diagram' && t.connectionId === tab.connectionId &&
             [...(t.diagramSchemas ?? [])].sort().join(',') === sortedSchemas
      );
      if (existing) {
        this.activeTabId = existing.id;
        if (this.splitMode) this._focusTabInPane(existing.id);
        return existing.id;
      }
    }

    const id = uuidv4();
    const newTab: Tab = { ...tab, id };
    this.tabs.push(newTab);
    this.activeTabId = id;

    // In split mode, add to active pane
    if (this.splitMode) {
      if (this.activePaneId === 'right') {
        this.rightPaneTabs = [...this.rightPaneTabs, id];
        this.activeRightTabId = id;
      } else {
        this.leftPaneTabs = [...this.leftPaneTabs, id];
        this.activeLeftTabId = id;
      }
    }

    this.persist();
    return id;
  }

  closeTab(id: string, force = false) {
    const tab = this.tabs.find(t => t.id === id);
    if (!tab) return;

    // Skip pinned tabs unless forced
    if (tab.pinned && !force) return;

    const idx = this.tabs.indexOf(tab);

    // Handle split mode pane removal
    if (this.splitMode) {
      const inLeft = this.leftPaneTabs.includes(id);
      const inRight = this.rightPaneTabs.includes(id);

      if (inLeft) {
        this.leftPaneTabs = this.leftPaneTabs.filter(tid => tid !== id);
        if (this.activeLeftTabId === id) {
          this.activeLeftTabId = this.leftPaneTabs[0] ?? null;
        }
        // Close split if pane is empty
        if (this.leftPaneTabs.length === 0) {
          this.closeSplit();
        }
      }
      if (inRight) {
        this.rightPaneTabs = this.rightPaneTabs.filter(tid => tid !== id);
        if (this.activeRightTabId === id) {
          this.activeRightTabId = this.rightPaneTabs[0] ?? null;
        }
        if (this.rightPaneTabs.length === 0) {
          this.closeSplit();
        }
      }
    }

    this.tabs.splice(idx, 1);
    if (this.activeTabId === id) {
      if (this.tabs.length > 0) {
        const newIdx = Math.min(idx, this.tabs.length - 1);
        this.activeTabId = this.tabs[newIdx].id;
      } else {
        this.activeTabId = null;
      }
    }
    this.persist();
  }

  closeAll() {
    const toClose = this.tabs.filter(t => !t.pinned).map(t => t.id);
    for (const id of toClose) this.closeTab(id);
  }

  closeOthers(keepId: string) {
    const toClose = this.tabs.filter(t => t.id !== keepId && !t.pinned).map(t => t.id);
    for (const id of toClose) this.closeTab(id);
  }

  togglePin(id: string) {
    const tab = this.tabs.find(t => t.id === id);
    if (tab) {
      tab.pinned = !tab.pinned;
      this.tabs = [...this.tabs]; // trigger reactivity
      this.persist();
    }
  }

  duplicateTab(id: string) {
    const tab = this.tabs.find(t => t.id === id);
    if (!tab) return;

    const newId = uuidv4();
    const copy: Tab = {
      ...tab,
      id: newId,
      title: tab.title + ' (copy)',
      pinned: false,
    };
    this.tabs.push(copy);
    this.activeTabId = newId;

    if (this.splitMode) {
      if (this.activePaneId === 'right') {
        this.rightPaneTabs = [...this.rightPaneTabs, newId];
        this.activeRightTabId = newId;
      } else {
        this.leftPaneTabs = [...this.leftPaneTabs, newId];
        this.activeLeftTabId = newId;
      }
    }

    this.persist();
    return newId;
  }

  setActive(id: string) {
    this.activeTabId = id;
    if (this.splitMode) this._focusTabInPane(id);
    this.persist();
  }

  updateTabSql(id: string, sql: string) {
    const tab = this.tabs.find(t => t.id === id);
    if (tab) {
      tab.sql = sql;
      this.debouncedPersist();
    }
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

  // --- Drag and drop ---
  moveTab(fromIndex: number, toIndex: number) {
    if (fromIndex === toIndex) return;
    const [tab] = this.tabs.splice(fromIndex, 1);
    this.tabs.splice(toIndex, 0, tab);
    this.tabs = [...this.tabs]; // trigger reactivity
    this.persist();
  }

  // --- Split panes ---
  splitTab(tabId: string) {
    if (this.splitMode) return; // already split
    if (this.tabs.length < 1) return;

    this.splitMode = true;
    this.splitRatio = 0.5;
    this.activePaneId = 'right';

    // All existing tabs go to left pane, the target tab goes to right pane
    const otherTabs = this.tabs.filter(t => t.id !== tabId).map(t => t.id);
    this.leftPaneTabs = otherTabs.length > 0 ? otherTabs : [];
    this.rightPaneTabs = [tabId];
    this.activeLeftTabId = otherTabs[0] ?? null;
    this.activeRightTabId = tabId;
    this.activeTabId = tabId;

    // If only one tab existed, the left pane will be empty — handle by keeping tab in both
    if (otherTabs.length === 0) {
      // Need at least 2 tabs for split — cancel
      this.splitMode = false;
      this.leftPaneTabs = [];
      this.rightPaneTabs = [];
    }
  }

  closeSplit() {
    this.splitMode = false;
    // Keep the active tab from whichever pane was active
    const activeId = this.activePaneId === 'left' ? this.activeLeftTabId : this.activeRightTabId;
    if (activeId) this.activeTabId = activeId;
    this.leftPaneTabs = [];
    this.rightPaneTabs = [];
    this.activeLeftTabId = null;
    this.activeRightTabId = null;
  }

  moveTabToPane(tabId: string, targetPane: 'left' | 'right') {
    if (!this.splitMode) return;

    // Remove from current pane
    this.leftPaneTabs = this.leftPaneTabs.filter(id => id !== tabId);
    this.rightPaneTabs = this.rightPaneTabs.filter(id => id !== tabId);

    // Add to target pane
    if (targetPane === 'left') {
      this.leftPaneTabs = [...this.leftPaneTabs, tabId];
      this.activeLeftTabId = tabId;
    } else {
      this.rightPaneTabs = [...this.rightPaneTabs, tabId];
      this.activeRightTabId = tabId;
    }

    // Close split if a pane becomes empty
    if (this.leftPaneTabs.length === 0 || this.rightPaneTabs.length === 0) {
      this.closeSplit();
    }
  }

  private _focusTabInPane(id: string) {
    if (this.leftPaneTabs.includes(id)) {
      this.activeLeftTabId = id;
      this.activePaneId = 'left';
    } else if (this.rightPaneTabs.includes(id)) {
      this.activeRightTabId = id;
      this.activePaneId = 'right';
    }
  }
}

export const tabStore = new TabStore();
