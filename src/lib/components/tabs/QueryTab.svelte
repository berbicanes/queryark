<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import { schemaStore } from '$lib/stores/schema.svelte';
  import { savedQueriesStore } from '$lib/stores/savedQueries.svelte';
  import { transactionStore } from '$lib/stores/transaction.svelte';
  import { uiStore } from '$lib/stores/ui.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import * as queryService from '$lib/services/queryService';
  import * as schemaService from '$lib/services/schemaService';
  import * as tauri from '$lib/services/tauri';
  import { DB_METADATA } from '$lib/types/database';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, SortColumn, CellValue } from '$lib/types/query';
  import { extractCellValue } from '$lib/utils/formatters';
  import { buildSqlNamespace, splitStatements, parseErrorPosition, buildExplainQuery } from '$lib/utils/sqlHelpers';
  import SqlEditor from '$lib/components/editor/SqlEditor.svelte';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import Pagination from '$lib/components/grid/Pagination.svelte';
  import ExportMenu from '$lib/components/grid/ExportMenu.svelte';
  import QueryHistory from '$lib/components/editor/QueryHistory.svelte';
  import SavedQueries from '$lib/components/editor/SavedQueries.svelte';
  import QueryPlanViewer from '$lib/components/editor/QueryPlanViewer.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  function getInitialSql() { return tab.sql ?? ''; }
  let sqlValue = $state(getInitialSql());
  let results = $state<QueryResponse[]>([]);
  let isExecuting = $state(false);
  let activeQueryId = $state<string | null>(null);
  let errorMessage = $state<string | null>(null);
  let errorStatementIndex = $state<number | null>(null);
  let errorRange = $state<{ from: number; to: number } | null>(null);

  // Panel state
  let showHistory = $state(false);
  let showSaved = $state(false);
  let saveName = $state('');
  let showSaveInput = $state(false);

  // Transaction state
  let inTransaction = $derived(transactionStore.isInTransaction(tab.connectionId));

  // Explain / query plan
  let showPlan = $state(false);
  let planResult = $state<QueryResponse | null>(null);

  // Client-side sorting per result set
  let sortColumnsMap = $state<Record<number, SortColumn[]>>({});

  // Client-side pagination per result set
  let paginationState = $state<Record<number, { page: number; pageSize: number }>>({});

  // Server-side pagination state per result set
  let serverPaginated = $state<Record<number, boolean>>({});
  let originalSql = $state<Record<number, string>>({});
  let totalRowCounts = $state<Record<number, number | null>>({});
  let pageLoading = $state<Record<number, boolean>>({});

  function getPagination(i: number) {
    return paginationState[i] ?? { page: 1, pageSize: settingsStore.defaultPageSize };
  }

  function getPaginatedRows(i: number): CellValue[][] {
    if (serverPaginated[i]) {
      // Server-paginated: rows are already the correct page
      return results[i]?.rows ?? [];
    }
    const sorted = getSortedRows(i);
    const { page, pageSize } = getPagination(i);
    const start = (page - 1) * pageSize;
    return sorted.slice(start, start + pageSize);
  }

  function getTotalRows(i: number): number {
    if (serverPaginated[i]) {
      return totalRowCounts[i] ?? results[i]?.row_count ?? 0;
    }
    return getSortedRows(i).length;
  }

  async function handlePageChange(i: number, page: number) {
    paginationState[i] = { ...getPagination(i), page };
    paginationState = { ...paginationState };

    if (serverPaginated[i]) {
      await fetchServerPage(i);
    }
  }

  async function handlePageSizeChange(i: number, pageSize: number) {
    paginationState[i] = { page: 1, pageSize };
    paginationState = { ...paginationState };

    if (serverPaginated[i]) {
      await fetchServerPage(i);
    }
  }

  async function fetchServerPage(i: number) {
    const sql = originalSql[i];
    if (!sql) return;

    const { page, pageSize } = getPagination(i);
    const offset = (page - 1) * pageSize;

    pageLoading[i] = true;
    pageLoading = { ...pageLoading };

    try {
      const sorts = sortColumnsMap[i];
      const response = await queryService.executeQueryPage(
        tab.connectionId,
        sql,
        pageSize,
        offset,
        undefined,
        sorts
      );
      if (response) {
        results[i] = response;
        results = [...results];
      }
    } finally {
      pageLoading[i] = false;
      pageLoading = { ...pageLoading };
    }
  }

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

  const DESTRUCTIVE_SQL_PATTERN = /\b(DROP|TRUNCATE|DELETE\s+FROM)\b/i;

  async function executeQuery() {
    if (isExecuting) return;
    if (!sqlValue.trim()) return;

    // Check for destructive SQL and confirm if needed
    if (settingsStore.confirmBeforeDelete && DESTRUCTIVE_SQL_PATTERN.test(sqlValue)) {
      return new Promise<void>((resolve) => {
        uiStore.confirm(
          'This query contains a destructive operation (DROP, TRUNCATE, or DELETE). Execute anyway?',
          () => { doExecuteQuery(); resolve(); }
        );
      });
    }

    return doExecuteQuery();
  }

  async function doExecuteQuery() {
    if (isExecuting) return;
    if (!sqlValue.trim()) return;

    const queryId = crypto.randomUUID();
    activeQueryId = queryId;
    isExecuting = true;
    errorMessage = null;
    errorStatementIndex = null;
    errorRange = null;
    results = [];
    serverPaginated = {};
    originalSql = {};
    totalRowCounts = {};
    pageLoading = {};

    try {
      const statements = splitStatements(sqlValue);

      if (statements.length <= 1) {
        // Single statement — use simple execution
        const response = await queryService.executeQuery(tab.connectionId, sqlValue, queryId);
        if (response) {
          results = [response];
          sortColumnsMap = {};
          paginationState = {};

          // If truncated, enable server-side pagination
          if (response.truncated) {
            serverPaginated = { 0: true };
            originalSql = { 0: sqlValue.trim() };
            totalRowCounts = { 0: null };
            // Async fetch total count
            queryService.countQueryRows(tab.connectionId, sqlValue.trim()).then(count => {
              totalRowCounts = { ...totalRowCounts, 0: count };
            });
          }

          onqueryresult?.({
            executionTime: response.execution_time_ms,
            rowCount: response.row_count
          });
        }
      } else {
        // Multi-statement execution
        const multiResult = await queryService.executeStatements(tab.connectionId, statements, queryId);
        results = multiResult.results;
        sortColumnsMap = {};
        paginationState = {};

        // Check each result for truncation
        for (let i = 0; i < multiResult.results.length; i++) {
          if (multiResult.results[i].truncated) {
            serverPaginated = { ...serverPaginated, [i]: true };
            originalSql = { ...originalSql, [i]: statements[i].trim() };
            totalRowCounts = { ...totalRowCounts, [i]: null };
            const stmtSql = statements[i].trim();
            const idx = i;
            queryService.countQueryRows(tab.connectionId, stmtSql).then(count => {
              totalRowCounts = { ...totalRowCounts, [idx]: count };
            });
          }
        }

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
      activeQueryId = null;
    }

    tabStore.updateTabSql(tab.id, sqlValue);

    // Auto-invalidate schema cache if any DDL statement was executed successfully
    if (!errorMessage) {
      const DDL_PATTERN = /^\s*(CREATE|ALTER|DROP|RENAME|TRUNCATE)\b/im;
      const statements = splitStatements(sqlValue);
      const hasDdl = statements.some(stmt => DDL_PATTERN.test(stmt));
      if (hasDdl) {
        const connId = tab.connectionId;
        schemaStore.clearConnection(connId);
        const conn = connectionStore.activeConnection;
        const dbType = conn?.config.db_type;
        const category = dbType ? DB_METADATA[dbType]?.category : null;
        if (category === 'Relational' || category === 'Analytics' || category === 'WideColumn') {
          schemaService.refreshSchema(connId);
        } else {
          schemaService.refreshContainers(connId);
        }
      }
    }
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

  async function handleSort(resultIndex: number, sorts: SortColumn[]) {
    sortColumnsMap[resultIndex] = sorts;
    sortColumnsMap = { ...sortColumnsMap };
    // Reset to page 1 on sort change
    if (paginationState[resultIndex]) {
      paginationState[resultIndex] = { ...paginationState[resultIndex], page: 1 };
      paginationState = { ...paginationState };
    }

    if (serverPaginated[resultIndex]) {
      await fetchServerPage(resultIndex);
    }
  }

  async function handleExpandCell(resultIndex: number, rowIndex: number, colIndex: number) {
    const result = results[resultIndex];
    if (!result) return;

    const column = result.columns[colIndex];
    if (!column) return;

    // Determine the absolute row offset in the original result set
    let absoluteRowOffset: number;
    if (serverPaginated[resultIndex]) {
      const { page, pageSize } = getPagination(resultIndex);
      absoluteRowOffset = (page - 1) * pageSize + rowIndex;
    } else {
      const { page, pageSize } = getPagination(resultIndex);
      absoluteRowOffset = (page - 1) * pageSize + rowIndex;
    }

    const sql = originalSql[resultIndex] ?? sqlValue.trim();

    const fullCell = await queryService.fetchFullCell(
      tab.connectionId,
      sql,
      column.name,
      absoluteRowOffset
    );

    if (fullCell) {
      // Replace the cell value in the results
      const newResults = [...results];
      const newRows = [...newResults[resultIndex].rows];
      const newRow = [...newRows[rowIndex]];
      newRow[colIndex] = fullCell;
      newRows[rowIndex] = newRow;
      newResults[resultIndex] = { ...newResults[resultIndex], rows: newRows };
      results = newResults;
    }
  }

  async function handleCancel() {
    if (!activeQueryId) return;
    try {
      await tauri.cancelQuery(activeQueryId);
    } catch (err) {
      uiStore.showError(`Failed to cancel query: ${err instanceof Error ? err.message : String(err)}`);
    }
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

  async function handleBeginTransaction() {
    try {
      await tauri.beginTransaction(tab.connectionId);
      transactionStore.setInTransaction(tab.connectionId, true);
      uiStore.showSuccess('Transaction started');
    } catch (err) {
      uiStore.showError(`Failed to begin transaction: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function handleCommitTransaction() {
    try {
      await tauri.commitTransaction(tab.connectionId);
      transactionStore.setInTransaction(tab.connectionId, false);
      uiStore.showSuccess('Transaction committed');
    } catch (err) {
      uiStore.showError(`Failed to commit: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function handleRollbackTransaction() {
    try {
      await tauri.rollbackTransaction(tab.connectionId);
      transactionStore.setInTransaction(tab.connectionId, false);
      uiStore.showSuccess('Transaction rolled back');
    } catch (err) {
      uiStore.showError(`Failed to rollback: ${err instanceof Error ? err.message : String(err)}`);
    }
  }

  async function handleExplain() {
    if (!sqlValue.trim()) return;
    showPlan = false;
    planResult = null;

    try {
      const explainSql = buildExplainQuery(sqlValue.trim(), dialect);
      const response = await queryService.executeQuery(tab.connectionId, explainSql);
      if (response) {
        planResult = response;
        showPlan = true;
      }
    } catch (err) {
      uiStore.showError(`Explain failed: ${err instanceof Error ? err.message : String(err)}`);
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
    window.addEventListener('queryark:execute-query', handleGlobalExecute);
  });

  onDestroy(() => {
    window.removeEventListener('queryark:execute-query', handleGlobalExecute);
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
      <div class="toolbar-separator"></div>
      <button
        class="toolbar-btn"
        onclick={handleExplain}
        title="Explain Query Plan"
      >
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
          <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
        </svg>
        <span>Explain</span>
      </button>
      <div class="toolbar-separator"></div>
      {#if inTransaction}
        <span class="txn-badge">TXN</span>
        <button
          class="toolbar-btn txn-commit"
          onclick={handleCommitTransaction}
          title="Commit Transaction"
        >Commit</button>
        <button
          class="toolbar-btn txn-rollback"
          onclick={handleRollbackTransaction}
          title="Rollback Transaction"
        >Rollback</button>
      {:else}
        <button
          class="toolbar-btn"
          onclick={handleBeginTransaction}
          title="Begin Transaction"
        >Begin</button>
      {/if}
    </div>
    <div class="toolbar-right">
      {#if results.length > 0}
        <ExportMenu
          columns={results[0].columns}
          rows={results[0].rows}
          connectionId={tab.connectionId}
        />
      {/if}
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
        <button class="cancel-btn" onclick={handleCancel} title="Cancel query">Cancel</button>
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
          {#if serverPaginated[i]}
            <div class="server-pagination-info">
              <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="10"></circle>
                <line x1="12" y1="16" x2="12" y2="12"></line>
                <line x1="12" y1="8" x2="12.01" y2="8"></line>
              </svg>
              Server-paginated — {totalRowCounts[i] != null ? `${totalRowCounts[i]!.toLocaleString()} total rows` : 'counting rows...'}
              {#if pageLoading[i]}
                <span class="page-spinner"></span>
              {/if}
            </div>
          {:else if result.truncated}
            <div class="truncation-warning">
              Results limited to {result.max_rows_limit?.toLocaleString()} rows. Full result set may be larger.
            </div>
          {/if}
          <div class="result-grid" class:multi={results.length > 1}>
            <DataGrid
              columns={result.columns}
              rows={getPaginatedRows(i)}
              sortColumns={sortColumnsMap[i] ?? []}
              onSort={(sorts) => handleSort(i, sorts)}
              onExpandCell={(rowIndex, colIndex) => handleExpandCell(i, rowIndex, colIndex)}
            />
          </div>
          {#if getTotalRows(i) > 0}
            <Pagination
              currentPage={getPagination(i).page}
              totalRows={getTotalRows(i)}
              pageSize={getPagination(i).pageSize}
              onPageChange={(page) => handlePageChange(i, page)}
              onPageSizeChange={(size) => handlePageSizeChange(i, size)}
            />
          {/if}
        {/each}
        {#if errorMessage}
          <div class="error-state partial">
            <span class="error-icon">!</span>
            <span>{errorMessage}</span>
          </div>
        {/if}
      </div>
    {:else if showPlan && planResult}
      <div class="plan-container">
        <div class="plan-header">
          <span>Query Plan</span>
          <button class="toolbar-btn" onclick={() => { showPlan = false; planResult = null; }}>Close</button>
        </div>
        <QueryPlanViewer planData={planResult} {dialect} />
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
    transition: background var(--transition-micro, 80ms ease);
  }

  .split-handle:hover,
  .query-tab.resizing .split-handle {
    background: rgba(122, 162, 247, 0.1);
  }

  .handle-bar {
    width: 32px;
    height: 2px;
    background: var(--text-muted);
    border-radius: 1px;
    opacity: 0.4;
    transition: opacity var(--transition-micro, 80ms ease);
  }

  .split-handle:hover .handle-bar {
    opacity: 0.7;
    background: var(--accent);
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

  .page-spinner {
    display: inline-block;
    width: 10px;
    height: 10px;
    border: 1.5px solid var(--border-color);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin-left: 4px;
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
    border-left: 3px solid var(--error);
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

  .toolbar-separator {
    width: 1px;
    height: 16px;
    background: var(--border-color);
    margin: 0 4px;
  }

  .txn-badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 6px;
    font-size: 9px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--warning, #fab387);
    background: rgba(250, 179, 135, 0.15);
    border: 1px solid rgba(250, 179, 135, 0.3);
    border-radius: var(--radius-sm);
  }

  .txn-commit {
    color: var(--success, #a6e3a1) !important;
  }

  .txn-rollback {
    color: var(--error) !important;
  }

  .plan-container {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }

  .plan-header {
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

  .truncation-warning {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--warning, #fab387);
    background: rgba(250, 179, 135, 0.08);
    border-bottom: 1px solid rgba(250, 179, 135, 0.2);
    flex-shrink: 0;
  }

  .server-pagination-info {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 4px 12px;
    font-size: 11px;
    color: var(--accent);
    background: rgba(137, 180, 250, 0.06);
    border-bottom: 1px solid rgba(137, 180, 250, 0.15);
    flex-shrink: 0;
  }

  .cancel-btn {
    padding: 4px 12px;
    font-size: 12px;
    font-family: var(--font-sans);
    color: var(--error);
    background: none;
    border: 1px solid var(--error);
    border-radius: var(--radius-sm);
    cursor: pointer;
    margin-left: 8px;
    transition: background var(--transition-micro, 80ms ease);
  }

  .cancel-btn:hover {
    background: rgba(243, 139, 168, 0.15);
  }
</style>
