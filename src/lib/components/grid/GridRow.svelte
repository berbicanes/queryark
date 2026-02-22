<script lang="ts">
  import type { ColumnDef, CellValue } from '$lib/types/query';
  import GridCell from './GridCell.svelte';

  let {
    row,
    columns,
    columnWidths,
    rowIndex,
    editable = false,
    selected = false,
    showCheckbox = false,
    onCellEdit,
    onCellSetNull,
    onSelect,
    onContextMenu,
  }: {
    row: CellValue[];
    columns: ColumnDef[];
    columnWidths: Record<string, number>;
    rowIndex: number;
    editable?: boolean;
    selected?: boolean;
    showCheckbox?: boolean;
    onCellEdit?: (colIndex: number, value: string) => void;
    onCellSetNull?: (colIndex: number) => void;
    onSelect?: (rowIndex: number, e: MouseEvent) => void;
    onContextMenu?: (rowIndex: number, colIndex: number, e: MouseEvent) => void;
  } = $props();

  let isEven = $derived(rowIndex % 2 === 0);

  function handleCheckboxClick(e: MouseEvent) {
    onSelect?.(rowIndex, e);
  }

  function handleRowContextMenu(colIndex: number, e: MouseEvent) {
    onContextMenu?.(rowIndex, colIndex, e);
  }
</script>

<div class="grid-row" class:even={isEven} class:odd={!isEven} class:selected>
  {#if showCheckbox}
    <div class="checkbox-cell">
      <input
        type="checkbox"
        checked={selected}
        onclick={handleCheckboxClick}
        class="row-checkbox"
      />
    </div>
  {/if}
  <div class="row-number-cell">
    {rowIndex + 1}
  </div>
  {#each row as cell, colIndex}
    {@const w = columnWidths[columns[colIndex]?.name] ?? 150}
    <GridCell
      value={cell}
      column={columns[colIndex]}
      width={w}
      {editable}
      onEdit={onCellEdit ? (val) => onCellEdit!(colIndex, val) : undefined}
      onSetNull={onCellSetNull ? () => onCellSetNull!(colIndex) : undefined}
      onContextMenu={(e) => handleRowContextMenu(colIndex, e)}
    />
  {/each}
</div>

<style>
  .grid-row {
    display: flex;
    height: 32px;
    border-bottom: 1px solid rgba(69, 71, 90, 0.3);
  }

  .grid-row.even {
    background: var(--bg-primary);
  }

  .grid-row.odd {
    background: rgba(42, 42, 60, 0.3);
  }

  .grid-row:hover {
    background: var(--bg-hover) !important;
  }

  .grid-row.selected {
    background: rgba(137, 180, 250, 0.1) !important;
  }

  .grid-row.selected:hover {
    background: rgba(137, 180, 250, 0.15) !important;
  }

  .row-number-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 50px;
    max-width: 50px;
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    border-right: 1px solid var(--border-color);
    user-select: none;
    flex-shrink: 0;
  }

  .checkbox-cell {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 32px;
    max-width: 32px;
    flex-shrink: 0;
    border-right: 1px solid rgba(69, 71, 90, 0.3);
  }

  .row-checkbox {
    width: 14px;
    height: 14px;
    cursor: pointer;
    accent-color: var(--accent);
  }
</style>
