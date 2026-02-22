<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { tabStore } from '$lib/stores/tabs.svelte';
  import { connectionStore } from '$lib/stores/connections.svelte';
  import * as queryService from '$lib/services/queryService';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, SortColumn, CellValue } from '$lib/types/query';
  import { extractCellValue } from '$lib/utils/formatters';
  import SqlEditor from '$lib/components/editor/SqlEditor.svelte';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  let sqlValue = $state(tab.sql ?? '');
  let result = $state<QueryResponse | null>(null);
  let isExecuting = $state(false);
  let errorMessage = $state<string | null>(null);

  // Client-side sorting for query results
  let sortColumns = $state<SortColumn[]>([]);

  // Resizable split
  let splitRatio = $state(0.45);
  let isResizing = $state(false);
  let containerEl: HTMLDivElement;

  // Determine the SQL dialect from the connection
  let dialect = $derived.by(() => {
    const conn = connectionStore.connections.find(c => c.config.id === tab.connectionId);
    return conn?.config.db_type ?? 'PostgreSQL';
  });

  // Sort rows client-side
  let sortedRows = $derived.by(() => {
    if (!result || sortColumns.length === 0) return result?.rows ?? [];

    const rows = [...result.rows];
    rows.sort((a, b) => {
      for (const sort of sortColumns) {
        const colIdx = result!.columns.findIndex(c => c.name === sort.column);
        if (colIdx === -1) continue;

        const aVal = extractCellValue(a[colIdx]);
        const bVal = extractCellValue(b[colIdx]);

        // Try numeric comparison
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
  });

  async function executeQuery() {
    if (isExecuting) return;
    if (!sqlValue.trim()) return;

    isExecuting = true;
    errorMessage = null;

    try {
      const response = await queryService.executeQuery(tab.connectionId, sqlValue);
      if (response) {
        result = response;
        sortColumns = [];
        onqueryresult?.({
          executionTime: response.execution_time_ms,
          rowCount: response.row_count
        });
      }
    } catch (err) {
      errorMessage = err instanceof Error ? err.message : String(err);
    } finally {
      isExecuting = false;
    }

    tabStore.updateTabSql(tab.id, sqlValue);
  }

  function handleSort(sorts: SortColumn[]) {
    sortColumns = sorts;
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
  <div class="editor-panel" style="height: {splitRatio * 100}%">
    <SqlEditor
      bind:value={sqlValue}
      onexecute={executeQuery}
      {dialect}
    />
  </div>

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
    {:else if errorMessage}
      <div class="error-state">
        <span class="error-icon">!</span>
        <span>{errorMessage}</span>
      </div>
    {:else if result}
      <DataGrid
        columns={result.columns}
        rows={sortedRows}
        {sortColumns}
        onSort={handleSort}
      />
    {:else}
      <div class="empty-state">
        <span class="text-muted">Run a query to see results</span>
        <span class="text-muted" style="font-size: 11px;">Press Ctrl+Enter to execute</span>
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

  .editor-panel {
    flex-shrink: 0;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 60px;
  }

  .results-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    min-height: 60px;
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
