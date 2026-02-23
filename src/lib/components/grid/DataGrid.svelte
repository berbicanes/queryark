<script lang="ts">
  import { untrack } from 'svelte';
  import type { ColumnDef, CellValue, SortColumn, FilterCondition } from '$lib/types/query';
  import { extractCellValue, isNull } from '$lib/utils/formatters';
  import { copyAsCsv, copyAsMarkdown } from '$lib/services/exportService';
  import GridHeader from './GridHeader.svelte';
  import GridRow from './GridRow.svelte';
  import FilterBar from './FilterBar.svelte';
  import ContextMenu from './ContextMenu.svelte';

  let {
    columns,
    rows,
    editable = false,
    schema,
    table,
    sortColumns = [],
    filters = [],
    modifiedCells,
    deletedRows,
    onCellEdit,
    onCellSetNull,
    onSort,
    onFiltersChange,
    onFilterByValue,
    onExpandCell,
  }: {
    columns: ColumnDef[];
    rows: CellValue[][];
    editable?: boolean;
    schema?: string;
    table?: string;
    sortColumns?: SortColumn[];
    filters?: FilterCondition[];
    modifiedCells?: Map<number, Set<number>>;
    deletedRows?: Set<number>;
    onCellEdit?: (rowIndex: number, colIndex: number, value: string) => void;
    onCellSetNull?: (rowIndex: number, colIndex: number) => void;
    onSort?: (sorts: SortColumn[]) => void;
    onFiltersChange?: (filters: FilterCondition[]) => void;
    onFilterByValue?: (column: string, value: string) => void;
    onExpandCell?: (rowIndex: number, colIndex: number) => void;
  } = $props();

  const ROW_HEIGHT = 32;
  const BUFFER_ROWS = 10;
  const DEFAULT_COL_WIDTH = 150;

  let scrollContainer = $state<HTMLDivElement | undefined>(undefined);
  let gridWrapper: HTMLDivElement;
  let scrollTop = $state(0);
  let containerHeight = $state(400);

  // Column widths
  let columnWidths = $state<Record<string, number>>({});

  // Column order: array of indices into the original columns array
  let columnOrder = $state<number[]>([]);

  // Row selection
  let selectedRows = $state<Set<number>>(new Set());
  let lastSelectedRow = $state<number | null>(null);

  // Filter bar visibility
  let showFilterBar = $state(false);

  // Context menu
  let contextMenu = $state<{
    x: number;
    y: number;
    rowIndex: number;
    colIndex: number;
  } | null>(null);

  // Initialize column widths and order when columns change
  // Use untrack for columnWidths read to avoid a read-write reactive cycle
  $effect(() => {
    const cols = columns;
    const prevWidths = untrack(() => columnWidths);
    const newWidths: Record<string, number> = {};
    for (const col of cols) {
      newWidths[col.name] = prevWidths[col.name] ?? DEFAULT_COL_WIDTH;
    }
    columnWidths = newWidths;
    columnOrder = cols.map((_, i) => i);
  });

  // Reorder columns based on columnOrder
  let orderedColumns = $derived(columnOrder.map(i => columns[i]).filter(Boolean));

  // Reorder each row's cells to match column order
  function reorderRow(row: CellValue[]): CellValue[] {
    return columnOrder.map(i => row[i]).filter((c): c is CellValue => c !== undefined);
  }

  // Map from ordered index back to original column index
  function originalColIndex(orderedIdx: number): number {
    return columnOrder[orderedIdx] ?? orderedIdx;
  }

  // Virtual scroll
  let totalHeight = $derived(rows.length * ROW_HEIGHT);
  let startIndex = $derived(Math.max(0, Math.floor(scrollTop / ROW_HEIGHT) - BUFFER_ROWS));
  let endIndex = $derived(Math.min(rows.length, Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT) + BUFFER_ROWS));
  let visibleRows = $derived(rows.slice(startIndex, endIndex));
  let offsetY = $derived(startIndex * ROW_HEIGHT);

  let allSelected = $derived(rows.length > 0 && selectedRows.size === rows.length);
  let someSelected = $derived(selectedRows.size > 0 && selectedRows.size < rows.length);

  function handleScroll() {
    if (scrollContainer) {
      scrollTop = scrollContainer.scrollTop;
    }
  }

  function handleResize() {
    if (scrollContainer) {
      containerHeight = scrollContainer.clientHeight;
    }
  }

  $effect(() => {
    if (scrollContainer) {
      containerHeight = scrollContainer.clientHeight;
    }
  });

  function handleColumnResize(column: string, width: number) {
    columnWidths = { ...columnWidths, [column]: width };
  }

  function handleCellEdit(rowIndex: number, orderedColIndex: number, value: string) {
    onCellEdit?.(startIndex + rowIndex, originalColIndex(orderedColIndex), value);
  }

  function handleCellSetNull(rowIndex: number, orderedColIndex: number) {
    onCellSetNull?.(startIndex + rowIndex, originalColIndex(orderedColIndex));
  }

  function handleExpandCell(rowIndex: number, orderedColIndex: number) {
    onExpandCell?.(startIndex + rowIndex, originalColIndex(orderedColIndex));
  }

  function handleSelectAll(selected: boolean) {
    if (selected) {
      selectedRows = new Set(rows.map((_, i) => i));
    } else {
      selectedRows = new Set();
    }
  }

  function handleRowSelect(absoluteRowIndex: number, e: MouseEvent) {
    const newSelection = new Set(selectedRows);

    if (e.shiftKey && lastSelectedRow !== null) {
      // Range select
      const start = Math.min(lastSelectedRow, absoluteRowIndex);
      const end = Math.max(lastSelectedRow, absoluteRowIndex);
      for (let i = start; i <= end; i++) {
        newSelection.add(i);
      }
    } else if (e.ctrlKey || e.metaKey) {
      // Toggle individual
      if (newSelection.has(absoluteRowIndex)) {
        newSelection.delete(absoluteRowIndex);
      } else {
        newSelection.add(absoluteRowIndex);
      }
    } else {
      // Single select
      if (newSelection.has(absoluteRowIndex) && newSelection.size === 1) {
        newSelection.delete(absoluteRowIndex);
      } else {
        newSelection.clear();
        newSelection.add(absoluteRowIndex);
      }
    }

    selectedRows = newSelection;
    lastSelectedRow = absoluteRowIndex;
  }

  function handleReorder(fromIndex: number, toIndex: number) {
    const newOrder = [...columnOrder];
    const [moved] = newOrder.splice(fromIndex, 1);
    newOrder.splice(toIndex, 0, moved);
    columnOrder = newOrder;
  }

  function handleContextMenu(rowIndex: number, colIndex: number, e: MouseEvent) {
    e.preventDefault();
    contextMenu = {
      x: e.clientX,
      y: e.clientY,
      rowIndex: startIndex + rowIndex,
      colIndex: originalColIndex(colIndex),
    };

    // Also select the row if not already selected
    if (!selectedRows.has(startIndex + rowIndex)) {
      selectedRows = new Set([startIndex + rowIndex]);
      lastSelectedRow = startIndex + rowIndex;
    }
  }

  function handleFilterToggle() {
    showFilterBar = !showFilterBar;
    if (!showFilterBar && filters.length > 0) {
      onFiltersChange?.([]);
    }
  }

  // Copy helpers
  function getSelectedRowsData(): CellValue[][] {
    if (selectedRows.size === 0) return [];
    return Array.from(selectedRows).sort((a, b) => a - b).map(i => rows[i]).filter(Boolean);
  }

  function cellToString(cell: CellValue): string {
    if (isNull(cell)) return '';
    return extractCellValue(cell);
  }

  function rowToTsv(row: CellValue[]): string {
    return orderedColumns.map((col, i) => {
      const origIdx = originalColIndex(i);
      return cellToString(row[origIdx]);
    }).join('\t');
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text);
  }

  function handleCopyCell() {
    if (!contextMenu) return;
    const cell = rows[contextMenu.rowIndex]?.[contextMenu.colIndex];
    if (cell) {
      copyToClipboard(cellToString(cell));
    }
  }

  function handleCopyRows() {
    const dataRows = getSelectedRowsData();
    if (dataRows.length === 0 && contextMenu) {
      const row = rows[contextMenu.rowIndex];
      if (row) {
        copyToClipboard(rowToTsv(row));
        return;
      }
    }
    const header = orderedColumns.map(c => c.name).join('\t');
    const body = dataRows.map(r => rowToTsv(r)).join('\n');
    copyToClipboard(header + '\n' + body);
  }

  function handleCopyAsInsert() {
    const dataRows = getSelectedRowsData();
    const targetRows = dataRows.length > 0 ? dataRows : (contextMenu ? [rows[contextMenu.rowIndex]].filter(Boolean) : []);
    if (targetRows.length === 0) return;

    const colNames = orderedColumns.map(c => `"${c.name}"`).join(', ');
    const tableName = schema && table ? `"${schema}"."${table}"` : '"table"';
    const inserts = targetRows.map(row => {
      const vals = orderedColumns.map((_, i) => {
        const origIdx = originalColIndex(i);
        const cell = row[origIdx];
        if (isNull(cell)) return 'NULL';
        const v = extractCellValue(cell);
        if (cell.type === 'Int' || cell.type === 'Float' || cell.type === 'Bool') return v;
        return `'${v.replace(/'/g, "''")}'`;
      }).join(', ');
      return `INSERT INTO ${tableName} (${colNames}) VALUES (${vals});`;
    }).join('\n');

    copyToClipboard(inserts);
  }

  function handleCopyAsJson() {
    const dataRows = getSelectedRowsData();
    const targetRows = dataRows.length > 0 ? dataRows : (contextMenu ? [rows[contextMenu.rowIndex]].filter(Boolean) : []);
    if (targetRows.length === 0) return;

    const jsonArray = targetRows.map(row => {
      const obj: Record<string, unknown> = {};
      orderedColumns.forEach((col, i) => {
        const origIdx = originalColIndex(i);
        const cell = row[origIdx];
        if (isNull(cell)) {
          obj[col.name] = null;
        } else if (cell.type === 'Bool') {
          obj[col.name] = cell.value;
        } else if (cell.type === 'Int' || cell.type === 'Float') {
          obj[col.name] = cell.value;
        } else {
          obj[col.name] = extractCellValue(cell);
        }
      });
      return obj;
    });

    copyToClipboard(JSON.stringify(jsonArray, null, 2));
  }

  function handleSetNullFromMenu() {
    if (!contextMenu) return;
    onCellSetNull?.(contextMenu.rowIndex, contextMenu.colIndex);
  }

  function handleFilterByValue() {
    if (!contextMenu) return;
    const cell = rows[contextMenu.rowIndex]?.[contextMenu.colIndex];
    const col = columns[contextMenu.colIndex];
    if (cell && col && !isNull(cell)) {
      onFilterByValue?.(col.name, extractCellValue(cell));
    }
  }

  function handleCopyAsCsv() {
    const dataRows = getSelectedRowsData();
    const targetRows = dataRows.length > 0 ? dataRows : (contextMenu ? [rows[contextMenu.rowIndex]].filter(Boolean) : []);
    if (targetRows.length === 0) return;
    copyAsCsv(orderedColumns, targetRows.map(r => reorderRow(r)));
  }

  function handleCopyAsMarkdown() {
    const dataRows = getSelectedRowsData();
    const targetRows = dataRows.length > 0 ? dataRows : (contextMenu ? [rows[contextMenu.rowIndex]].filter(Boolean) : []);
    if (targetRows.length === 0) return;
    copyAsMarkdown(orderedColumns, targetRows.map(r => reorderRow(r)));
  }

  // Keyboard copy handler
  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 'c') {
      if (selectedRows.size > 0) {
        e.preventDefault();
        const header = orderedColumns.map(c => c.name).join('\t');
        const body = getSelectedRowsData().map(r => rowToTsv(r)).join('\n');
        copyToClipboard(header + '\n' + body);
      }
    }
  }
</script>

<svelte:window onresize={handleResize} />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="data-grid" bind:this={gridWrapper} onkeydown={handleKeydown}>
  {#if columns.length === 0}
    <div class="empty-state">
      <span class="text-muted">No results</span>
    </div>
  {:else}
    <div class="grid-header-wrapper">
      <GridHeader
        columns={orderedColumns}
        {columnWidths}
        {sortColumns}
        showSelectAll={editable || true}
        {allSelected}
        {someSelected}
        onResize={handleColumnResize}
        onSort={onSort}
        onSelectAll={handleSelectAll}
        onFilterToggle={onFiltersChange ? handleFilterToggle : undefined}
        {showFilterBar}
        onReorder={handleReorder}
      />
    </div>
    {#if showFilterBar}
      <FilterBar
        columns={orderedColumns}
        {columnWidths}
        {filters}
        showCheckbox={true}
        onFiltersChange={onFiltersChange}
      />
    {/if}
    <div
      class="grid-body"
      bind:this={scrollContainer}
      onscroll={handleScroll}
    >
      <div class="virtual-spacer" style="height: {totalHeight}px; position: relative;">
        <div class="virtual-rows" style="transform: translateY({offsetY}px);">
          {#each visibleRows as row, i}
            {@const absoluteIndex = startIndex + i}
            <GridRow
              row={reorderRow(row)}
              columns={orderedColumns}
              {columnWidths}
              rowIndex={absoluteIndex}
              {editable}
              selected={selectedRows.has(absoluteIndex)}
              showCheckbox={true}
              isDeleted={deletedRows?.has(absoluteIndex) ?? false}
              modifiedCells={modifiedCells?.get(absoluteIndex)}
              onCellEdit={(colIndex, value) => handleCellEdit(i, colIndex, value)}
              onCellSetNull={(colIndex) => handleCellSetNull(i, colIndex)}
              onSelect={handleRowSelect}
              onContextMenu={(rowIdx, colIdx, e) => handleContextMenu(i, colIdx, e)}
              onExpandCell={onExpandCell ? (colIndex) => handleExpandCell(i, colIndex) : undefined}
            />
          {/each}
        </div>
      </div>
    </div>
    <div class="grid-footer">
      <span class="row-info">{rows.length} row{rows.length !== 1 ? 's' : ''}</span>
      {#if selectedRows.size > 0}
        <span class="selection-info">{selectedRows.size} selected</span>
      {/if}
    </div>
  {/if}

  {#if contextMenu}
    {@const ctxRow = rows[contextMenu.rowIndex]}
    {@const ctxCell = ctxRow?.[contextMenu.colIndex]}
    {@const ctxCol = columns[contextMenu.colIndex]}
    {#if ctxRow && ctxCell && ctxCol}
      <ContextMenu
        x={contextMenu.x}
        y={contextMenu.y}
        cell={ctxCell}
        column={ctxCol}
        row={ctxRow}
        columns={orderedColumns}
        {schema}
        {table}
        selectedRows={getSelectedRowsData()}
        {editable}
        onClose={() => { contextMenu = null; }}
        onCopyCell={handleCopyCell}
        onCopyRows={handleCopyRows}
        onCopyAsInsert={handleCopyAsInsert}
        onCopyAsJson={handleCopyAsJson}
        onCopyAsCsv={handleCopyAsCsv}
        onCopyAsMarkdown={handleCopyAsMarkdown}
        onSetNull={editable ? handleSetNullFromMenu : undefined}
        onFilterByValue={onFilterByValue ? handleFilterByValue : undefined}
      />
    {/if}
  {/if}
</div>

<style>
  .data-grid {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
    background: var(--bg-primary);
    outline: none;
  }

  .grid-header-wrapper {
    flex-shrink: 0;
    overflow: hidden;
  }

  .grid-body {
    flex: 1;
    overflow: auto;
  }

  .virtual-spacer {
    width: 100%;
  }

  .virtual-rows {
    width: 100%;
  }

  .grid-footer {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 4px 12px;
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .row-info {
    font-size: 11px;
    color: var(--text-muted);
    font-family: var(--font-mono);
  }

  .selection-info {
    font-size: 11px;
    color: var(--accent);
    font-family: var(--font-mono);
  }

  .empty-state {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100%;
    font-size: 13px;
  }
</style>
