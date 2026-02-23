<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as tauri from '$lib/services/tauri';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { changeTracker } from '$lib/stores/changeTracker.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, SortColumn, FilterCondition } from '$lib/types/query';
  import { extractCellValue } from '$lib/utils/formatters';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import Pagination from '$lib/components/grid/Pagination.svelte';
  import ExportMenu from '$lib/components/grid/ExportMenu.svelte';
  import TableStructure from '$lib/components/structure/TableStructure.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  let activeSubTab = $state<'data' | 'structure' | 'ddl'>('data');
  let ddlText = $state<string | null>(null);
  let ddlLoading = $state(false);
  let ddlCopied = $state(false);
  let result = $state<QueryResponse | null>(null);
  let isLoading = $state(false);
  let totalRows = $state(0);
  let currentPage = $state(1);
  let pageSize = $state(50);

  // Sort & filter state
  let sortColumns = $state<SortColumn[]>([]);
  let filters = $state<FilterCondition[]>([]);

  // Bulk edit mode
  let bulkEditMode = $state(false);
  let hasChanges = $derived(changeTracker.hasChanges(tab.id));
  let changeCount = $derived(changeTracker.changeCount(tab.id));
  let canUndo = $derived(changeTracker.canUndo(tab.id));
  let canRedo = $derived(changeTracker.canRedo(tab.id));

  // Modified cells map for visual indicators
  let modifiedCells = $derived.by(() => {
    if (!bulkEditMode) return undefined;
    const map = new Map<number, Set<number>>();
    for (const change of changeTracker.getChanges(tab.id)) {
      if (change.type === 'cell_edit') {
        if (!map.has(change.rowIndex)) map.set(change.rowIndex, new Set());
        map.get(change.rowIndex)!.add(change.colIndex);
      }
    }
    return map;
  });

  let deletedRows = $derived.by(() => {
    if (!bulkEditMode) return undefined;
    const set = new Set<number>();
    for (const change of changeTracker.getChanges(tab.id)) {
      if (change.type === 'row_delete') set.add(change.rowIndex);
    }
    return set;
  });

  // Determine DB type for this connection
  let dbType = $derived.by(() => {
    const conn = connectionStore.connections.find(c => c.config.id === tab.connectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  let offset = $derived((currentPage - 1) * pageSize);

  async function loadData() {
    if (!tab.schema || !tab.table) return;

    isLoading = true;
    try {
      const response = await tauri.getTableData(
        tab.connectionId, tab.schema, tab.table,
        pageSize, offset, sortColumns, filters
      );
      result = response;
      onqueryresult?.({
        executionTime: response.execution_time_ms,
        rowCount: response.row_count
      });
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      uiStore.showError(`Failed to load table data: ${message}`);
    } finally {
      isLoading = false;
    }
  }

  async function loadTotalRows() {
    if (!tab.schema || !tab.table) return;
    try {
      totalRows = await tauri.getRowCount(tab.connectionId, tab.schema, tab.table, filters);
    } catch {
      totalRows = 0;
    }
  }

  async function loadDdl() {
    if (!tab.schema || !tab.table) return;
    ddlLoading = true;
    try {
      ddlText = await tauri.exportDdl(tab.connectionId, tab.schema, tab.table);
    } catch (err) {
      ddlText = `-- Failed to load DDL: ${err instanceof Error ? err.message : String(err)}`;
    } finally {
      ddlLoading = false;
    }
  }

  async function copyDdl() {
    if (!ddlText) return;
    try {
      await navigator.clipboard.writeText(ddlText);
      ddlCopied = true;
      setTimeout(() => { ddlCopied = false; }, 2000);
    } catch {
      uiStore.showError('Failed to copy DDL to clipboard');
    }
  }

  function handlePageChange(page: number) {
    currentPage = page;
    loadData();
  }

  function handlePageSizeChange(size: number) {
    pageSize = size;
    currentPage = 1;
    loadData();
  }

  function handleSort(sorts: SortColumn[]) {
    sortColumns = sorts;
    currentPage = 1;
    loadData();
  }

  function handleFiltersChange(newFilters: FilterCondition[]) {
    filters = newFilters;
    currentPage = 1;
    loadData();
    loadTotalRows();
  }

  function handleFilterByValue(column: string, value: string) {
    // Add or replace a filter for this column
    const newFilters = filters.filter(f => f.column !== column);
    newFilters.push({ column, operator: 'eq', value });
    filters = newFilters;
    currentPage = 1;
    loadData();
    loadTotalRows();
  }

  function handleCellEdit(rowIndex: number, colIndex: number, value: string) {
    if (!result || !tab.schema || !tab.table) return;
    const columns = result.columns;
    const row = result.rows[rowIndex];
    if (!row) return;

    if (bulkEditMode) {
      const oldValue = extractCellValue(row[colIndex]);
      changeTracker.addCellEdit(tab.id, rowIndex, colIndex, oldValue, value);
      return;
    }

    // Use first column as PK (simplistic — matches existing behavior)
    const pkColumns = [columns[0].name];
    const pkCell = row[0];
    const pkValues = [pkCell.type === 'Null' ? '' : ('value' in pkCell ? String(pkCell.value) : '')];
    const column = columns[colIndex].name;

    tauri.updateCell(tab.connectionId, tab.schema, tab.table, column, value, pkColumns, pkValues)
      .then(() => loadData())
      .catch(err => {
        const message = err instanceof Error ? err.message : String(err);
        uiStore.showError(`Failed to update cell: ${message}`);
      });
  }

  function handleCellSetNull(rowIndex: number, colIndex: number) {
    if (!result || !tab.schema || !tab.table) return;
    const columns = result.columns;
    const row = result.rows[rowIndex];
    if (!row) return;

    if (bulkEditMode) {
      const oldValue = extractCellValue(row[colIndex]);
      changeTracker.addCellEdit(tab.id, rowIndex, colIndex, oldValue, '', true);
      return;
    }

    const pkColumns = [columns[0].name];
    const pkCell = row[0];
    const pkValues = [pkCell.type === 'Null' ? '' : ('value' in pkCell ? String(pkCell.value) : '')];
    const column = columns[colIndex].name;

    tauri.updateCell(tab.connectionId, tab.schema, tab.table, column, '', pkColumns, pkValues, true)
      .then(() => loadData())
      .catch(err => {
        const message = err instanceof Error ? err.message : String(err);
        uiStore.showError(`Failed to set NULL: ${message}`);
      });
  }

  function toggleBulkEdit() {
    if (bulkEditMode && hasChanges) {
      uiStore.confirm('Discard unsaved changes?', () => {
        changeTracker.discard(tab.id);
        bulkEditMode = false;
      });
    } else {
      bulkEditMode = !bulkEditMode;
      if (!bulkEditMode) {
        changeTracker.discard(tab.id);
      }
    }
  }

  async function applyChanges() {
    if (!result || !tab.schema || !tab.table) return;
    const changes = changeTracker.getChanges(tab.id);
    if (changes.length === 0) return;

    isLoading = true;
    try {
      for (const change of changes) {
        if (change.type === 'cell_edit') {
          const columns = result.columns;
          const row = result.rows[change.rowIndex];
          if (!row) continue;
          const pkColumns = [columns[0].name];
          const pkCell = row[0];
          const pkValues = [pkCell.type === 'Null' ? '' : ('value' in pkCell ? String(pkCell.value) : '')];
          const column = columns[change.colIndex].name;
          await tauri.updateCell(
            tab.connectionId, tab.schema!, tab.table!,
            column, change.newValue, pkColumns, pkValues, change.isNull
          );
        } else if (change.type === 'row_delete') {
          await tauri.deleteRows(
            tab.connectionId, tab.schema!, tab.table!,
            change.pkColumns, [change.pkValues]
          );
        } else if (change.type === 'row_insert') {
          await tauri.insertRow(
            tab.connectionId, tab.schema!, tab.table!,
            change.columns, change.values
          );
        }
      }
      changeTracker.discard(tab.id);
      uiStore.showSuccess(`Applied ${changes.length} change(s)`);
      await loadData();
      await loadTotalRows();
      if (tab.schema && tab.table) {
        schemaStore.clearTableStats(tab.connectionId, tab.schema, tab.table);
      }
    } catch (err) {
      uiStore.showError(`Failed to apply changes: ${err instanceof Error ? err.message : String(err)}`);
    } finally {
      isLoading = false;
    }
  }

  function discardChanges() {
    changeTracker.discard(tab.id);
  }

  function handleUndo() {
    changeTracker.undo(tab.id);
  }

  function handleRedo() {
    changeTracker.redo(tab.id);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (!bulkEditMode) return;
    if ((e.ctrlKey || e.metaKey) && e.key === 'z' && !e.shiftKey) {
      e.preventDefault();
      handleUndo();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'z' && e.shiftKey) {
      e.preventDefault();
      handleRedo();
    } else if ((e.ctrlKey || e.metaKey) && e.key === 'Z') {
      e.preventDefault();
      handleRedo();
    }
  }

  function openCreateTable() {
    uiStore.showCreateTableModal = true;
    uiStore.createTableContext = {
      connectionId: tab.connectionId,
      schema: tab.schema ?? '',
      dbType,
    };
  }

  function openAlterTable() {
    uiStore.showAlterTableModal = true;
    uiStore.alterTableContext = {
      connectionId: tab.connectionId,
      schema: tab.schema ?? '',
      table: tab.table ?? '',
      dbType,
    };
  }

  function openIndexModal() {
    uiStore.showIndexModal = true;
    uiStore.indexModalContext = {
      connectionId: tab.connectionId,
      schema: tab.schema ?? '',
      table: tab.table ?? '',
      dbType,
    };
  }

  onMount(() => {
    loadData();
    loadTotalRows();
    window.addEventListener('keydown', handleKeydown);
  });

  onDestroy(() => {
    window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="table-tab">
  <div class="sub-tab-bar">
    <button
      class="sub-tab"
      class:active={activeSubTab === 'data'}
      onclick={() => { activeSubTab = 'data'; }}
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <rect x="1" y="1" width="14" height="14" rx="1" stroke="currentColor" stroke-width="1.2" fill="none"/>
        <line x1="1" y1="5" x2="15" y2="5" stroke="currentColor" stroke-width="1"/>
        <line x1="6" y1="5" x2="6" y2="15" stroke="currentColor" stroke-width="1"/>
      </svg>
      Data
    </button>
    <button
      class="sub-tab"
      class:active={activeSubTab === 'structure'}
      onclick={() => { activeSubTab = 'structure'; }}
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M2 4h12M2 8h12M2 12h8" stroke="currentColor" stroke-width="1.2" stroke-linecap="round"/>
      </svg>
      Structure
    </button>
    <button
      class="sub-tab"
      class:active={activeSubTab === 'ddl'}
      onclick={() => { activeSubTab = 'ddl'; if (!ddlText && !ddlLoading) loadDdl(); }}
    >
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M4 2h6l4 4v8a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1z" stroke="currentColor" stroke-width="1.2" fill="none"/>
        <polyline points="10 2 10 6 14 6" stroke="currentColor" stroke-width="1.2" fill="none"/>
      </svg>
      DDL
    </button>
    <span class="sub-tab-title truncate">{tab.schema}.{tab.table}</span>
    {#if activeSubTab === 'data'}
      <div class="sub-tab-actions">
        <button
          class="sub-tab-btn"
          class:active={bulkEditMode}
          onclick={toggleBulkEdit}
          title={bulkEditMode ? 'Exit bulk edit mode' : 'Enter bulk edit mode'}
        >
          {bulkEditMode ? 'Exit Bulk' : 'Bulk Edit'}
        </button>
        {#if bulkEditMode && hasChanges}
          <span class="change-count">{changeCount}</span>
          <button class="sub-tab-btn apply" onclick={applyChanges} title="Apply all changes">Apply</button>
          <button class="sub-tab-btn discard" onclick={discardChanges} title="Discard all changes">Discard</button>
          <button class="sub-tab-btn" onclick={handleUndo} disabled={!canUndo} title="Undo (Ctrl+Z)">Undo</button>
          <button class="sub-tab-btn" onclick={handleRedo} disabled={!canRedo} title="Redo (Ctrl+Shift+Z)">Redo</button>
        {/if}
      </div>
      {#if result}
        <ExportMenu
          columns={result.columns}
          rows={result.rows}
          connectionId={tab.connectionId}
          schema={tab.schema}
          table={tab.table}
          showDdl={true}
          showImport={true}
          showExportAll={true}
          onImportComplete={() => { loadData(); loadTotalRows(); }}
        />
      {/if}
    {:else if activeSubTab === 'structure'}
      <div class="sub-tab-actions">
        <button class="sub-tab-btn" onclick={openAlterTable} title="Alter Table">Alter Table</button>
        <button class="sub-tab-btn" onclick={openIndexModal} title="Manage Indexes">Indexes</button>
        <button class="sub-tab-btn" onclick={openCreateTable} title="Create New Table">New Table</button>
      </div>
    {:else if activeSubTab === 'ddl'}
      <div class="sub-tab-actions">
        <button class="sub-tab-btn" onclick={() => { ddlText = null; loadDdl(); }} title="Refresh DDL">Refresh</button>
        <button class="sub-tab-btn" class:copied={ddlCopied} onclick={copyDdl} disabled={!ddlText || ddlLoading} title="Copy DDL">
          {ddlCopied ? 'Copied!' : 'Copy'}
        </button>
      </div>
    {/if}
  </div>

  <div class="sub-tab-content">
    {#if activeSubTab === 'data'}
      <div class="data-view">
        {#if isLoading}
          <div class="loading-state">
            <span class="spinner"></span>
            <span>Loading data...</span>
          </div>
        {:else if result}
          <div class="grid-wrapper">
            <DataGrid
              columns={result.columns}
              rows={result.rows}
              editable={true}
              schema={tab.schema}
              table={tab.table}
              {sortColumns}
              {filters}
              {modifiedCells}
              {deletedRows}
              onCellEdit={handleCellEdit}
              onCellSetNull={handleCellSetNull}
              onSort={handleSort}
              onFiltersChange={handleFiltersChange}
              onFilterByValue={handleFilterByValue}
            />
          </div>
          <Pagination
            {currentPage}
            {totalRows}
            {pageSize}
            onPageChange={handlePageChange}
            onPageSizeChange={handlePageSizeChange}
          />
        {:else}
          <div class="empty-state">
            <span class="text-muted">No data loaded</span>
          </div>
        {/if}
      </div>
    {:else if activeSubTab === 'structure'}
      <TableStructure
        connectionId={tab.connectionId}
        schema={tab.schema ?? ''}
        table={tab.table ?? ''}
      />
    {:else if activeSubTab === 'ddl'}
      <div class="ddl-view">
        {#if ddlLoading}
          <div class="loading-state">
            <span class="spinner"></span>
            <span>Loading DDL...</span>
          </div>
        {:else if ddlText}
          <pre class="ddl-code"><code>{ddlText}</code></pre>
        {:else}
          <div class="empty-state">
            <span class="text-muted">No DDL available</span>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

<style>
  .table-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .sub-tab-bar {
    display: flex;
    align-items: center;
    gap: 0;
    height: 32px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    padding: 0 8px;
    flex-shrink: 0;
  }

  .sub-tab {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    font-size: 12px;
    color: var(--text-muted);
    border: none;
    background: none;
    cursor: pointer;
    position: relative;
    transition: color var(--transition-subtle, 150ms ease);
  }

  .sub-tab:hover {
    color: var(--text-primary);
  }

  .sub-tab.active {
    color: var(--accent);
    font-weight: 600;
  }

  .sub-tab.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--accent);
    border-radius: 1px 1px 0 0;
  }

  .sub-tab-title {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .sub-tab-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    margin-left: auto;
  }

  .sub-tab-btn {
    padding: 2px 8px;
    font-size: 11px;
    font-family: var(--font-sans);
    border: 1px solid var(--border-color);
    background: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

  .sub-tab-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .sub-tab-btn.active {
    background: var(--bg-active);
    color: var(--accent);
    border-color: var(--accent);
  }

  .sub-tab-btn.apply {
    color: var(--success, #a6e3a1);
    border-color: var(--success, #a6e3a1);
  }

  .sub-tab-btn.discard {
    color: var(--error);
    border-color: var(--error);
  }

  .sub-tab-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }

  .change-count {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    min-width: 18px;
    height: 18px;
    padding: 0 4px;
    font-size: 10px;
    font-weight: 700;
    color: var(--warning, #fab387);
    background: rgba(250, 179, 135, 0.15);
    border-radius: 9px;
    font-family: var(--font-mono);
  }

  .sub-tab-content {
    flex: 1;
    overflow: hidden;
  }

  .data-view {
    display: flex;
    flex-direction: column;
    height: 100%;
  }

  .grid-wrapper {
    flex: 1;
    overflow: hidden;
  }

  .loading-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 8px;
    color: var(--text-secondary);
    font-size: 13px;
  }

  .spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }

  .sub-tab-btn.copied {
    color: var(--success, #a6e3a1);
    border-color: var(--success, #a6e3a1);
  }

  .ddl-view {
    height: 100%;
    overflow: auto;
    background: var(--bg-primary);
  }

  .ddl-code {
    margin: 0;
    padding: 16px;
    font-family: var(--font-mono);
    font-size: 13px;
    line-height: 1.6;
    color: var(--text-primary);
    white-space: pre;
    tab-size: 2;
    overflow: auto;
    height: 100%;
  }
</style>
