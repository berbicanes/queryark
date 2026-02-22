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
