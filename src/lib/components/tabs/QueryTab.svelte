<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import * as queryService from '$lib/services/queryService';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, SortColumn, CellValue } from '$lib/types/query';
  import { extractCellValue } from '$lib/utils/formatters';
  import { buildSqlNamespace, splitStatements, parseErrorPosition } from '$lib/utils/sqlHelpers';
  import SqlEditor from '$lib/components/editor/SqlEditor.svelte';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import QueryHistory from '$lib/components/editor/QueryHistory.svelte';
  import SavedQueries from '$lib/components/editor/SavedQueries.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  function getInitialSql() { return tab.sql ?? ''; }
  let sqlValue = $state(getInitialSql());
  let results = $state<QueryResponse[]>([]);
  let isExecuting = $state(false);
  let errorMessage = $state<string | null>(null);
  let errorStatementIndex = $state<number | null>(null);
  let errorRange = $state<{ from: number; to: number } | null>(null);

  // Panel state
  let showHistory = $state(false);
  let showSaved = $state(false);
  let saveName = $state('');
  let showSaveInput = $state(false);

  // Client-side sorting per result set
  let sortColumnsMap = $state<Record<number, SortColumn[]>>({});

  // Resizable split
  let splitRatio = $state(0.45);
  let isResizing = $state(false);
  let containerEl: HTMLDivElement;

  // Determine the SQL dialect from the connection
  let dialect = $derived.by(() => {
    const conn = connectionStore.connections.find(c => c.config.id === tab.connectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  // Build schema namespace for autocomplete
  let schemaNamespace = $derived.by(() => {
    return buildSqlNamespace(tab.connectionId);
  });

  // Sort rows for a given result index
  function getSortedRows(resultIndex: number): CellValue[][] {
    const result = results[resultIndex];
    if (!result) return [];
    const sorts = sortColumnsMap[resultIndex] ?? [];
    if (sorts.length === 0) return result.rows;

    const rows = [...result.rows];
    rows.sort((a, b) => {
      for (const sort of sorts) {
        const colIdx = result.columns.findIndex(c => c.name === sort.column);
        if (colIdx === -1) continue;

        const aVal = extractCellValue(a[colIdx]);
        const bVal = extractCellValue(b[colIdx]);

        const aNum = Number(aVal);
        const bNum = Number(bVal);
        let cmp: number;
        if (!isNaN(aNum) && !isNaN(bNum)) {
          cmp = aNum - bNum;
        } else {
          cmp = aVal.localeCompare(bVal);
        }

        if (cmp !== 0) {
          return sort.direction === 'DESC' ? -cmp : cmp;
        }
      }
      return 0;
    });
    return rows;
  }

  async function executeQuery() {
    if (isExecuting) return;
    if (!sqlValue.trim()) return;

    isExecuting = true;
    errorMessage = null;
    errorStatementIndex = null;
    errorRange = null;
    results = [];

    try {
      const statements = splitStatements(sqlValue);

      if (statements.length <= 1) {
        // Single statement — use simple execution
        const response = await queryService.executeQuery(tab.connectionId, sqlValue);
        if (response) {
          results = [response];
          sortColumnsMap = {};
          onqueryresult?.({
            executionTime: response.execution_time_ms,
            rowCount: response.row_count
          });
        }
      } else {
        // Multi-statement execution
        const multiResult = await queryService.executeStatements(tab.connectionId, statements);
        results = multiResult.results;
        sortColumnsMap = {};

        if (multiResult.error) {
          errorStatementIndex = multiResult.error.index;
          errorMessage = `Statement ${multiResult.error.index + 1}: ${multiResult.error.message}`;

          // Compute error range in the original SQL by finding the offset of the failed statement
          const failedStmt = statements[multiResult.error.index];
          const failedOffset = findStatementOffset(sqlValue, statements, multiResult.error.index);
          const parsed = parseErrorPosition(multiResult.error.message, failedStmt);
          if (parsed && failedOffset >= 0) {
            errorRange = {
              from: failedOffset + parsed.from,
              to: failedOffset + parsed.to,
            };
          }
        } else {
          // Report totals
          const totalTime = results.reduce((s, r) => s + r.execution_time_ms, 0);
          const totalRows = results.reduce((s, r) => s + r.row_count, 0);
          onqueryresult?.({ executionTime: totalTime, rowCount: totalRows });
        }
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      errorMessage = msg;

      // Try to highlight the error position
      const parsed = parseErrorPosition(msg, sqlValue);
      if (parsed) {
        errorRange = parsed;
      }
    } finally {
      isExecuting = false;
    }

    tabStore.updateTabSql(tab.id, sqlValue);
  }

  /**
   * Find the character offset of the Nth statement in the original SQL string.
   */
  function findStatementOffset(fullSql: string, statements: string[], index: number): number {
    let searchFrom = 0;
    for (let i = 0; i <= index; i++) {
      const pos = fullSql.indexOf(statements[i], searchFrom);
      if (i === index) return pos >= 0 ? pos : 0;
      if (pos >= 0) searchFrom = pos + statements[i].length;
    }
    return 0;
  }

  function handleSort(resultIndex: number, sorts: SortColumn[]) {
    sortColumnsMap[resultIndex] = sorts;
    sortColumnsMap = { ...sortColumnsMap }; // trigger reactivity
  }

  function handleSplitMouseDown(e: MouseEvent) {
    isResizing = true;
    e.preventDefault();
  }

  function handleMouseMove(e: MouseEvent) {
    if (!isResizing || !containerEl) return;
    const rect = containerEl.getBoundingClientRect();
    const ratio = (e.clientY - rect.top) / rect.height;
    splitRatio = Math.max(0.15, Math.min(0.85, ratio));
  }

  function handleMouseUp() {
    isResizing = false;
  }

  function handleGlobalExecute() {
    if (tabStore.activeTabId === tab.id) {
      executeQuery();
    }
  }

  function handleHistorySelect(sql: string) {
    sqlValue = sql;
    showHistory = false;
  }

  function handleSavedSelect(sql: string) {
    sqlValue = sql;
    showSaved = false;
  }

  function handleSave() {
    if (showSaveInput) {
      if (saveName.trim() && sqlValue.trim()) {
        savedQueriesStore.save(saveName.trim(), tab.connectionId, sqlValue);
        saveName = '';
        showSaveInput = false;
      }
    } else {
      showSaveInput = true;
    }
  }

  function handleSaveKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      handleSave();
    } else if (e.key === 'Escape') {
      showSaveInput = false;
      saveName = '';
    }
  }

  function toggleHistory() {
    showHistory = !showHistory;
    if (showHistory) showSaved = false;
  }

  function toggleSaved() {
    showSaved = !showSaved;
    if (showSaved) showHistory = false;
  }

  onMount(() => {
    window.addEventListener('dataforge:execute-query', handleGlobalExecute);
  });

  onDestroy(() => {
    window.removeEventListener('dataforge:execute-query', handleGlobalExecute);
  });
</script>

<svelte:window
  onmousemove={handleMouseMove}
  onmouseup={handleMouseUp}
/>

<div class="query-tab" class:resizing={isResizing} bind:this={containerEl}>
  <div class="editor-section" style="height: {splitRatio * 100}%">
    {#if showHistory || showSaved}
      <div class="editor-with-panel">
        <div class="editor-main">
          <SqlEditor
            bind:value={sqlValue}
            onexecute={executeQuery}
            {dialect}
            {schemaNamespace}
            {errorRange}
          />
        </div>
        <div class="side-panel">
          {#if showHistory}
            <QueryHistory
              connectionId={tab.connectionId}
              onselect={handleHistorySelect}
              onclose={() => showHistory = false}
            />
          {:else if showSaved}
            <SavedQueries
              connectionId={tab.connectionId}
              onselect={handleSavedSelect}
              onclose={() => showSaved = false}
            />
          {/if}
        </div>
      </div>
    {:else}
      <SqlEditor
        bind:value={sqlValue}
        onexecute={executeQuery}
        {dialect}
        {schemaNamespace}
        {errorRange}
      />
    {/if}
  </div>

  <div class="editor-toolbar">
    <div class="toolbar-left">
      <button
        class="toolbar-btn"
        class:active={showHistory}
        onclick={toggleHistory}
        title="Query History"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="12" cy="12" r="10"></circle>
          <polyline points="12 6 12 12 16 14"></polyline>
        </svg>
        <span>History</span>
      </button>
      <button
        class="toolbar-btn"
        class:active={showSaved}
        onclick={toggleSaved}
        title="Saved Queries"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 21l-7-5-7 5V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
        </svg>
        <span>Saved</span>
      </button>
    </div>
    <div class="toolbar-right">
      {#if showSaveInput}
        <input
          class="save-input"
          type="text"
          placeholder="Query name..."
          bind:value={saveName}
          onkeydown={handleSaveKeydown}
          onblur={() => { if (!saveName.trim()) showSaveInput = false; }}
        />
      {/if}
      <button
        class="toolbar-btn"
        onclick={handleSave}
        title="Save Query"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2z"></path>
          <polyline points="17 21 17 13 7 13 7 21"></polyline>
          <polyline points="7 3 7 8 15 8"></polyline>
        </svg>
        <span>Save</span>
      </button>
    </div>
  </div>

  <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
  <div
    class="split-handle"
    onmousedown={handleSplitMouseDown}
    role="separator"
    aria-orientation="horizontal"
    tabindex="-1"
  >
    <div class="handle-bar"></div>
  </div>

  <div class="results-panel" style="height: {(1 - splitRatio) * 100}%">
    {#if isExecuting}
      <div class="loading-state">
        <span class="spinner"></span>
        <span>Executing query...</span>
      </div>
    {:else if errorMessage && results.length === 0}
      <div class="error-state">
        <span class="error-icon">!</span>
        <span>{errorMessage}</span>
      </div>
    {:else if results.length > 0}
      <div class="results-container">
        {#each results as result, i (i)}
          {#if results.length > 1}
            <div class="result-label">
              <span>Statement {i + 1}</span>
              <span class="result-meta">{result.row_count} rows, {result.execution_time_ms}ms</span>
            </div>
          {/if}
          <div class="result-grid" class:multi={results.length > 1}>
            <DataGrid
              columns={result.columns}
              rows={getSortedRows(i)}
              sortColumns={sortColumnsMap[i] ?? []}
              onSort={(sorts) => handleSort(i, sorts)}
            />
          </div>
        {/each}
        {#if errorMessage}
          <div class="error-state partial">
            <span class="error-icon">!</span>
            <span>{errorMessage}</span>
          </div>
        {/if}
      </div>
    {:else}
      <div class="empty-state">
        <span class="text-muted">Run a query to see results</span>
        <span class="text-muted" style="font-size: 11px;">Press Ctrl+Enter to execute | Ctrl+Shift+F to format | Ctrl+/ to comment</span>
      </div>
    {/if}
  </div>
</div>

<style>
  .query-tab {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .query-tab.resizing {
    user-select: none;
    cursor: row-resize;
  }

  .editor-section {
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 60px;
  }

  .editor-with-panel {
    display: flex;
    height: 100%;
  }

  .editor-main {
    flex: 1;
    overflow: hidden;
    min-width: 0;
  }

  .side-panel {
    width: 320px;
    flex-shrink: 0;
    border-left: 1px solid var(--border-color);
    overflow: hidden;
  }

  .editor-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 2px 8px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
    min-height: 28px;
  }

  .toolbar-left,
  .toolbar-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .toolbar-btn {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    border: none;
    background: none;
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-sans);
    cursor: pointer;
    border-radius: var(--radius-sm);
    white-space: nowrap;
  }

  .toolbar-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .toolbar-btn.active {
    background: var(--bg-active);
    color: var(--accent);
  }

  .save-input {
    padding: 2px 6px;
    font-size: 11px;
    background: var(--bg-primary);
    color: var(--text-primary);
    border: 1px solid var(--accent);
    border-radius: var(--radius-sm);
    outline: none;
    font-family: var(--font-sans);
    width: 140px;
  }

  .results-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 60px;
  }

  .results-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow-y: auto;
  }

  .result-label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 4px 8px;
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    font-size: 11px;
    font-weight: 600;
    color: var(--text-secondary);
    flex-shrink: 0;
  }

  .result-meta {
    font-weight: 400;
    color: var(--text-muted);
  }

  .result-grid {
    flex: 1;
    overflow: hidden;
    min-height: 150px;
  }

  .result-grid.multi {
    max-height: 300px;
    flex: none;
  }

  .split-handle {
    height: 5px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    border-bottom: 1px solid var(--border-color);
    cursor: row-resize;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .split-handle:hover,
  .query-tab.resizing .split-handle {
    background: var(--bg-hover);
  }

  .handle-bar {
    width: 32px;
    height: 2px;
    background: var(--text-muted);
    border-radius: 1px;
    opacity: 0.5;
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

  .error-state {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 12px 16px;
    color: var(--error);
    font-size: 13px;
    background: rgba(243, 139, 168, 0.05);
  }

  .error-state.partial {
    flex-shrink: 0;
    border-top: 1px solid var(--border-color);
  }

  .error-icon {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    background: var(--error);
    color: var(--bg-primary);
    font-weight: 700;
    font-size: 12px;
    flex-shrink: 0;
  }

  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    gap: 4px;
    font-size: 13px;
  }
</style>
