<script lang="ts">
  import { onMount } from 'svelte';
  import * as tauri from '$lib/services/tauri';
  import { uiStore } from '$lib/stores/ui.svelte';
  import type { Tab } from '$lib/types/tabs';
  import type { QueryResponse, SortColumn, FilterCondition } from '$lib/types/query';
  import DataGrid from '$lib/components/grid/DataGrid.svelte';
  import Pagination from '$lib/components/grid/Pagination.svelte';
  import TableStructure from '$lib/components/structure/TableStructure.svelte';

  let { tab, onqueryresult }: {
    tab: Tab;
    onqueryresult?: (detail: { executionTime: number; rowCount: number }) => void;
  } = $props();

  let activeSubTab = $state<'data' | 'structure'>('data');
  let result = $state<QueryResponse | null>(null);
  let isLoading = $state(false);
  let totalRows = $state(0);
  let currentPage = $state(1);
  let pageSize = $state(50);

  // Sort & filter state
  let sortColumns = $state<SortColumn[]>([]);
  let filters = $state<FilterCondition[]>([]);

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

  onMount(() => {
    loadData();
    loadTotalRows();
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
    <span class="sub-tab-title truncate">{tab.schema}.{tab.table}</span>
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
    {:else}
      <TableStructure
        connectionId={tab.connectionId}
        schema={tab.schema ?? ''}
        table={tab.table ?? ''}
      />
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
    color: var(--text-secondary);
    border: none;
    background: none;
    cursor: pointer;
    position: relative;
    transition: color var(--transition-fast);
  }

  .sub-tab:hover {
    color: var(--text-primary);
  }

  .sub-tab.active {
    color: var(--accent);
  }

  .sub-tab.active::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--accent);
    border-radius: 1px;
  }

  .sub-tab-title {
    margin-left: auto;
    font-size: 11px;
    color: var(--text-muted);
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
</style>
