<script lang="ts">
  import type { ColumnDef, FilterCondition } from '$lib/types/query';

  let {
    columns,
    columnWidths,
    filters = [],
    showCheckbox = false,
    onFiltersChange,
  }: {
    columns: ColumnDef[];
    columnWidths: Record<string, number>;
    filters?: FilterCondition[];
    showCheckbox?: boolean;
    onFiltersChange?: (filters: FilterCondition[]) => void;
  } = $props();

  const operators: { value: FilterCondition['operator']; label: string }[] = [
    { value: 'eq', label: '=' },
    { value: 'neq', label: '!=' },
    { value: 'gt', label: '>' },
    { value: 'gte', label: '>=' },
    { value: 'lt', label: '<' },
    { value: 'lte', label: '<=' },
    { value: 'contains', label: 'contains' },
    { value: 'starts_with', label: 'starts' },
    { value: 'is_null', label: 'NULL' },
    { value: 'is_not_null', label: 'NOT NULL' },
  ];

  function getFilter(colName: string): FilterCondition | undefined {
    return filters.find(f => f.column === colName);
  }

  function getOperator(colName: string): FilterCondition['operator'] {
    return getFilter(colName)?.operator ?? 'contains';
  }

  function getValue(colName: string): string {
    return getFilter(colName)?.value ?? '';
  }

  function updateFilter(colName: string, operator: FilterCondition['operator'], value: string) {
    const isNullOp = operator === 'is_null' || operator === 'is_not_null';
    const newFilters = filters.filter(f => f.column !== colName);
    if (isNullOp || value.trim() !== '') {
      newFilters.push({ column: colName, operator, value: isNullOp ? '' : value });
    }
    onFiltersChange?.(newFilters);
  }

  function handleOperatorChange(colName: string, e: Event) {
    const op = (e.target as HTMLSelectElement).value as FilterCondition['operator'];
    const val = getValue(colName);
    updateFilter(colName, op, val);
  }

  function handleValueKeydown(colName: string, e: KeyboardEvent) {
    if (e.key === 'Enter') {
      const op = getOperator(colName);
      const val = (e.target as HTMLInputElement).value;
      updateFilter(colName, op, val);
    }
  }

  function handleValueBlur(colName: string, e: FocusEvent) {
    const op = getOperator(colName);
    const val = (e.target as HTMLInputElement).value;
    updateFilter(colName, op, val);
  }

  function clearFilter(colName: string) {
    const newFilters = filters.filter(f => f.column !== colName);
    onFiltersChange?.(newFilters);
  }

  function clearAll() {
    onFiltersChange?.([]);
  }
</script>

<div class="filter-bar">
  <div class="filter-row">
    {#if showCheckbox}
      <div class="filter-spacer checkbox-spacer"></div>
    {/if}
    <div class="filter-spacer row-number-spacer"></div>
    {#each columns as col}
      {@const w = columnWidths[col.name] ?? 150}
      {@const filter = getFilter(col.name)}
      {@const op = getOperator(col.name)}
      {@const isNullOp = op === 'is_null' || op === 'is_not_null'}
      <div class="filter-cell" style="width: {w}px; min-width: {w}px; max-width: {w}px;">
        <select
          class="filter-operator"
          value={op}
          onchange={(e) => handleOperatorChange(col.name, e)}
        >
          {#each operators as o}
            <option value={o.value}>{o.label}</option>
          {/each}
        </select>
        {#if !isNullOp}
          <input
            class="filter-input"
            type="text"
            placeholder="filter..."
            value={getValue(col.name)}
            onkeydown={(e) => handleValueKeydown(col.name, e)}
            onblur={(e) => handleValueBlur(col.name, e)}
          />
        {/if}
        {#if filter}
          <button class="filter-clear" onclick={() => clearFilter(col.name)} title="Clear filter">&times;</button>
        {/if}
      </div>
    {/each}
    <div class="filter-actions">
      {#if filters.length > 0}
        <button class="clear-all-btn" onclick={clearAll} title="Clear all filters">Clear</button>
      {/if}
    </div>
  </div>
</div>

<style>
  .filter-bar {
    background: var(--bg-secondary);
    border-bottom: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .filter-row {
    display: flex;
    align-items: center;
  }

  .filter-spacer.checkbox-spacer {
    min-width: 32px;
    max-width: 32px;
    flex-shrink: 0;
  }

  .filter-spacer.row-number-spacer {
    min-width: 50px;
    max-width: 50px;
    flex-shrink: 0;
  }

  .filter-cell {
    display: flex;
    align-items: center;
    gap: 2px;
    padding: 3px 4px;
    flex: none;
    border-right: 1px solid rgba(69, 71, 90, 0.3);
    overflow: hidden;
  }

  .filter-operator {
    width: 56px;
    flex-shrink: 0;
    padding: 2px 2px;
    font-size: 10px;
    font-family: var(--font-mono);
    background: var(--bg-primary);
    color: var(--text-secondary);
    border: 1px solid var(--border-color);
    border-radius: 2px;
  }

  .filter-input {
    flex: 1;
    min-width: 0;
    padding: 2px 4px;
    font-size: 11px;
    font-family: var(--font-mono);
    color: var(--text-primary);
    background: var(--bg-primary);
    border: 1px solid var(--border-color);
    border-radius: 2px;
    outline: none;
  }

  .filter-input:focus {
    border-color: var(--accent);
  }

  .filter-input::placeholder {
    color: var(--text-muted);
    opacity: 0.5;
  }

  .filter-clear {
    flex-shrink: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 16px;
    height: 16px;
    padding: 0;
    font-size: 12px;
    background: none;
    border: none;
    color: var(--text-muted);
    cursor: pointer;
    border-radius: 2px;
  }

  .filter-clear:hover {
    color: var(--error);
    background: rgba(243, 139, 168, 0.1);
  }

  .filter-actions {
    display: flex;
    align-items: center;
    padding: 0 4px;
    flex-shrink: 0;
  }

  .clear-all-btn {
    padding: 2px 6px;
    font-size: 10px;
    color: var(--text-muted);
    background: none;
    border: 1px solid var(--border-color);
    border-radius: 2px;
    cursor: pointer;
    white-space: nowrap;
  }

  .clear-all-btn:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }
</style>
