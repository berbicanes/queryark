import { load, type Store } from '@tauri-apps/plugin-store';
import { DEFAULT_SHORTCUTS } from '$lib/types/shortcuts';

export type Theme = 'dark' | 'light';

class SettingsStore {
  theme = $state<Theme>('dark');
  shortcutOverrides = $state<Record<string, string>>({});
  editorFontSize = $state(13);
  gridFontSize = $state(12);
  defaultPageSize = $state(100);
  confirmBeforeDelete = $state(true);
  maxQueryRows = $state(10000);
  maxCellSize = $state(256);

  // Window state persistence
  sidebarWidth = $state(260);
  sidebarCollapsed = $state(false);
  windowMaximized = $state(false);
  windowWidth = $state(1280);
  windowHeight = $state(800);
  windowX = $state<number | null>(null);
  windowY = $state<number | null>(null);

  // Session restore
  restoreSession = $state(true);
  lastActiveConnectionId = $state<string | null>(null);

  // Schema visibility per connection
  schemaVisibility = $state<Record<string, string[]>>({});

  private store: Store | null = null;
  private initialized = false;

  async init() {
    if (this.initialized) return;
    this.store = await load('settings.json');
    const savedTheme = await this.store.get<Theme>('theme');
    if (savedTheme) {
      this.theme = savedTheme;
    }
    const savedShortcuts = await this.store.get<Record<string, string>>('shortcutOverrides');
    if (savedShortcuts) {
      this.shortcutOverrides = savedShortcuts;
    }
    const savedEditorFontSize = await this.store.get<number>('editorFontSize');
    if (savedEditorFontSize) this.editorFontSize = savedEditorFontSize;
    const savedGridFontSize = await this.store.get<number>('gridFontSize');
    if (savedGridFontSize) this.gridFontSize = savedGridFontSize;
    const savedPageSize = await this.store.get<number>('defaultPageSize');
    if (savedPageSize) this.defaultPageSize = savedPageSize;
    const savedConfirm = await this.store.get<boolean>('confirmBeforeDelete');
    if (savedConfirm !== null && savedConfirm !== undefined) this.confirmBeforeDelete = savedConfirm;
    const savedMaxQueryRows = await this.store.get<number>('maxQueryRows');
    if (savedMaxQueryRows) this.maxQueryRows = savedMaxQueryRows;
    const savedMaxCellSize = await this.store.get<number>('maxCellSize');
    if (savedMaxCellSize) this.maxCellSize = savedMaxCellSize;

    // Window state
    const savedSidebarWidth = await this.store.get<number>('sidebarWidth');
    if (savedSidebarWidth) this.sidebarWidth = savedSidebarWidth;
    const savedSidebarCollapsed = await this.store.get<boolean>('sidebarCollapsed');
    if (savedSidebarCollapsed !== null && savedSidebarCollapsed !== undefined) this.sidebarCollapsed = savedSidebarCollapsed;
    const savedWindowMaximized = await this.store.get<boolean>('windowMaximized');
    if (savedWindowMaximized !== null && savedWindowMaximized !== undefined) this.windowMaximized = savedWindowMaximized;
    const savedWindowWidth = await this.store.get<number>('windowWidth');
    if (savedWindowWidth) this.windowWidth = savedWindowWidth;
    const savedWindowHeight = await this.store.get<number>('windowHeight');
    if (savedWindowHeight) this.windowHeight = savedWindowHeight;
    const savedWindowX = await this.store.get<number | null>('windowX');
    if (savedWindowX !== undefined) this.windowX = savedWindowX;
    const savedWindowY = await this.store.get<number | null>('windowY');
    if (savedWindowY !== undefined) this.windowY = savedWindowY;

    // Session restore
    const savedRestoreSession = await this.store.get<boolean>('restoreSession');
    if (savedRestoreSession !== null && savedRestoreSession !== undefined) this.restoreSession = savedRestoreSession;
    const savedLastConnId = await this.store.get<string | null>('lastActiveConnectionId');
    if (savedLastConnId !== undefined) this.lastActiveConnectionId = savedLastConnId;

    const savedSchemaVisibility = await this.store.get<Record<string, string[]>>('schemaVisibility');
    if (savedSchemaVisibility) this.schemaVisibility = savedSchemaVisibility;

    this.applyTheme();
    this.applyFontSizes();
    this.initialized = true;
  }

  private async persist() {
    if (this.store) {
      await this.store.set('theme', this.theme);
      await this.store.set('shortcutOverrides', this.shortcutOverrides);
      await this.store.set('editorFontSize', this.editorFontSize);
      await this.store.set('gridFontSize', this.gridFontSize);
      await this.store.set('defaultPageSize', this.defaultPageSize);
      await this.store.set('confirmBeforeDelete', this.confirmBeforeDelete);
      await this.store.set('maxQueryRows', this.maxQueryRows);
      await this.store.set('maxCellSize', this.maxCellSize);
      await this.store.set('sidebarWidth', this.sidebarWidth);
      await this.store.set('sidebarCollapsed', this.sidebarCollapsed);
      await this.store.set('windowMaximized', this.windowMaximized);
      await this.store.set('windowWidth', this.windowWidth);
      await this.store.set('windowHeight', this.windowHeight);
      await this.store.set('windowX', this.windowX);
      await this.store.set('windowY', this.windowY);
      await this.store.set('restoreSession', this.restoreSession);
      await this.store.set('lastActiveConnectionId', this.lastActiveConnectionId);
      await this.store.set('schemaVisibility', this.schemaVisibility);
      await this.store.save();
    }
  }

  applyTheme() {
    document.documentElement.setAttribute('data-theme', this.theme);
  }

  applyFontSizes() {
    document.documentElement.style.setProperty('--editor-font-size', `${this.editorFontSize}px`);
    document.documentElement.style.setProperty('--grid-font-size', `${this.gridFontSize}px`);
  }

  setTheme(theme: Theme) {
    this.theme = theme;
    this.applyTheme();
    this.persist();
  }

  toggleTheme() {
    this.setTheme(this.theme === 'dark' ? 'light' : 'dark');
  }

  setEditorFontSize(size: number) {
    this.editorFontSize = Math.max(10, Math.min(24, size));
    this.applyFontSizes();
    this.persist();
  }

  setGridFontSize(size: number) {
    this.gridFontSize = Math.max(10, Math.min(24, size));
    this.applyFontSizes();
    this.persist();
  }

  setDefaultPageSize(size: number) {
    this.defaultPageSize = Math.max(10, Math.min(10000, size));
    this.persist();
  }

  setConfirmBeforeDelete(value: boolean) {
    this.confirmBeforeDelete = value;
    this.persist();
  }

  setMaxQueryRows(size: number) {
    this.maxQueryRows = Math.max(100, Math.min(100000, size));
    this.persist();
  }

  setMaxCellSize(size: number) {
    this.maxCellSize = Math.max(64, Math.min(10000, size));
    this.persist();
  }

  setWindowState(w: number, h: number, x: number, y: number, maximized: boolean) {
    this.windowWidth = w;
    this.windowHeight = h;
    this.windowX = x;
    this.windowY = y;
    this.windowMaximized = maximized;
    this.persist();
  }

  setSidebarLayout(width: number, collapsed: boolean) {
    this.sidebarWidth = width;
    this.sidebarCollapsed = collapsed;
    this.persist();
  }

  setRestoreSession(value: boolean) {
    this.restoreSession = value;
    this.persist();
  }

  setLastActiveConnectionId(id: string | null) {
    this.lastActiveConnectionId = id;
    this.persist();
  }

  getSchemaVisibility(connectionId: string): string[] | null {
    return this.schemaVisibility[connectionId] ?? null;
  }

  setSchemaVisibility(connectionId: string, schemas: string[] | null) {
    if (schemas === null) {
      const { [connectionId]: _, ...rest } = this.schemaVisibility;
      this.schemaVisibility = rest;
    } else {
      this.schemaVisibility = { ...this.schemaVisibility, [connectionId]: schemas };
    }
    this.persist();
  }

  getBinding(actionId: string): string {
    if (this.shortcutOverrides[actionId]) {
      return this.shortcutOverrides[actionId];
    }
    const action = DEFAULT_SHORTCUTS.find(s => s.id === actionId);
    return action?.defaultKey ?? '';
  }

  setShortcut(actionId: string, key: string) {
    this.shortcutOverrides = { ...this.shortcutOverrides, [actionId]: key };
    this.persist();
  }

  resetShortcut(actionId: string) {
    const { [actionId]: _, ...rest } = this.shortcutOverrides;
    this.shortcutOverrides = rest;
    this.persist();
  }

  resetAllShortcuts() {
    this.shortcutOverrides = {};
    this.persist();
  }

  /** Convert a KeyboardEvent to a normalized key string like "Ctrl+Shift+K" */
  private eventToKeyString(e: KeyboardEvent): string {
    const parts: string[] = [];
    if (e.ctrlKey || e.metaKey) parts.push('Ctrl');
    if (e.shiftKey) parts.push('Shift');
    if (e.altKey) parts.push('Alt');

    let key = e.key;
    // Normalize key names
    if (key === ' ') key = 'Space';
    else if (key === 'ArrowUp') key = 'Up';
    else if (key === 'ArrowDown') key = 'Down';
    else if (key === 'ArrowLeft') key = 'Left';
    else if (key === 'ArrowRight') key = 'Right';
    else if (key === 'Enter') key = 'Enter';
    else if (key === 'Tab') key = 'Tab';
    else if (key === 'Escape') key = 'Escape';
    else if (key.length === 1) key = key.toUpperCase();

    // Don't add modifier-only keys
    if (['Control', 'Shift', 'Alt', 'Meta'].includes(key)) return '';

    parts.push(key);
    return parts.join('+');
  }

  /** Match a KeyboardEvent against all bindings, return action ID or null */
  matchEvent(e: KeyboardEvent): string | null {
    const keyStr = this.eventToKeyString(e);
    if (!keyStr) return null;

    for (const action of DEFAULT_SHORTCUTS) {
      const binding = this.getBinding(action.id);
      if (binding === keyStr) {
        return action.id;
      }
    }
    return null;
  }

  /** Convert a KeyboardEvent to a display string for recording shortcuts */
  eventToDisplayString(e: KeyboardEvent): string {
    return this.eventToKeyString(e);
  }
}

export const settingsStore = new SettingsStore();
