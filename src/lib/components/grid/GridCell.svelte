<script lang="ts">
  import type { ColumnDef, CellValue } from '$lib/types/query';
  import { extractCellValue, isNull, isLargeValue, getLargeValueLength, truncateDisplay, formatCharCount } from '$lib/utils/formatters';

  let {
    value,
    column,
    width = 150,
    editable = false,
    isModified = false,
    onEdit,
    onSetNull,
    onContextMenu,
    onExpandCell,
  }: {
    value: CellValue;
    column: ColumnDef;
    width?: number;
    editable?: boolean;
    isModified?: boolean;
    onEdit?: (value: string) => void;
    onSetNull?: () => void;
    onContextMenu?: (e: MouseEvent) => void;
    onExpandCell?: () => void;
  } = $props();

  let isEditing = $state(false);
  let editValue = $state('');
  let inputEl = $state<HTMLInputElement | HTMLTextAreaElement | undefined>(undefined);

  let displayValue = $derived(extractCellValue(value));
  let cellIsNull = $derived(isNull(value));
  let isBool = $derived(value.type === 'Bool');
  let isNumeric = $derived(value.type === 'Int' || value.type === 'Float');
  let isJson = $derived(value.type === 'Json' || value.type === 'LargeJson' || column.data_type.toLowerCase().includes('json'));
  let isLongText = $derived(!isJson && displayValue.length > 100);
  let useTextarea = $derived(isJson || isLongText);
  let truncatedValue = $derived(truncateDisplay(displayValue));
  let isTruncatedDisplay = $derived(displayValue.length > 500);
  let isLarge = $derived(isLargeValue(value));
  let largeLength = $derived(isLarge ? getLargeValueLength(value) : 0);

  function handleDblClick() {
    if (!editable || !onEdit) return;
    if (isLarge) {
      // For large values, expand first instead of editing
      onExpandCell?.();
      return;
    }
    if (isBool && !cellIsNull) {
      // Toggle boolean directly without entering edit mode
      onEdit(value.type === 'Bool' && value.value ? 'false' : 'true');
      return;
    }
    isEditing = true;
    editValue = cellIsNull ? '' : displayValue;
    requestAnimationFrame(() => {
      inputEl?.focus();
      if (inputEl instanceof HTMLInputElement) inputEl.select();
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
    if (e.key === 'Enter' && (!useTextarea || e.ctrlKey || e.metaKey)) {
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

  function handleExpandClick(e: MouseEvent) {
    e.stopPropagation();
    onExpandCell?.();
  }
</script>

<div
  class="grid-cell"
  class:null-value={cellIsNull}
  class:numeric={isNumeric}
  class:bool-value={isBool}
  class:modified={isModified}
  style="width: {width}px; min-width: {width}px; max-width: {width}px;"
  ondblclick={handleDblClick}
  oncontextmenu={handleContextMenu}
  role="gridcell"
  tabindex="-1"
>
  {#if isEditing}
    {#if useTextarea}
      <textarea
        bind:this={inputEl}
        bind:value={editValue}
        class="cell-textarea"
        class:json={isJson}
        onblur={handleSave}
        onkeydown={handleKeydown}
        rows="4"
      ></textarea>
    {:else}
      <input
        bind:this={inputEl}
        bind:value={editValue}
        class="cell-input"
        onblur={handleSave}
        onkeydown={handleKeydown}
      />
    {/if}
    {#if onSetNull}
      <button
        class="null-btn"
        onmousedown={(e) => { e.preventDefault(); handleSetNull(); }}
        title="Set NULL"
      >NULL</button>
    {/if}
  {:else if cellIsNull}
    <span class="null-badge">NULL</span>
  {:else if isBool && editable}
    <input
      type="checkbox"
      class="bool-checkbox"
      checked={value.type === 'Bool' && value.value}
      onchange={() => { if (onEdit) onEdit(value.type === 'Bool' && value.value ? 'false' : 'true'); }}
    />
    <span class="cell-text">{displayValue}</span>
  {:else if isLarge}
    <span class="cell-text truncate">{truncatedValue}</span>
    <button
      class="expand-btn"
      onclick={handleExpandClick}
      title="Load full value ({formatCharCount(largeLength)} {value.type === 'LargeBinary' ? 'bytes' : 'chars'})"
    >
      <svg width="10" height="10" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2">
        <polyline points="4 6 8 10 12 6"></polyline>
      </svg>
      <span class="expand-size">{formatCharCount(largeLength)}</span>
    </button>
  {:else}
    <span class="cell-text truncate">{truncatedValue}</span>
    {#if isTruncatedDisplay}
      <span class="char-count-badge" title="{displayValue.length} characters">{formatCharCount(displayValue.length)}</span>
    {/if}
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

  .grid-cell.modified {
    border-left: 3px solid var(--warning, #fab387);
    padding-left: 7px;
  }

  .null-badge {
    display: inline-flex;
    align-items: center;
    padding: 1px 6px;
    font-size: 9px;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--text-muted);
    background: var(--bg-tertiary, rgba(69, 71, 90, 0.3));
    border: 1px solid var(--border-color);
    border-radius: 3px;
    opacity: 0.7;
    line-height: 1.2;
    letter-spacing: 0.5px;
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

  .cell-textarea {
    flex: 1;
    min-width: 0;
    min-height: 60px;
    padding: 4px 6px;
    font-size: 12px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    background: var(--bg-primary);
    border: 1px solid var(--accent);
    border-radius: 2px;
    outline: none;
    resize: vertical;
    line-height: 1.4;
  }

  .cell-textarea.json {
    font-family: var(--font-mono);
    tab-size: 2;
  }

  .char-count-badge {
    display: inline-flex;
    align-items: center;
    padding: 0 4px;
    font-size: 9px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--text-muted);
    background: var(--bg-tertiary, rgba(69, 71, 90, 0.3));
    border-radius: 3px;
    flex-shrink: 0;
    line-height: 1.4;
    opacity: 0.7;
  }

  .expand-btn {
    display: inline-flex;
    align-items: center;
    gap: 2px;
    padding: 1px 5px;
    font-size: 9px;
    font-weight: 600;
    font-family: var(--font-mono);
    color: var(--accent);
    background: rgba(137, 180, 250, 0.1);
    border: 1px solid rgba(137, 180, 250, 0.3);
    border-radius: 3px;
    cursor: pointer;
    flex-shrink: 0;
    line-height: 1.4;
  }

  .expand-btn:hover {
    background: rgba(137, 180, 250, 0.2);
    border-color: var(--accent);
  }

  .expand-size {
    font-size: 9px;
  }

  .bool-checkbox {
    width: 14px;
    height: 14px;
    accent-color: var(--accent);
    cursor: pointer;
    flex-shrink: 0;
  }
</style>
