<script lang="ts">
  import type { ColumnDef, CellValue } from '$lib/types/query';
  import { extractCellValue, isNull } from '$lib/utils/formatters';

  let {
    value,
    column,
    width = 150,
    editable = false,
    onEdit,
    onSetNull,
    onContextMenu,
  }: {
    value: CellValue;
    column: ColumnDef;
    width?: number;
    editable?: boolean;
    onEdit?: (value: string) => void;
    onSetNull?: () => void;
    onContextMenu?: (e: MouseEvent) => void;
  } = $props();

  let isEditing = $state(false);
  let editValue = $state('');
  let inputEl: HTMLInputElement;

  let displayValue = $derived(extractCellValue(value));
  let cellIsNull = $derived(isNull(value));
  let isBool = $derived(value.type === 'Bool');
  let isNumeric = $derived(value.type === 'Int' || value.type === 'Float');

  function handleDblClick() {
    if (!editable || !onEdit) return;
    isEditing = true;
    editValue = cellIsNull ? '' : displayValue;
    requestAnimationFrame(() => {
      inputEl?.focus();
      inputEl?.select();
    });
  }

  function handleSave() {
    isEditing = false;
    if (onEdit) {
      onEdit(editValue);
    }
  }

  function handleCancel() {
    isEditing = false;
  }

  function handleSetNull() {
    isEditing = false;
    onSetNull?.();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    }
  }

  function handleContextMenu(e: MouseEvent) {
    onContextMenu?.(e);
  }
</script>

<div
  class="grid-cell"
  class:null-value={cellIsNull}
  class:numeric={isNumeric}
  class:bool-value={isBool}
  style="width: {width}px; min-width: {width}px; max-width: {width}px;"
  ondblclick={handleDblClick}
  oncontextmenu={handleContextMenu}
>
  {#if isEditing}
    <input
      bind:this={inputEl}
      bind:value={editValue}
      class="cell-input"
      onblur={handleSave}
      onkeydown={handleKeydown}
    />
    {#if onSetNull}
      <button
        class="null-btn"
        onmousedown={(e) => { e.preventDefault(); handleSetNull(); }}
        title="Set NULL"
      >NULL</button>
    {/if}
  {:else}
    <span class="cell-text truncate">{displayValue}</span>
  {/if}
</div>

<style>
  .grid-cell {
    display: flex;
    align-items: center;
    padding: 0 10px;
    flex: none;
    border-right: 1px solid rgba(69, 71, 90, 0.3);
    overflow: hidden;
    cursor: default;
    font-size: 12px;
    font-family: var(--font-mono);
    gap: 4px;
  }

  .grid-cell.null-value .cell-text {
    color: var(--text-muted);
    font-style: italic;
    opacity: 0.6;
  }

  .grid-cell.numeric {
    justify-content: flex-end;
  }

  .grid-cell.bool-value .cell-text {
    color: var(--accent);
  }

  .cell-text {
    max-width: 100%;
    line-height: 32px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cell-input {
    flex: 1;
    min-width: 0;
    height: 28px;
    padding: 2px 6px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    background: var(--bg-primary);
    border: 1px solid var(--accent);
    border-radius: 2px;
    outline: none;
  }

  .null-btn {
    flex-shrink: 0;
    padding: 2px 6px;
    font-size: 9px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--text-muted);
    background: var(--bg-tertiary);
    border: 1px solid var(--border-color);
    border-radius: 2px;
    cursor: pointer;
    line-height: 1;
  }

  .null-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }
</style>
