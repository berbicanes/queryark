export interface ShortcutAction {
  id: string;
  label: string;
  category: string;
  defaultKey: string;
}

export const DEFAULT_SHORTCUTS: ShortcutAction[] = [
  { id: 'newQuery', label: 'New Query Tab', category: 'General', defaultKey: 'Ctrl+N' },
  { id: 'closeTab', label: 'Close Tab', category: 'Tabs', defaultKey: 'Ctrl+W' },
  { id: 'nextTab', label: 'Next Tab', category: 'Tabs', defaultKey: 'Ctrl+Tab' },
  { id: 'prevTab', label: 'Previous Tab', category: 'Tabs', defaultKey: 'Ctrl+Shift+Tab' },
  { id: 'globalSearch', label: 'Command Palette', category: 'General', defaultKey: 'Ctrl+P' },
  { id: 'toggleSidebar', label: 'Toggle Sidebar', category: 'General', defaultKey: 'Ctrl+B' },
  { id: 'runQuery', label: 'Run Query', category: 'Editor', defaultKey: 'Ctrl+Enter' },
  { id: 'formatSql', label: 'Format SQL', category: 'Editor', defaultKey: 'Ctrl+Shift+F' },
  { id: 'saveQuery', label: 'Save Query', category: 'Editor', defaultKey: 'Ctrl+S' },
  { id: 'refreshSchema', label: 'Refresh Schema', category: 'General', defaultKey: 'F5' },
  { id: 'toggleTheme', label: 'Toggle Theme', category: 'General', defaultKey: 'Ctrl+Shift+T' },
  { id: 'shortcuts', label: 'Keyboard Shortcuts', category: 'General', defaultKey: 'Ctrl+K' },
  { id: 'openDiagram', label: 'Open ER Diagram', category: 'General', defaultKey: '' },
  { id: 'openTableDiff', label: 'Table Structure Diff', category: 'General', defaultKey: '' },
  { id: 'openDataDiff', label: 'Data Diff', category: 'General', defaultKey: '' },
  { id: 'openVisualQuery', label: 'Visual Query Builder', category: 'General', defaultKey: '' },
  { id: 'openSnippets', label: 'Query Snippets', category: 'General', defaultKey: '' },
  { id: 'openWorkspaces', label: 'Workspace Profiles', category: 'General', defaultKey: '' },
  { id: 'openBookmarks', label: 'Result Bookmarks', category: 'General', defaultKey: '' },
  { id: 'toggleChart', label: 'Toggle Chart View', category: 'Query', defaultKey: '' },
  { id: 'compareResults', label: 'Compare Results', category: 'Query', defaultKey: '' },
];

export type ShortcutId = typeof DEFAULT_SHORTCUTS[number]['id'];
