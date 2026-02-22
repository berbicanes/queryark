<script lang="ts">
  import type { ColumnDef, SortColumn } from '$lib/types/query';

  let {
    columns,
    columnWidths,
    sortColumns = [],
    showSelectAll = false,
    allSelected = false,
    someSelected = false,
    onResize,
    onSort,
    onSelectAll,
    onFilterToggle,
    showFilterBar = false,
    onReorder,
  }: {
    columns: ColumnDef[];
    columnWidths: Record<string, number>;
    sortColumns?: SortColumn[];
    showSelectAll?: boolean;
    allSelected?: boolean;
    someSelected?: boolean;
    onResize?: (column: string, width: number) => void;
    onSort?: (sorts: SortColumn[]) => void;
    onSelectAll?: (selected: boolean) => void;
    onFilterToggle?: () => void;
    showFilterBar?: boolean;
    onReorder?: (fromIndex: number, toIndex: number) => void;
  } = $props();

  // Resize state
  let resizing = $state<{ column: string; startX: number; startWidth: number } | null>(null);

  // Drag reorder state
  let dragIndex = $state<number | null>(null);
  let dropIndex = $state<number | null>(null);

  function getSortInfo(colName: string): { direction: 'ASC' | 'DESC'; index: number } | null {
    const idx = sortColumns.findIndex(s => s.column === colName);
    if (idx === -1) return null;
    return { direction: sortColumns[idx].direction as 'ASC' | 'DESC', index: idx };
  }

  function handleSortClick(colName: string, e: MouseEvent) {
    if (!onSort) return;
    const existing = sortColumns.findIndex(s => s.column === colName);

    if (e.shiftKey) {
      // Multi-sort: add/toggle/remove
      const newSorts = [...sortColumns];
      if (existing >= 0) {
        if (newSorts[existing].direction === 'ASC') {
          newSorts[existing] = { ...newSorts[existing], direction: 'DESC' };
        } else {
          newSorts.splice(existing, 1);
        }
      } else {
        newSorts.push({ column: colName, direction: 'ASC' });
      }
      onSort(newSorts);
    } else {
      // Single sort: cycle none -> ASC -> DESC -> none
      if (existing >= 0) {
        if (sortColumns[existing].direction === 'ASC') {
          onSort([{ column: colName, direction: 'DESC' }]);
        } else {
          onSort([]);
        }
      } else {
        onSort([{ column: colName, direction: 'ASC' }]);
      }
    }
  }

  function handleResizeStart(column: string, e: MouseEvent) {
    e.preventDefault();
    e.stopPropagation();
    const w = columnWidths[column] ?? 150;
    resizing = { column, startX: e.clientX, startWidth: w };

    function onMove(ev: MouseEvent) {
      if (!resizing) return;
      const delta = ev.clientX - resizing.startX;
      const newWidth = Math.max(60, resizing.startWidth + delta);
      onResize?.(resizing.column, newWidth);
    }

    function onUp() {
      resizing = null;
      window.removeEventListener('mousemove', onMove);
      window.removeEventListener('mouseup', onUp);
    }

    window.addEventListener('mousemove', onMove);
    window.addEventListener('mouseup', onUp);
  }

  function handleDragStart(index: number, e: DragEvent) {
    dragIndex = index;
    e.dataTransfer!.effectAllowed = 'move';
    e.dataTransfer!.setData('text/plain', String(index));
  }

  function handleDragOver(index: number, e: DragEvent) {
    e.preventDefault();
    e.dataTransfer!.dropEffect = 'move';
    dropIndex = index;
  }

  function handleDragLeave() {
    dropIndex = null;
  }

  function handleDrop(index: number, e: DragEvent) {
    e.preventDefault();
    if (dragIndex !== null && dragIndex !== index) {
      onReorder?.(dragIndex, index);
    }
    dragIndex = null;
    dropIndex = null;
  }

  function handleDragEnd() {
    dragIndex = null;
    dropIndex = null;
  }

  function handleSelectAllChange() {
    onSelectAll?.(!allSelected);
  }
</script>

<div class="grid-header">
  <div class="header-row">
    {#if showSelectAll}
      <div class="header-cell checkbox-cell">
        <input
          type="checkbox"
          checked={allSelected}
          indeterminate={someSelected && !allSelected}
          onchange={handleSelectAllChange}
          class="row-checkbox"
        />
      </div>
    {/if}
    <div class="header-cell row-number">#</div>
    {#each columns as col, i}
      {@const sortInfo = getSortInfo(col.name)}
      {@const w = columnWidths[col.name] ?? 150}
      <div
        class="header-cell data-col"
        class:drop-target={dropIndex === i && dragIndex !== i}
        class:dragging={dragIndex === i}
        style="width: {w}px; min-width: {w}px; max-width: {w}px;"
        title="{col.name} ({col.data_type})"
        onclick={(e) => handleSortClick(col.name, e)}
        onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); handleSortClick(col.name, e as unknown as MouseEvent); } }}
        draggable="true"
        ondragstart={(e) => handleDragStart(i, e)}
        ondragover={(e) => handleDragOver(i, e)}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(i, e)}
        ondragend={handleDragEnd}
        role="columnheader"
        tabindex="-1"
      >
        <div class="col-info">
          <span class="col-name">{col.name}</span>
          <span class="col-type">{col.data_type}</span>
        </div>
        {#if sortInfo}
          <span class="sort-indicator">
            {#if sortInfo.direction === 'ASC'}
              <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
                <path d="M8 3l5 8H3z" fill="currentColor"/>
              </svg>
            {:else}
              <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
                <path d="M8 13l5-8H3z" fill="currentColor"/>
              </svg>
            {/if}
            {#if sortColumns.length > 1}
              <span class="sort-order">{sortInfo.index + 1}</span>
            {/if}
          </span>
        {/if}
        <!-- svelte-ignore a11y_no_noninteractive_element_interactions -->
        <div
          class="resize-handle"
          onmousedown={(e) => handleResizeStart(col.name, e)}
          role="separator"
          tabindex="-1"
        ></div>
      </div>
    {/each}
    {#if onFilterToggle}
      <div class="header-cell filter-toggle-cell">
        <button
          class="filter-toggle-btn"
          class:active={showFilterBar}
          onclick={(e) => { e.stopPropagation(); onFilterToggle?.(); }}
          title="Toggle filters"
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <path d="M1 3h14M3 8h10M5 13h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
          </svg>
        </button>
      </div>
    {/if}
  </div>
</div>

<style>
  .grid-header {
    background: var(--bg-tertiary);
    border-bottom: 2px solid var(--border-color);
  }

  .header-row {
    display: flex;
  }

  .header-cell {
    display: flex;
    flex-direction: column;
    justify-content: center;
    padding: 4px 10px;
    border-right: 1px solid var(--border-color);
    overflow: hidden;
    user-select: none;
  }

  .header-cell.data-col {
    flex: none;
    position: relative;
    cursor: pointer;
    flex-direction: row;
    align-items: center;
    gap: 4px;
  }

  .header-cell.data-col:hover {
    background: rgba(255, 255, 255, 0.03);
  }

  .header-cell.drop-target {
    background: rgba(137, 180, 250, 0.1);
    border-left: 2px solid var(--accent);
  }

  .header-cell.dragging {
    opacity: 0.4;
  }

  .header-cell.row-number {
    min-width: 50px;
    max-width: 50px;
    flex: 0;
    text-align: center;
    font-size: 10px;
    color: var(--text-muted);
    font-weight: 600;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .header-cell.checkbox-cell {
    min-width: 32px;
    max-width: 32px;
    flex: 0;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
  }

  .header-cell.filter-toggle-cell {
    min-width: 32px;
    max-width: 32px;
    flex: 0;
    flex-direction: row;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    border-right: none;
  }

  .col-info {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    flex: 1;
    min-width: 0;
  }

  .col-name {
    font-size: 12px;
    font-weight: 600;
    color: var(--text-primary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.3;
  }

  .col-type {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    line-height: 1.3;
  }

  .sort-indicator {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    color: var(--accent);
    flex-shrink: 0;
  }

  .sort-order {
    font-size: 9px;
    font-weight: 700;
  }

  .resize-handle {
    position: absolute;
    right: 0;
    top: 0;
    bottom: 0;
    width: 5px;
    cursor: col-resize;
    z-index: 1;
  }

  .resize-handle:hover {
    background: var(--accent);
    opacity: 0.4;
  }

  .row-checkbox {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--accent);
  }

  .filter-toggle-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    background: none;
    border: none;
    border-radius: var(--radius-sm);
    color: var(--text-muted);
    cursor: pointer;
    padding: 0;
  }

  .filter-toggle-btn:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .filter-toggle-btn.active {
    color: var(--accent);
  }
</style>
