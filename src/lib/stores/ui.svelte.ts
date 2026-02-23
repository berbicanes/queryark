import type { DatabaseType } from '$lib/types/connection';
import type { QuerySnippet, CellValue, ColumnDef } from '$lib/types/query';

export interface CreateTableContext {
  connectionId: string;
  schema: string;
  dbType: DatabaseType;
}

export interface AlterTableContext {
  connectionId: string;
  schema: string;
  table: string;
  dbType: DatabaseType;
}

export interface IndexModalContext {
  connectionId: string;
  schema: string;
  table: string;
  dbType: DatabaseType;
}

class UiStore {
  sidebarWidth = $state(260);
  showConnectionModal = $state(false);
  showConfirmDialog = $state(false);
  confirmDialogMessage = $state('');
  confirmDialogCallback = $state<(() => void) | null>(null);
  isLoading = $state(false);
  loadingMessage = $state('');
  errorMessage = $state<string | null>(null);
  successMessage = $state<string | null>(null);

  // Phase 7 — UX modals & sidebar
  showShortcutsModal = $state(false);
  showCommandPalette = $state(false);
  sidebarCollapsed = $state(false);

  // Home/welcome dashboard — visible on app start
  showHome = $state(true);

  // Phase 10 — Settings
  showSettingsModal = $state(false);

  // Phase 13 — About
  showAboutModal = $state(false);

  // Schema management modals
  showCreateTableModal = $state(false);
  createTableContext = $state<CreateTableContext | null>(null);
  showAlterTableModal = $state(false);
  alterTableContext = $state<AlterTableContext | null>(null);
  showIndexModal = $state(false);
  indexModalContext = $state<IndexModalContext | null>(null);

  // Phase 21 — Collaboration & Workflow
  showSnippetModal = $state(false);
  showSnippetLibrary = $state(false);
  snippetToEdit = $state<QuerySnippet | null>(null);
  showSnippetVariablePrompt = $state(false);
  snippetToInsert = $state<QuerySnippet | null>(null);
  showWorkspaceModal = $state(false);
  showBookmarkList = $state(false);

  // Phase 23 — Quality & Trust
  showWhatsNewModal = $state(false);

  // Database backup/dump
  showDatabaseBackupModal = $state(false);
  databaseBackupConnectionId = $state<string | null>(null);

  // Phase 22 — Advanced Query Features
  showParameterPrompt = $state(false);
  parameterPromptSql = $state('');
  parameterPromptCallback = $state<((sql: string) => void) | null>(null);
  comparisonBuffer = $state<{ columns: ColumnDef[]; rows: CellValue[][]; sql: string } | null>(null);

  dismissHome() {
    if (!this.showHome) return;
    this.showHome = false;
    this.sidebarCollapsed = false;
  }

  openConnectionModal() {
    this.showConnectionModal = true;
  }

  closeConnectionModal() {
    this.showConnectionModal = false;
  }

  confirm(message: string, callback: () => void) {
    this.confirmDialogMessage = message;
    this.confirmDialogCallback = callback;
    this.showConfirmDialog = true;
  }

  closeConfirmDialog() {
    this.showConfirmDialog = false;
    this.confirmDialogCallback = null;
  }

  setLoading(loading: boolean, message = '') {
    this.isLoading = loading;
    this.loadingMessage = message;
  }

  showError(message: string) {
    this.errorMessage = message;
    setTimeout(() => {
      this.errorMessage = null;
    }, 5000);
  }

  showSuccess(message: string) {
    this.successMessage = message;
    setTimeout(() => {
      this.successMessage = null;
    }, 3000);
  }
}

export const uiStore = new UiStore();
