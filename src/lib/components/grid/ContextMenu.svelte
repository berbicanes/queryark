<script lang="ts">
  import type { ColumnDef, CellValue } from '$lib/types/query';
  import { extractCellValue, isNull } from '$lib/utils/formatters';

  let {
    x,
    y,
    cell,
    column,
    row,
    columns,
    schema,
    table,
    selectedRows = [],
    editable = false,
    onClose,
    onCopyCell,
    onCopyRows,
    onCopyAsInsert,
    onCopyAsJson,
    onSetNull,
    onFilterByValue,
  }: {
    x: number;
    y: number;
    cell: CellValue;
    column: ColumnDef;
    row: CellValue[];
    columns: ColumnDef[];
    schema?: string;
    table?: string;
    selectedRows?: CellValue[][];
    editable?: boolean;
    onClose: () => void;
    onCopyCell?: () => void;
    onCopyRows?: () => void;
    onCopyAsInsert?: () => void;
    onCopyAsJson?: () => void;
    onSetNull?: () => void;
    onFilterByValue?: () => void;
  } = $props();

  let menuEl: HTMLDivElement;

  // Clamp position to viewport after first render
  let clamped = $state({ x: 0, y: 0 });

  $effect(() => {
    const cx = x;
    const cy = y;
    if (menuEl) {
      const rect = menuEl.getBoundingClientRect();
      const vw = window.innerWidth;
      const vh = window.innerHeight;
      clamped = {
        x: cx + rect.width > vw ? Math.max(0, vw - rect.width - 4) : cx,
        y: cy + rect.height > vh ? Math.max(0, vh - rect.height - 4) : cy,
      };
    } else {
      clamped = { x: cx, y: cy };
    }
  });

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      onClose();
    }
  }

  function handleClickOutside(e: MouseEvent) {
    if (menuEl && !menuEl.contains(e.target as Node)) {
      onClose();
    }
  }

  let cellIsNull = $derived(isNull(cell));
  let cellDisplay = $derived(extractCellValue(cell));
  let hasSelection = $derived(selectedRows.length > 1);
</script>

<svelte:window onmousedown={handleClickOutside} onkeydown={handleKeydown} />

<div
  class="context-menu"
  style="left: {clamped.x}px; top: {clamped.y}px;"
  bind:this={menuEl}
  role="menu"
>
  <button class="menu-item" onclick={() => { onCopyCell?.(); onClose(); }} role="menuitem">
    <span class="menu-label">Copy cell</span>
    <span class="menu-hint">{cellIsNull ? 'NULL' : cellDisplay.length > 20 ? cellDisplay.slice(0, 20) + '...' : cellDisplay}</span>
  </button>

  <button class="menu-item" onclick={() => { onCopyRows?.(); onClose(); }} role="menuitem">
    <span class="menu-label">{hasSelection ? `Copy ${selectedRows.length} rows` : 'Copy row'}</span>
    <span class="menu-hint">TSV</span>
  </button>

  <div class="menu-separator"></div>

  {#if schema && table}
    <button class="menu-item" onclick={() => { onCopyAsInsert?.(); onClose(); }} role="menuitem">
      <span class="menu-label">Copy as INSERT</span>
    </button>
  {/if}

  <button class="menu-item" onclick={() => { onCopyAsJson?.(); onClose(); }} role="menuitem">
    <span class="menu-label">Copy as JSON</span>
  </button>

  {#if editable && onSetNull}
    <div class="menu-separator"></div>
    <button class="menu-item" onclick={() => { onSetNull?.(); onClose(); }} role="menuitem">
      <span class="menu-label">Set NULL</span>
    </button>
  {/if}

  {#if onFilterByValue && !cellIsNull}
    <div class="menu-separator"></div>
    <button class="menu-item" onclick={() => { onFilterByValue?.(); onClose(); }} role="menuitem">
      <span class="menu-label">Filter by this value</span>
      <span class="menu-hint">{column.name} = {cellDisplay.length > 15 ? cellDisplay.slice(0, 15) + '...' : cellDisplay}</span>
    </button>
  {/if}
</div>

<style>
  .context-menu {
    position: fixed;
    z-index: 10000;
    min-width: 200px;
    max-width: 320px;
    background: var(--bg-secondary);
    border: 1px solid var(--border-color);
    border-radius: var(--radius-md, 6px);
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    padding: 4px 0;
    font-size: 12px;
  }

  .menu-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    width: 100%;
    padding: 6px 12px;
    background: none;
    border: none;
    color: var(--text-primary);
    cursor: pointer;
    text-align: left;
    font-size: 12px;
  }

  .menu-item:hover {
    background: var(--bg-hover);
  }

  .menu-label {
    white-space: nowrap;
  }

  .menu-hint {
    font-size: 10px;
    color: var(--text-muted);
    font-family: var(--font-mono);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    max-width: 140px;
  }

  .menu-separator {
    height: 1px;
    background: var(--border-color);
    margin: 4px 0;
  }
</style>
